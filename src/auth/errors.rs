use async_trait::async_trait;
use dco3_crypto::DracoonCryptoError;
use reqwest::{Error as ClientError, Response};
use reqwest_middleware::Error as ReqError;
use thiserror::Error;

use crate::{nodes::models::S3ErrorResponse, utils::FromResponse};

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
    #[error("Connection to DRACOON failed")]
    ConnectionFailed,
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
            ReqError::Middleware(error) => DracoonClientError::ConnectionFailed,
            ReqError::Reqwest(error) => {
                if error.is_timeout() {
                    return DracoonClientError::ConnectionFailed;
                }

                if error.is_connect() {
                    return DracoonClientError::ConnectionFailed;
                }

                DracoonClientError::Unknown
            }
        }
    }
}

impl From<ClientError> for DracoonClientError {
    fn from(value: ClientError) -> Self {
        if value.is_timeout() {
            return DracoonClientError::ConnectionFailed;
        }

        if value.is_connect() {
            return DracoonClientError::ConnectionFailed;
        }

        DracoonClientError::Unknown
    }
}

#[async_trait]
impl FromResponse for DracoonClientError {
    async fn from_response(value: Response) -> Result<Self, DracoonClientError> {
        if !value.status().is_success() {
            let error = value.json::<DracoonErrorResponse>().await?;
            return Ok(DracoonClientError::Http(error));
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
