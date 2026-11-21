//! E2E harness: isolated, self-cleaning tests against a real DRACOON instance.
//!
//! Test run
//!  - a top level room per run (`e2e-run-<id>`), created on first use - all nodes are created inside
//!  - encrypted tests (opt-in, `E2E_CRYPTO=1`): the e2e user gets a new keypair with a random secret and an
//!    encrypted top level room (`e2e-run-<id>-crypto`) - use a dedicated test user, its keypair is replaced
//!  - the run rooms are deleted at the end of the run, also if tests failed (see `main.rs`)
//!
//! Isolation
//!  - every test connects on its own and works in its own folders / rooms / users / groups with unique
//!    names (`e2e-<test>-<random>`) - tests share no state and can run in parallel
//!
//! Cleanup
//!  - everything created via the `Ctx` helpers is deleted after the test (also on failure or panic),
//!    in reverse order
//!
//! Configuration (environment or `.env`, see `.env.example`) - tests are reported as ignored if missing

use std::{
    collections::hash_map::RandomState,
    future::Future,
    hash::{BuildHasher, Hasher},
    io::Cursor,
    panic::AssertUnwindSafe,
    pin::Pin,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use dco3::{
    auth::Connected,
    groups::{CreateGroupRequest, Group},
    nodes::{
        CreateFolderRequest, CreateRoomRequest, EncryptRoomRequest, FileMeta, Node,
        ResolutionStrategy, UploadOptions,
    },
    shares::{CreateDownloadShareRequest, CreateUploadShareRequest, DownloadShare, UploadShare},
    user::UserAuthData,
    users::{CreateUserRequest, UserData},
    Download, DownloadShares, Dracoon, DracoonClientError, Folders, Groups, Nodes, OAuth2Flow,
    Rooms, Upload, UploadShares, UserAccountKeyPairs, Users,
};
use futures_util::FutureExt;
use tokio::{io::BufReader, sync::OnceCell};

pub type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub type TestFn = for<'a> fn(&'a Ctx) -> BoxFuture<'a, TestResult>;

/// required environment variables
pub const REQUIRED: [&str; 3] = ["E2E_BASE_URL", "E2E_USER_NAME", "E2E_PASSWORD"];

/// which run resources a test needs
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// unencrypted run room
    Plain,
    /// encrypted run room + unlocked keypair (opt-in)
    Crypto,
}

#[derive(Clone)]
pub struct Config {
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub client_id: String,
    pub client_secret: String,
    pub email_domain: String,
    pub crypto: bool,
}

impl Config {
    /// reads the configuration from the environment (and `.env`) - returns the missing variables
    pub fn from_env() -> Result<Self, Vec<&'static str>> {
        // a missing .env file is fine - the environment may be set directly (e.g. in CI)
        let _ = dotenvy::dotenv();

        let var = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
        let missing: Vec<_> = REQUIRED
            .into_iter()
            .filter(|name| var(name).is_none())
            .collect();
        if !missing.is_empty() {
            return Err(missing);
        }

        Ok(Self {
            base_url: var("E2E_BASE_URL").unwrap(),
            username: var("E2E_USER_NAME").unwrap(),
            password: var("E2E_PASSWORD").unwrap(),
            client_id: var("CLIENT_ID").unwrap_or_else(|| "dracoon_legacy_scripting".into()),
            client_secret: var("CLIENT_SECRET").unwrap_or_default(),
            email_domain: var("E2E_EMAIL_DOMAIN").unwrap_or_else(|| "example.com".into()),
            crypto: var("E2E_CRYPTO").as_deref() == Some("1"),
        })
    }

    /// connects the e2e user (password flow) - with an encryption secret, the keypair is unlocked
    pub async fn connect(
        &self,
        encryption_secret: Option<&str>,
    ) -> Result<Dracoon<Connected>, DracoonClientError> {
        let mut builder = Dracoon::builder()
            .with_base_url(&self.base_url)
            .with_client_id(&self.client_id)
            .with_client_secret(&self.client_secret)
            .with_user_agent("dco3-e2e");
        if let Some(secret) = encryption_secret {
            builder = builder.with_encryption_password(secret);
        }

        builder
            .build()?
            .connect(OAuth2Flow::password_flow(&self.username, &self.password))
            .await
    }
}

/// random hex string (not cryptographically strong - used for unique names and disposable test secrets)
pub fn random_hex() -> String {
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u128(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|time| time.as_nanos())
            .unwrap_or_default(),
    );
    format!("{:016x}", hasher.finish())
}

/// pseudo-random test data (xorshift) - reproducible per seed, not compressible
pub fn test_data(size: usize, seed: u64) -> Vec<u8> {
    let mut state = seed | 1;
    (0..size)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 24) as u8
        })
        .collect()
}

/// per run resources - created once on first use, removed by `cleanup()`
pub struct TestRun {
    pub config: Config,
    pub run_id: String,
    room: OnceCell<Result<Node, String>>,
    crypto: OnceCell<Result<(Node, String), String>>,
    rooms: Mutex<Vec<Node>>,
}

impl TestRun {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            run_id: random_hex()[..8].to_string(),
            room: OnceCell::new(),
            crypto: OnceCell::new(),
            rooms: Mutex::new(Vec::new()),
        }
    }

    /// the unencrypted run room - a failed setup fails every test that needs it
    pub async fn room(&self) -> Result<Node, String> {
        self.room
            .get_or_init(|| async {
                self.create_run_room(None)
                    .await
                    .map_err(|err| format!("E2E test run setup failed (room): {err:?}"))
            })
            .await
            .clone()
    }

    /// (encrypted run room, keypair secret)
    pub async fn crypto(&self) -> Result<(Node, String), String> {
        self.crypto
            .get_or_init(|| async {
                let setup = async {
                    // the room admin needs a keypair to encrypt the room
                    let secret = self.reset_keypair().await?;
                    let room = self.create_run_room(Some(&secret)).await?;
                    Ok::<_, DracoonClientError>((room, secret))
                };
                setup
                    .await
                    .map_err(|err| format!("E2E test run setup failed (crypto): {err:?}"))
            })
            .await
            .clone()
    }

    async fn create_run_room(&self, secret: Option<&str>) -> Result<Node, DracoonClientError> {
        let dracoon = self.config.connect(secret).await?;
        let user_id = dracoon.get_user_info().await?.id;

        let name = match secret {
            Some(_) => format!("e2e-run-{}-crypto", self.run_id),
            None => format!("e2e-run-{}", self.run_id),
        };
        let req = CreateRoomRequest::builder(&name)
            .with_inherit_permissions(false)
            .with_admin_ids(vec![user_id])
            .build();
        let room = dracoon.nodes().create_room(req).await?;
        self.rooms.lock().unwrap().push(room.clone());

        if secret.is_some() {
            let req = EncryptRoomRequest::builder(true).build();
            return dracoon.nodes().encrypt_room(room.id, req).await;
        }

        Ok(room)
    }

    /// replaces the e2e user's keypair with a new one protected by a random secret
    async fn reset_keypair(&self) -> Result<String, DracoonClientError> {
        let secret = format!("{}{}aA1!", random_hex(), random_hex());
        let dracoon = self.config.connect(None).await?;

        // only "not found" means there is no keypair - a wrong secret means there is one
        let has_keypair = match dracoon.user().get_user_keypair(&secret).await {
            Err(err) if err.is_not_found() => false,
            Ok(_) | Err(DracoonClientError::CryptoError(_)) => true,
            Err(err) => return Err(err),
        };

        if has_keypair {
            // known server behaviour: after an encrypted room with deleted files was removed,
            // deleting the keypair can fail with 404 "Node not found" - the keypair then has to be
            // reset server-side before encrypted tests can run again
            dracoon.user().delete_user_keypair().await?;
        }
        dracoon.user().set_user_keypair(&secret).await?;

        Ok(secret)
    }

    /// deletes the run rooms (room deletion is permanent - no recycle bin) - returns errors
    pub async fn cleanup(&self) -> Vec<String> {
        let rooms = std::mem::take(&mut *self.rooms.lock().unwrap());
        if rooms.is_empty() {
            return Vec::new();
        }

        let dracoon = match self.config.connect(None).await {
            Ok(dracoon) => dracoon,
            Err(err) => {
                return rooms
                    .iter()
                    .map(|room| format!("{} ({}): cannot connect: {err:?}", room.name, room.id))
                    .collect()
            }
        };

        let mut errors = Vec::new();
        for room in rooms {
            if let Err(err) = dracoon.nodes().delete_node(room.id).await {
                errors.push(format!("{} ({}): {err:?}", room.name, room.id));
            }
        }
        errors
    }

    /// runs a single test: connect, run, clean up (also on failure / panic)
    pub async fn run_test(&self, name: &str, kind: Kind, test: TestFn) -> Result<(), String> {
        let room = self.room().await?;
        let crypto = match kind {
            Kind::Plain => None,
            Kind::Crypto => Some(self.crypto().await?),
        };

        let dracoon = self
            .config
            .connect(crypto.as_ref().map(|(_, secret)| secret.as_str()))
            .await
            .map_err(|err| format!("connect failed: {err:?}"))?;

        let ctx = Ctx {
            dracoon,
            config: self.config.clone(),
            test_name: name.rsplit("::").next().unwrap_or(name).to_string(),
            room,
            crypto_room: crypto.map(|(room, _)| room),
            cleanups: Mutex::new(Vec::new()),
        };

        let outcome = AssertUnwindSafe(test(&ctx)).catch_unwind().await;
        let cleanup_errors = ctx.run_cleanups().await;

        let result = match outcome {
            Ok(Ok(())) => Ok(()),
            Ok(Err(err)) => Err(format!("{err}\n{err:?}")),
            Err(panic) => Err(panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|msg| msg.to_string()))
                .unwrap_or_else(|| "test panicked".into())),
        };

        match (result, cleanup_errors.is_empty()) {
            (Ok(()), true) => Ok(()),
            (Ok(()), false) => Err(format!("cleanup failed:\n{}", cleanup_errors.join("\n"))),
            (Err(err), true) => Err(err),
            (Err(err), false) => Err(format!(
                "{err}\ncleanup failed:\n{}",
                cleanup_errors.join("\n")
            )),
        }
    }
}

type Cleanup = (String, BoxFuture<'static, Result<(), DracoonClientError>>);

/// per test context: connected client, run rooms and cleanup helpers
pub struct Ctx {
    pub dracoon: Dracoon<Connected>,
    pub config: Config,
    pub test_name: String,
    /// unencrypted run room
    pub room: Node,
    /// encrypted run room (crypto tests only)
    pub crypto_room: Option<Node>,
    cleanups: Mutex<Vec<Cleanup>>,
}

impl Ctx {
    /// `e2e-<test>-<random>[-suffix]`
    pub fn unique_name(&self, suffix: &str) -> String {
        let name = format!("e2e-{}-{}", self.test_name, &random_hex()[..8]);
        if suffix.is_empty() {
            name
        } else {
            format!("{name}-{suffix}")
        }
    }

    /// deletes a created object after the test - objects already deleted by the test are ignored
    pub fn defer<F>(&self, what: impl Into<String>, cleanup: F)
    where
        F: Future<Output = Result<(), DracoonClientError>> + Send + 'static,
    {
        self.cleanups
            .lock()
            .unwrap()
            .push((what.into(), Box::pin(cleanup)));
    }

    async fn run_cleanups(&self) -> Vec<String> {
        let cleanups = std::mem::take(&mut *self.cleanups.lock().unwrap());
        let mut errors = Vec::new();
        // reverse order: e.g. shares before their nodes, users before groups they joined
        for (what, cleanup) in cleanups.into_iter().rev() {
            match cleanup.await {
                Err(err) if !err.is_not_found() => errors.push(format!("{what}: {err:?}")),
                _ => {}
            }
        }
        errors
    }

    /// encrypted run room - only available in crypto tests
    pub fn crypto_room(&self) -> &Node {
        self.crypto_room
            .as_ref()
            .expect("crypto room is only available in crypto tests")
    }

    // nodes

    /// folder in the given parent (default: run room) - deleted after the test
    pub async fn create_folder(&self, parent_id: Option<u64>) -> Result<Node, DracoonClientError> {
        let req =
            CreateFolderRequest::builder(self.unique_name(""), parent_id.unwrap_or(self.room.id))
                .build();
        let folder = self.dracoon.nodes().create_folder(req).await?;
        self.defer_node_deletion(&folder);
        Ok(folder)
    }

    /// sub room of the run room - deleted after the test
    /// (own permissions with the e2e user as admin, or inherited permissions)
    pub async fn create_room(&self, own_permissions: bool) -> Result<Node, DracoonClientError> {
        let mut req = CreateRoomRequest::builder(&self.unique_name(""))
            .with_parent_id(self.room.id)
            .with_inherit_permissions(!own_permissions);
        if own_permissions {
            req = req.with_admin_ids(vec![self.dracoon.get_user_info().await?.id]);
        }
        let room = self.dracoon.nodes().create_room(req.build()).await?;
        self.defer_node_deletion(&room);
        Ok(room)
    }

    fn defer_node_deletion(&self, node: &Node) {
        let dracoon = self.dracoon.clone();
        let id = node.id;
        self.defer(format!("node {} ({id})", node.name), async move {
            dracoon.nodes().delete_node(id).await
        });
    }

    // transfers

    /// uploads `data` into `parent` (cleaned up with the parent) - returns the file node
    pub async fn upload(
        &self,
        parent: &Node,
        name: &str,
        data: Vec<u8>,
        chunk_size: Option<usize>,
        resolution_strategy: Option<ResolutionStrategy>,
    ) -> Result<Node, DracoonClientError> {
        let file_meta = FileMeta::builder(name, data.len() as u64).build();
        let mut options = UploadOptions::builder(file_meta);
        if let Some(strategy) = resolution_strategy {
            options = options.with_resolution_strategy(strategy);
        }
        let reader = BufReader::new(Cursor::new(data));

        self.dracoon
            .upload(parent, options.build(), reader, None, chunk_size)
            .await
    }

    /// downloads a file into memory
    pub async fn download(&self, node: &Node) -> Result<Vec<u8>, DracoonClientError> {
        let mut buffer = Vec::new();
        self.dracoon.download(node, &mut buffer, None, None).await?;
        Ok(buffer)
    }

    // users, groups, shares

    /// local user (no notification) - deleted after the test
    pub async fn create_user(&self) -> Result<UserData, DracoonClientError> {
        let name = self.unique_name("");
        let email = format!("{name}@{}", self.config.email_domain);
        // without notification (no invitation mail) an initial password is required
        let password = format!("{}{}aA1!", random_hex(), random_hex());
        let req = CreateUserRequest::builder("E2E", &name)
            .with_user_name(&email)
            .with_email(&email)
            .with_auth_data(UserAuthData::new_basic(Some(password), Some(true)))
            .with_notify_user(false)
            .build();

        let user = self.dracoon.users().create_user(req).await?;
        let dracoon = self.dracoon.clone();
        let id = user.id;
        self.defer(format!("user {} ({id})", user.user_name), async move {
            dracoon.users().delete_user(id).await
        });
        Ok(user)
    }

    /// group - deleted after the test
    pub async fn create_group(&self) -> Result<Group, DracoonClientError> {
        let req = CreateGroupRequest::new(self.unique_name(""), None);
        let group = self.dracoon.groups().create_group(req).await?;
        let dracoon = self.dracoon.clone();
        let id = group.id;
        self.defer(format!("group {} ({id})", group.name), async move {
            dracoon.groups().delete_group(id).await
        });
        Ok(group)
    }

    /// download share - deleted after the test (also removed with its node)
    pub async fn create_download_share(
        &self,
        node_id: u64,
        password: Option<&str>,
    ) -> Result<DownloadShare, DracoonClientError> {
        let mut req = CreateDownloadShareRequest::builder(node_id)
            .with_name(self.unique_name(""))
            .with_notes("e2e");
        if let Some(password) = password {
            req = req.with_password(password);
        }
        let share = self
            .dracoon
            .shares()
            .create_download_share(req.build())
            .await?;
        let dracoon = self.dracoon.clone();
        let id = share.id;
        self.defer(
            format!("download share {} ({id})", share.name),
            async move { dracoon.shares().delete_download_share(id).await },
        );
        Ok(share)
    }

    /// file request (upload share) - deleted after the test (also removed with its node)
    pub async fn create_upload_share(
        &self,
        target_id: u64,
        max_slots: Option<u32>,
    ) -> Result<UploadShare, DracoonClientError> {
        let mut req = CreateUploadShareRequest::builder(target_id).with_name(self.unique_name(""));
        if let Some(max_slots) = max_slots {
            req = req.with_max_slots(max_slots);
        }
        let share = self
            .dracoon
            .shares()
            .create_upload_share(req.build())
            .await?;
        let dracoon = self.dracoon.clone();
        let id = share.id;
        self.defer(format!("upload share {} ({id})", share.name), async move {
            dracoon.shares().delete_upload_share(id).await
        });
        Ok(share)
    }
}
