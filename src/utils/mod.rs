use async_trait::async_trait;
use reqwest::{Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_xml_rs::from_str;
use tracing::error;

use super::{
    client::{
        errors::DracoonClientError,
        models::{DracoonErrorResponse, StatusCodeState},
    },
    nodes::models::S3ErrorResponse,
};

/// Parses the response body and returns the result into desired JSON parsed response or error
pub async fn parse_body<T, E>(res: Response) -> Result<T, DracoonClientError>
where
    T: DeserializeOwned,
    E: DeserializeOwned + Into<DracoonClientError>,
{
    let status = res.status();
    match Into::<StatusCodeState>::into(status) {
        StatusCodeState::Ok(_) => Ok(res.json::<T>().await.map_err(|err| {
            error!("{}", err);
            err
        })?),
        StatusCodeState::Error(_) => {
            let parsed_error = res.json::<E>().await;
            match parsed_error {
                Ok(body) => Err(build_error_body::<E>(body)),
                Err(err) => {
                    error!("Failed to parse error body ({}): {}", status, err);
                    let fallback_error = fallback_http_error(status, "failed to parse error body");
                    Err(DracoonClientError::Http(fallback_error))
                }
            }
        }
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
    let status = response.status();
    let Ok(text) = response.text().await else {
        error!("Failed to read S3 XML error body: {}", status);
        let fallback = fallback_http_error(status, "failed to read S3 error body");
        return DracoonClientError::Http(fallback);
    };

    let Ok(error) = from_str(&text) else {
        error!("Failed to parse S3 XML error response: {}", text);
        let fallback = fallback_http_error(status, "failed to parse S3 error body");
        return DracoonClientError::Http(fallback);
    };

    let err_response = S3ErrorResponse::from_xml_error(status, error);
    DracoonClientError::S3Error(Box::new(err_response))
}

#[async_trait]
pub trait FromResponse {
    /// Trait that allows to convert a response into a specific type (async)
    async fn from_response(res: Response) -> Result<Self, DracoonClientError>
    where
        Self: Sized;
}

pub(crate) fn fallback_http_error(status: StatusCode, context: &str) -> DracoonErrorResponse {
    let reason = status.canonical_reason().unwrap_or("Unexpected error");
    let message = if context.is_empty() {
        reason.to_string()
    } else {
        format!("{reason} ({context})")
    };
    DracoonErrorResponse::new(status.as_u16() as i32, message.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Response as HttpResponse;
    use reqwest::Body;
    use serde_json::Value;

    fn build_response(status: StatusCode, body: &str) -> Response {
        HttpResponse::builder()
            .status(status)
            .body(Body::from(body.to_string()))
            .unwrap()
            .into()
    }

    #[tokio::test]
    async fn parse_body_returns_http_error_when_error_body_cannot_be_parsed() {
        let response = build_response(StatusCode::INTERNAL_SERVER_ERROR, "not-json");
        let err = parse_body::<Value, DracoonErrorResponse>(response)
            .await
            .expect_err("expected parse_body to return error");

        match err {
            DracoonClientError::Http(error) => {
                assert_eq!(
                    error.code(),
                    StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32
                );
                assert!(
                    error.error_message().contains("failed to parse error body"),
                    "unexpected error message: {}",
                    error.error_message()
                );
            }
            other => panic!("expected Http error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn build_s3_error_returns_http_error_when_xml_cannot_be_parsed() {
        let response = build_response(StatusCode::BAD_REQUEST, "not-xml");
        let err = build_s3_error(response).await;

        match err {
            DracoonClientError::Http(error) => {
                assert_eq!(error.code(), StatusCode::BAD_REQUEST.as_u16() as i32);
                assert!(
                    error
                        .error_message()
                        .contains("failed to parse S3 error body"),
                    "unexpected error message: {}",
                    error.error_message()
                );
            }
            other => panic!("expected Http error, got {other:?}"),
        }
    }

    #[test]
    fn fallback_http_error_includes_context() {
        let status = StatusCode::IM_A_TEAPOT;
        let error = fallback_http_error(status, "context info");
        assert_eq!(error.code(), status.as_u16() as i32);
        assert_eq!(error.error_message(), "I'm a teapot (context info)");
    }
}
