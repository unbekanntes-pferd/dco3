mod models;
use async_trait::async_trait;
use reqwest::header;

use crate::{
    auth::Connected,
    constants::{
        DRACOON_API_PREFIX, PUBLIC_BASE, PUBLIC_INFO, PUBLIC_SOFTWARE_BASE, PUBLIC_SYSTEM_BASE,
        PUBLIC_VERSION,
    },
    utils::FromResponse,
    DracoonClientError,
};

pub use self::models::*;

#[async_trait]
pub trait Public {
    async fn get_software_version(&self) -> Result<SoftwareVersionData, DracoonClientError>;
    async fn get_system_info(&self) -> Result<SystemInfo, DracoonClientError>;
}

#[async_trait]
impl Public for PublicEndpoint<Connected> {
    /// Get software version information for the DRACOON backend (API).
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Public};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::password_flow("username", "password"))
    /// #  .await
    /// #  .unwrap();
    ///
    /// let software_version = dracoon.public.get_software_version().await.unwrap();
    ///
    /// # }
    ///
    async fn get_software_version(&self) -> Result<SoftwareVersionData, DracoonClientError> {
        let url_part =
            format!("{DRACOON_API_PREFIX}/{PUBLIC_BASE}/{PUBLIC_SOFTWARE_BASE}/{PUBLIC_VERSION}");

        let url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(url)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        Ok(SoftwareVersionData::from_response(response).await?)
    }

    /// Get system information for the DRACOON backend (API).
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Public};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::password_flow("username", "password"))
    /// #  .await
    /// #  .unwrap();
    ///
    /// let system_info = dracoon.public.get_system_info().await.unwrap();
    ///
    /// # }
    async fn get_system_info(&self) -> Result<SystemInfo, DracoonClientError> {
        let url_part =
            format!("{DRACOON_API_PREFIX}/{PUBLIC_BASE}/{PUBLIC_SYSTEM_BASE}/{PUBLIC_INFO}");

        let url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(url)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        Ok(SystemInfo::from_response(response).await?)
    }
}

#[cfg(test)]
mod tests {

    use chrono::Datelike;

    use crate::{tests::dracoon::get_connected_client, Public};

    #[tokio::test]
    async fn test_get_system_info() {
        let (client, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let system_info_res = include_str!("../tests/responses/public/system_info_ok.json");

        let system_info_mock = mock_server
            .mock("GET", "/api/v4/public/system/info")
            .with_status(200)
            .with_body(system_info_res)
            .with_header("content-type", "application/json")
            .create();

        let system_info = client.public.get_system_info().await.unwrap();

        system_info_mock.assert();

        assert!(system_info.s3_enforce_direct_upload);
        assert!(system_info.use_s3_storage);
        assert_eq!(system_info.language_default, "de-DE");
        assert_eq!(system_info.s3_hosts.len(), 1);
        assert_eq!(system_info.s3_hosts.first().unwrap(), "test.s3.dracoon.com");
    }

    #[tokio::test]
    async fn test_get_software_version() {
        let (client, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let software_version_res = include_str!("../tests/responses/public/version_ok.json");

        let software_version_mock = mock_server
            .mock("GET", "/api/v4/public/software/version")
            .with_status(200)
            .with_body(software_version_res)
            .with_header("content-type", "application/json")
            .create();

        let software_version = client.public.get_software_version().await.unwrap();

        software_version_mock.assert();

        assert_eq!(software_version.rest_api_version, "5.4.6");
        assert_eq!(software_version.sds_server_version, "5.4.6");
        assert_eq!(software_version.build_date.month(), 3);
        assert_eq!(software_version.build_date.day(), 19);
        assert_eq!(software_version.build_date.year(), 2024);
        assert!(software_version.is_dracoon_cloud.unwrap());
    }
}
