//! This module is responsible for the authentication with DRACOON and implements
//! the [DracoonClient] struct which is used to interact with the DRACOON API.
use jiff::Timestamp;
use reqwest::{Client, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use retry_policies::Jitter;
use secrecy::{ExposeSecret, SecretString};
use std::{marker::PhantomData, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing::{debug, error};

use base64::{engine::general_purpose, Engine};

mod auth;
pub mod errors;
pub mod models;

pub use models::*;

use crate::{
    constants::{
        DRACOON_TOKEN_REVOKE_URL, DRACOON_TOKEN_URL, MAX_RETRIES, MAX_RETRY_DELAY, MAX_TOKEN_COUNT,
        MIN_RETRY_DELAY, MIN_TOKEN_COUNT, TOKEN_EXPIRY_SKEW_SECS, TOKEN_TYPE_HINT_ACCESS_TOKEN,
    },
    models::Container,
    ConnectedClient,
};

use self::errors::DracoonClientError;
use super::constants::{APP_USER_AGENT, TOKEN_TYPE_HINT_REFRESH_TOKEN};

/// represents the possible `OAuth2` flows
pub enum OAuth2Flow {
    PasswordFlow(String, String),
    AuthCodeFlow(String),
    RefreshToken(String),
    Simple(String),
}

impl OAuth2Flow {
    pub fn authorization_code(code: impl Into<String>) -> Self {
        OAuth2Flow::AuthCodeFlow(code.into())
    }

    pub fn password_flow(username: impl Into<String>, password: impl Into<String>) -> Self {
        OAuth2Flow::PasswordFlow(username.into(), password.into())
    }

    pub fn refresh_token(refresh_token: impl Into<String>) -> Self {
        OAuth2Flow::RefreshToken(refresh_token.into())
    }

    pub fn simple(token: impl Into<String>) -> Self {
        OAuth2Flow::Simple(token.into())
    }
}

/// connected state of [DracoonClient]
#[derive(Debug, Clone)]
pub struct Connected;
/// disconnected state of [DracoonClient]
#[derive(Debug, Clone)]
pub struct Disconnected;

/// provisioning state of [DracoonClient]
#[derive(Debug, Clone)]
pub struct Provisioning;

impl ConnectedClient for Connected {}
impl ConnectedClient for Provisioning {}

/// represents a connection to DRACOON (`OAuth2` tokens)
#[derive(Debug, Clone)]
pub struct Connection {
    access_token: SecretString,
    refresh_token: SecretString,
    expires_in: u64,
    connected_at: Timestamp,
}

impl Connection {
    pub fn refresh_token(&self) -> &str {
        self.refresh_token.expose_secret()
    }

    pub fn access_token(&self) -> &str {
        self.access_token.expose_secret()
    }

    pub fn expires_in(&self) -> u64 {
        self.expires_in
    }

    pub fn connected_at(&self) -> Timestamp {
        self.connected_at
    }

    /// Returns true if the access token is expired or about to expire.
    ///
    /// A token is treated as expired [`TOKEN_EXPIRY_SKEW_SECS`] early (at most half of its lifetime),
    /// so that it does not expire while a request (including retries) is in flight.
    pub fn is_expired(&self) -> bool {
        self.is_expired_at(Timestamp::now())
    }

    fn is_expired_at(&self, now: Timestamp) -> bool {
        // this handles OAuth2Flow::Simple (expires_in is not known)
        // the access token is valid or fails with 401
        if self.expires_in == u64::MAX {
            return false;
        }

        let skew = TOKEN_EXPIRY_SKEW_SECS.min(self.expires_in / 2);
        let valid_for = i64::try_from(self.expires_in - skew).unwrap_or(i64::MAX);

        match self
            .connected_at
            .checked_add(jiff::SignedDuration::from_secs(valid_for))
        {
            Ok(expires_at) => now >= expires_at,
            // lifetime too large to represent - treat as not expiring
            Err(_) => false,
        }
    }

    pub fn update_tokens(&mut self, connection: Connection) {
        self.access_token = connection.access_token;
        self.refresh_token = connection.refresh_token;
        self.expires_in = connection.expires_in;
        self.connected_at = connection.connected_at;
    }

    pub fn new_from_access_token(access_token: String) -> Self {
        Self {
            access_token: SecretString::from(access_token),
            refresh_token: SecretString::from(String::new()),
            expires_in: u64::MAX,
            connected_at: Timestamp::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CurrentConnection {
    Main,
    Additional(u8),
}

#[derive(Clone)]
/// represents the DRACOON client (stateful)
pub struct DracoonClient<State = Disconnected> {
    base_url: Url,
    redirect_uri: Option<Url>,
    client_id: String,
    client_secret: SecretString,
    /// HTTP client for API calls (retries; when connected, also refreshes stale tokens per attempt)
    pub http: ClientWithMiddleware,
    pub stream_http: Client,
    /// HTTP client for `OAuth2` token calls (retries only, no token handling)
    token_http: ClientWithMiddleware,
    /// serializes token refreshes, so concurrent requests trigger a single refresh
    refresh_lock: Arc<Mutex<()>>,
    connection: Container<Connection>,
    token_rotation: Option<u8>,
    additional_connections: Container<Vec<Connection>>,
    curr_connection: Container<CurrentConnection>,
    state: PhantomData<State>,
    provisioning_token: Option<SecretString>,
}

/// Builder for the [DracoonClient] struct.
#[derive(Default)]
pub struct DracoonClientBuilder {
    base_url: Option<String>,
    redirect_uri: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    user_agent: Option<String>,
    max_retries: Option<u32>,
    min_retry_delay: Option<u64>,
    max_retry_delay: Option<u64>,
    token_rotation: Option<u8>,
    provisioning_token: Option<String>,
}

impl DracoonClientBuilder {
    /// Creates a new [DracoonClientBuilder]
    pub fn new() -> Self {
        Self {
            base_url: None,
            redirect_uri: None,
            client_id: None,
            client_secret: None,
            user_agent: None,
            max_retries: None,
            min_retry_delay: None,
            max_retry_delay: None,
            provisioning_token: None,
            token_rotation: None,
        }
    }

    /// Sets the base url for the DRACOON instance
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Sets the redirect uri for the OAuth2 flow (required for the auth code flow)
    pub fn with_redirect_uri(mut self, redirect_uri: impl Into<String>) -> Self {
        self.redirect_uri = Some(redirect_uri.into());
        self
    }

    /// Sets the client id for the OAuth2 flow
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    /// Sets the client secret for the OAuth2 flow
    pub fn with_client_secret(mut self, client_secret: impl Into<String>) -> Self {
        self.client_secret = Some(client_secret.into());
        self
    }

    /// Sets the user agent (custom string)
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Sets max retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Sets min retry delay
    pub fn with_min_retry_delay(mut self, min_retry_delay: u64) -> Self {
        self.min_retry_delay = Some(min_retry_delay);
        self
    }

    /// Sets max retry delay
    pub fn with_max_retry_delay(mut self, max_retry_delay: u64) -> Self {
        self.max_retry_delay = Some(max_retry_delay);
        self
    }

    /// Sets the provisioning token for the provisioning API
    pub fn with_provisioning_token(mut self, token: impl Into<String>) -> Self {
        self.provisioning_token = Some(token.into());
        self
    }

    #[doc(hidden)]
    pub fn with_token_rotation(mut self, token_rotation: u8) -> Self {
        self.token_rotation = Some(token_rotation);
        self
    }

    /// Builds reqwest clients with configured middleware
    fn build_clients(&self) -> Result<(ClientWithMiddleware, Client), DracoonClientError> {
        let max_retries = self
            .max_retries
            .unwrap_or(MAX_RETRIES)
            .clamp(1, MAX_RETRIES);
        let min_retry_delay = self
            .min_retry_delay
            .unwrap_or(MIN_RETRY_DELAY)
            .clamp(300, MIN_RETRY_DELAY);
        let max_retry_delay = self
            .max_retry_delay
            .unwrap_or(MAX_RETRY_DELAY)
            .clamp(min_retry_delay, MAX_RETRY_DELAY);

        let retry_policy: ExponentialBackoff = ExponentialBackoff::builder()
            .jitter(Jitter::Bounded)
            .retry_bounds(
                Duration::from_millis(min_retry_delay),
                Duration::from_millis(max_retry_delay),
            )
            .build_with_max_retries(max_retries);

        let user_agent = match &self.user_agent {
            Some(user_agent) => format!("{}|{}", user_agent, APP_USER_AGENT),
            None => APP_USER_AGENT.to_string(),
        };

        let http = Client::builder().user_agent(user_agent).build()?;
        let upload_http = http.clone();

        let http = ClientBuilder::new(http)
            .with(RetryTransientMiddleware::new_with_policy_and_strategy(
                retry_policy,
                DracoonCustomRetryStrategy,
            ))
            .build();

        Ok((http, upload_http))
    }
    /// Builds the [DracoonClient] struct for the provisioning API
    pub fn build_provisioning(self) -> Result<DracoonClient<Provisioning>, DracoonClientError> {
        let Some(ref provisioning_token) = self.provisioning_token else {
            return Err(DracoonClientError::MissingArgument);
        };

        let Some(base_url) = self.base_url.clone() else {
            error!("Missing base url");
            return Err(DracoonClientError::MissingBaseUrl);
        };

        let base_url = Url::parse(&base_url)?;

        let (http, upload_http) = self.build_clients()?;

        Ok(DracoonClient {
            base_url,
            redirect_uri: None,
            client_id: String::new(),
            client_secret: SecretString::from(String::new()),
            token_http: http.clone(),
            http,
            stream_http: upload_http,
            refresh_lock: Arc::new(Mutex::new(())),
            state: PhantomData,
            connection: Container::new(),
            additional_connections: Container::new(),
            token_rotation: None,
            curr_connection: Container::new(),
            provisioning_token: Some(SecretString::from(provisioning_token.to_string())),
        })
    }

    /// Builds the [DracoonClient] struct - returns an error if any of the required fields are missing
    pub fn build(self) -> Result<DracoonClient<Disconnected>, DracoonClientError> {
        let (http, upload_http) = self.build_clients()?;

        let Some(base_url) = self.base_url.clone() else {
            error!("Missing base url");
            return Err(DracoonClientError::MissingBaseUrl);
        };

        let base_url = Url::parse(&base_url)?;

        let Some(client_id) = self.client_id else {
            error!("Missing client id");
            return Err(DracoonClientError::MissingClientId);
        };

        let Some(client_secret) = self.client_secret else {
            error!("Missing client secret");
            return Err(DracoonClientError::MissingClientSecret);
        };

        let redirect_uri = match self.redirect_uri {
            Some(url) => Url::parse(&url)?,
            None => Url::parse(&format!(
                "{}/oauth/callback",
                self.base_url.expect("missing base url already checked")
            ))?,
        };

        let token_rotation = self
            .token_rotation
            .unwrap_or(MIN_TOKEN_COUNT)
            .clamp(MIN_TOKEN_COUNT, MAX_TOKEN_COUNT);

        let token_rotation = if token_rotation > 1 {
            Some(token_rotation)
        } else {
            None
        };

        Ok(DracoonClient {
            base_url,
            redirect_uri: Some(redirect_uri),
            client_id,
            client_secret: SecretString::from(client_secret),
            connection: Container::<Connection>::new(),
            additional_connections: Container::new(),
            token_rotation,
            state: PhantomData,
            token_http: http.clone(),
            http,
            refresh_lock: Arc::new(Mutex::new(())),
            curr_connection: Container::new_from(CurrentConnection::Main),
            stream_http: upload_http,
            provisioning_token: None,
        })
    }
}

/// [DracoonClient] implementation for Disconnected state
impl DracoonClient<Disconnected> {
    pub fn builder() -> DracoonClientBuilder {
        DracoonClientBuilder::new()
    }

    /// Connects to DRACOON using any of the supported OAuth2 flows
    pub async fn connect(
        &self,
        oauth_flow: OAuth2Flow,
    ) -> Result<DracoonClient<Connected>, DracoonClientError> {
        let connection = match oauth_flow {
            OAuth2Flow::PasswordFlow(username, password) => {
                debug!("Connecting with password flow");
                self.connect_password_flow(&username, &password).await?
            }
            OAuth2Flow::AuthCodeFlow(code) => {
                debug!("Connecting with auth code flow");
                self.connect_authcode_flow(&code).await?
            }
            OAuth2Flow::RefreshToken(token) => {
                debug!("Connecting with refresh token flow");
                self.connect_refresh_token(&token).await?
            }
            OAuth2Flow::Simple(token) => Connection::new_from_access_token(token),
        };

        if let Some(token_rotation) = self.token_rotation {
            let mut additional_connections = Vec::new();
            for _ in 0..token_rotation - 1 {
                let new_connection = self
                    .connect_refresh_token(connection.refresh_token.expose_secret())
                    .await?;
                additional_connections.push(new_connection);
            }

            self.additional_connections
                .set(additional_connections)
                .await;
        }

        let connected = DracoonClient {
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            connection: Container::new_from(connection),
            additional_connections: self.additional_connections.clone(),
            token_rotation: self.token_rotation,
            curr_connection: self.curr_connection.clone(),
            base_url: self.base_url.clone(),
            redirect_uri: self.redirect_uri.clone(),
            state: PhantomData,
            http: self.token_http.clone(),
            token_http: self.token_http.clone(),
            stream_http: self.stream_http.clone(),
            refresh_lock: Arc::new(Mutex::new(())),
            provisioning_token: None,
        };

        // API calls get a middleware *inside* the retry middleware, which replaces a stale
        // access token before every attempt (the token state is shared via `Container`s).
        // The middleware's own client only uses `token_http`, so there is no reference cycle.
        let http = reqwest_middleware::ClientBuilder::from_client(self.token_http.clone())
            .with(auth::RefreshAuthHeader::new(Arc::new(connected.clone())))
            .build();

        Ok(DracoonClient { http, ..connected })
    }

    /// returns client id and client secret base64 encoded for the basic auth header
    fn client_credentials(&self) -> String {
        // RFC 7617: standard base64 alphabet with padding
        let client_credentials =
            format!("{}:{}", self.client_id, self.client_secret.expose_secret());

        general_purpose::STANDARD.encode(client_credentials)
    }

    /// Returns the authorize url for the OAuth2 auth code flow
    pub fn get_authorize_url(&self) -> String {
        let default_redirect = self
            .base_url
            .join("oauth/callback")
            .expect("Base url cannot be parsed");
        let redirect_uri = self
            .redirect_uri
            .as_ref()
            .unwrap_or(&default_redirect)
            .clone();

        let mut authorize_url = self
            .base_url
            .join("oauth/authorize")
            .expect("Base url cannot be parsed");
        let authorize_url = authorize_url
            .query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", redirect_uri.as_ref())
            .append_pair("scope", "all")
            .finish();

        authorize_url.to_string()
    }

    /// Returns the token url for any OAuth2 flow
    fn get_token_url(&self) -> Url {
        self.base_url
            .join(DRACOON_TOKEN_URL)
            .expect("Base url cannot be parsed")
    }

    /// Connects to DRACOON using the password flow
    async fn connect_password_flow(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Connection, DracoonClientError> {
        let token_url = self.get_token_url();

        let auth = OAuth2PasswordFlow::new(username, password);
        let auth_header = format!("Basic {}", self.client_credentials());

        let res = self
            .token_http
            .post(token_url)
            .header("Authorization", auth_header)
            .form(&auth)
            .send()
            .await
            .map_err(|err| {
                error!("Error connecting with password flow: {}", err);
                err
            })?;
        Ok(OAuth2TokenResponse::from_response(res).await?.into())
    }

    /// Connects to DRACOON using the auth code flow
    async fn connect_authcode_flow(&self, code: &str) -> Result<Connection, DracoonClientError> {
        let token_url = self.get_token_url();

        let auth = OAuth2AuthCodeFlow::new(
            &self.client_id,
            self.client_secret.expose_secret(),
            code,
            self.redirect_uri
                .as_ref()
                .expect("redirect uri is set")
                .as_str(),
        );

        let res = self
            .token_http
            .post(token_url)
            .form(&auth)
            .send()
            .await
            .map_err(|err| {
                error!("Error connecting with auth code flow: {}", err);
                err
            })?;

        drop(auth);
        Ok(OAuth2TokenResponse::from_response(res).await?.into())
    }

    /// Connects to DRACOON using the refresh token flow
    async fn connect_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<Connection, DracoonClientError> {
        let token_url = self.get_token_url();

        let auth = OAuth2RefreshTokenFlow::new(
            &self.client_id,
            self.client_secret.expose_secret(),
            refresh_token,
        );

        let res = self
            .token_http
            .post(token_url)
            .form(&auth)
            .send()
            .await
            .map_err(|err| {
                error!("Error connecting with refresh token flow: {}", err);
                err
            })?;
        Ok(OAuth2TokenResponse::from_response(res).await?.into())
    }
}

/// Maps a failed token revoke response to an error
async fn check_revoke_response(res: reqwest::Response) -> Result<(), DracoonClientError> {
    let status = res.status();
    if status.is_success() {
        return Ok(());
    }

    match res.json::<DracoonAuthErrorResponse>().await {
        Ok(error) => Err(error.into()),
        Err(err) => {
            error!("Failed to parse revoke error body ({}): {}", status, err);
            Err(DracoonClientError::Http(crate::utils::fallback_http_error(
                status,
                "failed to parse error body",
            )))
        }
    }
}

/// `DracoonClient` implementation for Connected state
impl DracoonClient<Connected> {
    /// disconnects the client and optionally revokes the access and refresh token
    /// access token is revoked by default, refresh token is *not* revoked by default
    pub async fn disconnect(
        self,
        revoke_access_token: Option<bool>,
        revoke_refresh_token: Option<bool>,
    ) -> Result<DracoonClient<Disconnected>, DracoonClientError> {
        let revoke_access_token = revoke_access_token.unwrap_or(true);
        let revoke_refresh_token = revoke_refresh_token.unwrap_or(false);

        if revoke_access_token {
            debug!("Revoking access token");
            self.revoke_acess_token().await?;
        }

        if revoke_refresh_token {
            debug!("Revoking refresh token");
            self.revoke_refresh_token().await?;
        }

        Ok(DracoonClient {
            client_id: self.client_id,
            client_secret: self.client_secret,
            connection: Container::<Connection>::new(),
            additional_connections: Container::new(),
            token_rotation: self.token_rotation,
            curr_connection: Container::new_from(CurrentConnection::Main),
            base_url: self.base_url,
            redirect_uri: self.redirect_uri,
            state: PhantomData,
            http: self.token_http.clone(),
            token_http: self.token_http,
            stream_http: self.stream_http,
            refresh_lock: Arc::new(Mutex::new(())),
            provisioning_token: None,
        })
    }

    /// Returns the token url for any OAuth2 flow
    fn get_token_url(&self) -> Url {
        self.base_url
            .join(DRACOON_TOKEN_URL)
            .expect("Base url cannot be parsed")
    }

    /// Revokes the access token
    async fn revoke_acess_token(&self) -> Result<(), DracoonClientError> {
        let access_token = self
            .connection
            .get()
            .await
            .expect("Connected client has no connection")
            .access_token()
            .to_string();

        let api_url = self
            .base_url
            .join(DRACOON_TOKEN_REVOKE_URL)
            .expect("Base url cannot be parsed");

        let auth = OAuth2TokenRevoke::new(
            &self.client_id,
            self.client_secret.expose_secret(),
            TOKEN_TYPE_HINT_ACCESS_TOKEN,
            &access_token,
        );

        let res = self.token_http.post(api_url).form(&auth).send().await?;

        check_revoke_response(res).await
    }

    /// Revokes the refresh token
    async fn revoke_refresh_token(&self) -> Result<(), DracoonClientError> {
        let refresh_token = self
            .connection
            .get()
            .await
            .expect("Connected client has no connection")
            .refresh_token()
            .to_string();

        let api_url = self
            .base_url
            .join(DRACOON_TOKEN_REVOKE_URL)
            .expect("Correct base url");

        let auth = OAuth2TokenRevoke::new(
            &self.client_id,
            self.client_secret.expose_secret(),
            TOKEN_TYPE_HINT_REFRESH_TOKEN,
            &refresh_token,
        );

        let res = self.token_http.post(api_url).form(&auth).send().await?;

        check_revoke_response(res).await
    }

    /// Fetches new tokens using available refresh token from the current connection
    async fn connect_refresh_token(&self) -> Result<Connection, DracoonClientError> {
        let token_url = self.get_token_url();

        let refresh_token = self
            .connection
            .get()
            .await
            .expect("Connected client has no connection")
            .refresh_token
            .clone();

        // this happens for OAuth2Flow::Simple (no refresh token provided)
        if refresh_token.expose_secret().is_empty() {
            return Err(DracoonClientError::Auth(
                DracoonAuthErrorResponse::new_unauthorized(),
            ));
        }

        let auth = OAuth2RefreshTokenFlow::new(
            &self.client_id,
            self.client_secret.expose_secret(),
            refresh_token.expose_secret(),
        );

        let res = self.token_http.post(token_url).form(&auth).send().await?;
        Ok(OAuth2TokenResponse::from_response(res).await?.into())
    }

    /// Returns the necessary token header for any API call that requires authentication in DRACOON
    ///
    /// Expired (or soon expiring) tokens are refreshed first. Concurrent callers share a single refresh.
    pub async fn get_auth_header(&self) -> Result<String, DracoonClientError> {
        let slot = self
            .curr_connection
            .get()
            .await
            .unwrap_or(CurrentConnection::Main);
        let connection = self.connection_in_slot(&slot).await;

        if connection.is_expired() {
            // no need to rotate, there's a new access token
            let connection = self.refresh_connection_in_slot(&slot).await?;
            return Ok(format!("Bearer {}", connection.access_token()));
        }

        if let Some(token_rotation) = self.token_rotation {
            // rotate the connection
            let next_connection = match slot {
                CurrentConnection::Main => CurrentConnection::Additional(0),
                CurrentConnection::Additional(idx) => {
                    if idx + 1 < token_rotation - 1 {
                        CurrentConnection::Additional(idx + 1)
                    } else {
                        CurrentConnection::Main
                    }
                }
            };

            self.curr_connection.set(next_connection).await;
        }

        Ok(format!("Bearer {}", connection.access_token()))
    }

    /// Returns the connection stored in the given rotation slot
    async fn connection_in_slot(&self, slot: &CurrentConnection) -> Connection {
        match slot {
            CurrentConnection::Main => self
                .connection
                .get()
                .await
                .expect("Connected client has no connection"),
            CurrentConnection::Additional(idx) => self
                .additional_connections
                .get()
                .await
                .expect("Connected client has no additional connections")
                .get(*idx as usize)
                .expect("Invalid connection index")
                .clone(),
        }
    }

    /// Refreshes the connection in the given slot - unless another task refreshed it in the meantime
    async fn refresh_connection_in_slot(
        &self,
        slot: &CurrentConnection,
    ) -> Result<Connection, DracoonClientError> {
        let _guard = self.refresh_lock.lock().await;

        // double check: a concurrent caller may have refreshed while we waited for the lock
        let current = self.connection_in_slot(slot).await;
        if !current.is_expired() {
            return Ok(current);
        }

        debug!("Access token expired - refreshing");
        let new_connection = self.connect_refresh_token().await?;

        match slot {
            CurrentConnection::Main => self.connection.set(new_connection.clone()).await,
            CurrentConnection::Additional(idx) => {
                let mut additional_connections = self
                    .additional_connections
                    .get()
                    .await
                    .expect("Connected client has no additional connections");
                additional_connections[*idx as usize] = new_connection.clone();
                self.additional_connections
                    .set(additional_connections)
                    .await;
            }
        }

        Ok(new_connection)
    }

    /// Checks whether an `Authorization` header still carries a valid access token of this client.
    /// Returns false if the token was replaced by a refresh or is (about to be) expired.
    /// Headers that do not carry a bearer token are left alone (returns true).
    pub(crate) async fn is_auth_header_current(&self, auth_header: &str) -> bool {
        let Some(token) = auth_header.strip_prefix("Bearer ") else {
            return true;
        };

        let main = self.connection.get().await;
        let additional = self.additional_connections.get().await.unwrap_or_default();

        main.into_iter()
            .chain(additional)
            .find(|connection| connection.access_token() == token)
            .is_some_and(|connection| !connection.is_expired())
    }

    /// Returns the refresh token
    pub async fn get_refresh_token(&self) -> String {
        self.connection
            .get()
            .await
            .expect("Connected client has no connection")
            .refresh_token()
            .to_string()
    }
}

impl DracoonClient<Provisioning> {
    /// Returns the X-SDS-Service-Token for provisioning API calls
    pub fn get_service_token(&self) -> String {
        self.provisioning_token
            .as_ref()
            .expect("Provisioning client has no token")
            .expose_secret()
            .to_string()
    }
}

impl<S> DracoonClient<S> {
    /// Returns the base url of the DRACOON instance
    pub fn get_base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn build_api_url(&self, url_part: &str) -> Url {
        self.base_url
            .join(url_part)
            .expect("Invalid base url or url part")
    }
}

#[cfg(test)]
mod tests {

    use reqwest::header::USER_AGENT;

    use super::*;

    fn token_response(token: &str) -> String {
        include_str!("tests/auth_ok_placeholder.json").replace("$token", token)
    }

    /// a connection with `token` that (incl. skew) expires `expires_in_ms` from now
    fn connection_expiring_in(token: &str, expires_in_ms: i64) -> Connection {
        let lifetime = 3600;
        let valid_for_ms = (lifetime - TOKEN_EXPIRY_SKEW_SECS) as i64 * 1000;
        Connection {
            access_token: SecretString::from(token.to_string()),
            refresh_token: SecretString::from("refresh_token".to_string()),
            expires_in: lifetime,
            connected_at: Timestamp::now()
                - jiff::SignedDuration::from_millis(valid_for_ms - expires_in_ms),
        }
    }

    async fn connect_with_token(
        mock_server: &mut mockito::ServerGuard,
        token: &str,
    ) -> DracoonClient<Connected> {
        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(token_response(token))
            .create_async()
            .await;

        let client = get_test_client(&mock_server.url())
            .connect(OAuth2Flow::refresh_token("refresh_token"))
            .await
            .expect("connect");
        auth_mock.remove_async().await;
        client
    }

    #[tokio::test]
    async fn test_password_flow_uses_standard_base64_basic_auth() {
        let mut mock_server = mockito::Server::new_async().await;

        // "client_id:x?~" encodes to a '/' and needs padding in standard base64
        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .match_header("authorization", "Basic Y2xpZW50X2lkOng/fg==")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(token_response("token1"))
            .expect(1)
            .create_async()
            .await;

        let client = DracoonClientBuilder::new()
            .with_base_url(mock_server.url())
            .with_client_id("client_id")
            .with_client_secret("x?~")
            .with_max_retries(1)
            .build()
            .unwrap();

        let res = client
            .connect(OAuth2Flow::password_flow("username", "password"))
            .await;

        auth_mock.assert_async().await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_disconnect_returns_error_if_revoke_fails() {
        let mut mock_server = mockito::Server::new_async().await;
        let client = connect_with_token(&mut mock_server, "token1").await;

        let revoke_mock = mock_server
            .mock("POST", "/oauth/revoke")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(include_str!("tests/auth_error.json"))
            .create_async()
            .await;

        let res = client.disconnect(None, None).await;

        revoke_mock.assert_async().await;
        let Err(err) = res else {
            panic!("expected revoke error");
        };
        assert!(err.is_auth_error(), "unexpected error: {err:?}");
    }

    #[tokio::test]
    async fn test_password_flow_without_refresh_token() {
        // e.g. the dracoon_legacy_scripting client issues no refresh token (null or missing)
        for body in [
            r#"{"access_token": "token1", "refresh_token": null, "token_type": "bearer", "expires_in": 28800}"#,
            r#"{"access_token": "token1", "token_type": "bearer", "expires_in": 28800}"#,
        ] {
            let mut mock_server = mockito::Server::new_async().await;
            let auth_mock = mock_server
                .mock("POST", "/oauth/token")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(body)
                .expect(1)
                .create_async()
                .await;

            let client = get_test_client(&mock_server.url())
                .connect(OAuth2Flow::password_flow("username", "password"))
                .await
                .expect("connect without refresh token");

            assert_eq!(client.get_auth_header().await.unwrap(), "Bearer token1");
            assert_eq!(client.get_refresh_token().await, "");

            // once expired, there is nothing to refresh with: auth error, no token request
            client
                .connection
                .set(Connection {
                    refresh_token: SecretString::from(String::new()),
                    ..connection_expiring_in("token1", -1_000)
                })
                .await;
            let err = client.get_auth_header().await.unwrap_err();
            assert!(err.is_auth_error(), "unexpected error: {err:?}");
            auth_mock.assert_async().await;
        }
    }

    #[test]
    fn test_connection_is_expired_with_skew() {
        let connected_at = Timestamp::now();
        let connection = Connection {
            access_token: SecretString::from("a".to_string()),
            refresh_token: SecretString::from("r".to_string()),
            expires_in: 3600,
            connected_at,
        };
        let at = |secs: i64| connected_at + jiff::SignedDuration::from_secs(secs);

        assert!(!connection.is_expired_at(at(0)));
        assert!(!connection.is_expired_at(at(3539)));
        // expired 60 seconds before the token actually expires
        assert!(connection.is_expired_at(at(3540)));
        assert!(connection.is_expired_at(at(3601)));

        // short-lived tokens: skew is capped at half the lifetime
        let short = Connection {
            expires_in: 60,
            ..connection.clone()
        };
        assert!(!short.is_expired_at(at(29)));
        assert!(short.is_expired_at(at(30)));

        // simple flow (unknown lifetime) and absurd lifetimes never expire / never panic
        let simple = Connection::new_from_access_token("a".to_string());
        assert!(!simple.is_expired());
        let huge = Connection {
            expires_in: u64::MAX - 1,
            ..connection
        };
        assert!(!huge.is_expired_at(at(1)));
    }

    #[tokio::test]
    async fn test_get_auth_header_refreshes_token_about_to_expire() {
        let mut mock_server = mockito::Server::new_async().await;
        let client = connect_with_token(&mut mock_server, "token1").await;

        // still valid for 30 seconds - inside the skew window
        client
            .connection
            .set(connection_expiring_in("token1", -30_000))
            .await;

        let refresh_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(token_response("token2"))
            .expect(1)
            .create_async()
            .await;

        let header = client.get_auth_header().await.unwrap();

        refresh_mock.assert_async().await;
        assert_eq!(header, "Bearer token2");
    }

    #[tokio::test]
    async fn test_get_auth_header_concurrent_callers_share_one_refresh() {
        let mut mock_server = mockito::Server::new_async().await;
        let client = Arc::new(connect_with_token(&mut mock_server, "token1").await);

        client
            .connection
            .set(connection_expiring_in("token1", -1_000))
            .await;

        let refresh_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(token_response("token2"))
            .expect(1)
            .create_async()
            .await;

        let calls = (0..10).map(|_| {
            let client = Arc::clone(&client);
            tokio::spawn(async move { client.get_auth_header().await })
        });
        let headers = futures_util::future::join_all(calls).await;

        refresh_mock.assert_async().await;
        for header in headers {
            assert_eq!(header.unwrap().unwrap(), "Bearer token2");
        }
    }

    #[tokio::test]
    async fn test_stale_token_is_replaced_before_sending() {
        let mut mock_server = mockito::Server::new_async().await;
        let client = connect_with_token(&mut mock_server, "token1").await;

        // the caller built the request with token1, which has expired since
        client
            .connection
            .set(connection_expiring_in("token1", -1_000))
            .await;

        let refresh_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(token_response("token2"))
            .expect(1)
            .create_async()
            .await;
        let api_mock = mock_server
            .mock("GET", "/api/v4/user/account")
            .match_header("authorization", "Bearer token2")
            .with_status(200)
            .expect(1)
            .create_async()
            .await;

        let res = client
            .http
            .get(client.build_api_url("api/v4/user/account"))
            .header(reqwest::header::AUTHORIZATION, "Bearer token1")
            .send()
            .await
            .unwrap();

        assert!(res.status().is_success());
        refresh_mock.assert_async().await;
        api_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_retry_after_401_uses_token_refreshed_in_between() {
        let mut mock_server = mockito::Server::new_async().await;
        let client = connect_with_token(&mut mock_server, "token1").await;

        // token1 is valid for the first attempt, but expires before the retry (min delay 300 ms)
        client
            .connection
            .set(connection_expiring_in("token1", 150))
            .await;
        let header = client.get_auth_header().await.unwrap();
        assert_eq!(header, "Bearer token1");

        let refresh_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(token_response("token2"))
            .expect(1)
            .create_async()
            .await;
        let gateway_401 = mock_server
            .mock("GET", "/api/v4/user/account")
            .match_header("authorization", "Bearer token1")
            .with_status(401)
            .expect(1)
            .create_async()
            .await;
        let api_ok = mock_server
            .mock("GET", "/api/v4/user/account")
            .match_header("authorization", "Bearer token2")
            .with_status(200)
            .expect(1)
            .create_async()
            .await;

        let res = client
            .http
            .get(client.build_api_url("api/v4/user/account"))
            .header(reqwest::header::AUTHORIZATION, header)
            .send()
            .await
            .unwrap();

        assert!(res.status().is_success());
        gateway_401.assert_async().await;
        refresh_mock.assert_async().await;
        api_ok.assert_async().await;
    }

    #[tokio::test]
    async fn test_connection_error_is_not_reported_as_middleware_error() {
        // nothing listens on port 1
        let client = get_test_client("http://127.0.0.1:1");

        let Err(err) = client
            .connect(OAuth2Flow::refresh_token("refresh_token"))
            .await
        else {
            panic!("expected connection error");
        };

        assert_eq!(
            err,
            DracoonClientError::ConnectionFailed("Connection failed".into())
        );
    }

    #[tokio::test]
    async fn test_failed_refresh_in_middleware_returns_auth_error() {
        let mut mock_server = mockito::Server::new_async().await;
        let client = connect_with_token(&mut mock_server, "token1").await;
        client
            .connection
            .set(connection_expiring_in("token1", -1_000))
            .await;

        let _refresh_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(include_str!("tests/auth_error.json"))
            .create_async()
            .await;

        let err = client
            .http
            .get(client.build_api_url("api/v4/user/account"))
            .header(reqwest::header::AUTHORIZATION, "Bearer token1")
            .send()
            .await
            .map_err(DracoonClientError::from)
            .unwrap_err();

        assert!(err.is_auth_error(), "unexpected error: {err:?}");
    }

    fn get_test_client(url: &str) -> DracoonClient<Disconnected> {
        DracoonClientBuilder::new()
            .with_base_url(url)
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .with_max_retries(1)
            .with_max_retry_delay(600)
            .with_min_retry_delay(300)
            .build()
            .expect("valid client config")
    }

    fn get_test_client_with_token_rotation(url: &str, rotation: u8) -> DracoonClient<Disconnected> {
        DracoonClientBuilder::new()
            .with_base_url(url)
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .with_user_agent("test_client")
            .with_max_retries(1)
            .with_max_retry_delay(600)
            .with_min_retry_delay(300)
            .with_token_rotation(rotation)
            .build()
            .expect("valid client config")
    }

    struct TokenGenrator {
        tokens: [&'static str; 5],
        idx: usize,
    }

    impl TokenGenrator {
        pub fn new() -> Self {
            TokenGenrator {
                tokens: ["token1", "token2", "token3", "token4", "token5"],
                idx: 0,
            }
        }
        pub fn get_token(&mut self) -> &'static str {
            let token = self.tokens[self.idx];
            self.idx = (self.idx + 1) % 5;
            token
        }

        pub fn get_auth_response(&mut self) -> String {
            let auth_res = include_str!("tests/auth_ok_placeholder.json");
            auth_res.replace("$token", self.get_token())
        }
    }

    #[tokio::test]
    async fn test_auth_code_authentication() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let auth_res = include_str!("./tests/auth_ok.json");

        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(auth_res)
            .create();

        let dracoon = DracoonClientBuilder::new()
            .with_base_url(base_url)
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .expect("valid client config");

        let auth_code = OAuth2Flow::AuthCodeFlow("hello world".to_string());

        let res = dracoon.connect(auth_code).await;

        auth_mock.assert();
        assert!(&res.is_ok());

        assert!(res.unwrap().connection.is_some().await);
    }

    #[tokio::test]
    async fn test_refresh_token_authentication() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let auth_res = include_str!("./tests/auth_ok.json");

        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(auth_res)
            .create();

        let dracoon = get_test_client(base_url.as_str());

        let refresh_token_auth = OAuth2Flow::RefreshToken("hello world".to_string());

        let res = dracoon.connect(refresh_token_auth).await;

        auth_mock.assert();
        assert!(&res.is_ok());

        assert!(res.as_ref().unwrap().connection.is_some().await);

        let access_token = res
            .as_ref()
            .unwrap()
            .connection
            .get()
            .await
            .unwrap()
            .access_token()
            .to_string();

        let refresh_token = res
            .as_ref()
            .unwrap()
            .connection
            .get()
            .await
            .unwrap()
            .refresh_token()
            .to_string();

        let expires_in = res
            .as_ref()
            .unwrap()
            .connection
            .get()
            .await
            .unwrap()
            .expires_in();

        assert_eq!(access_token, "access_token");
        assert_eq!(refresh_token, "refresh_token");
        assert_eq!(expires_in, 3600);
    }

    #[tokio::test]
    async fn test_auth_error_handling() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let auth_res = include_str!("./tests/auth_error.json");

        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(auth_res)
            .create();

        let dracoon = get_test_client(base_url.as_str());

        let auth_code = OAuth2Flow::AuthCodeFlow("hello world".to_string());

        let res = dracoon.connect(auth_code).await;

        auth_mock.assert();

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_get_auth_header() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let auth_res = include_str!("./tests/auth_ok.json");

        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(auth_res)
            .create();

        let dracoon = get_test_client(base_url.as_str());
        let refresh_token_auth = OAuth2Flow::RefreshToken("hello world".to_string());

        let res = dracoon.connect(refresh_token_auth).await;
        let connected_client = res.unwrap();

        let access_token = connected_client.get_auth_header().await.unwrap();

        auth_mock.assert();
        assert_eq!(access_token, "Bearer access_token");
    }

    #[tokio::test]
    async fn test_token_rotation_creation() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let mut token_generator = TokenGenrator::new();

        let auth_mock_1 = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(token_generator.get_auth_response())
            .create();

        let auth_mock_2 = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(token_generator.get_auth_response())
            .create();

        let auth_mock_3 = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(token_generator.get_auth_response())
            .create();

        let auth_mock_4 = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(token_generator.get_auth_response())
            .create();

        let auth_mock_5 = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(token_generator.get_auth_response())
            .create();
        let dracoon = get_test_client_with_token_rotation(&base_url, 5);

        let dracoon = dracoon
            .connect(OAuth2Flow::AuthCodeFlow("test".to_string()))
            .await
            .unwrap();

        auth_mock_1.assert();
        auth_mock_2.assert();
        auth_mock_3.assert();
        auth_mock_4.assert();
        auth_mock_5.assert();

        assert_eq!(dracoon.token_rotation, Some(5));
        assert_eq!(dracoon.additional_connections.get().await.unwrap().len(), 4);
        assert_eq!(
            dracoon.curr_connection.get().await.unwrap(),
            CurrentConnection::Main
        );
        assert_eq!(
            dracoon
                .additional_connections
                .get()
                .await
                .unwrap()
                .first()
                .unwrap()
                .access_token
                .expose_secret(),
            "token2"
        );
        assert_eq!(
            dracoon
                .additional_connections
                .get()
                .await
                .unwrap()
                .get(1)
                .unwrap()
                .access_token
                .expose_secret(),
            "token3"
        );
        assert_eq!(
            dracoon
                .additional_connections
                .get()
                .await
                .unwrap()
                .get(2)
                .unwrap()
                .access_token
                .expose_secret(),
            "token4"
        );
        assert_eq!(
            dracoon
                .additional_connections
                .get()
                .await
                .unwrap()
                .get(3)
                .unwrap()
                .access_token
                .expose_secret(),
            "token5"
        );
    }

    #[tokio::test]
    async fn test_token_rotation_creation_above_limit() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let auth_res = include_str!("./tests/auth_ok.json");

        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(auth_res)
            .expect(5)
            .create();

        let dracoon = get_test_client_with_token_rotation(&base_url, 10);

        let dracoon = dracoon
            .connect(OAuth2Flow::AuthCodeFlow("test".to_string()))
            .await
            .unwrap();

        auth_mock.assert();

        assert_eq!(dracoon.token_rotation, Some(5));
        assert_eq!(dracoon.additional_connections.get().await.unwrap().len(), 4);
        assert_eq!(
            dracoon.curr_connection.get().await.unwrap(),
            CurrentConnection::Main
        );
        assert_eq!(
            dracoon
                .additional_connections
                .get()
                .await
                .unwrap()
                .first()
                .unwrap()
                .access_token
                .expose_secret(),
            "access_token"
        );
        assert_eq!(
            dracoon
                .additional_connections
                .get()
                .await
                .unwrap()
                .get(1)
                .unwrap()
                .access_token
                .expose_secret(),
            "access_token"
        );
        assert_eq!(
            dracoon
                .additional_connections
                .get()
                .await
                .unwrap()
                .get(2)
                .unwrap()
                .access_token
                .expose_secret(),
            "access_token"
        );
        assert_eq!(
            dracoon
                .additional_connections
                .get()
                .await
                .unwrap()
                .get(3)
                .unwrap()
                .access_token
                .expose_secret(),
            "access_token"
        );
    }

    #[tokio::test]
    async fn test_token_rotation_with_lower_limit() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let auth_res = include_str!("./tests/auth_ok.json");

        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(auth_res)
            .expect(1)
            .create();

        let dracoon = get_test_client_with_token_rotation(&base_url, 1)
            .connect(OAuth2Flow::authorization_code("test".to_string()))
            .await
            .unwrap();

        auth_mock.assert();

        assert_eq!(dracoon.token_rotation, None);
        assert!(dracoon.additional_connections.is_none().await);
    }

    #[tokio::test]
    async fn test_get_token_url() {
        let base_url = "https://dracoon.team";

        let dracoon = get_test_client(base_url);

        let token_url = dracoon.get_token_url();

        assert_eq!(token_url.as_str(), "https://dracoon.team/oauth/token");
    }

    #[tokio::test]
    async fn test_get_base_url() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let auth_res = include_str!("./tests/auth_ok.json");

        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(auth_res)
            .create();

        let dracoon = get_test_client(&base_url);
        let dracoon = dracoon
            .connect(OAuth2Flow::AuthCodeFlow("hello world".to_string()))
            .await
            .unwrap();

        let base_url = dracoon.get_base_url();

        auth_mock.assert();
        assert_eq!(base_url.as_str(), format!("{}/", mock_server.url()));
    }

    #[tokio::test]
    async fn test_get_refresh_token() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let auth_res = include_str!("./tests/auth_ok.json");

        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(auth_res)
            .create();

        let dracoon = get_test_client(&base_url);
        let dracoon = dracoon
            .connect(OAuth2Flow::AuthCodeFlow("hello world".to_string()))
            .await
            .unwrap();

        let refresh_token = dracoon.get_refresh_token().await;

        auth_mock.assert();
        assert_eq!(refresh_token, "refresh_token");
    }

    #[tokio::test]
    async fn test_retry_policy() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(429)
            .with_header("content-type", "application/json")
            .with_body("Internal Server Error")
            .expect(2)
            .create();

        let dracoon = get_test_client(&base_url);
        let dracoon = dracoon
            .connect(OAuth2Flow::AuthCodeFlow("hello world".to_string()))
            .await;

        auth_mock.assert();
        assert!(dracoon.is_err());
    }

    #[tokio::test]
    async fn test_token_refresh() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();
        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(include_str!("./tests/auth_ok_expired.json"))
            .expect(2)
            .create();

        let dracoon = get_test_client(&base_url);
        let dracoon = dracoon
            .connect(OAuth2Flow::AuthCodeFlow("hello world".to_string()))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;

        let header = dracoon.get_auth_header().await.unwrap();

        // two requests - one for initial auth, one for refresh
        auth_mock.assert();

        assert_eq!(header, "Bearer access_token");
    }

    #[tokio::test]
    async fn test_get_service_token() {
        let dracoon = DracoonClient::builder()
            .with_base_url("https://test.dracoon.com")
            .with_provisioning_token("TopSecret1234!")
            .build_provisioning();

        assert!(dracoon.is_ok());
        let dracoon = dracoon.unwrap();

        assert_eq!(dracoon.get_service_token(), "TopSecret1234!");
    }

    #[tokio::test]
    async fn test_fail_build_with_missing_token() {
        let dracoon = DracoonClient::builder()
            .with_base_url("https://test.dracoon.com")
            .build_provisioning();

        assert!(dracoon.is_err());
    }

    #[tokio::test]
    async fn test_default_user_agent() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let auth_res = include_str!("./tests/auth_ok.json");

        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(auth_res)
            .match_header(USER_AGENT.as_str(), APP_USER_AGENT)
            .create();

        let dracoon = get_test_client(&base_url);
        let _dracoon = dracoon
            .connect(OAuth2Flow::AuthCodeFlow("hello world".to_string()))
            .await
            .unwrap();

        auth_mock.assert();
    }

    #[tokio::test]
    async fn test_custom_user_agent() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let auth_res = include_str!("./tests/auth_ok.json");

        let custom_user_agent = format!("{}|{}", "test_client", APP_USER_AGENT);

        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(auth_res)
            .match_header(USER_AGENT.as_str(), custom_user_agent.as_str())
            .create();

        let _dracoon = DracoonClientBuilder::new()
            .with_base_url(base_url.as_str())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .with_user_agent("test_client")
            .build()
            .expect("valid client config")
            .connect(OAuth2Flow::AuthCodeFlow("hello world".to_string()))
            .await
            .unwrap();

        auth_mock.assert();
    }

    #[tokio::test]
    async fn test_simple_connection() {
        let dracoon = DracoonClient::builder()
            .with_base_url("https://test.dracoon.com")
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .expect("valid client config")
            .connect(OAuth2Flow::simple("access_token"))
            .await;

        assert!(dracoon.is_ok());

        let connected_client = dracoon.unwrap();

        let access_token = connected_client.get_auth_header().await.unwrap();

        assert_eq!(access_token, "Bearer access_token");

        let conn = connected_client.connection.get().await.unwrap();
        assert_eq!(conn.access_token.expose_secret(), "access_token");
        assert_eq!(conn.refresh_token.expose_secret(), "");
        assert_eq!(conn.expires_in, u64::MAX);
    }
}
