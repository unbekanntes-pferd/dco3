#![allow(dead_code)]
#![allow(unused_variables)]

//! # dco3 - DRACOON API wrapper in Rust
//!
//! `dco3` is an async wrapper around API calls in DRACOON.
//! DRACOON is a cloud service provider - more information can be found on <https://dracoon.com>.
//! The name is based on several other projects pointing to oxide (Rust) and DRACOON.
//!
//! ## Usage
//! All API calls are implemented by the `Dracoon` struct. It can be created by using the `builder()` method.
//!
//! In order to access specific API calls, the `Dracoon` struct needs to be in the `Connected` state.
//! This can be achieved by calling the `connect` method.
//! To use specific endpoints, you need to import relevant traits.
//! Currently, the following traits are implemented:
//!
//! * [User] - for user account management
//! * [UserAccountKeyPairs] - for user keypair management
//! * [Nodes] - for node operations (folders, rooms, upload and download are excluded)
//! * [Download] - for downloading files
//! * [Upload] - for uploading files
//! * [Folders] - for folder operations
//! * [Rooms] - for room operations
//! * [DownloadShares] - for download share operations
//! * [UploadShares] - for upload share operations
//! * [Groups] - for group operations
//! * [Users] - for user management operations
//! * [CustomerProvisioning] - for customer provisioning operations
//! * [MissingFileKeys] - for distributing missing keys using the user keypair
//! * [RescueKeyPair] - for distributing missing keys using the rescue key
//! * [Config] - for general configuration information
//! * [AuthenticationMethods] - for authentication methods information
//! * [Eventlog] - for eventlog information
//! * [Public] - for public information
//! * [PublicDownload] - for public download via share
//! * [PublicUpload] - for public upload via file request
//! * [Roles] - for role operations
//!
//! ### Example
//! ```no_run
//! use dco3::{Dracoon, OAuth2Flow, User};
//!
//! #[tokio::main]
//! async fn main() {
//!    let dracoon = Dracoon::builder()
//!       .with_base_url("https://dracoon.team")
//!       .with_client_id("client_id")
//!       .with_client_secret("client_secret")
//!       .build()
//!       .unwrap()
//!       .connect(OAuth2Flow::password_flow("username", "password"))
//!       .await
//!       .unwrap();
//!
//!   let user_info = dracoon.user().get_user_account().await.unwrap();
//!   println!("User info: {:?}", user_info);
//! }
//!```
//!
//! ## Authentication
//!
//! All supported OAuth2 flows are implemented.
//!
//! ### Password Flow
//! ```no_run
//! use dco3::{Dracoon, OAuth2Flow};
//!
//! #[tokio::main]
//! async fn main() {
//!
//!    // you can instantiate the required flow by using the `OAuth2Flow` enum
//!    let password_flow = OAuth2Flow::password_flow("username", "password");
//!
//!    let dracoon = Dracoon::builder()
//!       .with_base_url("https://dracoon.team")
//!       .with_client_id("client_id")
//!       .with_client_secret("client_secret")
//!       .build()
//!       .unwrap()
//!       .connect(password_flow)
//!       .await
//!       .unwrap();
//! }
//!```
//! ### Authorization Code Flow
//! ```no_run
//! use dco3::{Dracoon, OAuth2Flow};
//!
//! #[tokio::main]
//! async fn main() {
//!
//!    let mut dracoon = Dracoon::builder()
//!       .with_base_url("https://dracoon.team")
//!       .with_client_id("client_id")
//!       .with_client_secret("client_secret")
//!       .with_redirect_uri("https://redirect.uri")
//!       .build()
//!       .unwrap();
//!
//!    // initiate the authorization code flow
//!    let authorize_url = dracoon.get_authorize_url();
//!
//!    // get auth code
//!    let auth_code = "some_auth_code";
//!
//!    // you can instantiate the required flow by using the `OAuth2Flow` enum
//!    let auth_code_flow = OAuth2Flow::authorization_code(auth_code);
//!
//!    let dracoon = dracoon.connect(auth_code_flow).await.unwrap();
//! }
//!```
//!
//! ### Refresh Token
//!
//! ```no_run
//! use dco3::{Dracoon, OAuth2Flow};
//!
//! #[tokio::main]
//! async fn main() {
//!
//!   let refresh_token = "some_refresh_token";
//!
//!   let dracoon = Dracoon::builder()
//!     .with_base_url("https://dracoon.team")
//!     .with_client_id("client_id")
//!     .with_client_secret("client_secret")
//!     .build()
//!     .unwrap()
//!     .connect(OAuth2Flow::refresh_token(refresh_token))
//!     .await
//!     .unwrap();
//!
//! }
//! ```
//!
//! ### Simple
//!
//! ```no_run
//! use dco3::{Dracoon, OAuth2Flow};
//!
//! #[tokio::main]
//! async fn main() {
//!
//!  // you can also pass the access token directly
//!  let dracoon = Dracoon::builder()
//!   .with_base_url("https://dracoon.team")
//!   .with_client_id("client_id")
//!   .with_client_secret("client_secret")
//!   .build()
//!   .unwrap()
//!   .connect(OAuth2Flow::simple("access_token"))
//!   .await
//!   .unwrap();
//!
//!  // be aware that the access token refresh will *not* work
//!  // once the token is expired, you need to pass a new token
//!
//! }
//! ```
//!
//!
//! ## Error handling
//!
//! All errors are wrapped in the [DracoonClientError] enum.
//!
//! Most errrors are related to general usage (like missing parameters).
//!
//! All API errors are wrapped in the `DracoonClientError::Http` variant.
//! The variant contains response with relevant status code and message.
//!
//! You can check if the underlying error message if a specific API error by using the `is_*` methods.
//!
//! ```no_run
//! use dco3::{Dracoon, OAuth2Flow, Nodes};
//!
//! #[tokio::main]
//!
//! async fn main() {
//!
//!  let dracoon = Dracoon::builder()
//!    .with_base_url("https://dracoon.team")
//!    .with_client_id("client_id")
//!    .with_client_secret("client_secret")
//!    .build()
//!    .unwrap()
//!    .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
//!    .await
//!    .unwrap();
//!
//! let node = dracoon.nodes().get_node(123).await;
//!
//! match node {
//!  Ok(node) => println!("Node info: {:?}", node),
//! Err(err) => {
//!  if err.is_not_found() {
//!     println!("Node not found");
//!     } else {
//!          println!("Error: {:?}", err);
//!            }
//!         }
//!       }
//!  }
//! ```
//! If you need more information about the error, you can use the `get_http_error` method.
//!
//! ```no_run
//! use dco3::{Dracoon, OAuth2Flow, Nodes};
//!
//! #[tokio::main]
//!
//! async fn main() {
//!
//!  let dracoon = Dracoon::builder()
//!    .with_base_url("https://dracoon.team")
//!    .with_client_id("client_id")
//!    .with_client_secret("client_secret")
//!    .build()
//!    .unwrap()
//!    .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
//!    .await
//!    .unwrap();
//!
//! let node = dracoon.nodes().get_node(123).await;
//!
//! match node {
//!  Ok(node) => println!("Node info: {:?}", node),
//! Err(err) => {
//!   if let Some(http_err) = err.get_http_error() {
//!    // check error type
//!   if http_err.is_not_found() {
//!    // do something
//!    println!("Node not found");
//!    // check error message
//!   println!("Error message: {}", http_err.error_message());
//!    // access error details
//!   println!("Error details: {}", http_err.debug_info().unwrap());
//!     }
//!    }
//!   }
//!  }
//! }
//!````
//! ### Retries
//! The client will automatically retry failed requests.
//! You can configure the retry behavior by passing your config during client creation.
//!
//! Default values are: 5 retries, min delay 600ms, max delay 20s.
//! Keep in mind that you cannot set arbitrary values - for all values, minimum and maximum values are defined.
//!
//! ```
//!
//! use dco3::{Dracoon, OAuth2Flow};
//!
//! #[tokio::main]
//! async fn main() {
//!
//!  let dracoon = Dracoon::builder()
//!   .with_base_url("https://dracoon.team")
//!   .with_client_id("client_id")
//!   .with_client_secret("client_secret")
//!   .with_max_retries(3)
//!   .with_min_retry_delay(400)
//!   .with_max_retry_delay(1000)
//!   .build();
//!
//! }
//!
//! ```
//!
//! ## Building requests
//!
//! All API calls are implemented as traits.
//! Each API call that requires a sepcific payload has a corresponding builder.
//! To access the builder, you can call the builder() method.
//!
//! ```no_run
//! # use dco3::{Dracoon, OAuth2Flow, Rooms, nodes::CreateRoomRequest};
//! # #[tokio::main]
//! # async fn main() {
//! # let dracoon = Dracoon::builder()
//! #  .with_base_url("https://dracoon.team")
//! #  .with_client_id("client_id")
//! #  .with_client_secret("client_secret")
//! #  .build()
//! #  .unwrap()
//! #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
//! #  .await
//! #  .unwrap();
//! let room = CreateRoomRequest::builder("My Room")
//!            .with_parent_id(123)
//!            .with_admin_ids(vec![1, 2, 3])
//!            .build();
//!
//! let room = dracoon.nodes().create_room(room).await.unwrap();
//!
//! # }
//! ```
//! Some requests do not have any complicated fields - in these cases, use the `new()` method.
//! ```no_run
//! # use dco3::{Dracoon, OAuth2Flow, Groups, groups::CreateGroupRequest};
//! # #[tokio::main]
//! # async fn main() {
//! # let dracoon = Dracoon::builder()
//! #  .with_base_url("https://dracoon.team")
//! #  .with_client_id("client_id")
//! #  .with_client_secret("client_secret")
//! #  .build()
//! #  .unwrap()
//! #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
//! #  .await
//! #  .unwrap();
//!
//! // this takes a mandatory name and optional expiration
//! let group = CreateGroupRequest::new("My Group", None);
//! let group = dracoon.groups().create_group(group).await.unwrap();
//!
//! # }
//! ```
//!
//! ## Pagination
//!
//! GET endpoints are limited to 500 returned items - therefore you must paginate the content to fetch
//! remaining items.
//!
//! ```no_run
//! # use dco3::{Dracoon, auth::OAuth2Flow, Nodes, ListAllParams};
//! # #[tokio::main]
//! # async fn main() {
//! # let dracoon = Dracoon::builder()
//! #  .with_base_url("https://dracoon.team")
//! #  .with_client_id("client_id")
//! #  .with_client_secret("client_secret")
//! #  .build()
//! #  .unwrap()
//! #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
//! #  .await
//! #  .unwrap();
//!

//! // This fetches the first 500 nodes without any param
//!  let mut nodes = dracoon.nodes().get_nodes(None, None, None).await.unwrap();
//!
//! // Iterate over the remaining nodes
//!  for offset in (0..nodes.range.total).step_by(500) {
//!  let params = ListAllParams::builder()
//!   .with_offset(offset)
//!   .build();
//!  let next_nodes = dracoon.nodes().get_nodes(None, None, Some(params)).await.unwrap();
//!  
//!   nodes.items.extend(next_nodes.items);
//!
//! };
//! # }
//! ```
//! ## Cryptography support
//! All API calls (specifically up- and downloads) support encryption and decryption.
//! In order to use encryption, you can pass the encryption password while building the client.
//!
//! ```no_run
//!  use dco3::{Dracoon, OAuth2Flow};
//!  #[tokio::main]
//!  async fn main() {
//!  let dracoon = Dracoon::builder()
//!   .with_base_url("https://dracoon.team")
//!   .with_client_id("client_id")
//!   .with_client_secret("client_secret")
//!   .with_encryption_password("my secret")
//!   .build()
//!   .unwrap()
//!   .connect(OAuth2Flow::password_flow("username", "password"))
//!   .await
//!   .unwrap();
//! // keypair is now set and can be fetched without passing a secret
//! let kp = dracoon.get_keypair(None).await.unwrap();
//! # }
//! ```
//!
//! It is also possible to pass the encryption secret after connecting by using the `get_keypair` method.
//!
//! ```no_run
//!  use dco3::{Dracoon, OAuth2Flow};
//!  #[tokio::main]
//!  async fn main() {
//!  let dracoon = Dracoon::builder()
//!   .with_base_url("https://dracoon.team")
//!   .with_client_id("client_id")
//!   .with_client_secret("client_secret")
//!   .build()
//!   .unwrap()
//!   .connect(OAuth2Flow::password_flow("username", "password"))
//!   .await
//!   .unwrap();
//! // check and provide the keypair by passing the encryption secret
//! let secret = "my secret".to_string();
//! let kp = dracoon.get_keypair(Some(secret)).await.unwrap();
//! # }
//! ```
//! ## Provisioning
//! In order to use the provisioning API to manage customers of a tenant, you can instantiate
//! a client with the `Provisioning` state.
//! All API calls are implemented in the [CustomerProvisioning] trait.
//!
//! ```no_run
//!
//! use dco3::{Dracoon, OAuth2Flow, CustomerProvisioning};
//!
//! #[tokio::main]
//! async fn main() {
//! // the client only requires passing the base url and a provisioning token
//! // other API calls are *not* supported in this state.
//! let dracoon = Dracoon::builder()
//!    .with_base_url("https://dracoon.team")
//!    .with_provisioning_token("some_token")
//!    .build_provisioning()
//!    .unwrap();
//!
//! // the client is now in the provisioning state and can be used to manage customers
//! let customers = dracoon.provisioning().get_customers(None).await.unwrap();
//!
//! }
//! ```
//!
//! ## Examples
//! For an example client implementation, see the [dccmd-rs](https://github.com/unbekanntes-pferd/dccmd-rs) repository.

use std::{marker::PhantomData, sync::Arc};

use client::{GetClient, Provisioning};
use config::ConfigEndpoint;
use dco3_crypto::PlainUserKeyPairContainer;
use eventlog::EventlogEndpoint;
use groups::GroupsEndpoint;
use nodes::NodesEndpoint;
use provisioning::ProvisioningEndpoint;
use public::{PublicEndpoint, SystemInfo};
use reqwest::Url;
use roles::RolesEndpoint;
use secrecy::{ExposeSecret, Secret};
use settings::SettingsEndpoint;
use shares::SharesEndpoint;
use system::SystemEndpoint;
use user::UserEndpoint;
use users::UsersEndpoint;

use self::{
    client::{Connected, Disconnected},
    client::{DracoonClient, DracoonClientBuilder},
    user::models::UserAccount,
};

// re-export traits and base models
pub use self::{
    client::errors::DracoonClientError,
    client::OAuth2Flow,
    config::Config,
    eventlog::Eventlog,
    groups::Groups,
    models::*,
    nodes::{Download, Folders, MissingFileKeys, Nodes, Rooms, Upload},
    provisioning::CustomerProvisioning,
    public::{Public, PublicDownload, PublicUpload},
    roles::Roles,
    settings::RescueKeyPair,
    shares::{DownloadShares, UploadShares},
    system::AuthenticationMethods,
    user::{User, UserAccountKeyPairs},
    users::Users,
};

pub mod client;
pub mod config;
pub mod constants;
pub mod eventlog;
pub mod groups;
pub mod models;
pub mod nodes;
pub mod provisioning;
pub mod public;
pub mod roles;
pub mod settings;
pub mod shares;
pub mod system;
mod tests;
pub mod user;
pub mod users;
pub mod utils;

/// DRACOON struct - implements all API calls via traits
#[derive(Clone)]
pub struct Dracoon<State = Disconnected> {
    client: Arc<DracoonClient<State>>,
    state: PhantomData<State>,
    user_info: Container<UserAccount>,
    keypair: Container<Secret<WrappedUserKeypair>>,
    system_info: Container<SystemInfo>,
    encryption_secret: Option<Secret<String>>,
    endpoints: Endpoints<State>,
}

impl<S: Send + Sync> GetClient<S> for Dracoon<S> {
    fn get_client(&self) -> &DracoonClient<S> {
        &self.client
    }
}

/// Builder for the `Dracoon` struct.
/// Requires a base url, client id and client secret.
/// Optionally, a redirect uri can be provided.
/// For convenience, use the [Dracoon] builder method.
#[derive(Default)]
pub struct DracoonBuilder {
    client_builder: DracoonClientBuilder,
    encryption_secret: Option<Secret<String>>,
}

impl DracoonBuilder {
    /// Creates a new `DracoonBuilder`
    pub fn new() -> Self {
        let client_builder = DracoonClientBuilder::new();
        Self {
            client_builder,
            encryption_secret: None,
        }
    }

    /// Sets the encryption password - it is *not* permanently stored in the client.
    /// The secret will be consumed, once a connection is tried to establish via the `connect` method.
    /// The client will then either fail to connect due to wrong encryption secret or permanently store
    /// a user's keypair.
    pub fn with_encryption_password(mut self, encryption_secret: impl Into<String>) -> Self {
        self.encryption_secret = Some(Secret::new(encryption_secret.into()));
        self
    }

    /// Sets the base url for the DRACOON instance
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.client_builder = self.client_builder.with_base_url(base_url);
        self
    }

    /// Sets the client id for the DRACOON instance
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_builder = self.client_builder.with_client_id(client_id);
        self
    }

    /// Sets the client secret for the DRACOON instance
    pub fn with_client_secret(mut self, client_secret: impl Into<String>) -> Self {
        self.client_builder = self.client_builder.with_client_secret(client_secret);
        self
    }

    /// Sets the redirect uri for the DRACOON instance
    pub fn with_redirect_uri(mut self, redirect_uri: impl Into<String>) -> Self {
        self.client_builder = self.client_builder.with_redirect_uri(redirect_uri);
        self
    }

    /// Sets a custom user agent prefix for the client
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.client_builder = self.client_builder.with_user_agent(user_agent);
        self
    }

    /// Sets a custom max. retry count (default: 5)
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.client_builder = self.client_builder.with_max_retries(max_retries);
        self
    }

    /// Sets a custom min. retry delay
    pub fn with_min_retry_delay(mut self, min_retry_delay: u64) -> Self {
        self.client_builder = self.client_builder.with_min_retry_delay(min_retry_delay);
        self
    }

    /// Sets a custom max. retry delay
    pub fn with_max_retry_delay(mut self, max_retry_delay: u64) -> Self {
        self.client_builder = self.client_builder.with_max_retry_delay(max_retry_delay);
        self
    }

    /// Sets X-SDS-Service-token for DRACOON customer provisioning
    pub fn with_provisioning_token(mut self, provisioning_token: impl Into<String>) -> Self {
        self.client_builder = self
            .client_builder
            .with_provisioning_token(provisioning_token);
        self
    }

    /// Sets the token rotation for the client (use amount of tokens per client)
    #[doc(hidden = "Experimental")]
    pub fn with_token_rotation(mut self, token_rotation: u8) -> Self {
        self.client_builder = self.client_builder.with_token_rotation(token_rotation);
        self
    }

    fn build_endpoints<S>(client: &Arc<DracoonClient<S>>) -> Endpoints<S> {
        client.into()
    }

    /// Builds the [Dracoon] struct - fails, if any of the required fields are missing
    pub fn build(self) -> Result<Dracoon<Disconnected>, DracoonClientError> {
        let dracoon = self.client_builder.build()?;
        let dracoon = Arc::new(dracoon);
        let endpoints = Self::build_endpoints(&dracoon);

        Ok(Dracoon {
            client: dracoon,
            state: PhantomData,
            user_info: Container::new(),
            keypair: Container::new(),
            system_info: Container::new(),
            encryption_secret: self.encryption_secret,
            endpoints,
        })
    }

    /// Builds the [Dracoon] struct set up for provisioning - fails if any of the required fields are missing
    pub fn build_provisioning(self) -> Result<Dracoon<Provisioning>, DracoonClientError> {
        let dracoon = self.client_builder.build_provisioning()?;
        let dracoon = Arc::new(dracoon);
        let endpoints = Self::build_endpoints(&dracoon);

        Ok(Dracoon {
            client: dracoon,
            state: PhantomData,
            user_info: Container::new(),
            keypair: Container::new(),
            system_info: Container::new(),
            encryption_secret: None,
            endpoints,
        })
    }
}

impl Dracoon<Disconnected> {
    pub fn builder() -> DracoonBuilder {
        DracoonBuilder::new()
    }

    pub async fn connect(
        self,
        oauth_flow: OAuth2Flow,
    ) -> Result<Dracoon<Connected>, DracoonClientError> {
        let client = self.client.connect(oauth_flow).await?;

        let connected_client = Arc::new(client);
        let endpoints = DracoonBuilder::build_endpoints(&connected_client);

        let mut dracoon = Dracoon {
            client: connected_client,
            state: PhantomData,
            user_info: Container::new(),
            keypair: Container::new(),
            system_info: Container::new(),
            encryption_secret: self.encryption_secret,
            endpoints,
        };

        if let Some(encryption_secret) = dracoon.encryption_secret.clone() {
            let kp = dracoon.user().get_user_keypair(encryption_secret.expose_secret()).await?;
            dracoon.encryption_secret = None;
            dracoon.keypair.set(Secret::new(WrappedUserKeypair::new(kp))).await;
            drop(encryption_secret)
        }

        Ok(dracoon)
    }

    pub fn get_authorize_url(&self) -> String {
        self.client.get_authorize_url()
    }
}

impl Dracoon<Connected> {
    pub async fn get_auth_header(&self) -> Result<String, DracoonClientError> {
        self.client.get_auth_header().await
    }

    pub fn get_base_url(&self) -> &Url {
        self.client.get_base_url()
    }

    pub async fn get_refresh_token(&self) -> String {
        self.client.get_refresh_token().await
    }

    pub async fn get_user_info(&self) -> Result<UserAccount, DracoonClientError> {
        if self.user_info.is_none().await {
            let user_info = self.user().get_user_account().await?;
            self.user_info.set(user_info).await;
        }

        let user_info = self.user_info.get().await.expect("No user info set");
        Ok(user_info)
    }

    pub async fn get_system_info(&self) -> Result<SystemInfo, DracoonClientError> {
        if self.system_info.is_none().await {
            let system_info = self.public().get_system_info().await?;
            self.system_info.set(system_info).await;
        }

        let system_info = self.system_info.get().await.expect("No system info set");

        Ok(system_info)
    }

    pub async fn get_keypair(
        &self,
        secret: Option<String>,
    ) -> Result<PlainUserKeyPairContainer, DracoonClientError> {
        if self.keypair.is_none().await {
            if let Some(secret) = secret {
                let keypair = self.user().get_user_keypair(&secret).await?;
                self.keypair.set(Secret::new(WrappedUserKeypair::new(keypair))).await;
            } else {
                return Err(DracoonClientError::MissingEncryptionSecret);
            }
        }

        let keypair = self.keypair.get().await.expect("No keypair set");
        Ok(keypair.expose_secret().keypair().clone())
    }
}

impl Dracoon<Provisioning> {
    pub fn get_service_token(&self) -> String {
        self.client.get_service_token()
    }
}

impl<S> Dracoon<S> {
    pub fn public(&self) -> &PublicEndpoint<S> {
        &self.endpoints.public
    }
}

impl<S: ConnectedClient> Dracoon<S> {
    pub fn build_api_url(&self, url_part: &str) -> Url {
        self.client
            .get_base_url()
            .join(url_part)
            .expect("Invalid base url")
    }

    /// Returns endpoint for all config routes `/api/v4/config`
    pub fn config(&self) -> &ConfigEndpoint<S> {
        &self.endpoints.config
    }

    /// Returns endpoint for all eventlog routes `/api/v4/eventlog`
    pub fn eventlog(&self) -> &EventlogEndpoint<S> {
        &self.endpoints.eventlog
    }

    /// Returns endpoint for all groups routes `/api/v4/groups`
    pub fn groups(&self) -> &GroupsEndpoint<S> {
        &self.endpoints.groups
    }

    /// Returns endpoint for all nodes routes `/api/v4/nodes`
    pub fn nodes(&self) -> &NodesEndpoint<S> {
        &self.endpoints.nodes
    }

    /// Returns endpoint for all roles routes `/api/v4/roles`
    pub fn roles(&self) -> &RolesEndpoint<S> {
        &self.endpoints.roles
    }

    /// Returns endpoint for all settings routes `/api/v4/settings`
    pub fn settings(&self) -> &SettingsEndpoint<S> {
        &self.endpoints.settings
    }

    /// Returns endpoint for all shares routes `/api/v4/shares`
    pub fn shares(&self) -> &SharesEndpoint<S> {
        &self.endpoints.shares
    }

    /// Returns endpoint for all system routes `/api/v4/system`
    pub fn system(&self) -> &SystemEndpoint<S> {
        &self.endpoints.system
    }

    /// Returns endpoint for all provisioning routes `/api/v4/provisioning`
    pub fn provisioning(&self) -> &ProvisioningEndpoint<S> {
        &self.endpoints.provisioning
    }

    /// Returns endpoint for all user routes `/api/v4/user`
    pub fn user(&self) -> &UserEndpoint<S> {
        &self.endpoints.user
    }

    /// Returns endpoint for all users routes `/api/v4/users`
    pub fn users(&self) -> &UsersEndpoint<S> {
        &self.endpoints.users
    }
}


pub mod auth {
    /// OAuth2 flow enum - used to pass the required flow to the client
    pub use crate::client::OAuth2Flow;
}