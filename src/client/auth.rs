//! Middleware that keeps the `Authorization` header of API calls current across retries.

use std::sync::Arc;

use async_trait::async_trait;
use http::Extensions;
use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Request, Response,
};
use reqwest_middleware::{Middleware, Next};
use tracing::debug;

use super::{Connected, DracoonClient};

/// Runs *inside* the retry middleware, i.e. before every attempt of a request.
///
/// Callers set the `Authorization` header once when building a request. If a request is retried
/// (e.g. after an intermittent 401 from the API gateway or a 5xx), the header would otherwise be
/// resent unchanged even if the token expired or was refreshed in the meantime.
/// This middleware swaps a stale bearer token for a current one (refreshing it if necessary).
///
/// Requests without a bearer token (`OAuth2` token calls, S3 / download URLs) pass through unchanged.
pub(crate) struct RefreshAuthHeader {
    /// client sharing the token state; its own `http` does not include this middleware
    client: Arc<DracoonClient<Connected>>,
}

impl RefreshAuthHeader {
    pub(crate) fn new(client: Arc<DracoonClient<Connected>>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Middleware for RefreshAuthHeader {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let current_header = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);

        if let Some(current_header) = current_header {
            if !self.client.is_auth_header_current(&current_header).await {
                debug!("Replacing stale access token before sending request");
                let fresh = self
                    .client
                    .get_auth_header()
                    .await
                    .map_err(reqwest_middleware::Error::middleware)?;
                let mut fresh =
                    HeaderValue::from_str(&fresh).map_err(reqwest_middleware::Error::middleware)?;
                fresh.set_sensitive(true);
                req.headers_mut().insert(AUTHORIZATION, fresh);
            }
        }

        next.run(req, extensions).await
    }
}
