use async_trait::async_trait;
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde_xml_rs::from_str;
use tracing::{debug, error};

use super::{
    auth::{errors::DracoonClientError, models::StatusCodeState},
    nodes::models::S3ErrorResponse,
};

/// Parses the response body and returns the result into desired JSON parsed response or error
pub async fn parse_body<T, E>(res: Response) -> Result<T, DracoonClientError>
where
    T: DeserializeOwned,
    E: DeserializeOwned + Into<DracoonClientError>,
{
    match Into::<StatusCodeState>::into(res.status()) {
        StatusCodeState::Ok(_) => Ok(res.json::<T>().await.map_err(|err| {
            eprintln!("Failed to parse response body: {}", err);
            error!("{}", err);
            err
        })?),
        StatusCodeState::Error(_) => Err(build_error_body::<E>(res.json::<E>().await.map_err(
            |err| {
                error!("Failed to parse error body: {}", err);
                err
            },
        )?)),
    }
}

/// Builds the error body from the response
fn build_error_body<E>(body: E) -> DracoonClientError
where
    E: DeserializeOwned + Into<DracoonClientError>,
{
    body.into()
}

/// Builds the error body from the response for S3 errors (XML)
pub async fn build_s3_error(response: Response) -> DracoonClientError {
    let status = &response.status();
    let Ok(text) = response.text().await else {
        debug!("Failed to read S3 XML error body: {}", status);
        return DracoonClientError::Unknown;
    };

    let Ok(error) = from_str(&text) else {
        debug!("Failed to parse S3 XML error response: {}", text);
        return DracoonClientError::Unknown;
    };
    let err_response = S3ErrorResponse::from_xml_error(*status, error);
    DracoonClientError::S3Error(Box::new(err_response))
}

#[async_trait]
pub trait FromResponse {
    /// Trait that allows to convert a response into a specific type (async)
    async fn from_response(res: Response) -> Result<Self, DracoonClientError>
    where
        Self: Sized;
}
