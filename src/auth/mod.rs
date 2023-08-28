//! This module is responsible for the authentication with DRACOON and implements
//! the [DracoonClient] struct which is used to interact with the DRACOON API.
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::{marker::PhantomData, time::Duration};
use tracing::{debug, error};

use base64::{
    self, alphabet,
    engine::{self, general_purpose},
    Engine,
};

pub mod errors;
pub mod models;

pub use models::*;

use crate::{
    constants::{
        DRACOON_TOKEN_REVOKE_URL, DRACOON_TOKEN_URL, EXPONENTIAL_BACKOFF_BASE, MAX_RETRIES,
        MAX_RETRY_DELAY, MIN_RETRY_DELAY, TOKEN_TYPE_HINT_ACCESS_TOKEN,
    },
    models::Container,
};

use self::errors::DracoonClientError;
use super::constants::{APP_USER_AGENT, TOKEN_TYPE_HINT_REFRESH_TOKEN};

/// represents the possible `OAuth2` flows
pub enum OAuth2Flow {
    PasswordFlow(String, String),
    AuthCodeFlow(String),
    RefreshToken(String),
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

/// represents a connection to DRACOON (`OAuth2` tokens)
#[derive(Debug, Clone)]
pub struct Connection {
    access_token: String,
    refresh_token: String,
    expires_in: u32,
    connected_at: DateTime<Utc>,
}

impl Connection {
    pub fn refresh_token(&self) -> String {
        self.refresh_token.clone()
    }

    pub fn access_token(&self) -> String {
        self.access_token.clone()
    }

    pub fn expires_in(&self) -> u32 {
        self.expires_in
    }

    pub fn connected_at(&self) -> DateTime<Utc> {
        self.connected_at
    }

    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let expires_at = self.connected_at + chrono::Duration::seconds(self.expires_in as i64);
        now > expires_at
    }

    pub fn update_tokens(&mut self, connection: Connection) {
        self.access_token = connection.access_token;
        self.refresh_token = connection.refresh_token;
        self.expires_in = connection.expires_in;
        self.connected_at = connection.connected_at;
    }
}

#[derive(Clone)]
/// represents the DRACOON client (stateful)
pub struct DracoonClient<State = Disconnected> {
    base_url: Url,
    redirect_uri: Option<Url>,
    client_id: String,
    client_secret: String,
    pub http: ClientWithMiddleware,
    pub stream_http: Client,
    connection: Container<Connection>,
    connected: PhantomData<State>,
    provisioning_token: Option<String>,
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

    pub fn with_provisioning_token(mut self, token: impl Into<String>) -> Self {
        self.provisioning_token = Some(token.into());
        self
    }

    pub fn build_provisioning(self) -> Result<DracoonClient<Provisioning>, DracoonClientError> {
        let Some(provisioning_token) = self.provisioning_token else {
            return Err(DracoonClientError::MissingArgument)
        };

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
            .backoff_exponent(EXPONENTIAL_BACKOFF_BASE)
            .retry_bounds(
                Duration::from_millis(min_retry_delay),
                Duration::from_millis(max_retry_delay),
            )
            .build_with_max_retries(max_retries);

        let user_agent = match self.user_agent {
            Some(user_agent) => format!("{}|{}", user_agent, APP_USER_AGENT),
            None => APP_USER_AGENT.to_string(),
        };

        let http = Client::builder().user_agent(APP_USER_AGENT).build()?;
        let upload_http = http.clone();

        let http = ClientBuilder::new(http)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        let Some(base_url) = self.base_url.clone() else {
        error!("Missing base url");
        return Err(DracoonClientError::MissingBaseUrl)
    };

        let base_url = Url::parse(&base_url)?;

        Ok(DracoonClient {
            base_url,
            redirect_uri: None,
            client_id: String::new(),
            client_secret: String::new(),
            http,
            stream_http: upload_http,
            connected: PhantomData,
            connection: Container::new(),
            provisioning_token: Some(provisioning_token),
        })
    }

    /// Builds the [DracoonClient] struct - returns an error if any of the required fields are missing
    pub fn build(self) -> Result<DracoonClient<Disconnected>, DracoonClientError> {
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
            .backoff_exponent(EXPONENTIAL_BACKOFF_BASE)
            .retry_bounds(
                Duration::from_millis(min_retry_delay),
                Duration::from_millis(max_retry_delay),
            )
            .build_with_max_retries(max_retries);

        let user_agent = match self.user_agent {
            Some(user_agent) => format!("{}|{}", user_agent, APP_USER_AGENT),
            None => APP_USER_AGENT.to_string(),
        };

        let http = Client::builder().user_agent(APP_USER_AGENT).build()?;
        let upload_http = http.clone();

        let http = ClientBuilder::new(http)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        let Some(base_url) = self.base_url.clone() else {
            error!("Missing base url");
            return Err(DracoonClientError::MissingBaseUrl)
        };

        let base_url = Url::parse(&base_url)?;

        let Some(client_id) = self.client_id else {
            error!("Missing client id");
            return Err(DracoonClientError::MissingClientId)
        };

        let Some(client_secret) = self.client_secret else {
            error!("Missing client secret");
            return Err(DracoonClientError::MissingClientSecret)
        };

        let redirect_uri = match self.redirect_uri {
            Some(url) => Url::parse(&url)?,
            None => Url::parse(&format!(
                "{}/oauth/callback",
                self.base_url.expect("missing base url already checked")
            ))?,
        };

        Ok(DracoonClient {
            base_url,
            redirect_uri: Some(redirect_uri),
            client_id,
            client_secret,
            connection: Container::<Connection>::new(),
            connected: PhantomData,
            http,
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
        self,
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
        };

        Ok(DracoonClient {
            client_id: self.client_id,
            client_secret: self.client_secret,
            connection: Container::new_from(connection),
            base_url: self.base_url,
            redirect_uri: self.redirect_uri,
            connected: PhantomData,
            http: self.http,
            stream_http: self.stream_http,
            provisioning_token: None,
        })
    }

    /// returns client id and client secret bas64 encoded for the basic auth header
    fn client_credentials(&self) -> String {
        const B64_URLSAFE: engine::GeneralPurpose =
            engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);
        let client_credentials = format!("{}:{}", &self.client_id, &self.client_secret);

        B64_URLSAFE.encode(client_credentials)
    }

    /// Returns the authorize url for the OAuth2 auth code flow
    pub fn get_authorize_url(&mut self) -> String {
        let default_redirect = self
            .base_url
            .join("oauth/callback")
            .expect("Correct base url");
        let redirect_uri = self
            .redirect_uri
            .as_ref()
            .unwrap_or(&default_redirect)
            .clone();

        self.redirect_uri = Some(redirect_uri.clone());

        let mut authorize_url = self
            .base_url
            .join("oauth/authorize")
            .expect("Correct base url");
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
            .expect("Correct base url")
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
            .http
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
            &self.client_secret,
            code,
            self.redirect_uri
                .as_ref()
                .expect("redirect uri is set")
                .as_str(),
        );

        let res = self
            .http
            .post(token_url)
            .form(&auth)
            .send()
            .await
            .map_err(|err| {
                error!("Error connecting with auth code flow: {}", err);
                err
            })?;
        Ok(OAuth2TokenResponse::from_response(res).await?.into())
    }

    /// Connects to DRACOON using the refresh token flow
    async fn connect_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<Connection, DracoonClientError> {
        let token_url = self.get_token_url();

        let auth = OAuth2RefreshTokenFlow::new(&self.client_id, &self.client_secret, refresh_token);

        let res = self
            .http
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
            base_url: self.base_url,
            redirect_uri: self.redirect_uri,
            connected: PhantomData,
            http: self.http,
            stream_http: self.stream_http,
            provisioning_token: None,
        })
    }

    /// Returns the base url of the DRACOON instance
    pub fn get_base_url(&self) -> &Url {
        &self.base_url
    }

    /// Returns the token url for any OAuth2 flow
    fn get_token_url(&self) -> Url {
        self.base_url
            .join(DRACOON_TOKEN_URL)
            .expect("Correct base url")
    }

    /// Revokes the access token
    async fn revoke_acess_token(&self) -> Result<(), DracoonClientError> {
        let access_token = self
            .connection
            .get()
            .expect("Connected client has a connection")
            .access_token
            .clone();

        let api_url = self
            .base_url
            .join(DRACOON_TOKEN_REVOKE_URL)
            .expect("Correct base url");

        let auth = OAuth2TokenRevoke::new(
            &self.client_id,
            &self.client_secret,
            TOKEN_TYPE_HINT_ACCESS_TOKEN,
            &access_token,
        );

        self.http.post(api_url).form(&auth).send().await?;

        Ok(())
    }

    /// Revokes the refresh token
    async fn revoke_refresh_token(&self) -> Result<(), DracoonClientError> {
        let refresh_token = self
            .connection
            .get()
            .expect("Connected client has a connection")
            .refresh_token
            .clone();

        let api_url = self
            .base_url
            .join(DRACOON_TOKEN_REVOKE_URL)
            .expect("Correct base url");

        let auth = OAuth2TokenRevoke::new(
            &self.client_id,
            &self.client_secret,
            TOKEN_TYPE_HINT_REFRESH_TOKEN,
            &refresh_token,
        );

        self.http.post(api_url).form(&auth).send().await?;

        Ok(())
    }

    /// Fetches new tokens using available refresh token from the current connection
    async fn connect_refresh_token(&self) -> Result<Connection, DracoonClientError> {
        let token_url = self.get_token_url();

        let refresh_token = self
            .connection
            .get()
            .expect("Connected client has a connection")
            .refresh_token
            .clone();

        let auth =
            OAuth2RefreshTokenFlow::new(&self.client_id, &self.client_secret, &refresh_token);

        let res = self.http.post(token_url).form(&auth).send().await?;
        Ok(OAuth2TokenResponse::from_response(res).await?.into())
    }

    /// Returns the necessary token header for any API call that requires authentication in DRACOON
    pub async fn get_auth_header(&self) -> Result<String, DracoonClientError> {
        if self.is_connection_expired() {
            let new_connection = self.connect_refresh_token().await?;
            let mut connection = self.connection.get();
            let connection = connection
                .as_mut()
                .expect("Connected client has a connection");
            connection.update_tokens(new_connection);
        }

        Ok(format!(
            "Bearer {}",
            self.connection
                .get()
                .expect("Connected client has a connection")
                .access_token
        ))
    }

    /// Returns the refresh token
    pub fn get_refresh_token(&self) -> String {
        self.connection
            .get()
            .expect("Connected client has a connection")
            .refresh_token()
    }

    /// Checks if the access token is still valid
    fn is_connection_expired(&self) -> bool {
        self.connection
            .get()
            .expect("Connected client has a connection")
            .is_expired()
    }
}

impl DracoonClient<Provisioning> {
    /// Returns the X-SDS-Service-Token for provisioning API calls
    pub fn get_service_token(&self) -> String {
        self.provisioning_token
            .as_ref()
            .expect("Provisioning client has no token")
            .to_string()
    }

    /// Returns the base url of the DRACOON instance
    pub fn get_base_url(&self) -> &Url {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use tokio_test::assert_ok;

    use super::*;

    fn get_test_client(url: &str) -> DracoonClient<Disconnected> {
        DracoonClientBuilder::new()
            .with_base_url(url)
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .with_user_agent("test_client")
            .with_max_retries(1)
            .with_max_retry_delay(600)
            .with_min_retry_delay(300)
            .build()
            .expect("valid client config")
    }

    #[tokio::test]
    async fn test_auth_code_authentication() {
        let mut mock_server = mockito::Server::new();
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
        assert_ok!(&res);

        assert!(res.unwrap().connection.is_some());
    }

    #[tokio::test]
    async fn test_refresh_token_authentication() {
        let mut mock_server = mockito::Server::new();
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
        assert_ok!(&res);

        assert!(res.as_ref().unwrap().connection.is_some());

        let access_token = res
            .as_ref()
            .unwrap()
            .connection
            .get()
            .unwrap()
            .access_token();

        let refresh_token = res
            .as_ref()
            .unwrap()
            .connection
            .get()
            .unwrap()
            .refresh_token();

        let expires_in = res.as_ref().unwrap().connection.get().unwrap().expires_in();

        assert_eq!(access_token, "access_token");
        assert_eq!(refresh_token, "refresh_token");
        assert_eq!(expires_in, 3600);
    }

    #[tokio::test]
    async fn test_auth_error_handling() {
        let mut mock_server = mockito::Server::new();
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
        let mut mock_server = mockito::Server::new();
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
    async fn test_get_token_url() {
        let base_url = "https://dracoon.team";

        let dracoon = get_test_client(base_url);

        let token_url = dracoon.get_token_url();

        assert_eq!(token_url.as_str(), "https://dracoon.team/oauth/token");
    }

    #[tokio::test]
    async fn test_get_base_url() {
        let mut mock_server = mockito::Server::new();
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
        let mut mock_server = mockito::Server::new();
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

        let refresh_token = dracoon.get_refresh_token();

        auth_mock.assert();
        assert_eq!(refresh_token, "refresh_token");
    }

    #[tokio::test]
    async fn test_retry_policy() {
        let mut mock_server = mockito::Server::new();
        let base_url = mock_server.url();

        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(429)
            .with_header("content-type", "application/json")
            .with_body("Internal Server Error")
            .expect_at_least(2)
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
        let mut mock_server = mockito::Server::new();
        let base_url = mock_server.url();
        let auth_mock = mock_server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(include_str!("./tests/auth_ok_expired.json"))
            .expect_at_least(2)
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
}
