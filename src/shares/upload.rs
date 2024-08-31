use async_trait::async_trait;
use reqwest::header;

use crate::constants::{DRACOON_API_PREFIX, SHARES_BASE, SHARES_EMAIL, SHARES_UPLOAD};
use crate::models::ListAllParams;
use crate::utils::FromResponse;
use crate::{client::Connected, DracoonClientError};

use super::models::*;
use super::UploadShares;

#[async_trait]
impl UploadShares for SharesEndpoint<Connected> {
    async fn get_upload_shares(
        &self,
        params: Option<ListAllParams>,
    ) -> Result<UploadSharesList, DracoonClientError> {
        let params = params.unwrap_or_default();
        let url_part = format!("{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_UPLOAD}");

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

        UploadSharesList::from_response(response).await
    }

    async fn update_upload_shares(
        &self,
        update: UpdateUploadSharesBulkRequest,
    ) -> Result<(), DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_UPLOAD}");

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

    async fn delete_upload_shares(
        &self,
        delete: DeleteUploadSharesRequest,
    ) -> Result<(), DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_UPLOAD}");

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

    async fn create_upload_share(
        &self,
        create: CreateUploadShareRequest,
    ) -> Result<UploadShare, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_UPLOAD}");

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

        UploadShare::from_response(response).await
    }

    async fn get_upload_share(
        &self,
        upload_share_id: u64,
    ) -> Result<UploadShare, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_UPLOAD}/{id}",
            id = upload_share_id
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

        UploadShare::from_response(response).await
    }

    async fn update_upload_share(
        &self,
        upload_share_id: u64,
        update: UpdateUploadShareRequest,
    ) -> Result<UploadShare, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_UPLOAD}/{id}",
            id = upload_share_id
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

        UploadShare::from_response(response).await
    }

    async fn delete_upload_share(&self, upload_share_id: u64) -> Result<(), DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_UPLOAD}/{id}",
            id = upload_share_id
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

    async fn send_upload_share_email(
        &self,
        upload_share_id: u64,
        email: UploadShareLinkEmail,
    ) -> Result<(), DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{SHARES_BASE}/{SHARES_UPLOAD}/{id}/{SHARES_EMAIL}",
            id = upload_share_id
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
