use super::{
    models::{DownloadProgressCallback, DownloadUrlResponse, Node},
    Download,
};
use crate::{
    client::{errors::DracoonClientError, Connected},
    constants::{
        DEFAULT_DOWNLOAD_CHUNK_SIZE, DRACOON_API_PREFIX, FILES_BASE, FILES_FILE_KEY, NODES_BASE,
        NODES_DOWNLOAD_URL,
    },
    utils::{build_s3_error, build_s3_protocol_error, FromResponse},
    Dracoon,
};
use async_trait::async_trait;
use dco3_crypto::{DracoonCrypto, FileKey};
use futures_util::TryStreamExt;
use reqwest::{
    header::{self, CONTENT_RANGE, RANGE},
    Response, StatusCode,
};
use std::cmp::min;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tracing::{debug, error};

#[async_trait]
impl Download for Dracoon<Connected> {
    async fn download<'w>(
        &'w self,
        node: &Node,
        writer: &'w mut (dyn AsyncWrite + Send + Unpin),
        callback: Option<DownloadProgressCallback>,
        chunksize: Option<usize>,
    ) -> Result<(), DracoonClientError> {
        if node.is_encrypted.unwrap_or(false) {
            self.download_encrypted(node, writer, callback, chunksize)
                .await
        } else {
            self.download_unencrypted(node, writer, callback, chunksize)
                .await
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
        node: &Node,
        writer: &mut (dyn AsyncWrite + Send + Unpin),
        mut callback: Option<DownloadProgressCallback>,
        chunksize: Option<usize>,
    ) -> Result<(), DracoonClientError>;

    async fn download_encrypted(
        &self,
        node: &Node,
        writer: &mut (dyn AsyncWrite + Send + Unpin),
        mut callback: Option<DownloadProgressCallback>,
        chunksize: Option<usize>,
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
        node: &Node,
        writer: &mut (dyn AsyncWrite + Send + Unpin),
        mut callback: Option<DownloadProgressCallback>,
        chunksize: Option<usize>,
    ) -> Result<(), DracoonClientError> {
        let url = self.get_download_url(node.id).await?.download_url;

        // if size is given, use it
        let content_length = match node.size {
            Some(size) => size,
            None => probe_content_length(&self.client.http, &url).await?,
        };

        // offset (in bytes)
        let mut downloaded_bytes = 0u64;
        let chunksize = chunksize.unwrap_or(DEFAULT_DOWNLOAD_CHUNK_SIZE);

        debug!("Content length: {}", content_length);

        // loop until all bytes are downloaded
        let mut range_guard = RangeProgressGuard::default();
        while downloaded_bytes < content_length {
            range_guard.check(downloaded_bytes)?;
            let url = if downloaded_bytes == 0 {
                url.clone()
            } else {
                self.get_download_url(node.id).await?.download_url
            };

            // calculate range
            let start = downloaded_bytes;
            let end = min(start + chunksize as u64 - 1, content_length - 1);
            let range = format!("bytes={start}-{end}");

            // get chunk
            let response = self
                .client
                .http
                .get(url)
                .header(RANGE, range)
                .send()
                .await
                .map_err(|err| {
                    error!("Error while downloading chunk: {}", err);
                    err
                })?;

            let response =
                check_range_response(response, content_length > chunksize as u64).await?;

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
        node: &Node,
        writer: &mut (dyn AsyncWrite + Send + Unpin),
        mut callback: Option<DownloadProgressCallback>,
        chunksize: Option<usize>,
    ) -> Result<(), DracoonClientError> {
        // get file key
        let file_key = self.get_file_key(node.id).await?;

        let keypair = self.get_keypair(None).await?;

        let url = self.get_download_url(node.id).await?.download_url;

        // if size is given, use it
        let content_length = match node.size {
            Some(size) => size,
            None => probe_content_length(&self.client.http, &url).await?,
        };

        let mut crypter = DracoonCrypto::file_decryptor(file_key, keypair)?;

        // offset (in bytes)
        let mut downloaded_bytes = 0u64;
        let chunksize = chunksize.unwrap_or(DEFAULT_DOWNLOAD_CHUNK_SIZE);

        debug!("Content length: {}", content_length);

        // loop until all bytes are downloaded
        let mut range_guard = RangeProgressGuard::default();
        while downloaded_bytes < content_length {
            range_guard.check(downloaded_bytes)?;
            let url = if downloaded_bytes == 0 {
                url.clone()
            } else {
                self.get_download_url(node.id).await?.download_url
            };
            // calculate range
            let start = downloaded_bytes;
            let end = min(start + chunksize as u64 - 1, content_length - 1);
            let range = format!("bytes={start}-{end}");

            // get chunk
            let response = self
                .client
                .http
                .get(url)
                .header(RANGE, range)
                .send()
                .await
                .map_err(|err| {
                    error!("Error while downloading chunk: {}", err);
                    err
                })?;

            let response =
                check_range_response(response, content_length > chunksize as u64).await?;

            // write chunk to writer
            let mut stream = response.bytes_stream();

            while let Some(chunk) = stream.try_next().await? {
                let len = chunk.len() as u64;
                let plain = crypter.update(&chunk)?;

                writer
                    .write_all(&plain)
                    .await
                    .or(Err(DracoonClientError::IoError))?;

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

        let final_chunk = crypter.finalize()?;
        writer
            .write_all(&final_chunk)
            .await
            .or(Err(DracoonClientError::IoError))?;
        Ok(())
    }

    async fn get_file_key(&self, node_id: u64) -> Result<FileKey, DracoonClientError> {
        let url_part =
            format!("{DRACOON_API_PREFIX}/{NODES_BASE}/{FILES_BASE}/{node_id}/{FILES_FILE_KEY}");

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

/// Determines the size of a download via a `Range: bytes=0-0` request (`Content-Range: bytes 0-0/<size>`)
async fn probe_content_length(
    http: &reqwest_middleware::ClientWithMiddleware,
    url: &str,
) -> Result<u64, DracoonClientError> {
    let response = http
        .get(url)
        .header(RANGE, "bytes=0-0")
        .send()
        .await
        .map_err(|err| {
            debug!("Error while getting content length: {}", err);
            err
        })?;

    let status = response.status();
    if !status.is_success() {
        return Err(build_s3_error(response).await);
    }

    response
        .headers()
        .get(CONTENT_RANGE)
        .and_then(|val| val.to_str().ok())
        .and_then(|val| val.rsplit('/').next())
        .and_then(|len| len.parse().ok())
        .ok_or_else(|| {
            error!("Download size unknown: no valid Content-Range in probe response");
            build_s3_protocol_error(
                status,
                "unknown_content_length",
                "Download size could not be determined (missing Content-Range)",
            )
        })
}

/// Checks the response to a ranged download request.
/// If the file is downloaded in several ranges, the server must answer with `206 Partial Content` -
/// a server ignoring `Range` would otherwise make us write the content several times.
pub(crate) async fn check_range_response(
    response: Response,
    multiple_ranges: bool,
) -> Result<Response, DracoonClientError> {
    let status = response.status();
    if !status.is_success() {
        return Err(build_s3_error(response).await);
    }

    if multiple_ranges && status != StatusCode::PARTIAL_CONTENT {
        error!(
            "Expected partial content for ranged download, got {}",
            status
        );
        return Err(build_s3_protocol_error(
            status,
            "range_not_supported",
            "Download server ignored the requested byte range",
        ));
    }

    Ok(response)
}

/// Fails a download loop if a range request did not make any progress (empty response body),
/// instead of requesting the same range forever.
#[derive(Default)]
pub(crate) struct RangeProgressGuard {
    last_offset: Option<u64>,
}

impl RangeProgressGuard {
    pub(crate) fn check(&mut self, offset: u64) -> Result<(), DracoonClientError> {
        if self.last_offset == Some(offset) {
            error!("Download made no progress at offset {}", offset);
            return Err(build_s3_protocol_error(
                StatusCode::BAD_GATEWAY,
                "empty_range_response",
                "Download server returned no data for the requested byte range",
            ));
        }
        self.last_offset = Some(offset);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // separate from test folder due to internal trait (DownloadInternal)

    use std::pin::Pin;
    use std::task::{Context, Poll};

    use dco3_crypto::{DracoonRSACrypto, FileKeyVersion};
    use tokio::io::AsyncWrite;

    use super::*;

    use crate::tests::dracoon::get_connected_client;

    #[derive(Default)]
    struct RecordingAsyncWriter {
        writes: Vec<Vec<u8>>,
    }

    impl RecordingAsyncWriter {
        fn flattened(&self) -> Vec<u8> {
            self.writes.iter().flatten().copied().collect()
        }
    }

    impl AsyncWrite for RecordingAsyncWriter {
        fn poll_write(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            self.writes.push(buf.to_vec());
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    fn encrypt_streaming(plaintext: impl AsRef<[u8]>) -> (Vec<u8>, dco3_crypto::PlainFileKey) {
        let mut encryptor = DracoonCrypto::file_encryptor().unwrap();
        let mut ciphertext = encryptor.update(plaintext.as_ref()).unwrap();
        let finalized = encryptor.finalize().unwrap();
        ciphertext.extend_from_slice(&finalized.final_chunk);
        (ciphertext, finalized.plain_file_key)
    }

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
        assert_eq!(file_key.version, FileKeyVersion::RSA4096_AES256GCM);
    }

    #[tokio::test]
    async fn test_download_unencrypted() {
        let (dracoon, mut mock_server) = get_connected_client().await;

        // create bytes for mocking byte response
        let mock_bytes: [u8; 16] = [
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ];

        let download_mock = mock_server
            .mock("GET", "/some/download/url")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(mock_bytes)
            .create();

        let download_url = format!("{}some/download/url", dracoon.get_base_url());

        let buffer = Vec::with_capacity(16);

        let mut writer = tokio::io::BufWriter::new(buffer);

        let node_json = include_str!("../tests/responses/nodes/node_ok.json");
        let node: Node = serde_json::from_str(node_json).unwrap();

        let download_url_res =
            include_str!("../tests/responses/download/download_url_ok_template.json");
        let download_url_res = download_url_res.replace("$url", &download_url);

        let download_url_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/2/downloads")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(download_url_res)
            .create();

        dracoon
            .download_unencrypted(&node, &mut writer, None, None)
            .await
            .unwrap();

        download_mock.assert();

        assert_eq!(writer.buffer(), mock_bytes.to_vec());
    }

    #[tokio::test]
    async fn test_download_encrypted() {
        let (dracoon, mut mock_server) = get_connected_client().await;

        // create bytes for mocking byte response
        let mock_bytes: [u8; 16] = [
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ];
        let mock_bytes_compare = mock_bytes;

        let mock_bytes_encrypted = encrypt_streaming(mock_bytes);
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

        let file_key_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/2/user_file_key")
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

        let download_url = format!("{}some/download/url", dracoon.get_base_url());

        let _kp = dracoon
            .get_keypair(Some("TopSecret1234!".into()))
            .await
            .unwrap();

        let buffer = Vec::with_capacity(16);

        // create a writer
        let mut writer = tokio::io::BufWriter::new(buffer);

        let node_json = include_str!("../tests/responses/nodes/node_ok.json");
        let node: Node = serde_json::from_str(node_json).unwrap();

        let download_url_res =
            include_str!("../tests/responses/download/download_url_ok_template.json");
        let download_url_res = download_url_res.replace("$url", &download_url);

        let download_url_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/2/downloads")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(download_url_res)
            .create();

        dracoon
            .download_encrypted(&node, &mut writer, None, None)
            .await
            .unwrap();

        keypair_mock.assert();

        download_mock.assert();

        file_key_mock.assert();

        assert_eq!(writer.buffer(), mock_bytes_compare.to_vec());
    }

    #[tokio::test]
    async fn test_download_encrypted_no_keypair() {
        let (dracoon, mut mock_server) = get_connected_client().await;

        // create bytes for mocking byte response
        let mock_bytes: [u8; 16] = [
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ];

        let mock_bytes_encrypted = encrypt_streaming(mock_bytes);
        let plain_key = mock_bytes_encrypted.1.clone();

        let keypair =
            DracoonCrypto::create_plain_user_keypair(dco3_crypto::UserKeyPairVersion::RSA4096)
                .unwrap();
        let file_key = DracoonCrypto::encrypt_file_key(plain_key, keypair).unwrap();

        let file_key_json = serde_json::to_string(&file_key).unwrap();

        let file_key_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/2/user_file_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(file_key_json)
            .create();

        let download_url = format!("{}some/download/url", dracoon.get_base_url());

        let buffer = Vec::with_capacity(16);

        // create a writer
        let mut writer = tokio::io::BufWriter::new(buffer);

        let node_json = include_str!("../tests/responses/nodes/node_ok.json");
        let node: Node = serde_json::from_str(node_json).unwrap();

        let download_url_res =
            include_str!("../tests/responses/download/download_url_ok_template.json");
        let download_url_res = download_url_res.replace("$url", &download_url);

        let download_url_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/2/downloads")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(download_url_res)
            .create();

        let download_res = dracoon
            .download_encrypted(&node, &mut writer, None, None)
            .await;

        assert!(download_res.is_err());
        assert_eq!(
            download_res.err().unwrap(),
            DracoonClientError::MissingEncryptionSecret
        );
    }

    #[tokio::test]
    async fn test_download_unencrypted_chunked() {
        let (dracoon, mut mock_server) = get_connected_client().await;

        // create bytes for mocking byte response
        let mock_bytes: [u8; 16] = [
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ];

        let download_mock_1 = mock_server
            .mock("GET", "/some/download/url")
            .with_status(206)
            .with_header("content-type", "application/octet-stream")
            .with_body(&mock_bytes[0..8])
            .match_header("Range", "bytes=0-7")
            .create();

        let download_mock_2 = mock_server
            .mock("GET", "/some/download/url")
            .with_status(206)
            .with_header("content-type", "application/octet-stream")
            .with_body(&mock_bytes[8..])
            .match_header("Range", "bytes=8-15")
            .create();

        let download_url = format!("{}some/download/url", dracoon.get_base_url());

        let buffer = Vec::with_capacity(16);

        let mut writer = tokio::io::BufWriter::new(buffer);

        let node_json = include_str!("../tests/responses/nodes/node_ok.json");
        let node: Node = serde_json::from_str(node_json).unwrap();

        let download_url_res =
            include_str!("../tests/responses/download/download_url_ok_template.json");
        let download_url_res = download_url_res.replace("$url", &download_url);

        let download_url_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/2/downloads")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(download_url_res)
            .expect(2) // 2 hits for 2 chunks
            .create();

        dracoon
            .download_unencrypted(&node, &mut writer, None, Some(8))
            .await
            .unwrap();

        download_mock_1.assert();
        download_mock_2.assert();

        download_url_mock.assert();

        assert_eq!(writer.buffer(), mock_bytes.to_vec());
    }

    /// mocks the download url endpoint for node 2 and returns the download url
    async fn mock_download_url(mock_server: &mut mockito::ServerGuard, base_url: &str) -> String {
        let download_url = format!("{base_url}some/download/url");
        let download_url_res =
            include_str!("../tests/responses/download/download_url_ok_template.json")
                .replace("$url", &download_url);

        mock_server
            .mock("POST", "/api/v4/nodes/files/2/downloads")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(download_url_res)
            .create_async()
            .await;

        download_url
    }

    #[tokio::test]
    async fn test_download_without_size_fails_if_content_length_is_unknown() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;

        // probe response without Content-Range header
        let probe_mock = mock_server
            .mock("GET", "/some/download/url")
            .match_header("Range", "bytes=0-0")
            .with_status(200)
            .with_body([0u8])
            .create_async()
            .await;

        let node_json = include_str!("../tests/responses/nodes/node_ok.json");
        let mut node: Node = serde_json::from_str(node_json).unwrap();
        node.size = None;

        let mut writer = RecordingAsyncWriter::default();
        let res = dracoon
            .download_unencrypted(&node, &mut writer, None, None)
            .await;

        probe_mock.assert_async().await;
        assert!(res.is_err(), "expected error, got {res:?}");
        assert!(writer.flattened().is_empty());
    }

    #[tokio::test]
    async fn test_download_chunked_fails_if_server_ignores_range() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;

        // full content with 200 instead of the requested range
        let _download_mock = mock_server
            .mock("GET", "/some/download/url")
            .with_status(200)
            .with_body([7u8; 16])
            .create_async()
            .await;

        let node_json = include_str!("../tests/responses/nodes/node_ok.json");
        let node: Node = serde_json::from_str(node_json).unwrap();

        let mut writer = RecordingAsyncWriter::default();
        let res = dracoon
            .download_unencrypted(&node, &mut writer, None, Some(8))
            .await;

        assert!(res.is_err(), "expected error, got {res:?}");
    }

    #[tokio::test]
    async fn test_download_fails_on_empty_range_response_instead_of_looping() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;

        let _download_mock = mock_server
            .mock("GET", "/some/download/url")
            .with_status(206)
            .with_body([])
            .create_async()
            .await;

        let node_json = include_str!("../tests/responses/nodes/node_ok.json");
        let node: Node = serde_json::from_str(node_json).unwrap();

        let mut writer = RecordingAsyncWriter::default();
        let res = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            dracoon.download_unencrypted(&node, &mut writer, None, Some(8)),
        )
        .await
        .expect("download must not loop forever");

        assert!(res.is_err(), "expected error, got {res:?}");
    }

    #[tokio::test]
    async fn test_download_encrypted_chunked() {
        let (dracoon, mut mock_server) = get_connected_client().await;

        // Create bytes for mocking byte response
        let mock_bytes: [u8; 16] = [
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ];
        let mock_bytes_compare = mock_bytes;

        // Encrypt the mock bytes and get the plain file key
        let mock_bytes_encrypted = encrypt_streaming(mock_bytes);
        let plain_key = mock_bytes_encrypted.1.clone();

        // Create a user keypair and encrypt the file key with it
        let keypair =
            DracoonCrypto::create_plain_user_keypair(dco3_crypto::UserKeyPairVersion::RSA4096)
                .unwrap();
        let enc_keypair =
            DracoonCrypto::encrypt_private_key("TopSecret1234!", keypair.clone()).unwrap();
        let enc_keypair_json = serde_json::to_string(&enc_keypair).unwrap();
        let file_key = DracoonCrypto::encrypt_file_key(plain_key, keypair).unwrap();
        let file_key_json = serde_json::to_string(&file_key).unwrap();

        // Mock the download URL
        let download_url = format!("{}some/download/url", dracoon.get_base_url());

        // Mock the download URL generation endpoint
        let download_url_res =
            include_str!("../tests/responses/download/download_url_ok_template.json");
        let download_url_res = download_url_res.replace("$url", &download_url);

        let download_url_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/2/downloads")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(download_url_res)
            .expect(2) // 2 hits for 2 chunks
            .create();

        // Mock the file key retrieval endpoint
        let file_key_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/2/user_file_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(file_key_json)
            .expect(1)
            .create();

        // Mock the user keypair retrieval endpoint
        let keypair_mock = mock_server
            .mock("GET", "/api/v4/user/account/keypair")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(enc_keypair_json)
            .expect(1)
            .create();

        // Mock the first chunk download
        let download_mock_1 = mock_server
            .mock("GET", "/some/download/url")
            .match_header("Range", "bytes=0-7")
            .with_status(206)
            .with_header("content-type", "application/octet-stream")
            .with_body(&mock_bytes_encrypted.0[0..8])
            .expect(1)
            .create();

        // Mock the second chunk download
        let download_mock_2 = mock_server
            .mock("GET", "/some/download/url")
            .match_header("Range", "bytes=8-15")
            .with_status(206)
            .with_header("content-type", "application/octet-stream")
            .with_body(&mock_bytes_encrypted.0[8..16])
            .expect(1)
            .create();

        let mut writer = RecordingAsyncWriter::default();

        // Deserialize the node object
        let node_json = include_str!("../tests/responses/nodes/node_ok.json");
        let node: Node = serde_json::from_str(node_json).unwrap();

        // Retrieve the keypair before downloading (if required)
        let _kp = dracoon
            .get_keypair(Some("TopSecret1234!".into()))
            .await
            .unwrap();

        // Call the download_encrypted method
        dracoon
            .download_encrypted(&node, &mut writer, None, Some(8))
            .await
            .unwrap();

        // Assert that all mocks were called as expected
        download_url_mock.assert();
        file_key_mock.assert();
        keypair_mock.assert();
        download_mock_1.assert();
        download_mock_2.assert();

        assert_eq!(writer.flattened(), mock_bytes_compare.to_vec());
        assert!(writer.writes.len() > 1);
        assert!(writer
            .writes
            .iter()
            .all(|write| write.len() < mock_bytes_compare.len()));
    }
}
