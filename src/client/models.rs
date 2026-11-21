use reqwest_retry::{default_on_request_failure, Retryable, RetryableStrategy};
use secrecy::SecretString;
use std::fmt::{Display, Formatter};
use url::ParseError;

use jiff::Timestamp;
use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{GRANT_TYPE_AUTH_CODE, GRANT_TYPE_PASSWORD, GRANT_TYPE_REFRESH_TOKEN},
    utils::parse_body,
};

use super::{errors::DracoonClientError, Connection, DracoonClient};

pub(crate) trait GetClient<S> {
    fn get_client(&self) -> &DracoonClient<S>;
}

/// represents form data payload for `OAuth2` password flow
#[derive(Serialize, Deserialize)]
pub struct OAuth2PasswordFlow {
    pub username: String,
    pub password: String,
    pub grant_type: String,
}

impl OAuth2PasswordFlow {
    /// creates a new password flow payload
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
            grant_type: GRANT_TYPE_PASSWORD.to_string(),
        }
    }
}

/// represents form data payload for `OAuth2` authorization code flow
#[derive(Serialize, Deserialize)]
pub struct OAuth2AuthCodeFlow {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
}

impl OAuth2AuthCodeFlow {
    /// creates a new authorization code flow payload
    pub fn new(client_id: &str, client_secret: &str, code: &str, redirect_uri: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            grant_type: GRANT_TYPE_AUTH_CODE.to_string(),
            code: code.to_string(),
            redirect_uri: redirect_uri.to_string(),
        }
    }
}

/// represents form data payload for `OAuth2` refresh token flow
#[derive(Serialize, Deserialize)]
pub struct OAuth2RefreshTokenFlow {
    client_id: String,
    client_secret: String,
    grant_type: String,
    refresh_token: String,
}

impl OAuth2RefreshTokenFlow {
    /// creates a new refresh token flow payload
    pub fn new(client_id: &str, client_secret: &str, refresh_token: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            grant_type: GRANT_TYPE_REFRESH_TOKEN.to_string(),
            refresh_token: refresh_token.to_string(),
        }
    }
}

/// represents form data payload for `OAuth2` token revoke
#[derive(Serialize, Deserialize)]
pub struct OAuth2TokenRevoke {
    client_id: String,
    client_secret: String,
    token_type_hint: String,
    token: String,
}

impl OAuth2TokenRevoke {
    /// creates a new token revoke payload
    pub fn new(client_id: &str, client_secret: &str, token_type_hint: &str, token: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            token_type_hint: token_type_hint.to_string(),
            token: token.to_string(),
        }
    }
}

/// DRACOON `OAuth2` token response
#[derive(Serialize, Deserialize)]
pub struct OAuth2TokenResponse {
    access_token: String,
    /// not issued by every OAuth client (e.g. `dracoon_legacy_scripting`)
    #[serde(default)]
    refresh_token: Option<String>,
    token_type: Option<String>,
    expires_in: u64,
    expires_in_inactive: Option<u64>,
    scope: Option<String>,
}

const REDACTED: &str = "[redacted]";

// manual `Debug` implementations: never print passwords, client secrets, codes or tokens

impl std::fmt::Debug for OAuth2PasswordFlow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuth2PasswordFlow")
            .field("username", &self.username)
            .field("password", &REDACTED)
            .field("grant_type", &self.grant_type)
            .finish()
    }
}

impl std::fmt::Debug for OAuth2AuthCodeFlow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuth2AuthCodeFlow")
            .field("client_id", &self.client_id)
            .field("client_secret", &REDACTED)
            .field("grant_type", &self.grant_type)
            .field("code", &REDACTED)
            .field("redirect_uri", &self.redirect_uri)
            .finish()
    }
}

impl std::fmt::Debug for OAuth2RefreshTokenFlow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuth2RefreshTokenFlow")
            .field("client_id", &self.client_id)
            .field("client_secret", &REDACTED)
            .field("grant_type", &self.grant_type)
            .field("refresh_token", &REDACTED)
            .finish()
    }
}

impl std::fmt::Debug for OAuth2TokenRevoke {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuth2TokenRevoke")
            .field("client_id", &self.client_id)
            .field("client_secret", &REDACTED)
            .field("token_type_hint", &self.token_type_hint)
            .field("token", &REDACTED)
            .finish()
    }
}

impl std::fmt::Debug for OAuth2TokenResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuth2TokenResponse")
            .field("access_token", &REDACTED)
            .field("refresh_token", &REDACTED)
            .field("token_type", &self.token_type)
            .field("expires_in", &self.expires_in)
            .field("expires_in_inactive", &self.expires_in_inactive)
            .field("scope", &self.scope)
            .finish()
    }
}

/// DRACOON http error response
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DracoonErrorResponse {
    code: i32,
    message: String,
    debug_info: Option<String>,
    error_code: Option<i32>,
}

impl Display for DracoonErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let dbg_info = self.debug_info.as_deref().unwrap_or("No details");
        let error_code = self.error_code.unwrap_or(0);
        write!(
            f,
            "{} {} - {dbg_info} ({})",
            self.code, self.message, error_code
        )
    }
}

impl DracoonErrorResponse {
    /// creates a DRACOON compatible error type
    pub fn new(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            debug_info: None,
            error_code: None,
        }
    }

    /// Checks if error is 403 Forbidden
    pub fn is_forbidden(&self) -> bool {
        self.code == 403
    }

    /// Checks if error is 404 Not Found
    pub fn is_not_found(&self) -> bool {
        self.code == 404
    }

    /// Checks if error is 409 Conflict
    pub fn is_conflict(&self) -> bool {
        self.code == 409
    }

    /// Checks if error is 429 Too Many Requests
    pub fn is_too_many_requests(&self) -> bool {
        self.code == 429
    }

    /// Checks if error is 500 Internal Server Error
    pub fn is_server_error(&self) -> bool {
        self.code >= 500
    }

    /// Checks if error is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        self.code >= 400 && self.code < 500
    }

    /// Checks if error is 401 Unauthorized
    pub fn is_unauthorized(&self) -> bool {
        self.code == 401
    }

    /// Checks if error is 400 Bad Request
    pub fn is_bad_request(&self) -> bool {
        self.code == 400
    }

    /// Checks if error is 402 Payment Required
    pub fn is_payment_required(&self) -> bool {
        self.code == 402
    }

    /// Checks if error is 412 Precondition Failed
    pub fn is_precondition_failed(&self) -> bool {
        self.code == 412
    }

    // Returns DRACOON API error code if available
    pub fn error_code(&self) -> Option<i32> {
        self.error_code
    }

    /// Returns the HTTP status code
    pub fn code(&self) -> i32 {
        self.code
    }

    /// Returns the error message
    pub fn error_message(&self) -> String {
        self.message.clone()
    }

    /// Returns the debug info
    pub fn debug_info(&self) -> Option<String> {
        self.debug_info.clone()
    }
}

/// DRACOON `OAuth2` error response
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DracoonAuthErrorResponse {
    error: String,
    error_description: Option<String>,
}

impl DracoonAuthErrorResponse {
    pub fn new_unauthorized() -> Self {
        Self {
            error: "Unauthorized".to_string(),
            error_description: None,
        }
    }
}

impl Display for DracoonAuthErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error: {} ({})",
            self.error_description
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            self.error
        )
    }
}

impl OAuth2TokenResponse {
    /// transforms a response into a DRACOON `OAuth2` token response
    /// on error will return a DRACOON auth error response
    pub async fn from_response(res: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonAuthErrorResponse>(res).await
    }
}

/// represents the state of a status code
///  - Ok: 2xx
/// - Error: 4xx or 5xx
pub enum StatusCodeState {
    Ok(StatusCode),
    Error(StatusCode),
}

impl From<StatusCode> for StatusCodeState {
    /// transforms a status code into a status code state
    fn from(value: StatusCode) -> Self {
        match value {
            StatusCode::OK
            | StatusCode::CREATED
            | StatusCode::ACCEPTED
            | StatusCode::NO_CONTENT => StatusCodeState::Ok(value),
            _ => StatusCodeState::Error(value),
        }
    }
}

impl From<OAuth2TokenResponse> for Connection {
    /// transforms a `OAuth2` token response into a connection for the client
    fn from(value: OAuth2TokenResponse) -> Self {
        Self {
            connected_at: Timestamp::now(),
            access_token: SecretString::from(value.access_token),
            // without a refresh token, an expired access token results in an auth error
            refresh_token: SecretString::from(value.refresh_token.unwrap_or_default()),
            expires_in: value.expires_in,
        }
    }
}

impl From<DracoonAuthErrorResponse> for DracoonClientError {
    /// transforms a DRACOON auth error response into a DRACOON client error
    fn from(value: DracoonAuthErrorResponse) -> Self {
        Self::Auth(value)
    }
}

impl From<DracoonErrorResponse> for DracoonClientError {
    /// transforms a DRACOON error response into a DRACOON client error
    fn from(value: DracoonErrorResponse) -> Self {
        Self::Http(value)
    }
}

impl From<ParseError> for DracoonClientError {
    /// transforms a URL parse error into a DRACOON client error
    fn from(_v: ParseError) -> Self {
        Self::InvalidUrl("parsing url failed (invalid)".to_string())
    }
}

pub(crate) struct DracoonCustomRetryStrategy;

impl RetryableStrategy for DracoonCustomRetryStrategy {
    fn handle(
        &self,
        res: &Result<reqwest::Response, reqwest_middleware::Error>,
    ) -> Option<Retryable> {
        match res {
            Ok(success) => default_on_request_success(success),
            Err(error) => default_on_request_failure(error),
        }
    }
}

// this overwrites the default retry strategy to handle 401 invalid JWT token errors that need to be retried
// it is not possible to access the response body due to async / sync code limitations
// therefore all 401 requests are retried
// in any case, token validity is checked before a request
/// matches `/oauth/...` also behind a base url with a path prefix (e.g. `/dracoon/oauth/token`)
fn is_oauth_path(path: &str) -> bool {
    path.split('/').any(|segment| segment == "oauth")
}

fn default_on_request_success(success: &reqwest::Response) -> Option<Retryable> {
    let status = success.status();
    if status.is_server_error() {
        Some(Retryable::Transient)
    } else if status.is_client_error()
        && status != StatusCode::REQUEST_TIMEOUT
        && status != StatusCode::TOO_MANY_REQUESTS
        && status != StatusCode::UNAUTHORIZED
    {
        Some(Retryable::Fatal)
    } else if status.is_success() {
        None
    } else if status == StatusCode::REQUEST_TIMEOUT
        || status == StatusCode::TOO_MANY_REQUESTS
        || status == StatusCode::UNAUTHORIZED
    {
        // never retry 401 on OAuth2 endpoints (wrong credentials / invalid refresh token);
        // other endpoints can fail intermittently with 401 at the API gateway and are retried
        if status == StatusCode::UNAUTHORIZED && is_oauth_path(success.url().path()) {
            Some(Retryable::Fatal)
        } else {
            Some(Retryable::Transient)
        }
    } else {
        Some(Retryable::Fatal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_payloads_do_not_leak_secrets_in_debug_output() {
        let outputs = [
            format!("{:?}", OAuth2PasswordFlow::new("user", "s3cr3t-password")),
            format!(
                "{:?}",
                OAuth2AuthCodeFlow::new("id", "s3cr3t-client", "s3cr3t-code", "https://cb")
            ),
            format!(
                "{:?}",
                OAuth2RefreshTokenFlow::new("id", "s3cr3t-client", "s3cr3t-refresh")
            ),
            format!(
                "{:?}",
                OAuth2TokenRevoke::new("id", "s3cr3t-client", "access_token", "s3cr3t-token")
            ),
            format!(
                "{:?}",
                serde_json::from_str::<OAuth2TokenResponse>(
                    r#"{"access_token": "s3cr3t-access", "refresh_token": "s3cr3t-refresh", "expires_in": 3600}"#
                )
                .unwrap()
            ),
        ];

        for output in outputs {
            assert!(!output.contains("s3cr3t"), "secret leaked: {output}");
            assert!(output.contains("[redacted]"), "unexpected output: {output}");
        }
    }

    #[test]
    fn test_is_oauth_path() {
        assert!(is_oauth_path("/oauth/token"));
        assert!(is_oauth_path("/oauth/revoke"));
        // base url with a path prefix
        assert!(is_oauth_path("/dracoon/oauth/token"));
        assert!(!is_oauth_path("/api/v4/user/account"));
        assert!(!is_oauth_path("/api/v4/nodes/oauthfiles"));
    }
}
