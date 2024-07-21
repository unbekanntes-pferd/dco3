use std::cmp::min;

use async_trait::async_trait;
use dco3_crypto::{ChunkedEncryption, Decrypter, DracoonCrypto, DracoonRSACrypto};
use futures_util::TryStreamExt;
use reqwest::header::{self, RANGE};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tracing::error;

use crate::{
    constants::{
        DEFAULT_CHUNK_SIZE, DRACOON_API_PREFIX, PUBLIC_BASE, PUBLIC_DOWNLOAD_SHARES,
        PUBLIC_SHARES_BASE,
    },
    nodes::DownloadProgressCallback,
    utils::{build_s3_error, FromResponse},
    DracoonClientError,
};

use super::{
    PublicDownload, PublicDownloadShare, PublicDownloadTokenGenerateRequest,
    PublicDownloadTokenGenerateResponse, PublicEndpoint, PublicShareEncryption,
};

#[async_trait]
impl<S: Send + Sync> PublicDownload for PublicEndpoint<S> {
    async fn download<'w>(
        &'w self,
        access_key: impl Into<String> + Send + Sync,
        share: PublicDownloadShare,
        password: Option<String>,
        writer: &'w mut (dyn AsyncWrite + Send + Unpin),
        callback: Option<DownloadProgressCallback>,
        chunksize: Option<usize>,
    ) -> Result<(), DracoonClientError> {
        if password.is_none() && (share.is_protected || share.is_encrypted.unwrap_or(false)) {
            return Err(DracoonClientError::MissingArgument);
        }

        let download_url = if share.is_protected {
            let Some(ref password) = password else {
                return Err(DracoonClientError::MissingArgument);
            };
            self.generate_download_url(
                access_key.into(),
                PublicDownloadTokenGenerateRequest::new(password),
            )
            .await?
        } else {
            self.generate_download_url(
                access_key.into(),
                PublicDownloadTokenGenerateRequest::default(),
            )
            .await?
        };

        match share.is_encrypted.unwrap_or(false) {
            true => {
                let password = password.ok_or(DracoonClientError::MissingEncryptionSecret)?;
                let file_key = share
                    .file_key
                    .ok_or(DracoonClientError::MissingEncryptionSecret)?;
                let private_key_container = share
                    .private_key_container
                    .ok_or(DracoonClientError::MissingEncryptionSecret)?;

                self.download_encrypted(
                    &download_url.download_url,
                    password,
                    PublicShareEncryption::new(file_key, private_key_container),
                    writer,
                    share.size,
                    callback,
                )
                .await?;
            }
            false => {
                self.download_unencrypted(&download_url.download_url, writer, share.size, callback)
                    .await?;
            }
        }

        Ok(())
    }
}

#[async_trait]
trait PublicDownloadInternal {
    async fn generate_download_url(
        &self,
        access_key: String,
        req: PublicDownloadTokenGenerateRequest,
    ) -> Result<PublicDownloadTokenGenerateResponse, DracoonClientError>;

    async fn download_unencrypted(
        &self,
        url: &str,
        writer: &mut (dyn AsyncWrite + Send + Unpin),
        size: u64,
        mut callback: Option<DownloadProgressCallback>,
    ) -> Result<(), DracoonClientError>;

    async fn download_encrypted(
        &self,
        url: &str,
        password: String,
        encryption_info: PublicShareEncryption,
        writer: &mut (dyn AsyncWrite + Send + Unpin),
        size: u64,
        mut callback: Option<DownloadProgressCallback>,
    ) -> Result<(), DracoonClientError>;
}

#[async_trait]
impl<S: Send + Sync> PublicDownloadInternal for PublicEndpoint<S> {
    async fn generate_download_url(
        &self,
        access_key: String,
        req: PublicDownloadTokenGenerateRequest,
    ) -> Result<PublicDownloadTokenGenerateResponse, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{PUBLIC_BASE}/{PUBLIC_SHARES_BASE}/{PUBLIC_DOWNLOAD_SHARES}/{}",
            access_key
        );

        let url = self.client().build_api_url(&url_part);

        let response = if !req.has_password() {
            self.client()
                .http
                .post(url)
                .header(header::CONTENT_TYPE, "application/json")
                .send()
                .await?
        } else {
            self.client()
                .http
                .post(url)
                .header(header::CONTENT_TYPE, "application/json")
                .json(&req)
                .send()
                .await?
        };

        PublicDownloadTokenGenerateResponse::from_response(response).await
    }

    async fn download_unencrypted(
        &self,
        url: &str,
        writer: &mut (dyn AsyncWrite + Send + Unpin),
        size: u64,
        mut callback: Option<DownloadProgressCallback>,
    ) -> Result<(), DracoonClientError> {
        // offset (in bytes)
        let mut downloaded_bytes = 0u64;

        // loop until all bytes are downloaded
        while downloaded_bytes < size {
            // calculate range
            let start = downloaded_bytes;
            let end = min(start + DEFAULT_CHUNK_SIZE as u64 - 1, size - 1);
            let range = format!("bytes={start}-{end}");

            // get chunk
            let response = self
                .client()
                .http
                .get(url)
                .header(RANGE, range)
                .send()
                .await
                .map_err(|err| {
                    error!("Error while downloading chunk: {}", err);
                    err
                })?;

            // handle error
            if response.error_for_status_ref().is_err() {
                let error = build_s3_error(response).await;
                return Err(error);
            }

            // write chunk to writer
            let mut stream = response.bytes_stream();

            while let Some(chunk) = stream.try_next().await? {
                let len = chunk.len() as u64;
                writer
                    .write_all(&chunk)
                    .await
                    .or(Err(DracoonClientError::IoError))?;
                downloaded_bytes += len;

                // call progress callback if provided
                if let Some(ref mut callback) = callback {
                    callback(len, size);
                }
                if downloaded_bytes >= size {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn download_encrypted(
        &self,
        url: &str,
        password: String,
        encryption_info: PublicShareEncryption,
        writer: &mut (dyn AsyncWrite + Send + Unpin),
        size: u64,
        mut callback: Option<DownloadProgressCallback>,
    ) -> Result<(), DracoonClientError> {
        let plain_private_key =
            DracoonCrypto::decrypt_private_key(&password, &encryption_info.private_key_container)?;
        let plain_key =
            DracoonCrypto::decrypt_file_key(encryption_info.file_key, plain_private_key)?;

        // this is safe, because the maximum size of a file (encrypted) is 60 GB
        #[allow(clippy::cast_possible_truncation)]
        let mut buffer = vec![0u8; size as usize];

        let mut crypter = DracoonCrypto::decrypter(plain_key, &mut buffer)?;

        // offset (in bytes)
        let mut downloaded_bytes = 0u64;

        // loop until all bytes are downloaded
        while downloaded_bytes < size {
            // calculate range
            let start = downloaded_bytes;
            let end = min(start + DEFAULT_CHUNK_SIZE as u64 - 1, size - 1);
            let range = format!("bytes={start}-{end}");

            // get chunk
            let response = self
                .client()
                .http
                .get(url)
                .header(RANGE, range)
                .send()
                .await
                .map_err(|err| {
                    error!("Error while downloading chunk: {}", err);
                    err
                })?;

            // handle error
            if response.error_for_status_ref().is_err() {
                let error = build_s3_error(response).await;
                return Err(error);
            }

            // write chunk to writer
            let mut stream = response.bytes_stream();

            while let Some(chunk) = stream.try_next().await? {
                let len = chunk.len() as u64;

                crypter.update(&chunk)?;
                downloaded_bytes += len;

                // call progress callback if provided
                if let Some(ref mut callback) = callback {
                    callback(len, size);
                }
                if downloaded_bytes >= size {
                    break;
                }
            }
        }

        crypter.finalize()?;

        writer
            .write_all(&buffer)
            .await
            .or(Err(DracoonClientError::IoError))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dco3_crypto::{DracoonCrypto, DracoonRSACrypto, Encrypt};

    use crate::{
        public::{
            download::PublicDownloadInternal, PublicDownloadTokenGenerateRequest,
            PublicShareEncryption,
        },
        Dracoon,
    };

    #[tokio::test]
    async fn test_generate_download_url() {
        let mut mock_server = mockito::Server::new_async().await;

        let client = Dracoon::builder()
            .with_base_url(mock_server.url())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();

        let res = include_str!("../tests/responses/download/download_url_ok.json");

        let url_mock = mock_server
            .mock("POST", "/api/v4/public/shares/downloads/123456")
            .with_status(200)
            .with_body(res)
            .create();

        let url = client
            .public
            .generate_download_url(
                "123456".to_string(),
                PublicDownloadTokenGenerateRequest::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            url.download_url,
            "https://test.dracoon.com/not/real/download_url"
        );
    }

    #[tokio::test]
    async fn test_download_unencrypted() {
        let mut mock_server = mockito::Server::new_async().await;

        // create bytes for mocking byte response
        let mock_bytes: [u8; 16] = [
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ];

        let client = Dracoon::builder()
            .with_base_url(mock_server.url())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();

        let download_mock = mock_server
            .mock("GET", "/some/download/url")
            .with_status(200)
            .with_body(mock_bytes)
            .create();

        let download_url = format!("{}/some/download/url", mock_server.url());

        let buffer = Vec::with_capacity(16);

        let mut writer = tokio::io::BufWriter::new(buffer);

        client
            .public
            .download_unencrypted(&download_url, &mut writer, 16, None)
            .await
            .unwrap();

        download_mock.assert();

        assert_eq!(writer.buffer().len(), 16);

        assert_eq!(writer.buffer(), &mock_bytes.to_vec());
    }

    #[tokio::test]
    async fn test_download_encrypted() {
        let mut mock_server = mockito::Server::new_async().await;

        let dracoon = Dracoon::builder()
            .with_base_url(mock_server.url())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();

        // create bytes for mocking byte response
        let mock_bytes: [u8; 16] = [
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ];
        let mock_bytes_compare = mock_bytes;

        let mock_bytes_encrypted = DracoonCrypto::encrypt(mock_bytes).unwrap();
        let plain_key = mock_bytes_encrypted.1.clone();

        let keypair =
            DracoonCrypto::create_plain_user_keypair(dco3_crypto::UserKeyPairVersion::RSA4096)
                .unwrap();
        let enc_keypair =
            DracoonCrypto::encrypt_private_key("TopSecret1234!", keypair.clone()).unwrap();
        let enc_keypair_json = serde_json::to_string(&enc_keypair).unwrap();
        let file_key = DracoonCrypto::encrypt_file_key(plain_key, keypair).unwrap();

        let file_key_json = serde_json::to_string(&file_key).unwrap();

        let download_mock = mock_server
            .mock("GET", "/some/download/url")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(&mock_bytes_encrypted.0)
            .create();

        let download_url = format!("{}/some/download/url", mock_server.url());

        let secret = "TopSecret1234!";

        let buffer = Vec::with_capacity(16);

        // create a writer
        let mut writer = tokio::io::BufWriter::new(buffer);

        dracoon
            .public
            .download_encrypted(
                &download_url,
                secret.to_string(),
                PublicShareEncryption::new(file_key, enc_keypair.private_key_container),
                &mut writer,
                16,
                None,
            )
            .await
            .unwrap();

        download_mock.assert();

        assert_eq!(writer.buffer(), mock_bytes_compare.to_vec());
    }
}
