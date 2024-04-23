use std::fmt::{Display, Formatter};
use url::ParseError;

use chrono::Utc;
use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{GRANT_TYPE_AUTH_CODE, GRANT_TYPE_PASSWORD, GRANT_TYPE_REFRESH_TOKEN},
    utils::parse_body,
};

use super::{errors::DracoonClientError, Connection};

/// represents form data payload for `OAuth2` password flow
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuth2TokenResponse {
    access_token: String,
    refresh_token: String,
    token_type: Option<String>,
    expires_in: u64,
    expires_in_inactive: Option<u64>,
    scope: Option<String>,
}

/// DRACOON http error response
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
#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DracoonAuthErrorResponse {
    error: String,
    error_description: Option<String>,
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
            connected_at: Utc::now(),
            access_token: value.access_token,
            refresh_token: value.refresh_token,
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
