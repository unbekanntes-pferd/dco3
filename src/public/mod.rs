mod models;
use async_trait::async_trait;
use reqwest::header;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

use crate::{
    constants::{
        DRACOON_API_PREFIX, PUBLIC_BASE, PUBLIC_DOWNLOAD_SHARES, PUBLIC_INFO, PUBLIC_SHARES_BASE,
        PUBLIC_SOFTWARE_BASE, PUBLIC_SYSTEM_BASE, PUBLIC_UPLOAD_SHARES, PUBLIC_VERSION,
    },
    nodes::{DownloadProgressCallback, UploadOptions, UploadProgressCallback},
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

    /// Get public upload share information for a DRACOON upload share.
    ///
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
    /// let public_upload_share = dracoon.public.get_public_upload_share(access_key.to_string()).await.unwrap();
    ///
    /// # }
    /// ```
    async fn get_public_upload_share(
        &self,
        access_key: impl Into<String> + Send + Sync,
    ) -> Result<PublicUploadShare, DracoonClientError>;
}

#[async_trait]
pub trait PublicDownload {
    /// Download a file from a public download share.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Public, PublicDownload};
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
    /// // the password must be set for encrypted shares and for protected shares
    /// let password = Some("TopSecret123!".to_string());
    ///
    /// let mut writer = tokio::io::BufWriter::new(tokio::fs::File::create("test.txt").await.unwrap());
    ///
    /// dracoon.public.download(access_key.to_string(), share, password, &mut writer, None, None).await.unwrap();
    ///
    /// // or with a progress callback
    /// let share = dracoon.public.get_public_download_share(access_key.to_string()).await.unwrap();
    /// let password = Some("TopSecret123!".to_string());
    /// dracoon.public.download(access_key.to_string(), share, password, &mut writer, Some(Box::new(|progress, total| {
    ///    println!("Downloaded: {}%", progress);
    /// })), None).await.unwrap();
    /// # }
    /// ```
    async fn download<'w>(
        &'w self,
        access_key: impl Into<String> + Send + Sync,
        share: PublicDownloadShare,
        password: Option<String>,
        writer: &'w mut (dyn AsyncWrite + Send + Unpin),
        mut callback: Option<DownloadProgressCallback>,
        chunksize: Option<usize>,
    ) -> Result<(), DracoonClientError>;
}

#[async_trait]
pub trait PublicUpload<R: AsyncRead> {
    async fn upload<'r>(
        &'r self,
        access_key: impl Into<String> + Send + Sync,
        share: PublicUploadShare,
        upload_options: UploadOptions,
        mut reader: BufReader<R>,
        mut callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<FileName, DracoonClientError>;
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

    async fn get_public_upload_share(
        &self,
        access_key: impl Into<String> + Send + Sync,
    ) -> Result<PublicUploadShare, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{PUBLIC_BASE}/{PUBLIC_SHARES_BASE}/{PUBLIC_UPLOAD_SHARES}/{}",
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

        Ok(PublicUploadShare::from_response(response).await?)
    }
}

#[cfg(test)]
mod tests {

    use chrono::Datelike;
    use dco3_crypto::{
        DracoonCrypto, DracoonRSACrypto, Encrypt, FileKeyVersion, UserKeyPairVersion,
    };

    use crate::{
        nodes::{FileMeta, UploadOptions},
        public::{PublicDownloadTokenGenerateRequest, PublicUpload},
        tests::dracoon::get_connected_client,
        Dracoon, Public, PublicDownload,
    };

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

        let public_download_share_res =
            include_str!("../tests/responses/public/download_share_ok.json");

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
        assert_eq!(
            public_download_share.creator_name,
            Some("string".to_string())
        );
        assert_eq!(public_download_share.media_type, "string");
        assert!(public_download_share.is_protected);
        assert!(!public_download_share.limit_reached);
        assert!(public_download_share.has_download_limit);
        assert_eq!(public_download_share.created_at.month(), 1);
        assert_eq!(public_download_share.created_at.day(), 1);
        assert_eq!(public_download_share.created_at.year(), 2021);
        assert_eq!(public_download_share.notes, Some("string".to_string()));
        assert_eq!(
            public_download_share.expire_at,
            Some("2021-01-01T00:00:00Z".parse().unwrap())
        );
        assert_eq!(
            public_download_share.creator_username,
            Some("string".to_string())
        );
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

        let public_download_share_res =
            include_str!("../tests/responses/public/download_share_ok.json");

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
        assert_eq!(
            public_download_share.creator_name,
            Some("string".to_string())
        );
        assert_eq!(public_download_share.media_type, "string");
        assert!(public_download_share.is_protected);
        assert!(!public_download_share.limit_reached);
        assert!(public_download_share.has_download_limit);
        assert_eq!(public_download_share.created_at.month(), 1);
        assert_eq!(public_download_share.created_at.day(), 1);
        assert_eq!(public_download_share.created_at.year(), 2021);
        assert_eq!(public_download_share.notes, Some("string".to_string()));
        assert_eq!(
            public_download_share.expire_at,
            Some("2021-01-01T00:00:00Z".parse().unwrap())
        );
        assert_eq!(
            public_download_share.creator_username,
            Some("string".to_string())
        );
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
    async fn test_public_download_unencrypted() {
        let mut mock_server = mockito::Server::new_async().await;

        let client = Dracoon::builder()
            .with_base_url(mock_server.url())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();

        let public_download_share_res =
            include_str!("../tests/responses/public/download_share_ok_templated.json");

        // must be unencrypted
        let public_download_share_res =
            public_download_share_res.replace(r#""$ENCRYPTED""#, "false");

        // size must be 16 (mock_bytes below)
        let public_download_share_res = public_download_share_res.replace(r#""$SIZE""#, "16");

        // remove private key container
        let public_download_share_res = public_download_share_res
            .replace(r#""privateKeyContainer": "$PRIVATE_KEY_CONTAINER","#, "");

        // remove file key
        let public_download_share_res =
            public_download_share_res.replace(r#""fileKey": "$FILE_KEY","#, "");

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

        let mock_bytes = b"testtesttesttest".to_vec();
        let mock_compare = mock_bytes.clone();

        let buffer = Vec::with_capacity(16);

        let mut writer = tokio::io::BufWriter::new(buffer);

        let download_url_res = r#"{"downloadUrl": "/some/download/url"}"#;
        let download_url_res = download_url_res.replace(
            "/some/download/url",
            format!("{}/some/download/url", mock_server.url()).as_str(),
        );

        let payload = PublicDownloadTokenGenerateRequest::new("TopSecret1234!");
        let payload = serde_json::to_string(&payload).unwrap();

        let url_mock = mock_server
            .mock("POST", "/api/v4/public/shares/downloads/test")
            .with_status(200)
            .with_body(download_url_res)
            .match_body(&*payload)
            .create();

        let download_mock = mock_server
            .mock("GET", "/some/download/url")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(mock_bytes)
            .create();

        client
            .public
            .download(
                "test",
                public_download_share,
                Some("TopSecret1234!".to_string()),
                &mut writer,
                None,
                None,
            )
            .await
            .unwrap();

        url_mock.assert();
        download_mock.assert();

        assert_eq!(writer.buffer().len(), 16);

        assert_eq!(writer.buffer(), mock_compare);
    }

    #[tokio::test]
    async fn test_public_download_encrypted() {
        let mut mock_server = mockito::Server::new_async().await;

        let client = Dracoon::builder()
            .with_base_url(mock_server.url())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();

        let public_download_share_res =
            include_str!("../tests/responses/public/download_share_ok_templated.json");

        // size must be 16 (mock_bytes below)
        let public_download_share_res =
            public_download_share_res.replace(r#""size": 123456"#, r#""size": 16"#);

        let mock_bytes = b"testtesttesttest".to_vec();
        let mock_compare = mock_bytes.clone();
        let mock_bytes_encrypted = DracoonCrypto::encrypt(mock_bytes).unwrap();

        let plain_key = mock_bytes_encrypted.1.clone();

        let keypair =
            DracoonCrypto::create_plain_user_keypair(dco3_crypto::UserKeyPairVersion::RSA4096)
                .unwrap();
        let enc_keypair =
            DracoonCrypto::encrypt_private_key("TopSecret1234!", keypair.clone()).unwrap();

        let private_key_container = enc_keypair.private_key_container.clone();
        let private_key_json = serde_json::to_string(&private_key_container).unwrap();
        let file_key = DracoonCrypto::encrypt_file_key(plain_key, keypair).unwrap();
        let file_key_json = serde_json::to_string(&file_key).unwrap();

        // must be encrypted
        let public_download_share_res =
            public_download_share_res.replace(r#""$ENCRYPTED""#, "true");

        // size must be 16 (mock_bytes below)
        let public_download_share_res = public_download_share_res.replace(r#""$SIZE""#, "16");

        // inject private key container
        let public_download_share_res =
            public_download_share_res.replace(r#""$PRIVATE_KEY_CONTAINER""#, &private_key_json);

        // inject file key
        let public_download_share_res =
            public_download_share_res.replace(r#""$FILE_KEY""#, &file_key_json);

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

        let buffer = Vec::with_capacity(16);

        let mut writer = tokio::io::BufWriter::new(buffer);

        let download_url_res = r#"{"downloadUrl": "/some/download/url"}"#;
        let download_url_res = download_url_res.replace(
            "/some/download/url",
            format!("{}/some/download/url", mock_server.url()).as_str(),
        );

        let url_mock = mock_server
            .mock("POST", "/api/v4/public/shares/downloads/test")
            .with_status(200)
            .with_body(download_url_res)
            .create();

        let download_mock = mock_server
            .mock("GET", "/some/download/url")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(mock_bytes_encrypted.0)
            .create();

        client
            .public
            .download(
                "test",
                public_download_share,
                Some("TopSecret1234!".to_string()),
                &mut writer,
                None,
                None,
            )
            .await
            .unwrap();

        url_mock.assert();
        download_mock.assert();

        assert_eq!(writer.buffer().len(), 16);

        assert_eq!(writer.buffer(), mock_compare);
    }

    #[tokio::test]
    #[ignore = "not needed in CI (only for manual testing)"]
    async fn test_download_unencrypted_staging() {
        let access_key = "aXBpSgdf8pP8yV2axsruGrtt8f81Fbfo";
        let expected_content = b"Blububububbububu";

        let client = Dracoon::builder()
            .with_base_url("https://staging.dracoon.com")
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();

        let public_download_share = client
            .public
            .get_public_download_share(access_key)
            .await
            .unwrap();

        let buffer = Vec::with_capacity(16);

        let mut writer = tokio::io::BufWriter::new(buffer);
        let password = "Test1234!".to_string();

        client
            .public
            .download(
                access_key,
                public_download_share,
                Some(password),
                &mut writer,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(writer.buffer().len(), 16);
        assert_eq!(writer.buffer(), expected_content);
    }

    #[tokio::test]
    #[ignore = "not needed in CI (only for manual testing)"]
    async fn test_download_encrypted_staging() {
        let access_key = "Nyg9kGoBaQzPfK0pTz10AifibzrQues4";
        let expected_content = b"Blububububbububu";

        let client = Dracoon::builder()
            .with_base_url("https://staging.dracoon.com")
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();

        let public_download_share = client
            .public
            .get_public_download_share(access_key)
            .await
            .unwrap();

        let buffer = Vec::with_capacity(16);

        let mut writer = tokio::io::BufWriter::new(buffer);
        let password = "Test1234!".to_string();

        client
            .public
            .download(
                access_key,
                public_download_share,
                Some(password),
                &mut writer,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(writer.buffer().len(), 16);
        assert_eq!(writer.buffer(), expected_content);
    }

    #[tokio::test]
    async fn test_get_public_upload_share_disconnected() {
        let mut mock_server = mockito::Server::new_async().await;

        let client = Dracoon::builder()
            .with_base_url(mock_server.url())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();

        let public_upload_share_res =
            include_str!("../tests/responses/public/upload_share_ok.json");

        let public_upload_share_mock = mock_server
            .mock("GET", "/api/v4/public/shares/uploads/test")
            .with_status(200)
            .with_body(public_upload_share_res)
            .with_header("content-type", "application/json")
            .create();

        let public_upload_share = client.public.get_public_upload_share("test").await.unwrap();

        public_upload_share_mock.assert();

        assert!(public_upload_share.is_protected);
        assert_eq!(public_upload_share.created_at.month(), 1);
        assert_eq!(public_upload_share.created_at.day(), 1);
        assert_eq!(public_upload_share.created_at.year(), 2021);
        assert_eq!(public_upload_share.name, Some("string".to_string()));
        assert_eq!(public_upload_share.notes, Some("string".to_string()));
        assert_eq!(
            public_upload_share.expire_at,
            Some("2021-01-01T00:00:00Z".parse().unwrap())
        );
        assert_eq!(public_upload_share.show_uploaded_files, Some(true));
        assert_eq!(public_upload_share.remaining_size, None);
        assert_eq!(public_upload_share.remaining_slots, Some(1));
        assert_eq!(public_upload_share.is_encrypted, Some(false));
        assert_eq!(public_upload_share.is_protected, true);
        let uploaded_files = public_upload_share.uploaded_files.unwrap();
        assert_eq!(uploaded_files.len(), 1);
        let uploaded_file = uploaded_files.first().unwrap();
        assert_eq!(uploaded_file.name, "string");
        assert_eq!(uploaded_file.size, 16);
    }

    #[tokio::test]
    #[ignore = "not needed in CI (only for manual testing)"]
    async fn test_upload_unencrypted_staging() {
        let access_key = "Eb41f2Ac8nxhln99ddrp9KyIhlO5fwoi";

        let client = Dracoon::builder()
            .with_base_url("https://staging.dracoon.com")
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();

        let public_upload_share = client
            .public
            .get_public_upload_share(access_key)
            .await
            .unwrap();

        let mock_bytes = b"Blububububbububu";

        let reader = tokio::io::BufReader::new(mock_bytes.as_slice());

        let file_meta = FileMeta::builder()
            .with_name("test.txt".to_string())
            .with_size(16)
            .build();

        let upload_opts = UploadOptions::builder(file_meta).build();

        let file_name = client
            .public
            .upload(
                access_key,
                public_upload_share,
                upload_opts,
                reader,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(file_name, "test.txt");
    }

    #[tokio::test]
    #[ignore = "not needed in CI (only for manual testing)"]
    async fn test_upload_encrypted_staging() {
        let access_key = "xwHSQhIpdtXXLEnNbkwLDCUQuIpoLSCn";

        let client = Dracoon::builder()
            .with_base_url("https://staging.dracoon.com")
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();

        let public_upload_share = client
            .public
            .get_public_upload_share(access_key)
            .await
            .unwrap();

        let mock_bytes = b"Blububububbububu";

        let reader = tokio::io::BufReader::new(mock_bytes.as_slice());

        let file_meta = FileMeta::builder()
            .with_name("test.txt".to_string())
            .with_size(16)
            .build();

        let upload_opts = UploadOptions::builder(file_meta).build();

        let file_name = client
            .public
            .upload(
                access_key,
                public_upload_share,
                upload_opts,
                reader,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(file_name, "test.txt");
    }
}
