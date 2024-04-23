mod models;
use async_trait::async_trait;
use reqwest::header;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

use crate::{
    constants::{
        DRACOON_API_PREFIX, PUBLIC_BASE, PUBLIC_DOWNLOAD_SHARES, PUBLIC_INFO, PUBLIC_SHARES_BASE,
        PUBLIC_SOFTWARE_BASE, PUBLIC_SYSTEM_BASE, PUBLIC_VERSION,
    },
    nodes::{DownloadProgressCallback, FileMeta, UploadOptions, UploadProgressCallback},
    utils::FromResponse,
    DracoonClientError,
};

mod download;
mod upload;

pub use self::models::*;

#[async_trait]
pub trait Public {
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
    async fn get_software_version(&self) -> Result<SoftwareVersionData, DracoonClientError>;
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
    async fn get_system_info(&self) -> Result<SystemInfo, DracoonClientError>;
    /// Get public download share information for a DRACOON download share.
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
    /// let access_key = "access_key";
    ///
    /// let public_download_share = dracoon.public.get_public_download_share(access_key.to_string()).await.unwrap();
    ///
    /// # }
    ///
    /// ```
    async fn get_public_download_share(
        &self,
        access_key: impl Into<String> + Send + Sync,
    ) -> Result<PublicDownloadShare, DracoonClientError>;
}

#[async_trait]
pub trait PublicDownload {
    /// Download a file from a public download share.
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
    /// let access_key = "access_key";
    /// let share = dracoon.public.get_public_download_share(access_key.to_string()).await.unwrap();
    /// let password = Some("TopSecret123!".to_string());
    /// 
    /// let mut writer = tokio::io::BufWriter::new(tokio::fs::File::create("test.txt").await.unwrap());
    /// 
    /// dracoon.public.download(access_key.to_string(), share, password, &mut writer, None).await.unwrap();
    /// 
    /// // or with a progress callback
    /// 
    /// dracoon.public.download(access_key.to_string(), share, password, &mut writer, Some(Box::new(|progress| {
    ///    println!("Downloaded: {}%", progress);
    /// }))).await.unwrap();
    /// # }
    /// ```
    async fn download<'w>(
        &'w self,
        access_key: String,
        share: PublicDownloadShare,
        password: Option<String>,
        writer: &'w mut (dyn AsyncWrite + Send + Unpin),
        mut callback: Option<DownloadProgressCallback>,
    ) -> Result<(), DracoonClientError>;
}

#[async_trait]
pub trait PublicUpload<R: AsyncRead> {
    async fn upload<'r>(
        &'r self,
        file_meta: FileMeta,
        share: String,
        upload_options: UploadOptions,
        mut reader: BufReader<R>,
        mut callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<S3ShareUploadStatus, DracoonClientError>;
}

#[async_trait]
impl<S: Send + Sync> Public for PublicEndpoint<S> {
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

    async fn get_public_download_share(
        &self,
        access_key: impl Into<String> + Send + Sync,
    ) -> Result<PublicDownloadShare, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{PUBLIC_BASE}/{PUBLIC_SHARES_BASE}/{PUBLIC_DOWNLOAD_SHARES}/{}",
            access_key.into()
        );

        let url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(url)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        Ok(PublicDownloadShare::from_response(response).await?)
    }
}

#[cfg(test)]
mod tests {

    use chrono::Datelike;
    use dco3_crypto::{FileKeyVersion, UserKeyPairVersion};

    use crate::{tests::dracoon::get_connected_client, Dracoon, Public};

    #[tokio::test]
    async fn test_get_system_info_connected() {
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
    async fn test_get_software_version_connected() {
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

    #[tokio::test]
    async fn test_get_software_version_disconnected() {
        let mut mock_server = mockito::Server::new_async().await;

        let client = Dracoon::builder()
            .with_base_url(mock_server.url())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();

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

    #[tokio::test]
    async fn test_get_system_info_disconnected() {
        let mut mock_server = mockito::Server::new_async().await;

        let client = Dracoon::builder()
            .with_base_url(mock_server.url())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();

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
    async fn test_get_public_download_share_connected() {
        let (client, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let public_download_share_res = include_str!("../tests/responses/public/download_share_ok.json");

        let public_download_share_mock = mock_server
            .mock("GET", "/api/v4/public/shares/downloads/test")
            .with_status(200)
            .with_body(public_download_share_res)
            .with_header("content-type", "application/json")
            .create();

        let public_download_share = client
            .public
            .get_public_download_share("test")
            .await
            .unwrap();

        public_download_share_mock.assert();

        assert_eq!(public_download_share.file_name, "string");
        assert_eq!(public_download_share.size, 123456);
        assert_eq!(public_download_share.creator_name, "string");
        assert_eq!(public_download_share.media_type, "string");
        assert!(public_download_share.is_protected);
        assert!(!public_download_share.limit_reached);
        assert!(public_download_share.has_download_limit);
        assert_eq!(public_download_share.created_at.month(), 1);
        assert_eq!(public_download_share.created_at.day(), 1);
        assert_eq!(public_download_share.created_at.year(), 2021);
        assert_eq!(public_download_share.notes, Some("string".to_string()));
        assert_eq!(public_download_share.expire_at, Some("2021-01-01T00:00:00Z".parse().unwrap()));
        assert_eq!(public_download_share.creator_username, Some("string".to_string()));
        assert_eq!(public_download_share.name, Some("string".to_string()));

        let file_key = public_download_share.file_key.unwrap();
        assert_eq!(file_key.key, "string");
        assert_eq!(file_key.iv, "string");
        assert_eq!(file_key.tag, Some("string".to_string()));
        assert_eq!(file_key.version, FileKeyVersion::RSA4096_AES256GCM);

        let private_key_container = public_download_share.private_key_container.unwrap();
        assert_eq!(private_key_container.version, UserKeyPairVersion::RSA4096);
        assert_eq!(private_key_container.created_by, Some(1));

    }

    #[tokio::test]
    async fn test_get_public_download_share_disconnected() {
        let mut mock_server = mockito::Server::new_async().await;

        let client = Dracoon::builder()
            .with_base_url(mock_server.url())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();

        let public_download_share_res = include_str!("../tests/responses/public/download_share_ok.json");

        let public_download_share_mock = mock_server
            .mock("GET", "/api/v4/public/shares/downloads/test")
            .with_status(200)
            .with_body(public_download_share_res)
            .with_header("content-type", "application/json")
            .create();

        let public_download_share = client
            .public
            .get_public_download_share("test")
            .await
            .unwrap();

        public_download_share_mock.assert();

        assert_eq!(public_download_share.file_name, "string");
        assert_eq!(public_download_share.size, 123456);
        assert_eq!(public_download_share.creator_name, "string");
        assert_eq!(public_download_share.media_type, "string");
        assert!(public_download_share.is_protected);
        assert!(!public_download_share.limit_reached);
        assert!(public_download_share.has_download_limit);
        assert_eq!(public_download_share.created_at.month(), 1);
        assert_eq!(public_download_share.created_at.day(), 1);
        assert_eq!(public_download_share.created_at.year(), 2021);
        assert_eq!(public_download_share.notes, Some("string".to_string()));
        assert_eq!(public_download_share.expire_at, Some("2021-01-01T00:00:00Z".parse().unwrap()));
        assert_eq!(public_download_share.creator_username, Some("string".to_string()));
        assert_eq!(public_download_share.name, Some("string".to_string()));

        let file_key = public_download_share.file_key.unwrap();
        assert_eq!(file_key.key, "string");
        assert_eq!(file_key.iv, "string");
        assert_eq!(file_key.tag, Some("string".to_string()));
        assert_eq!(file_key.version, FileKeyVersion::RSA4096_AES256GCM);

        let private_key_container = public_download_share.private_key_container.unwrap();
        assert_eq!(private_key_container.version, UserKeyPairVersion::RSA4096);
        assert_eq!(private_key_container.created_by, Some(1));
}
}