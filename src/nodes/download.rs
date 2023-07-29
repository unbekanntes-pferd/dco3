use super::{
    models::{DownloadUrlResponse, Node, DownloadProgressCallback},
    Download,
};
use crate::{
    auth::{errors::DracoonClientError, Connected},
    constants::{
        CHUNK_SIZE, DRACOON_API_PREFIX, FILES_BASE, FILES_FILE_KEY, NODES_BASE, NODES_DOWNLOAD_URL,
    },
    utils::{build_s3_error, FromResponse},
    Dracoon,
};
use async_trait::async_trait;
use dco3_crypto::{FileKey, DracoonCrypto, DracoonRSACrypto, Decrypter, ChunkedEncryption};
use futures_util::TryStreamExt;
use reqwest::header::{self, CONTENT_LENGTH, RANGE};
use std::{cmp::min, io::Write};
use tracing::debug;

#[async_trait]
impl Download for Dracoon<Connected> {
    async fn download<'w>(
        &'w self,
        node: &Node,
        writer: &'w mut (dyn Write + Send),
        callback: Option<DownloadProgressCallback>,
    ) -> Result<(), DracoonClientError> {
        let download_url_response = self.get_download_url(node.id).await?;

        match node.is_encrypted {
            Some(encrypted) => {
                if encrypted {
                    self.download_encrypted(
                        &download_url_response.download_url,
                        node.id,
                        writer,
                        node.size,
                        callback,
                    )
                    .await
                } else {
                    self.download_unencrypted(
                        &download_url_response.download_url,
                        writer,
                        node.size,
                        callback,
                    )
                    .await
                }
            }
            None => {
                self.download_unencrypted(
                    &download_url_response.download_url,
                    writer,
                    node.size,
                    callback,
                )
                .await
            }
        }
    }
}

#[async_trait]
trait DownloadInternal {
    async fn get_download_url(
        &self,
        node_id: u64,
    ) -> Result<DownloadUrlResponse, DracoonClientError>;

    async fn get_file_key(&self, node_id: u64) -> Result<FileKey, DracoonClientError>;

    async fn download_unencrypted(
        &self,
        url: &str,
        writer: &mut (dyn Write + Send),
        size: Option<u64>,
        mut callback: Option<DownloadProgressCallback>,
    ) -> Result<(), DracoonClientError>;

    async fn download_encrypted(
        &self,
        url: &str,
        node_id: u64,
        writer: &mut (dyn Write + Send),
        size: Option<u64>,
        mut callback: Option<DownloadProgressCallback>,
    ) -> Result<(), DracoonClientError>;
}

#[async_trait]
impl DownloadInternal for Dracoon<Connected> {
    async fn get_download_url(
        &self,
        node_id: u64,
    ) -> Result<DownloadUrlResponse, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{NODES_BASE}/{FILES_BASE}/{node_id}/{NODES_DOWNLOAD_URL}"
        );


        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .post(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        DownloadUrlResponse::from_response(response).await
    }

    async fn download_unencrypted(
        &self,
        url: &str,
        writer: &mut (dyn Write + Send),
        size: Option<u64>,
        mut callback: Option<DownloadProgressCallback>,
    ) -> Result<(), DracoonClientError> {
        // get content length from header
        let content_length = self
            .client
            .http
            .head(url)
            .send()
            .await?
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse().ok())
            .unwrap_or(0);

        // if size is given, use it
        let content_length = size.unwrap_or(content_length);

        // offset (in bytes)
        let mut downloaded_bytes = 0u64;

        debug!("Content length: {}", content_length);

        // loop until all bytes are downloaded
        while downloaded_bytes < content_length {
            // calculate range
            let start = downloaded_bytes;
            let end = min(start + CHUNK_SIZE as u64 - 1, content_length - 1);
            let range = format!("bytes={start}-{end}");

            // get chunk
            let response = self
                .client
                .http
                .get(url)
                .header(RANGE, range)
                .send()
                .await?;

            // handle error
            if response.error_for_status_ref().is_err() {
                let error = build_s3_error(response).await;
                return Err(error);
            }

            // write chunk to writer
            let mut stream = response.bytes_stream();

            while let Some(chunk) = stream.try_next().await? {
                let len = chunk.len() as u64;
                writer.write_all(&chunk).or(Err(DracoonClientError::IoError))?;
                downloaded_bytes += len;

                // call progress callback if provided
                if let Some(ref mut callback) = callback {
                    callback(len, content_length);
                }
                if downloaded_bytes >= content_length {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn download_encrypted(
        &self,
        url: &str,
        node_id: u64,
        writer: &mut (dyn Write + Send),
        size: Option<u64>,
        mut callback: Option<DownloadProgressCallback>,
    ) -> Result<(), DracoonClientError> {
        // get file key
        let file_key = self.get_file_key(node_id).await?;

        let keypair = self.get_keypair(None).await?;
 
        let plain_key = DracoonCrypto::decrypt_file_key(file_key, &keypair)?;


        // get content length from header
        let content_length = self
            .client
            .http
            .head(url)
            .send()
            .await?
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse().ok())
            .unwrap_or(0);

        // if size is given, use it
        let content_length = size.unwrap_or(content_length);

        // this is safe, because the maximum size of a file (encrypted) is 60 GB
        #[allow(clippy::cast_possible_truncation)]
        let mut buffer = vec![0u8; content_length as usize];

        let mut crypter = DracoonCrypto::decrypter(plain_key, &mut buffer)?;


        // offset (in bytes)
        let mut downloaded_bytes = 0u64;

        debug!("Content length: {}", content_length);

        // loop until all bytes are downloaded
        while downloaded_bytes < content_length {
            // calculate range
            let start = downloaded_bytes;
            let end = min(start + CHUNK_SIZE as u64 - 1, content_length - 1);
            let range = format!("bytes={start}-{end}");

            // get chunk
            let response = self
                .client
                .http
                .get(url)
                .header(RANGE, range)
                .send()
                .await?;

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
                    callback(len, content_length);
                }
                if downloaded_bytes >= content_length {
                    break;
                }
            }

        }

        crypter.finalize()?;

        writer.write_all(&buffer).or(Err(DracoonClientError::IoError))?;
        Ok(())
    }

    async fn get_file_key(&self, node_id: u64) -> Result<FileKey, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{NODES_BASE}/{FILES_BASE}/{node_id}/{FILES_FILE_KEY}"
        );

        let response = self
            .client
            .http
            .get(self.build_api_url(&url_part))
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .send()
            .await?;

        FileKey::from_response(response).await
    }
}

#[cfg(test)]
mod tests {
    // separate from test folder due to internal trait (DownloadInternal)

    use dco3_crypto::Encrypt;

    use super::*;

    use crate::tests::dracoon::get_connected_client;

    #[tokio::test]
    async fn test_get_download_url() {

        let download_url_str = "https://test.dracoon.com/not/real/download_url";

        let (dracoon, mut mock_server) = get_connected_client().await;

        let download_url_res = include_str!("../tests/responses/download/download_url_ok.json");

        let download_url_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/1234/downloads")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(download_url_res)
            .create();

        let download_url = dracoon.get_download_url(1234).await.unwrap();

        download_url_mock.assert();

        assert_eq!(download_url.download_url, download_url_str);

    }

    #[tokio::test]
    async fn test_get_file_key() {

        let (dracoon, mut mock_server) = get_connected_client().await;

        let file_key_res = include_str!("../tests/responses/download/file_key_ok.json");

        let file_key_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/1234/user_file_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(file_key_res)
            .create();

        let file_key = dracoon.get_file_key(1234).await.unwrap();

        file_key_mock.assert();

        assert_eq!(file_key.key, "string");
        assert_eq!(file_key.iv, "string");
        assert!(file_key.tag.is_some());
        assert_eq!(file_key.tag.unwrap(), "string");
        // TODO: implement PartialEq for FileKeyVersion
        // assert_eq!(file_key.version, FileKeyVersion::RSA4096_AES256GCM);


    }
    
    #[tokio::test]
    async fn test_download_unencrypted() {

        let (dracoon, mut mock_server) = get_connected_client().await;

        let content_length_mock = mock_server
            .mock("HEAD", "/some/download/url")
            .with_status(200)
            .with_header("content-length", "16")
            .create();

        // create bytes for mocking byte response
        let mock_bytes: [u8; 16] = [0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44];

        let download_mock = mock_server
            .mock("GET", "/some/download/url")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(&mock_bytes)
            .create();

        let download_url = format!("{}some/download/url",dracoon.get_base_url().to_string());

        let buffer = Vec::with_capacity(16);

        // create a writer 
        let mut writer = std::io::BufWriter::new(buffer);

        dracoon.download_unencrypted(&download_url, &mut writer, Some(16), None).await.unwrap();

        content_length_mock.assert();

        download_mock.assert();

        assert_eq!(writer.into_inner().unwrap(), mock_bytes.to_vec());


    }

    #[tokio::test]
    async fn test_download_encrypted() {

        let (dracoon, mut mock_server) = get_connected_client().await;

        let content_length_mock = mock_server
            .mock("HEAD", "/some/download/url")
            .with_status(200)
            .with_header("content-length", "16")
            .create();

        // create bytes for mocking byte response
        let mock_bytes: [u8; 16] = [0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44];
        let mock_bytes_compare = mock_bytes.clone();

        let mock_bytes_encrypted = DracoonCrypto::encrypt(mock_bytes.to_vec()).unwrap();
        let plain_key = mock_bytes_encrypted.1.clone();

        let keypair = DracoonCrypto::create_plain_user_keypair(dco3_crypto::UserKeyPairVersion::RSA4096).unwrap();
        let enc_keypair = DracoonCrypto::encrypt_private_key("TopSecret1234!", keypair.clone()).unwrap();
        let enc_keypair_json = serde_json::to_string(&enc_keypair).unwrap();
        let file_key = DracoonCrypto::encrypt_file_key(plain_key, keypair).unwrap();

        let file_key_json = serde_json::to_string(&file_key).unwrap();


        let download_mock = mock_server
            .mock("GET", "/some/download/url")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(&mock_bytes_encrypted.0)
            .create();

        let file_key_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/1234/user_file_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(file_key_json)
            .create();

        let keypair_mock = mock_server
            .mock("GET", "/api/v4/user/account/keypair")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(enc_keypair_json)
            .create();

        let download_url = format!("{}some/download/url",dracoon.get_base_url().to_string());

        let _kp  = dracoon.get_keypair(Some("TopSecret1234!".into())).await.unwrap();

        let buffer = Vec::with_capacity(16);

        // create a writer 
        let mut writer = std::io::BufWriter::new(buffer);

        dracoon.download_encrypted(&download_url, 1234, &mut writer, None, None).await.unwrap();

        keypair_mock.assert();

        content_length_mock.assert();

        download_mock.assert();

        file_key_mock.assert();

        assert_eq!(writer.into_inner().unwrap(), mock_bytes_compare.to_vec());

    }

    #[tokio::test]
    async fn test_download_encrypted_no_keypair() {


        let (dracoon, mut mock_server) = get_connected_client().await;

        // create bytes for mocking byte response
        let mock_bytes: [u8; 16] = [0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44];

        let mock_bytes_encrypted = DracoonCrypto::encrypt(mock_bytes.to_vec()).unwrap();
        let plain_key = mock_bytes_encrypted.1.clone();

        let keypair = DracoonCrypto::create_plain_user_keypair(dco3_crypto::UserKeyPairVersion::RSA4096).unwrap();
        let file_key = DracoonCrypto::encrypt_file_key(plain_key, keypair).unwrap();

        let file_key_json = serde_json::to_string(&file_key).unwrap();

        let file_key_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/1234/user_file_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(file_key_json)
            .create();

        let download_url = format!("{}some/download/url",dracoon.get_base_url().to_string());

        let buffer = Vec::with_capacity(16);

        // create a writer 
        let mut writer = std::io::BufWriter::new(buffer);

        let download_res = dracoon.download_encrypted(&download_url, 1234, &mut writer, None, None).await;

        assert!(download_res.is_err());
        // TODO: implement PartialEq for DracoonClientError
        // assert_eq!(err, DracoonClientError::MissingEncryptionSecret);

    }

    async fn test_download_unencrypted_node() {
        todo!()

    }

    async fn test_download_full_encrypted_node() {
        todo!()
    }




}