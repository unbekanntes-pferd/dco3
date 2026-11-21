use super::{
    models::{DownloadProgressCallback, DownloadUrlResponse, Node, NodeType},
    Download,
};
use crate::{
    client::{errors::DracoonClientError, Connected},
    constants::{
        DEFAULT_DOWNLOAD_CHUNK_SIZE, DRACOON_API_PREFIX, FILES_BASE, FILES_FILE_KEY,
        MAX_DOWNLOAD_RANGE_SIZE, NODES_BASE, NODES_DOWNLOAD_URL,
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

    async fn download_range(
        &self,
        node: &Node,
        offset: u64,
        length: usize,
    ) -> Result<bytes::Bytes, DracoonClientError> {
        if length == 0 {
            return Ok(bytes::Bytes::new());
        }

        if node.node_type != NodeType::File {
            return Err(DracoonClientError::InvalidArgument(format!(
                "node {} is not a file",
                node.id
            )));
        }

        if node.is_encrypted.unwrap_or(false) {
            return Err(DracoonClientError::Unsupported(
                "byte-range reads of encrypted files (use download)".into(),
            ));
        }

        if length > MAX_DOWNLOAD_RANGE_SIZE {
            return Err(DracoonClientError::InvalidArgument(format!(
                "range length {length} exceeds the maximum of {MAX_DOWNLOAD_RANGE_SIZE} bytes"
            )));
        }

        let requested_end = u64::try_from(length - 1)
            .ok()
            .and_then(|len| offset.checked_add(len))
            .ok_or_else(|| {
                DracoonClientError::InvalidArgument(format!(
                    "range offset {offset} + length {length} overflows"
                ))
            })?;

        let end = match node.size {
            // nothing to read at or beyond the end of the file (incl. empty files)
            Some(size) if offset >= size => return Ok(bytes::Bytes::new()),
            Some(size) => requested_end.min(size - 1),
            None => requested_end,
        };

        let url = self.get_download_url(node.id).await?.download_url;

        // the download url is pre-authorized: no DRACOON bearer token
        let response = self
            .client
            .http
            .get(url)
            .header(RANGE, format!("bytes={offset}-{end}"))
            .send()
            .await
            .map_err(|err| {
                error!("Error while downloading range: {}", err);
                err
            })?;

        read_range_response(response, offset, end, node.size).await
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

/// Validates the response to a single range request `bytes=offset-end` and reads its body (bounded).
async fn read_range_response(
    response: Response,
    offset: u64,
    end: u64,
    size: Option<u64>,
) -> Result<bytes::Bytes, DracoonClientError> {
    let status = response.status();
    let content_range = response
        .headers()
        .get(CONTENT_RANGE)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_content_range);

    if status == StatusCode::RANGE_NOT_SATISFIABLE {
        // only an empty read if the server confirms the offset is at or beyond the end of the file
        return match (size, content_range) {
            (
                None,
                Some(ContentRange {
                    range: None,
                    total: Some(total),
                }),
            ) if offset >= total => Ok(bytes::Bytes::new()),
            _ => Err(build_s3_protocol_error(
                status,
                "range_not_satisfiable",
                "Download server rejected the requested byte range",
            )),
        };
    }

    let response = check_range_response(response, true).await?;

    let Some(ContentRange {
        range: Some((first, last)),
        total,
    }) = content_range
    else {
        return Err(build_s3_protocol_error(
            status,
            "invalid_content_range",
            "Partial content response without a valid Content-Range header",
        ));
    };

    // the server may only end the range early at the end of the file
    let ends_at_eof = total.is_some_and(|total| last + 1 == total);
    if first != offset || (last != end && !(last < end && ends_at_eof)) {
        return Err(build_s3_protocol_error(
            status,
            "range_mismatch",
            format!("Requested bytes {offset}-{end}, received bytes {first}-{last}"),
        ));
    }

    if let (Some(size), Some(total)) = (size, total) {
        if size != total {
            return Err(build_s3_protocol_error(
                status,
                "size_mismatch",
                format!("Expected a file size of {size} bytes, download has {total} bytes"),
            ));
        }
    }

    // bounded by MAX_DOWNLOAD_RANGE_SIZE: last <= end < offset + MAX_DOWNLOAD_RANGE_SIZE
    let expected = usize::try_from(last - first + 1).map_err(|_| {
        DracoonClientError::InvalidArgument("range does not fit into memory".into())
    })?;

    let mut body = bytes::BytesMut::with_capacity(expected);
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.try_next().await? {
        if body.len() + chunk.len() > expected {
            return Err(build_s3_protocol_error(
                status,
                "range_response_too_long",
                format!("Download server sent more than the requested {expected} bytes"),
            ));
        }
        body.extend_from_slice(&chunk);
    }

    if body.len() < expected {
        return Err(build_s3_protocol_error(
            status,
            "truncated_range_response",
            format!(
                "Download server sent {} of the requested {expected} bytes",
                body.len()
            ),
        ));
    }

    Ok(body.freeze())
}

/// A parsed `Content-Range` response header (RFC 9110): `bytes <first>-<last>/<total>`,
/// with `*` for an unknown total or (on `416`) an unsatisfied range.
#[derive(Debug, PartialEq)]
pub(crate) struct ContentRange {
    /// first and last byte position (inclusive)
    pub(crate) range: Option<(u64, u64)>,
    /// complete length of the representation, if known
    pub(crate) total: Option<u64>,
}

/// Parses a `Content-Range` header - returns `None` for anything malformed or inconsistent
pub(crate) fn parse_content_range(value: &str) -> Option<ContentRange> {
    fn number(value: &str) -> Option<u64> {
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        value.parse().ok()
    }

    let (unit, rest) = value.trim().split_once(' ')?;
    if !unit.eq_ignore_ascii_case("bytes") {
        return None;
    }

    let (range, total) = rest.trim().split_once('/')?;
    let total = match total {
        "*" => None,
        total => Some(number(total)?),
    };

    let range = match range {
        "*" => None,
        range => {
            let (first, last) = range.split_once('-')?;
            let (first, last) = (number(first)?, number(last)?);
            if last < first || total.is_some_and(|total| last >= total) {
                return None;
            }
            Some((first, last))
        }
    };

    if range.is_none() && total.is_none() {
        return None;
    }

    Some(ContentRange { range, total })
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

    const RANGE_DATA: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

    /// unencrypted file node with id 2 (download url endpoint `/api/v4/nodes/files/2/downloads`)
    fn range_file_node(size: Option<u64>) -> Node {
        let mut node: Node =
            serde_json::from_str(include_str!("../tests/responses/nodes/node_ok.json")).unwrap();
        node.node_type = NodeType::File;
        node.is_encrypted = Some(false);
        node.size = size;
        node
    }

    /// mocks a ranged GET on the download url
    async fn mock_range_get(
        mock_server: &mut mockito::ServerGuard,
        range: &str,
        status: usize,
        content_range: Option<&str>,
        body: &[u8],
    ) -> mockito::Mock {
        let mut mock = mock_server
            .mock("GET", "/some/download/url")
            .match_header("range", range)
            // the DRACOON bearer token must never be sent to the download host
            .match_header("authorization", mockito::Matcher::Missing)
            .with_status(status)
            .with_body(body);
        if let Some(content_range) = content_range {
            mock = mock.with_header("content-range", content_range);
        }
        mock.expect(1).create_async().await
    }

    /// mocks endpoints that must not be called (download url + any download)
    async fn mock_no_requests(
        mock_server: &mut mockito::ServerGuard,
    ) -> (mockito::Mock, mockito::Mock) {
        let url_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/2/downloads")
            .expect(0)
            .create_async()
            .await;
        let get_mock = mock_server
            .mock("GET", "/some/download/url")
            .expect(0)
            .create_async()
            .await;
        (url_mock, get_mock)
    }

    fn assert_protocol_error(err: &DracoonClientError, expected_code: &str) {
        match err {
            DracoonClientError::S3Error(err) => match &err.kind {
                crate::nodes::models::S3ErrorKind::Protocol { code, .. } => {
                    assert_eq!(*code, expected_code, "unexpected protocol error: {err}");
                }
                other => panic!("expected protocol error {expected_code}, got {other:?}"),
            },
            other => panic!("expected protocol error {expected_code}, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_download_range_middle() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;
        let get_mock = mock_range_get(
            &mut mock_server,
            "bytes=5-8",
            206,
            Some("bytes 5-8/16"),
            &RANGE_DATA[5..=8],
        )
        .await;

        let bytes = dracoon
            .download_range(&range_file_node(Some(16)), 5, 4)
            .await
            .unwrap();

        get_mock.assert_async().await;
        assert_eq!(bytes.as_ref(), &RANGE_DATA[5..=8]);
    }

    #[tokio::test]
    async fn test_download_range_last_range_is_clamped_to_size() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;
        let get_mock = mock_range_get(
            &mut mock_server,
            "bytes=12-15",
            206,
            Some("bytes 12-15/16"),
            &RANGE_DATA[12..],
        )
        .await;

        let bytes = dracoon
            .download_range(&range_file_node(Some(16)), 12, 8)
            .await
            .unwrap();

        get_mock.assert_async().await;
        assert_eq!(bytes.as_ref(), &RANGE_DATA[12..]);
    }

    #[tokio::test]
    async fn test_download_range_at_or_after_eof_and_zero_length_send_no_request() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        let (url_mock, get_mock) = mock_no_requests(&mut mock_server).await;
        let node = range_file_node(Some(16));

        assert!(dracoon
            .download_range(&node, 16, 4)
            .await
            .unwrap()
            .is_empty());
        assert!(dracoon
            .download_range(&node, 100, 4)
            .await
            .unwrap()
            .is_empty());
        assert!(dracoon
            .download_range(&node, 3, 0)
            .await
            .unwrap()
            .is_empty());
        // zero-byte file
        assert!(dracoon
            .download_range(&range_file_node(Some(0)), 0, 4)
            .await
            .unwrap()
            .is_empty());

        url_mock.assert_async().await;
        get_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_download_range_rejects_invalid_requests_without_request() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        let (url_mock, get_mock) = mock_no_requests(&mut mock_server).await;

        let mut encrypted = range_file_node(Some(16));
        encrypted.is_encrypted = Some(true);
        let err = dracoon.download_range(&encrypted, 0, 4).await.unwrap_err();
        assert!(err.is_unsupported(), "unexpected error: {err:?}");

        for node_type in [NodeType::Folder, NodeType::Room] {
            let mut node = range_file_node(Some(16));
            node.node_type = node_type;
            let err = dracoon.download_range(&node, 0, 4).await.unwrap_err();
            assert!(err.is_invalid_argument(), "unexpected error: {err:?}");
        }

        let err = dracoon
            .download_range(&range_file_node(None), 0, MAX_DOWNLOAD_RANGE_SIZE + 1)
            .await
            .unwrap_err();
        assert!(err.is_invalid_argument(), "unexpected error: {err:?}");

        let err = dracoon
            .download_range(&range_file_node(None), u64::MAX, 2)
            .await
            .unwrap_err();
        assert!(err.is_invalid_argument(), "unexpected error: {err:?}");

        url_mock.assert_async().await;
        get_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_download_range_max_size_is_allowed() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;
        // known size clamps the request to the file
        let get_mock = mock_range_get(
            &mut mock_server,
            "bytes=0-15",
            206,
            Some("bytes 0-15/16"),
            &RANGE_DATA,
        )
        .await;

        let bytes = dracoon
            .download_range(&range_file_node(Some(16)), 0, MAX_DOWNLOAD_RANGE_SIZE)
            .await
            .unwrap();

        get_mock.assert_async().await;
        assert_eq!(bytes.as_ref(), &RANGE_DATA);
    }

    #[tokio::test]
    async fn test_download_range_rejects_full_body_response() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;
        mock_range_get(&mut mock_server, "bytes=5-8", 200, None, &RANGE_DATA).await;

        let err = dracoon
            .download_range(&range_file_node(Some(16)), 5, 4)
            .await
            .unwrap_err();

        assert_protocol_error(&err, "range_not_supported");
    }

    #[tokio::test]
    async fn test_download_range_rejects_invalid_content_range() {
        for content_range in [None, Some("bytes 5-8"), Some("garbage")] {
            let (dracoon, mut mock_server) = get_connected_client().await;
            mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;
            mock_range_get(
                &mut mock_server,
                "bytes=5-8",
                206,
                content_range,
                &RANGE_DATA[5..=8],
            )
            .await;

            let err = dracoon
                .download_range(&range_file_node(Some(16)), 5, 4)
                .await
                .unwrap_err();

            assert_protocol_error(&err, "invalid_content_range");
        }
    }

    #[tokio::test]
    async fn test_download_range_rejects_mismatched_range() {
        for (content_range, body) in [
            ("bytes 4-7/16", &RANGE_DATA[4..=7]),
            // only the start differs
            ("bytes 6-8/16", &RANGE_DATA[6..=8]),
            ("bytes 5-7/16", &RANGE_DATA[5..=7]),
            ("bytes 5-9/16", &RANGE_DATA[5..=9]),
        ] {
            let (dracoon, mut mock_server) = get_connected_client().await;
            mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;
            mock_range_get(
                &mut mock_server,
                "bytes=5-8",
                206,
                Some(content_range),
                body,
            )
            .await;

            let err = dracoon
                .download_range(&range_file_node(Some(16)), 5, 4)
                .await
                .unwrap_err();

            assert_protocol_error(&err, "range_mismatch");
        }
    }

    #[tokio::test]
    async fn test_download_range_rejects_changed_size() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;
        mock_range_get(
            &mut mock_server,
            "bytes=5-8",
            206,
            Some("bytes 5-8/20"),
            &RANGE_DATA[5..=8],
        )
        .await;

        let err = dracoon
            .download_range(&range_file_node(Some(16)), 5, 4)
            .await
            .unwrap_err();

        assert_protocol_error(&err, "size_mismatch");
    }

    #[tokio::test]
    async fn test_download_range_rejects_truncated_and_oversized_bodies() {
        for (body, code) in [
            (&RANGE_DATA[5..=7], "truncated_range_response"),
            (&RANGE_DATA[5..=9], "range_response_too_long"),
        ] {
            let (dracoon, mut mock_server) = get_connected_client().await;
            mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;
            mock_range_get(
                &mut mock_server,
                "bytes=5-8",
                206,
                Some("bytes 5-8/16"),
                body,
            )
            .await;

            let err = dracoon
                .download_range(&range_file_node(Some(16)), 5, 4)
                .await
                .unwrap_err();

            assert_protocol_error(&err, code);
        }
    }

    #[tokio::test]
    async fn test_download_range_unknown_size_accepts_range_clamped_at_eof() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;
        let get_mock = mock_range_get(
            &mut mock_server,
            "bytes=12-19",
            206,
            Some("bytes 12-15/16"),
            &RANGE_DATA[12..],
        )
        .await;

        let bytes = dracoon
            .download_range(&range_file_node(None), 12, 8)
            .await
            .unwrap();

        get_mock.assert_async().await;
        assert_eq!(bytes.as_ref(), &RANGE_DATA[12..]);
    }

    #[tokio::test]
    async fn test_download_range_unknown_size_416_beyond_eof_is_empty() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;
        let get_mock = mock_range_get(
            &mut mock_server,
            "bytes=20-23",
            416,
            Some("bytes */16"),
            &[],
        )
        .await;

        let bytes = dracoon
            .download_range(&range_file_node(None), 20, 4)
            .await
            .unwrap();

        get_mock.assert_async().await;
        assert!(bytes.is_empty());
    }

    #[tokio::test]
    async fn test_download_range_416_without_confirmed_eof_is_an_error() {
        // known size (the request was within the file) / no valid total
        for (size, content_range) in [
            (Some(16), Some("bytes */16")),
            (None, None),
            // the offset is within the file according to the server
            (None, Some("bytes */16")),
        ] {
            let (dracoon, mut mock_server) = get_connected_client().await;
            mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;
            mock_range_get(&mut mock_server, "bytes=5-8", 416, content_range, &[]).await;

            let err = dracoon
                .download_range(&range_file_node(size), 5, 4)
                .await
                .unwrap_err();

            assert_protocol_error(&err, "range_not_satisfiable");
        }
    }

    #[tokio::test]
    async fn test_download_range_download_url_forbidden() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        mock_server
            .mock("POST", "/api/v4/nodes/files/2/downloads")
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"code": 403, "message": "Forbidden", "debugInfo": "no permission", "errorCode": -10000}"#)
            .create_async()
            .await;

        let err = dracoon
            .download_range(&range_file_node(Some(16)), 5, 4)
            .await
            .unwrap_err();

        assert!(err.is_forbidden(), "unexpected error: {err:?}");
    }

    #[tokio::test]
    async fn test_download_range_s3_error_keeps_details() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;
        mock_range_get(
            &mut mock_server,
            "bytes=5-8",
            403,
            None,
            br#"<?xml version="1.0" encoding="UTF-8"?><Error><Code>AccessDenied</Code><Message>Request has expired</Message><RequestId>1</RequestId><HostId>2</HostId></Error>"#,
        )
        .await;

        let err = dracoon
            .download_range(&range_file_node(Some(16)), 5, 4)
            .await
            .unwrap_err();

        match err {
            DracoonClientError::S3Error(err) => {
                assert_eq!(err.status, reqwest::StatusCode::FORBIDDEN);
                assert!(
                    matches!(err.kind, crate::nodes::models::S3ErrorKind::Xml(_)),
                    "unexpected error: {err}"
                );
            }
            other => panic!("expected S3 error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_download_range_reassembles_file() {
        let (dracoon, mut mock_server) = get_connected_client().await;
        mock_download_url(&mut mock_server, dracoon.get_base_url().as_str()).await;

        let mut get_mocks = Vec::new();
        for start in (0..16u64).step_by(3) {
            let last = (start + 2).min(15);
            get_mocks.push(
                mock_range_get(
                    &mut mock_server,
                    &format!("bytes={start}-{last}"),
                    206,
                    Some(&format!("bytes {start}-{last}/16")),
                    &RANGE_DATA[start as usize..=last as usize],
                )
                .await,
            );
        }

        let node = range_file_node(Some(16));
        let mut reassembled = Vec::new();
        for start in (0..16u64).step_by(3) {
            reassembled.extend_from_slice(&dracoon.download_range(&node, start, 3).await.unwrap());
        }

        for mock in get_mocks {
            mock.assert_async().await;
        }
        assert_eq!(reassembled, RANGE_DATA);
    }

    #[test]
    fn test_parse_content_range() {
        let parse = parse_content_range;

        assert_eq!(
            parse("bytes 5-8/16"),
            Some(ContentRange {
                range: Some((5, 8)),
                total: Some(16)
            })
        );
        // unknown total
        assert_eq!(
            parse("bytes 0-499/*"),
            Some(ContentRange {
                range: Some((0, 499)),
                total: None
            })
        );
        // unsatisfied range (416)
        assert_eq!(
            parse("bytes */16"),
            Some(ContentRange {
                range: None,
                total: Some(16)
            })
        );
        // unit is case-insensitive
        assert_eq!(parse("Bytes 1-1/2").unwrap().range, Some((1, 1)));

        for invalid in [
            "",
            "bytes",
            "bytes 5-8",
            "bytes 8-5/16",
            "bytes 5-16/16",
            "bytes */*",
            "bytes -1-4/16",
            "bytes +1-4/16",
            "bytes 1-x/16",
            "items 1-4/16",
            "bytes 1-18446744073709551616/18446744073709551617",
        ] {
            assert_eq!(
                parse(invalid),
                None,
                "accepted invalid Content-Range {invalid:?}"
            );
        }
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

        let _download_url_mock = mock_server
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

        let _download_url_mock = mock_server
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

        let _file_key_mock = mock_server
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

        let _download_url_mock = mock_server
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
