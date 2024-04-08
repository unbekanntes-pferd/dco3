use async_trait::async_trait;
use reqwest::header;

use crate::constants::{DRACOON_API_PREFIX, SHARES_BASE, SHARES_DOWNLOAD, SHARES_EMAIL};
use crate::models::ListAllParams;
use crate::utils::FromResponse;
use crate::{auth::Connected, DracoonClientError};

use super::models::*;
use super::DownloadShares;

#[async_trait]
impl DownloadShares for SharesEndpoint<Connected> {
    async fn get_download_shares(
        &self,
        params: Option<ListAllParams>,
    ) -> Result<DownloadSharesList, DracoonClientError> {
        let params = params.unwrap_or_default();
        let url_part = format!("{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_DOWNLOAD}");

        let mut api_url = self.client().build_api_url(&url_part);

        let filters = params.filter_to_string();
        let sorts = params.sort_to_string();

        api_url
            .query_pairs_mut()
            .extend_pairs(params.limit.map(|v| ("limit", v.to_string())))
            .extend_pairs(params.offset.map(|v| ("offset", v.to_string())))
            .extend_pairs(params.sort.map(|_| ("sort", sorts)))
            .extend_pairs(params.filter.map(|_| ("filter", filters)))
            .finish();

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        DownloadSharesList::from_response(response).await
    }

    async fn update_download_shares(
        &self,
        update: UpdateDownloadSharesBulkRequest,
    ) -> Result<(), DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_DOWNLOAD}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .put(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&update)
            .send()
            .await?;

        if response.status().is_server_error() || response.status().is_client_error() {
            return Err(DracoonClientError::from_response(response)
                .await
                .expect("Could not parse error response"));
        }

        Ok(())
    }

    async fn delete_download_shares(
        &self,
        delete: DeleteDownloadSharesRequest,
    ) -> Result<(), DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_DOWNLOAD}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .delete(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&delete)
            .send()
            .await?;

        if response.status().is_server_error() || response.status().is_client_error() {
            return Err(DracoonClientError::from_response(response)
                .await
                .expect("Could not parse error response"));
        }

        Ok(())
    }

    async fn create_download_share(
        &self,
        create: CreateDownloadShareRequest,
    ) -> Result<DownloadShare, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_DOWNLOAD}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .post(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&create)
            .send()
            .await?;

        DownloadShare::from_response(response).await
    }

    async fn get_download_share(
        &self,
        download_share_id: u64,
    ) -> Result<DownloadShare, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_DOWNLOAD}/{id}",
            id = download_share_id
        );

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        DownloadShare::from_response(response).await
    }

    async fn update_download_share(
        &self,
        download_share_id: u64,
        update: UpdateDownloadShareRequest,
    ) -> Result<DownloadShare, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_DOWNLOAD}/{id}",
            id = download_share_id
        );

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .put(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&update)
            .send()
            .await?;

        DownloadShare::from_response(response).await
    }

    async fn delete_download_share(
        &self,
        download_share_id: u64,
    ) -> Result<(), DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_DOWNLOAD}/{id}",
            id = download_share_id
        );

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .delete(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        if response.status().is_server_error() || response.status().is_client_error() {
            return Err(DracoonClientError::from_response(response)
                .await
                .expect("Could not parse error response"));
        }

        Ok(())
    }

    async fn send_download_share_email(
        &self,
        download_share_id: u64,
        email: DownloadShareLinkEmail,
    ) -> Result<(), DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_DOWNLOAD}/{id}/{SHARES_EMAIL}",
            id = download_share_id
        );

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .post(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&email)
            .send()
            .await?;

        if response.status().is_server_error() || response.status().is_client_error() {
            return Err(DracoonClientError::from_response(response)
                .await
                .expect("Could not parse error response"));
        }

        Ok(())
    }
}
