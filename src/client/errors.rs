use async_trait::async_trait;
use dco3_crypto::DracoonCryptoError;
use reqwest::{Error as ClientError, Response};
use reqwest_middleware::Error as ReqError;
use reqwest_retry::RetryError;
use thiserror::Error;
use tracing::{debug, error};

use crate::{
    nodes::models::S3ErrorResponse,
    utils::{fallback_http_error, FromResponse},
};

use super::models::{DracoonAuthErrorResponse, DracoonErrorResponse};

#[derive(Debug, Error, PartialEq)]
pub enum DracoonClientError {
    #[error("Client id required")]
    MissingClientId,
    #[error("Client secret required")]
    MissingClientSecret,
    #[error("Base url required")]
    MissingBaseUrl,
    #[error("Invalid DRACOON url")]
    InvalidUrl(String),
    #[error("Invalid DRACOON path")]
    InvalidPath(String),
    #[error("Connection to DRACOON failed: {0}")]
    ConnectionFailed(String),
    #[error("Unknown error")]
    Unknown,
    #[error("Internal error")]
    Internal,
    #[error("HTTP error")]
    Http(DracoonErrorResponse),
    #[error("S3 error")]
    S3Error(Box<S3ErrorResponse>),
    #[error("Authentication error")]
    Auth(DracoonAuthErrorResponse),
    #[error("IO error")]
    IoError,
    #[error("Crypto error")]
    CryptoError(DracoonCryptoError),
    #[error("Missing encryption secret")]
    MissingEncryptionSecret,
    #[error("Missing argument")]
    MissingArgument,
}

impl From<ReqError> for DracoonClientError {
    fn from(value: ReqError) -> Self {
        match value {
            ReqError::Middleware(error) => {
                // the retry middleware wraps every error (incl. timeouts / connection errors) - unwrap it
                let error = match error.downcast::<RetryError>() {
                    Ok(RetryError::WithRetries { err, .. } | RetryError::Error(err)) => {
                        return err.into()
                    }
                    Err(error) => error,
                };

                match error.downcast::<DracoonClientError>() {
                    // errors raised by our own middleware (e.g. failed token refresh) are passed through
                    Ok(error) => error,
                    Err(error) => {
                        error!("Error in middleware: {:#}", error);
                        DracoonClientError::ConnectionFailed("Error in middleware".into())
                    }
                }
            }
            ReqError::Reqwest(error) => error.into(),
        }
    }
}

impl From<ClientError> for DracoonClientError {
    fn from(error: ClientError) -> Self {
        // keep the full error chain in the logs - the variant only carries a message
        debug!("HTTP request failed: {:?}", error);

        if error.is_timeout() {
            return DracoonClientError::ConnectionFailed("Timeout".into());
        }

        if error.is_connect() {
            return DracoonClientError::ConnectionFailed("Connection failed".into());
        }

        DracoonClientError::ConnectionFailed(error_chain(&error))
    }
}

/// formats an error with all its sources (`outer: inner: innermost`)
fn error_chain(error: &dyn std::error::Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(err) = source {
        message.push_str(": ");
        message.push_str(&err.to_string());
        source = err.source();
    }
    message
}

#[async_trait]
impl FromResponse for DracoonClientError {
    async fn from_response(value: Response) -> Result<Self, DracoonClientError> {
        let status = value.status();

        if !status.is_success() {
            let parsed = value.json::<DracoonErrorResponse>().await;
            return match parsed {
                Ok(error) => Ok(DracoonClientError::Http(error)),
                Err(err) => {
                    error!("Failed to parse error body ({}): {}", status, err);
                    let fallback = fallback_http_error(status, "failed to parse error body");
                    Ok(DracoonClientError::Http(fallback))
                }
            };
        }
        Err(DracoonClientError::Unknown)
    }
}

impl From<DracoonCryptoError> for DracoonClientError {
    fn from(value: DracoonCryptoError) -> Self {
        DracoonClientError::CryptoError(value)
    }
}

impl DracoonClientError {
    pub fn get_http_error(&self) -> Option<&DracoonErrorResponse> {
        match self {
            DracoonClientError::Http(error) => Some(error),
            _ => None,
        }
    }

    /// Check if the error is an authentication error
    pub fn is_auth_error(&self) -> bool {
        matches!(self, DracoonClientError::Auth(_))
    }

    /// Check if the error is an HTTP error
    pub fn is_http_error(&self) -> bool {
        matches!(self, DracoonClientError::Http(_))
    }

    /// Check if the error is an 401 Unauthorized error
    pub fn is_unauthorized(&self) -> bool {
        match self {
            DracoonClientError::Http(error) => error.is_unauthorized(),
            _ => false,
        }
    }

    /// Check if the error is an 402 Payment Required error
    pub fn is_payment_required(&self) -> bool {
        match self {
            DracoonClientError::Http(error) => error.is_payment_required(),
            _ => false,
        }
    }

    /// Check if the error is an 403 Forbidden error
    pub fn is_forbidden(&self) -> bool {
        match self {
            DracoonClientError::Http(error) => error.is_forbidden(),
            _ => false,
        }
    }

    /// Check if the error is an 404 Not Found error
    pub fn is_not_found(&self) -> bool {
        match self {
            DracoonClientError::Http(error) => error.is_not_found(),
            _ => false,
        }
    }

    /// Check if the error is an 409 Conflict error
    pub fn is_conflict(&self) -> bool {
        match self {
            DracoonClientError::Http(error) => error.is_conflict(),
            _ => false,
        }
    }

    /// Check if the error is an 412 Precondition Failed error
    pub fn is_precondition_failed(&self) -> bool {
        match self {
            DracoonClientError::Http(error) => error.is_precondition_failed(),
            _ => false,
        }
    }

    /// Check if the error is an 429 Too Many Requests error
    pub fn is_too_many_requests(&self) -> bool {
        match self {
            DracoonClientError::Http(error) => error.is_too_many_requests(),
            _ => false,
        }
    }

    /// Check if the error is an 500 Internal Server Error error
    pub fn is_server_error(&self) -> bool {
        match self {
            DracoonClientError::Http(error) => error.is_server_error(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Response as HttpResponse;
    use reqwest::{Body, StatusCode};

    fn build_response(status: StatusCode, body: &str) -> Response {
        HttpResponse::builder()
            .status(status)
            .body(Body::from(body.to_string()))
            .unwrap()
            .into()
    }

    #[tokio::test]
    async fn reqwest_errors_keep_their_description() {
        // a URL without a host fails while building the request
        let err = reqwest::Client::new()
            .get("http://")
            .send()
            .await
            .unwrap_err();

        let DracoonClientError::ConnectionFailed(message) = DracoonClientError::from(err) else {
            panic!("expected ConnectionFailed");
        };
        assert_ne!(message, "Unknown");
        assert!(
            message.contains("builder error"),
            "unexpected message: {message}"
        );
    }

    #[tokio::test]
    async fn from_response_returns_http_error_when_error_body_cannot_be_parsed() {
        let response = build_response(StatusCode::BAD_GATEWAY, "not-json");
        let error = DracoonClientError::from_response(response)
            .await
            .expect("expected fallback Http error");

        match error {
            DracoonClientError::Http(error) => {
                assert_eq!(error.code(), StatusCode::BAD_GATEWAY.as_u16() as i32);
                assert!(
                    error.error_message().contains("failed to parse error body"),
                    "unexpected error message: {}",
                    error.error_message()
                );
            }
            other => panic!("expected Http error, got {other:?}"),
        }
    }
}
