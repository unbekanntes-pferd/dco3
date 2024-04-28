use std::{cmp::min, time::Duration};

use super::{
    models::{
        CloneableUploadProgressCallback, CompleteS3FileUploadRequest, CreateFileUploadRequest,
        CreateFileUploadResponse, FileMeta, GeneratePresignedUrlsRequest, MissingKeysResponse,
        Node, PresignedUrl, PresignedUrlList, ResolutionStrategy, S3FileUploadStatus,
        S3UploadStatus, UploadOptions, UploadProgressCallback, UserFileKeySetBatchRequest,
    },
    CompleteUploadRequest, Upload,
};
use crate::{
    auth::{errors::DracoonClientError, Connected, GetClient},
    constants::{
        CHUNK_SIZE, DRACOON_API_PREFIX, FILES_BASE, FILES_KEYS, FILES_S3_COMPLETE, FILES_S3_URLS,
        FILES_UPLOAD, MISSING_FILE_KEYS, MISSING_KEYS_BATCH, NODES_BASE, POLLING_START_DELAY,
        UPLOADS_BASE,
    },
    models::ObjectExpiration,
    nodes::models::{S3FileUploadPart, UserFileKeySetRequest},
    utils::{build_s3_error, FromResponse},
    Dracoon,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dco3_crypto::{ChunkedEncryption, DracoonCrypto, DracoonRSACrypto, Encrypter};
use futures_util::{Stream, StreamExt};
use reqwest::{header, Body};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
use tracing::error;

#[async_trait]
impl<R: AsyncRead + Sync + Send + Unpin + 'static> Upload<R> for Dracoon<Connected> {
    async fn upload<'r>(
        &'r self,
        parent_node: &Node,
        upload_options: UploadOptions,
        reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<Node, DracoonClientError> {
        let is_s3_upload = self.get_system_info().await?.use_s3_storage;
        let is_encrypted = parent_node.is_encrypted.unwrap_or(false);

        let upload_fn = match (is_encrypted, is_s3_upload) {
            (true, true) => Self::upload_to_s3_encrypted,
            (true, false) => Self::upload_to_nfs_encrypted,
            (false, true) => Self::upload_to_s3_unencrypted,
            (false, false) => Self::upload_to_nfs_unencrypted,
        };

        upload_fn(
            self,
            parent_node,
            upload_options,
            reader,
            callback,
            chunk_size,
        )
        .await
    }
}

#[async_trait]
trait UploadInternal<R: AsyncRead> {
    async fn create_upload_channel(
        &self,
        create_file_upload_req: CreateFileUploadRequest,
    ) -> Result<CreateFileUploadResponse, DracoonClientError>;

    async fn create_s3_upload_urls(
        &self,
        upload_id: String,
        generate_urls_req: GeneratePresignedUrlsRequest,
    ) -> Result<PresignedUrlList, DracoonClientError>;

    async fn upload_to_s3_unencrypted(
        &self,
        parent_node: &Node,
        upload_options: UploadOptions,
        reader: BufReader<R>,
        mut callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<Node, DracoonClientError>;
    async fn upload_to_s3_encrypted(
        &self,
        parent_node: &Node,
        upload_options: UploadOptions,
        reader: BufReader<R>,
        mut callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<Node, DracoonClientError>;

    async fn finalize_upload(
        &self,
        upload_id: String,
        complete_file_upload_req: CompleteS3FileUploadRequest,
    ) -> Result<(), DracoonClientError>;

    async fn get_upload_status(
        &self,
        upload_id: String,
    ) -> Result<S3FileUploadStatus, DracoonClientError>;
    async fn get_missing_file_keys(
        &self,
        file_id: u64,
    ) -> Result<MissingKeysResponse, DracoonClientError>;

    async fn set_file_keys(
        &self,
        keys_batch_req: UserFileKeySetBatchRequest,
    ) -> Result<(), DracoonClientError>;
}

#[async_trait]
pub(crate) trait StreamUploadInternal<S>: GetClient<S> {
    async fn upload_stream_to_s3<'a>(
        &self,
        mut stream: impl Stream<Item = Result<bytes::Bytes, impl std::error::Error + Send + Sync + 'static>>
            + Sync
            + Send
            + Unpin
            + 'static,
        url: &PresignedUrl,
        file_meta: FileMeta,
        chunk_size: usize,
        curr_pos: Option<u64>,
        callback: Option<CloneableUploadProgressCallback>,
    ) -> Result<String, DracoonClientError> {
          // Initialize a variable to keep track of the number of bytes read
          let mut bytes_read = curr_pos.unwrap_or(0);
          let file_size = file_meta.1;
          // Create an async stream from the reader
          let async_stream = async_stream::stream! {
  
              while let Some(chunk) = stream.next().await {
                  if let Ok(chunk) = &chunk {
                      let processed = min(bytes_read + (chunk.len() as u64), file_meta.1);
                      bytes_read = processed;
  
                      if let Some(cb) = callback.clone() {
                          cb.call(bytes_read, file_meta.1);
                      }
                  }
                  yield chunk
              }
          };
  
          let body = Body::wrap_stream(async_stream);
  
          let res = self
              .get_client()
              .stream_http
              .put(&url.url)
              .body(body)
              .header(header::CONTENT_LENGTH, chunk_size)
              .send()
              .await
              .map_err(|e| {
                  error!("Connection error (S3 upload): {:?}", e);
                  e
              })?;
  
          // handle error
          if res.error_for_status_ref().is_err() {
              error!(
                  "Error uploading file to S3: {:?}",
                  res.error_for_status_ref().unwrap_err()
              );
              let error = build_s3_error(res).await;
              return Err(error);
          }
  
          let e_tag_header = res
              .headers()
              .get("etag")
              .expect("ETag header missing")
              .to_str()
              .expect("ETag header invalid");
          let e_tag = e_tag_header.trim_start_matches('"').trim_end_matches('"');
  
          Ok(e_tag.to_string())
    }

    async fn upload_stream_to_nfs<'a>(
        &self,
        mut stream: impl Stream<Item = Result<bytes::Bytes, impl std::error::Error + Send + Sync + 'static>>
            + Sync
            + Send
            + Unpin
            + 'static,
        url: &str,
        file_meta: FileMeta,
        chunk_size: usize,
        curr_pos: Option<u64>,
        callback: Option<CloneableUploadProgressCallback>,
    ) -> Result<(), DracoonClientError> {
            // Initialize a variable to keep track of the number of bytes read
            let mut bytes_read = curr_pos.unwrap_or(0);
            let file_size = file_meta.1;
            // Create an async stream from the reader
            let async_stream = async_stream::stream! {
    
                while let Some(chunk) = stream.next().await {
                    if let Ok(chunk) = &chunk {
                        let processed = min(bytes_read + (chunk.len() as u64), file_meta.1);
                        bytes_read = processed;
    
                        if let Some(cb) = callback.clone() {
                            cb.call(bytes_read, file_meta.1);
                        }
                    }
                    yield chunk
                }
            };
    
            let body = Body::wrap_stream(async_stream);
    
            let start_range = bytes_read;
            let end_range = if bytes_read + chunk_size as u64 > file_size {
                file_size
            } else {
                bytes_read + chunk_size as u64
            };
    
            let res = self
                .get_client()
                .stream_http
                .post(url)
                .body(body)
                .header(
                    header::CONTENT_RANGE,
                    format!("bytes {}-{}/{}", start_range, end_range, file_size),
                )
                .header(header::CONTENT_LENGTH, chunk_size)
                .send()
                .await
                .map_err(|e| {
                    error!("Connection error (NFS upload): {:?}", e);
                    e
                })?;
    
            // handle error
            if res.error_for_status_ref().is_err() {
                error!(
                    "Error uploading file to NFS: {:?}",
                    res.error_for_status_ref().unwrap_err()
                );
                return Err(DracoonClientError::from_response(res)
                    .await
                    .unwrap_or(DracoonClientError::Unknown));
            }
            Ok(())
    }
}

impl StreamUploadInternal<Connected> for Dracoon<Connected> {}

#[async_trait]
trait UploadInternalNfs<R: AsyncRead, S>: StreamUploadInternal<S> {
    async fn upload_to_nfs_unencrypted(
        &self,
        parent_node: &Node,
        upload_options: UploadOptions,
        reader: BufReader<R>,
        mut callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<Node, DracoonClientError>;
    async fn upload_to_nfs_encrypted(
        &self,
        parent_node: &Node,
        upload_options: UploadOptions,
        reader: BufReader<R>,
        mut callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<Node, DracoonClientError>;

    async fn finalize_nfs_upload(
        &self,
        upload_token: String,
        complete_file_upload_req: CompleteUploadRequest,
    ) -> Result<Node, DracoonClientError>;
}

#[async_trait]
impl<R: AsyncRead + Sync + Send + Unpin + 'static> UploadInternal<R> for Dracoon<Connected> {
    async fn create_upload_channel(
        &self,
        create_file_upload_req: CreateFileUploadRequest,
    ) -> Result<CreateFileUploadResponse, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{NODES_BASE}/{FILES_BASE}/{FILES_UPLOAD}");

        let api_url = self.build_api_url(&url_part);

        let res = self
            .client
            .http
            .post(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&create_file_upload_req)
            .send()
            .await?;

        CreateFileUploadResponse::from_response(res).await
    }

    async fn create_s3_upload_urls(
        &self,
        upload_id: String,
        generate_urls_req: GeneratePresignedUrlsRequest,
    ) -> Result<PresignedUrlList, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{NODES_BASE}/{FILES_BASE}/{FILES_UPLOAD}/{upload_id}/{FILES_S3_URLS}"
        );
        let api_url = self.build_api_url(&url_part);
        let res = self
            .client
            .http
            .post(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&generate_urls_req)
            .send()
            .await?;

        PresignedUrlList::from_response(res).await
    }

    async fn finalize_upload(
        &self,
        upload_id: String,
        complete_file_upload_req: CompleteS3FileUploadRequest,
    ) -> Result<(), DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{NODES_BASE}/{FILES_BASE}/{FILES_UPLOAD}/{upload_id}/{FILES_S3_COMPLETE}"
        );
        let api_url = self.build_api_url(&url_part);
        let res = self
            .client
            .http
            .put(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&complete_file_upload_req)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(DracoonClientError::from_response(res).await?)
        }
    }

    /// requests S3 upload status from DRACOON
    async fn get_upload_status(
        &self,
        upload_id: String,
    ) -> Result<S3FileUploadStatus, DracoonClientError> {
        let url_part =
            format!("{DRACOON_API_PREFIX}/{NODES_BASE}/{FILES_BASE}/{FILES_UPLOAD}/{upload_id}");
        let api_url = self.build_api_url(&url_part);
        let res = self
            .client
            .http
            .get(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        S3FileUploadStatus::from_response(res).await
    }

    #[allow(clippy::single_match_else)]
    #[allow(clippy::too_many_lines)]
    async fn upload_to_s3_unencrypted(
        &self,
        parent_node: &Node,
        upload_options: UploadOptions,
        mut reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<Node, DracoonClientError> {
        // parse upload options
        let (
            classification,
            timestamp_creation,
            timestamp_modification,
            expiration,
            resolution_strategy,
            keep_share_links,
        ) = parse_upload_options(&upload_options);

        let fm = upload_options.file_meta.clone();

        let chunk_size = chunk_size.unwrap_or(CHUNK_SIZE);

        // create upload channel
        let file_upload_req = CreateFileUploadRequest::builder(parent_node.id, fm.0)
            .with_classification(classification)
            .with_size(fm.1)
            .with_timestamp_modification(timestamp_modification)
            .with_timestamp_creation(timestamp_creation)
            .with_expiration(expiration)
            .build();

        let upload_channel = <Dracoon<Connected> as UploadInternal<R>>::create_upload_channel::<
            '_,
            '_,
        >(self, file_upload_req)
        .await
        .map_err(|err| {
            error!("Error creating upload channel: {}", err);
            err
        })?;

        let mut s3_parts = Vec::new();

        let (count_urls, last_chunk_size) = calculate_s3_url_count(fm.1, chunk_size as u64);
        let mut url_part: u32 = 1;

        let cloneable_callback = callback.map(CloneableUploadProgressCallback::new);

        if count_urls > 1 {
            while url_part < count_urls {
                let mut buffer = vec![0; chunk_size];

                match reader.read_exact(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        buffer.truncate(n);
                        let chunk = bytes::Bytes::from(buffer);

                        let stream: async_stream::__private::AsyncStream<
                            Result<bytes::Bytes, std::io::Error>,
                            _,
                        > = async_stream::stream! {
                            yield Ok(chunk);
                        };

                        let url_req = GeneratePresignedUrlsRequest::new(
                            n.try_into().expect("size not larger than 32 MB"),
                            url_part,
                            url_part,
                        );
                        let url =
                            <Dracoon<Connected> as UploadInternal<R>>::create_s3_upload_urls::<
                                '_,
                                '_,
                            >(
                                self, upload_channel.upload_id.clone(), url_req
                            )
                            .await?;
                        let url = url.urls.first().expect("Creating S3 url failed");

                        // truncation is safe because chunk_size is 32 MB
                        #[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
                        let curr_pos: u64 = ((url_part - 1) * (chunk_size as u32)) as u64;

                        let e_tag = <Dracoon<Connected> as StreamUploadInternal<Connected>>::upload_stream_to_s3(
                            self,
                            Box::pin(stream),
                            url,
                            upload_options.file_meta.clone(),
                            n,
                            Some(curr_pos),
                            cloneable_callback.clone(),
                        )
                        .await?;

                        s3_parts.push(S3FileUploadPart::new(url_part, e_tag));
                        url_part += 1;
                    }
                    Err(err) => {
                        error!("Error reading file: {}", err);
                        return Err(DracoonClientError::IoError);
                    }
                }
            }
        }

        // upload last chunk
        let mut buffer = vec![
            0;
            last_chunk_size
                .try_into()
                .expect("size not larger than 32 MB")
        ];
        match reader.read_exact(&mut buffer).await {
            Ok(n) => {
                buffer.truncate(n);
                let chunk = bytes::Bytes::from(buffer);
                let stream: async_stream::__private::AsyncStream<
                    Result<bytes::Bytes, std::io::Error>,
                    _,
                > = async_stream::stream! {
                    // TODO: chunk stream for better progress
                    // currently the progress is only updated per chunk
                    yield Ok(chunk);

                };

                let url_req = GeneratePresignedUrlsRequest::new(
                    n.try_into().expect("size not larger than 32 MB"),
                    url_part,
                    url_part,
                );
                let url =
                    <Dracoon<Connected> as UploadInternal<R>>::create_s3_upload_urls::<'_, '_>(
                        self,
                        upload_channel.upload_id.clone(),
                        url_req,
                    )
                    .await
                    .map_err(|err| {
                        error!("Error creating S3 upload urls: {}", err);
                        err
                    })?;

                let url = url.urls.first().expect("Creating S3 url failed");

                let curr_pos: u64 = (url_part - 1) as u64 * (CHUNK_SIZE as u64);

                let e_tag = <Dracoon<Connected> as StreamUploadInternal<Connected>>::upload_stream_to_s3(
                    self,
                    Box::pin(stream),
                    url,
                    upload_options.file_meta.clone(),
                    n,
                    Some(curr_pos),
                    cloneable_callback.clone(),
                )
                .await?;

                s3_parts.push(S3FileUploadPart::new(url_part, e_tag));
            }
            Err(err) => {
                error!("Error reading file: {}", err);
                return Err(DracoonClientError::IoError);
            }
        }

        // finalize upload
        let complete_upload_req = CompleteS3FileUploadRequest::builder(s3_parts)
            .with_resolution_strategy(resolution_strategy)
            .with_keep_share_links(keep_share_links)
            .build();

        <Dracoon<Connected> as UploadInternal<R>>::finalize_upload::<'_, '_>(
            self,
            upload_channel.upload_id.clone(),
            complete_upload_req,
        )
        .await
        .map_err(|err| {
            error!("Error finalizing upload: {}", err);
            err
        })?;

        // get upload status
        // return node if upload is done
        // return error if upload failed
        // polling with exponential backoff
        let mut sleep_duration = POLLING_START_DELAY;
        loop {
            let status_response = <Dracoon<Connected> as UploadInternal<R>>::get_upload_status(
                self,
                upload_channel.upload_id.clone(),
            )
            .await
            .map_err(|err| {
                error!("Error getting upload status: {}", err);
                err
            })?;

            match status_response.status {
                S3UploadStatus::Done => {
                    return Ok(status_response
                        .node
                        .expect("Node must be set if status is done"));
                }
                S3UploadStatus::Error => {
                    let response = status_response
                        .error_details
                        .expect("Error message must be set if status is error");
                    error!("Error uploading file: {}", response);
                    return Err(DracoonClientError::Http(response));
                }
                _ => {
                    tokio::time::sleep(Duration::from_millis(sleep_duration)).await;
                    sleep_duration *= 2;
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn upload_to_s3_encrypted(
        &self,
        parent_node: &Node,
        upload_options: UploadOptions,
        mut reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<Node, DracoonClientError> {
        let keypair = self.get_keypair(None).await?;

        let chunk_size = chunk_size.unwrap_or(CHUNK_SIZE);

        let mut crypto_buff =
            vec![0u8; upload_options.file_meta.1.try_into().expect("size not larger than 32 MB")];
        let mut read_buff = vec![0u8; upload_options.file_meta.1.try_into().expect("size not larger than 32 MB")];
        let mut crypter = DracoonCrypto::encrypter(&mut crypto_buff)?;

        while let Ok(chunk) = reader.read(&mut read_buff).await {
            if chunk == 0 {
                break;
            }
            crypter.update(&read_buff[..chunk])?;
        }
        crypter.finalize()?;
        // drop the read buffer after completing the encryption
        drop(read_buff);

        //TODO: rewrite without buffer clone
        let enc_bytes = crypter.get_message().clone();

        assert_eq!(enc_bytes.len() as u64, upload_options.file_meta.1);

        let mut crypto_reader = BufReader::new(enc_bytes.as_slice());
        let plain_file_key = crypter.get_plain_file_key();
        let file_key = DracoonCrypto::encrypt_file_key(plain_file_key.clone(), keypair)?;
        // drop the crypto buffer (enc bytes are still in the reader)
        drop(crypto_buff);

        let (
            classification,
            timestamp_creation,
            timestamp_modification,
            expiration,
            resolution_strategy,
            keep_share_links,
        ) = parse_upload_options(&upload_options);

        let fm = upload_options.file_meta.clone();

        // create upload channel
        let file_upload_req = CreateFileUploadRequest::builder(parent_node.id, fm.0)
            .with_classification(classification)
            .with_size(fm.1)
            .with_timestamp_modification(timestamp_modification)
            .with_timestamp_creation(timestamp_creation)
            .with_expiration(expiration)
            .build();

        let upload_channel = <Dracoon<Connected> as UploadInternal<R>>::create_upload_channel::<
            '_,
            '_,
        >(self, file_upload_req)
        .await
        .map_err(|err| {
            error!("Error creating upload channel: {}", err);
            err
        })?;

        let mut s3_parts = Vec::new();

        let (count_urls, last_chunk_size) = calculate_s3_url_count(fm.1, chunk_size as u64);
        let mut url_part: u32 = 1;

        let cloneable_callback = callback.map(CloneableUploadProgressCallback::new);

        if count_urls > 1 {
            while url_part < count_urls {
                let mut buffer = vec![0; chunk_size];

                match crypto_reader.read_exact(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk_len = n;
                        buffer.truncate(chunk_len);
                        let chunk = bytes::Bytes::from(buffer);

                        let stream: async_stream::__private::AsyncStream<
                            Result<bytes::Bytes, std::io::Error>,
                            _,
                        > = async_stream::stream! {
                            yield Ok(chunk);
                        };

                        let url_req = GeneratePresignedUrlsRequest::new(
                            chunk_len.try_into().expect("size not larger than 32 MB"),
                            url_part,
                            url_part,
                        );
                        let url =
                            <Dracoon<Connected> as UploadInternal<R>>::create_s3_upload_urls::<
                                '_,
                                '_,
                            >(
                                self, upload_channel.upload_id.clone(), url_req
                            )
                            .await
                            .map_err(|err| {
                                error!("Error creating S3 upload urls: {}", err);
                                err
                            })?;
                        let url = url.urls.first().expect("Creating S3 url failed");

                        let curr_pos: u64 = (url_part - 1) as u64 * (chunk_size as u64);

                        let e_tag = <Dracoon<Connected> as StreamUploadInternal<Connected>>::upload_stream_to_s3(
                            self,
                            Box::pin(stream),
                            url,
                            upload_options.file_meta.clone(),
                            chunk_len,
                            Some(curr_pos),
                            cloneable_callback.clone(),
                        )
                        .await
                        .map_err(|err| {
                            error!("Error uploading stream to S3: {}", err);
                            err
                        })?;

                        s3_parts.push(S3FileUploadPart::new(url_part, e_tag));
                        url_part += 1;
                    }
                    Err(err) => return Err(DracoonClientError::IoError),
                }
            }
        }

        // upload last chunk
        let mut buffer = vec![
            0;
            last_chunk_size
                .try_into()
                .expect("size not larger than 32 MB")
        ];
        match crypto_reader.read_exact(&mut buffer).await {
            Ok(n) => {
                buffer.truncate(n);
                let chunk = bytes::Bytes::from(buffer);
                let stream: async_stream::__private::AsyncStream<
                    Result<bytes::Bytes, std::io::Error>,
                    _,
                > = async_stream::stream! {
                    // TODO: chunk stream for better progress
                    yield Ok(chunk);

                };

                let url_req = GeneratePresignedUrlsRequest::new(
                    n.try_into().expect("size not larger than 32 MB"),
                    url_part,
                    url_part,
                );
                let url =
                    <Dracoon<Connected> as UploadInternal<R>>::create_s3_upload_urls::<'_, '_>(
                        self,
                        upload_channel.upload_id.clone(),
                        url_req,
                    )
                    .await
                    .map_err(|err| {
                        error!("Error creating S3 upload urls: {}", err);
                        err
                    })?;

                let url = url.urls.first().expect("Creating S3 url failed");

                // truncation is safe because chunk_size is 32 MB
                #[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
                let curr_pos: u64 = ((url_part - 1) * (CHUNK_SIZE as u32)) as u64;

                let e_tag = <Dracoon<Connected> as StreamUploadInternal<Connected>>::upload_stream_to_s3(
                    self,
                    Box::pin(stream),
                    url,
                    upload_options.file_meta.clone(),
                    n,
                    Some(curr_pos),
                    cloneable_callback.clone(),
                )
                .await
                .map_err(|err| {
                    error!("Error uploading stream to S3: {}", err);
                    err
                })?;

                s3_parts.push(S3FileUploadPart::new(url_part, e_tag));
            }

            Err(err) => {
                error!("Error reading file: {}", err);
                return Err(DracoonClientError::IoError);
            }
        }

        // finalize upload
        let complete_upload_req = CompleteS3FileUploadRequest::builder(s3_parts)
            .with_resolution_strategy(resolution_strategy)
            .with_keep_share_links(keep_share_links)
            .with_file_key(file_key)
            .build();

        <Dracoon<Connected> as UploadInternal<R>>::finalize_upload::<'_, '_>(
            self,
            upload_channel.upload_id.clone(),
            complete_upload_req,
        )
        .await
        .map_err(|err| {
            error!("Error finalizing upload: {}", err);
            err
        })?;

        // get upload status
        // return node if upload is done
        // return error if upload failed
        // polling with exponential backoff
        let mut sleep_duration = POLLING_START_DELAY;
        loop {
            let status_response = <Dracoon<Connected> as UploadInternal<R>>::get_upload_status(
                self,
                upload_channel.upload_id.clone(),
            )
            .await
            .map_err(|err| {
                error!("Error getting upload status: {}", err);
                err
            })?;

            match status_response.status {
                S3UploadStatus::Done => {
                    // fetch missing keys (limit 50)
                    let missing_keys =
                        <Dracoon<Connected> as UploadInternal<R>>::get_missing_file_keys(
                            self,
                            status_response
                                .node
                                .as_ref()
                                .expect("Node must be set if status is done")
                                .id,
                        )
                        .await
                        .map_err(|err| {
                            error!("Error getting missing file keys: {}", err);
                            err
                        })?;

                    // encrypt plain file key for each user
                    let key_reqs = missing_keys
                        .users
                        .into_iter()
                        .flat_map::<Result<UserFileKeySetRequest, DracoonClientError>, _>(|user| {
                            let user_id = user.id;
                            let file_id = status_response
                                .node
                                .as_ref()
                                .expect("Node must be set if status is done")
                                .id;
                            let public_key = user.public_key_container;
                            let file_key = DracoonCrypto::encrypt_file_key(
                                plain_file_key.clone(),
                                public_key,
                            )?;
                            let set_key_req =
                                UserFileKeySetRequest::new(user_id, file_id, file_key);
                            Ok(set_key_req)
                        })
                        .collect::<Vec<_>>();

                    drop(plain_file_key);
                    // set file keys
                    if !key_reqs.is_empty() {
                        <Dracoon<Connected> as UploadInternal<R>>::set_file_keys(
                            self,
                            key_reqs.into(),
                        )
                        .await
                        .map_err(|err| {
                            error!("Error setting file keys: {}", err);
                            err
                        })?;
                    }

                    return Ok(status_response
                        .node
                        .expect("Node must be set if status is done"));
                }
                S3UploadStatus::Error => {
                    return Err(DracoonClientError::Http(
                        status_response
                            .error_details
                            .expect("Error message must be set if status is error"),
                    ));
                }
                _ => {
                    tokio::time::sleep(Duration::from_millis(sleep_duration)).await;
                    sleep_duration *= 2;
                }
            }
        }
    }

    async fn get_missing_file_keys(
        &self,
        file_id: u64,
    ) -> Result<MissingKeysResponse, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{NODES_BASE}/{MISSING_FILE_KEYS}");

        let mut api_url = self.build_api_url(&url_part);

        api_url
            .query_pairs_mut()
            .append_pair("file_id", file_id.to_string().as_str())
            .append_pair("limit", MISSING_KEYS_BATCH.to_string().as_str())
            .finish();

        let response = self
            .client
            .http
            .get(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .send()
            .await?;

        MissingKeysResponse::from_response(response).await
    }

    async fn set_file_keys(
        &self,
        keys_batch_req: UserFileKeySetBatchRequest,
    ) -> Result<(), DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{NODES_BASE}/{FILES_BASE}/{FILES_KEYS}");

        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .post(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .json(&keys_batch_req)
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
/// helper to parse upload options (file meta and upload options)
pub fn parse_upload_options(
    upload_options: &UploadOptions,
) -> (
    u8,
    DateTime<Utc>,
    DateTime<Utc>,
    ObjectExpiration,
    ResolutionStrategy,
    bool,
) {
    let classification = upload_options.classification.unwrap_or(2);
    let timestamp_modification = upload_options.file_meta.3.unwrap_or_else(Utc::now);
    let timestamp_creation = upload_options.file_meta.2.unwrap_or_else(Utc::now);
    let expiration = upload_options.clone().expiration.unwrap_or_default();
    let resolution_strategy = upload_options
        .resolution_strategy
        .as_ref()
        .unwrap_or(&ResolutionStrategy::AutoRename);
    let keep_share_links = upload_options.keep_share_links.unwrap_or(false);

    (
        classification,
        timestamp_creation,
        timestamp_modification,
        expiration,
        resolution_strategy.clone(),
        keep_share_links,
    )
}

/// helper to calculate the number of S3 urls and the size of the last chunk
pub fn calculate_s3_url_count(total_size: u64, chunk_size: u64) -> (u32, u64) {
    let mut urls = total_size / chunk_size;
    if total_size % chunk_size != 0 {
        urls += 1;
    }

    // return url count and last chunk size
    (
        urls.try_into().expect("overflow size to chunk"),
        total_size % chunk_size,
    )
}

#[async_trait]
impl<R: AsyncRead + Sync + Send + Unpin + 'static> UploadInternalNfs<R, Connected> for Dracoon<Connected> {
    async fn upload_to_nfs_unencrypted(
        &self,
        parent_node: &Node,
        upload_options: UploadOptions,
        mut reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<Node, DracoonClientError> {
        // parse upload options
        let (
            classification,
            timestamp_creation,
            timestamp_modification,
            expiration,
            resolution_strategy,
            keep_share_links,
        ) = parse_upload_options(&upload_options);

        let fm = upload_options.file_meta.clone();

        let chunk_size = chunk_size.unwrap_or(CHUNK_SIZE);

        // create upload channel
        let file_upload_req = CreateFileUploadRequest::builder(parent_node.id, fm.0)
            .with_classification(classification)
            .with_size(fm.1)
            .with_timestamp_modification(timestamp_modification)
            .with_timestamp_creation(timestamp_creation)
            .with_expiration(expiration)
            .with_direct_s3_upload(false)
            .build();

        let upload_channel = <Dracoon<Connected> as UploadInternal<R>>::create_upload_channel::<
            '_,
            '_,
        >(self, file_upload_req)
        .await
        .map_err(|err| {
            error!("Error creating upload channel: {}", err);
            err
        })?;

        let (count_chunks, last_chunk_size) = calculate_s3_url_count(fm.1, chunk_size as u64);
        let mut chunk_part: u32 = 1;

        let cloneable_callback = callback.map(CloneableUploadProgressCallback::new);

        if count_chunks > 1 {
            while chunk_part < count_chunks {
                let mut buffer = vec![0; chunk_size];

                match reader.read_exact(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        buffer.truncate(n);
                        let chunk = bytes::Bytes::from(buffer);

                        let stream: async_stream::__private::AsyncStream<
                            Result<bytes::Bytes, std::io::Error>,
                            _,
                        > = async_stream::stream! {
                            yield Ok(chunk);
                        };

                        let url = upload_channel.upload_url.clone();

                        // truncation is safe because chunk_size is 32 MB
                        #[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
                        let curr_pos: u64 = ((chunk_part - 1) * (chunk_size as u32)) as u64;

                        self.upload_stream_to_nfs(
                            Box::pin(stream),
                            &url,
                            upload_options.file_meta.clone(),
                            n,
                            Some(curr_pos),
                            cloneable_callback.clone(),
                        )
                        .await?;

                        chunk_part += 1;
                    }
                    Err(err) => {
                        error!("Error reading file: {}", err);
                        return Err(DracoonClientError::IoError);
                    }
                }
            }
        }

        // upload last chunk
        let mut buffer = vec![
            0;
            last_chunk_size
                .try_into()
                .expect("size not larger than 32 MB")
        ];
        match reader.read_exact(&mut buffer).await {
            Ok(n) => {
                buffer.truncate(n);
                let chunk = bytes::Bytes::from(buffer);
                let stream: async_stream::__private::AsyncStream<
                    Result<bytes::Bytes, std::io::Error>,
                    _,
                > = async_stream::stream! {
                    // TODO: chunk stream for better progress
                    // currently the progress is only updated per chunk
                    yield Ok(chunk);

                };

                let url = upload_channel.upload_url.clone();

                let curr_pos: u64 = (chunk_part - 1) as u64 * (CHUNK_SIZE as u64);

                let e_tag = self.upload_stream_to_nfs(
                    Box::pin(stream),
                    &url,
                    upload_options.file_meta.clone(),
                    n,
                    Some(curr_pos),
                    cloneable_callback.clone(),
                )
                .await?;
            }
            Err(err) => {
                error!("Error reading file: {}", err);
                return Err(DracoonClientError::IoError);
            }
        }

        // finalize upload
        let complete_upload_req = CompleteUploadRequest::builder()
            .with_resolution_strategy(resolution_strategy)
            .with_keep_share_links(keep_share_links)
            .build();

        let node = <Dracoon<Connected> as UploadInternalNfs<R, Connected>>::finalize_nfs_upload::<'_, '_>(
            self,
            upload_channel.token.clone(),
            complete_upload_req,
        )
        .await
        .map_err(|err| {
            error!("Error finalizing upload: {}", err);
            err
        })?;

        Ok(node)
    }

    async fn upload_to_nfs_encrypted(
        &self,
        parent_node: &Node,
        upload_options: UploadOptions,
        mut reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<Node, DracoonClientError> {
        let keypair = self.get_keypair(None).await?;

        let chunk_size = chunk_size.unwrap_or(CHUNK_SIZE);

        let mut crypto_buff =
            vec![0u8; upload_options.file_meta.1.try_into().expect("size not larger than 32 MB")];
        let mut read_buff = vec![0u8; upload_options.file_meta.1.try_into().expect("size not larger than 32 MB")];
        let mut crypter = DracoonCrypto::encrypter(&mut crypto_buff)?;

        while let Ok(chunk) = reader.read(&mut read_buff).await {
            if chunk == 0 {
                break;
            }
            crypter.update(&read_buff[..chunk])?;
        }
        crypter.finalize()?;
        // drop the read buffer after completing the encryption
        drop(read_buff);

        //TODO: rewrite without buffer clone
        let enc_bytes = crypter.get_message().clone();

        assert_eq!(enc_bytes.len() as u64, upload_options.file_meta.1);

        let mut crypto_reader = BufReader::new(enc_bytes.as_slice());
        let plain_file_key = crypter.get_plain_file_key();
        let file_key = DracoonCrypto::encrypt_file_key(plain_file_key.clone(), keypair)?;
        // drop the crypto buffer (enc bytes are still in the reader)
        drop(crypto_buff);

        let (
            classification,
            timestamp_creation,
            timestamp_modification,
            expiration,
            resolution_strategy,
            keep_share_links,
        ) = parse_upload_options(&upload_options);

        let fm = upload_options.file_meta.clone();

        // create upload channel
        let file_upload_req = CreateFileUploadRequest::builder(parent_node.id, fm.0)
            .with_classification(classification)
            .with_size(fm.1)
            .with_timestamp_modification(timestamp_modification)
            .with_timestamp_creation(timestamp_creation)
            .with_expiration(expiration)
            .with_direct_s3_upload(false)
            .build();

        let upload_channel = <Dracoon<Connected> as UploadInternal<R>>::create_upload_channel::<
            '_,
            '_,
        >(self, file_upload_req)
        .await
        .map_err(|err| {
            error!("Error creating upload channel: {}", err);
            err
        })?;

        let (count_chunks, last_chunk_size) = calculate_s3_url_count(fm.1, chunk_size as u64);
        let mut chunk_part: u32 = 1;

        let cloneable_callback = callback.map(CloneableUploadProgressCallback::new);

        if count_chunks > 1 {
            while chunk_part < count_chunks {
                let mut buffer = vec![0; chunk_size];

                match crypto_reader.read_exact(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk_len = n;
                        buffer.truncate(chunk_len);
                        let chunk = bytes::Bytes::from(buffer);

                        let stream: async_stream::__private::AsyncStream<
                            Result<bytes::Bytes, std::io::Error>,
                            _,
                        > = async_stream::stream! {
                            yield Ok(chunk);
                        };

                        let url = upload_channel.upload_url.clone();

                        let curr_pos: u64 = (chunk_part - 1) as u64 * (chunk_size as u64);

                        self.upload_stream_to_nfs(
                            Box::pin(stream),
                            &url,
                            upload_options.file_meta.clone(),
                            chunk_len,
                            Some(curr_pos),
                            cloneable_callback.clone(),
                        )
                        .await
                        .map_err(|err| {
                            error!("Error uploading stream to S3: {}", err);
                            err
                        })?;

                        chunk_part += 1;
                    }
                    Err(err) => return Err(DracoonClientError::IoError),
                }
            }
        }

        // upload last chunk
        let mut buffer = vec![
            0;
            last_chunk_size
                .try_into()
                .expect("size not larger than 32 MB")
        ];
        match crypto_reader.read_exact(&mut buffer).await {
            Ok(n) => {
                buffer.truncate(n);
                let chunk = bytes::Bytes::from(buffer);
                let stream: async_stream::__private::AsyncStream<
                    Result<bytes::Bytes, std::io::Error>,
                    _,
                > = async_stream::stream! {
                    // TODO: chunk stream for better progress
                    yield Ok(chunk);

                };

                let url = upload_channel.upload_url.clone();

                // truncation is safe because chunk_size is 32 MB
                #[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
                let curr_pos: u64 = ((chunk_part - 1) * (CHUNK_SIZE as u32)) as u64;

                self.upload_stream_to_nfs(
                    Box::pin(stream),
                    &url,
                    upload_options.file_meta.clone(),
                    n,
                    Some(curr_pos),
                    cloneable_callback.clone(),
                )
                .await
                .map_err(|err| {
                    error!("Error uploading stream to NFS: {}", err);
                    err
                })?;
            }

            Err(err) => {
                error!("Error reading file: {}", err);
                return Err(DracoonClientError::IoError);
            }
        }

        // finalize upload
        let complete_upload_req = CompleteUploadRequest::builder()
            .with_resolution_strategy(resolution_strategy)
            .with_keep_share_links(keep_share_links)
            .with_file_key(file_key)
            .build();

        let node = <Dracoon<Connected> as UploadInternalNfs<R, Connected>>::finalize_nfs_upload::<'_, '_>(
            self,
            upload_channel.token.clone(),
            complete_upload_req,
        )
        .await
        .map_err(|err| {
            error!("Error finalizing upload: {}", err);
            err
        })?;

        // fetch missing keys (limit 50)
        let missing_keys =
            <Dracoon<Connected> as UploadInternal<R>>::get_missing_file_keys(self, node.id)
                .await
                .map_err(|err| {
                    error!("Error getting missing file keys: {}", err);
                    err
                })?;

        // encrypt plain file key for each user
        let key_reqs = missing_keys
            .users
            .into_iter()
            .flat_map::<Result<UserFileKeySetRequest, DracoonClientError>, _>(|user| {
                let user_id = user.id;
                let file_id = node.id;
                let public_key = user.public_key_container;
                let file_key = DracoonCrypto::encrypt_file_key(plain_file_key.clone(), public_key)?;
                let set_key_req = UserFileKeySetRequest::new(user_id, file_id, file_key);
                Ok(set_key_req)
            })
            .collect::<Vec<_>>();

        drop(plain_file_key);
        // set file keys
        if !key_reqs.is_empty() {
            <Dracoon<Connected> as UploadInternal<R>>::set_file_keys(self, key_reqs.into())
                .await
                .map_err(|err| {
                    error!("Error setting file keys: {}", err);
                    err
                })?;
        }

        Ok(node)
    }

    async fn finalize_nfs_upload(
        &self,
        upload_token: String,
        complete_file_upload_req: CompleteUploadRequest,
    ) -> Result<Node, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{UPLOADS_BASE}/{upload_token}");
        let api_url = self.build_api_url(&url_part);
        let res = self
            .client
            .http
            .put(api_url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&complete_file_upload_req)
            .send()
            .await?;

        Node::from_response(res).await
    }
}

#[cfg(test)]
mod tests {

    use std::io::Cursor;

    use dco3_crypto::FileKeyVersion;

    use crate::tests::dracoon::get_connected_client;
    use crate::tests::nodes::tests::assert_node;

    use super::*;

    #[tokio::test]
    async fn test_create_upload_channel() {
        let (client, mut mock_server) = get_connected_client().await;

        let channel_res = include_str!("../tests/responses/upload/upload_channel_ok.json");

        let upload_channel_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads")
            .with_status(200)
            .with_body(channel_res)
            .with_header("content-type", "application/json")
            .create();

        let channel_req = CreateFileUploadRequest::builder(123, "test".into()).build();

        let upload_channel =
            <Dracoon<Connected> as UploadInternal<BufReader<&[u8]>>>::create_upload_channel(
                &client,
                channel_req,
            )
            .await
            .unwrap();

        upload_channel_mock.assert();

        assert_eq!(upload_channel.upload_id, "string");
        assert_eq!(upload_channel.upload_url, "string");
        assert_eq!(upload_channel.token, "string");
    }

    #[tokio::test]
    async fn test_create_s3_upload_urls() {
        let (client, mut mock_server) = get_connected_client().await;

        let s3_urls_res = include_str!("../tests/responses/upload/s3_urls_ok.json");

        let s3_urls_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads/123/s3_urls")
            .with_status(200)
            .with_body(s3_urls_res)
            .with_header("content-type", "application/json")
            .create();

        let s3_urls_req = GeneratePresignedUrlsRequest::new(123456, 1, 1);

        let s3_urls =
            <Dracoon<Connected> as UploadInternal<BufReader<&[u8]>>>::create_s3_upload_urls(
                &client,
                "123".into(),
                s3_urls_req,
            )
            .await
            .unwrap();

        s3_urls_mock.assert();

        assert_eq!(s3_urls.urls.len(), 1);
        assert_eq!(
            s3_urls.urls.first().unwrap().url,
            "https://test.dracoon.com/not/real/upload_url"
        );
        assert_eq!(s3_urls.urls.first().unwrap().part_number, 1);
    }

    #[tokio::test]
    async fn test_finalize_upload() {
        let (client, mut mock_server) = get_connected_client().await;

        let finalize_mock = mock_server
            .mock("PUT", "/api/v4/nodes/files/uploads/123/s3")
            .with_status(202)
            .with_header("content-type", "application/json")
            .create();

        let req =
            CompleteS3FileUploadRequest::builder(vec![S3FileUploadPart::new(1, "123".into())])
                .build();

        <Dracoon<Connected> as UploadInternal<BufReader<&[u8]>>>::finalize_upload(
            &client,
            "123".into(),
            req,
        )
        .await
        .unwrap();

        finalize_mock.assert();
    }

    #[tokio::test]
    async fn test_get_upload_status() {
        let (client, mut mock_server) = get_connected_client().await;

        let status_res = include_str!("../tests/responses/upload/upload_status_ok.json");

        let status_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/uploads/123")
            .with_status(200)
            .with_body(status_res)
            .with_header("content-type", "application/json")
            .create();

        let upload_status =
            <Dracoon<Connected> as UploadInternal<BufReader<&[u8]>>>::get_upload_status(
                &client,
                "123".into(),
            )
            .await
            .unwrap();

        status_mock.assert();

        assert_eq!(upload_status.status, S3UploadStatus::Done);

        assert_node(&upload_status.node.unwrap());
    }

    #[tokio::test]
    async fn test_get_upload_status_pending() {
        let (client, mut mock_server) = get_connected_client().await;

        let status_res = include_str!("../tests/responses/upload/upload_status_pending_ok.json");

        let status_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/uploads/123")
            .with_status(200)
            .with_body(status_res)
            .with_header("content-type", "application/json")
            .create();

        let upload_status =
            <Dracoon<Connected> as UploadInternal<BufReader<&[u8]>>>::get_upload_status(
                &client,
                "123".into(),
            )
            .await
            .unwrap();

        status_mock.assert();

        assert_eq!(upload_status.status, S3UploadStatus::Finishing);
    }

    #[tokio::test]
    async fn test_get_missing_file_keys() {
        let (client, mut mock_server) = get_connected_client().await;
        let missing_keys_res = include_str!("../tests/responses/nodes/missing_file_keys_ok.json");

        let missing_keys_mock = mock_server
            .mock("GET", "/api/v4/nodes/missingFileKeys?file_id=123&limit=50")
            .with_status(200)
            .with_body(missing_keys_res)
            .with_header("content-type", "application/json")
            .create();

        let missing_keys =
            <Dracoon<Connected> as UploadInternal<BufReader<&[u8]>>>::get_missing_file_keys(
                &client, 123,
            )
            .await
            .unwrap();

        missing_keys_mock.assert();

        assert_eq!(missing_keys.items.len(), 1);
        assert_eq!(missing_keys.users.len(), 1);
        assert_eq!(missing_keys.files.len(), 1);
        assert_eq!(missing_keys.items.first().unwrap().file_id, 3);
        assert_eq!(missing_keys.items.first().unwrap().user_id, 2);
        assert_eq!(missing_keys.users.first().unwrap().id, 2);
        assert_eq!(missing_keys.files.first().unwrap().id, 3);
        assert_eq!(
            missing_keys.files.first().unwrap().file_key_container.key,
            "string"
        );
        assert_eq!(
            missing_keys.files.first().unwrap().file_key_container.iv,
            "string"
        );
        assert_eq!(
            missing_keys
                .files
                .first()
                .unwrap()
                .file_key_container
                .version,
            FileKeyVersion::RSA4096_AES256GCM
        );
    }

    #[tokio::test]
    async fn test_set_file_keys() {}

    #[tokio::test]
    async fn test_upload_stream_to_s3() {
        let mock_bytes: [u8; 16] = [
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ];

        let (client, mut mock_server) = get_connected_client().await;

        let chunk = bytes::Bytes::from(mock_bytes.to_vec());

        let stream: async_stream::__private::AsyncStream<Result<bytes::Bytes, std::io::Error>, _> = async_stream::stream! {
            yield Ok(chunk);
        };

        let upload_mock = mock_server
            .mock("PUT", "/some/upload/url")
            .with_status(202)
            .with_header("etag", "string")
            .create();

        let upload_url = format!("{}some/upload/url", client.get_base_url());

        let upload_url = PresignedUrl {
            url: upload_url,
            part_number: 1,
        };

        let file_meta = FileMeta::builder()
            .with_name("test".into())
            .with_size(16)
            .build();

        let e_tag = <Dracoon<Connected> as StreamUploadInternal<Connected>>::upload_stream_to_s3(
            &client,
            Box::pin(stream),
            &upload_url,
            file_meta,
            16,
            None,
            None,
        )
        .await
        .unwrap();

        upload_mock.assert();
        assert_eq!(e_tag, "string".to_string());
    }

    #[tokio::test]
    async fn test_upload_to_s3_unencrypted() {
        let (client, mut mock_server) = get_connected_client().await;

        let parent_node: Node =
            serde_json::from_str(include_str!("../tests/responses/nodes/node_ok.json")).unwrap();

        let mock_bytes: Vec<u8> = vec![
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ];

        let reader = Cursor::new(mock_bytes);
        let reader_clone = BufReader::new(reader);

        let file_meta = FileMeta::builder()
            .with_name("test".into())
            .with_size(16)
            .build();

        let upload_options = UploadOptions::builder(file_meta).build();

        // mock upload channel
        let channel_res = include_str!("../tests/responses/upload/upload_channel_ok.json");

        let upload_channel_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads")
            .with_status(201)
            .with_body(channel_res)
            .with_header("content-type", "application/json")
            .create();

        // mock S3 urls
        let s3_urls_response =
            include_str!("../tests/responses/upload/s3_urls_ok_with_placeholder.json");
        let s3_urls_response =
            s3_urls_response.replace("$base_url/", client.get_base_url().as_str());

        let s3_urls_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads/string/s3_urls")
            .with_status(201)
            .with_body(s3_urls_response.clone())
            .with_header("content-type", "application/json")
            .create();

        let upload_res =
            serde_json::from_str::<PresignedUrlList>(s3_urls_response.as_str()).unwrap();

        // mock upload to S3
        let upload_mock = mock_server
            .mock("PUT", "/upload_url")
            .with_status(202)
            .with_header("etag", "string")
            .create();

        // mock finalize upload
        let finalize_mock = mock_server
            .mock("PUT", "/api/v4/nodes/files/uploads/string/s3")
            .with_status(202)
            .create();

        // mock upload status
        let status_res = include_str!("../tests/responses/upload/upload_status_ok.json");
        let status_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/uploads/string")
            .with_status(200)
            .with_body(status_res)
            .with_header("content-type", "application/json")
            .create();

        let node =
            <Dracoon<Connected> as UploadInternal<Cursor<Vec<u8>>>>::upload_to_s3_unencrypted(
                &client,
                &parent_node,
                upload_options,
                reader_clone,
                None,
                None,
            )
            .await
            .unwrap();

        upload_channel_mock.assert();
        s3_urls_mock.assert();
        upload_mock.assert();
        finalize_mock.assert();
        status_mock.assert();

        assert_node(&node);
    }

    #[tokio::test]
    async fn test_upload_to_s3_unencrypted_no_content() {
        let (client, mut mock_server) = get_connected_client().await;

        let parent_node: Node =
            serde_json::from_str(include_str!("../tests/responses/nodes/node_ok.json")).unwrap();

        // test 0KB file
        let mock_bytes: Vec<u8> = vec![];

        let reader = Cursor::new(mock_bytes);
        let reader_clone = BufReader::new(reader);

        let file_meta = FileMeta::builder()
            .with_name("test".into())
            .with_size(0)
            .build();

        let upload_options = UploadOptions::builder(file_meta).build();

        // mock upload channel
        let channel_res = include_str!("../tests/responses/upload/upload_channel_ok.json");

        let upload_channel_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads")
            .with_status(201)
            .with_body(channel_res)
            .with_header("content-type", "application/json")
            .create();

        // mock S3 urls
        let s3_urls_response =
            include_str!("../tests/responses/upload/s3_urls_ok_with_placeholder.json");
        let s3_urls_response =
            s3_urls_response.replace("$base_url/", client.get_base_url().as_str());

        let s3_urls_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads/string/s3_urls")
            .with_status(201)
            .with_body(s3_urls_response.clone())
            .with_header("content-type", "application/json")
            .create();

        let upload_res =
            serde_json::from_str::<PresignedUrlList>(s3_urls_response.as_str()).unwrap();

        // mock upload to S3
        let upload_mock = mock_server
            .mock("PUT", "/upload_url")
            .with_status(202)
            .with_header("etag", "string")
            .create();

        // mock finalize upload
        let finalize_mock = mock_server
            .mock("PUT", "/api/v4/nodes/files/uploads/string/s3")
            .with_status(202)
            .create();

        // mock upload status
        let status_res = include_str!("../tests/responses/upload/upload_status_ok.json");
        let status_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/uploads/string")
            .with_status(200)
            .with_body(status_res)
            .with_header("content-type", "application/json")
            .create();

        let node =
            <Dracoon<Connected> as UploadInternal<Cursor<Vec<u8>>>>::upload_to_s3_unencrypted(
                &client,
                &parent_node,
                upload_options,
                reader_clone,
                None,
                None,
            )
            .await
            .unwrap();

        upload_channel_mock.assert();
        s3_urls_mock.assert();
        upload_mock.assert();
        finalize_mock.assert();
        status_mock.assert();

        assert_node(&node);
    }

    #[tokio::test]
    async fn test_upload_to_s3_encrypted() {
        let (client, mut mock_server) = get_connected_client().await;

        let parent_node: Node =
            serde_json::from_str(include_str!("../tests/responses/nodes/node_ok.json")).unwrap();

        let mock_bytes: Vec<u8> = vec![
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ];

        let reader = Cursor::new(mock_bytes);
        let reader_clone = BufReader::new(reader);

        let keypair =
            DracoonCrypto::create_plain_user_keypair(dco3_crypto::UserKeyPairVersion::RSA4096)
                .unwrap();
        let enc_keypair =
            DracoonCrypto::encrypt_private_key("TopSecret1234!", keypair.clone()).unwrap();
        let enc_keypair_json = serde_json::to_string(&enc_keypair).unwrap();

        let keypair_mock = mock_server
            .mock("GET", "/api/v4/user/account/keypair")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(enc_keypair_json)
            .create();

        let _kp = client
            .get_keypair(Some("TopSecret1234!".into()))
            .await
            .unwrap();

        keypair_mock.assert();

        let file_meta = FileMeta::builder()
            .with_name("test".into())
            .with_size(16)
            .build();

        let upload_options = UploadOptions::builder(file_meta).build();

        // mock upload channel
        let channel_res = include_str!("../tests/responses/upload/upload_channel_ok.json");

        let upload_channel_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads")
            .with_status(201)
            .with_body(channel_res)
            .with_header("content-type", "application/json")
            .create();

        // mock S3 urls
        let s3_urls_response =
            include_str!("../tests/responses/upload/s3_urls_ok_with_placeholder.json");
        let s3_urls_response =
            s3_urls_response.replace("$base_url/", client.get_base_url().as_str());

        let s3_urls_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads/string/s3_urls")
            .with_status(201)
            .with_body(s3_urls_response.clone())
            .with_header("content-type", "application/json")
            .create();

        let upload_res =
            serde_json::from_str::<PresignedUrlList>(s3_urls_response.as_str()).unwrap();

        // mock upload to S3
        let upload_mock = mock_server
            .mock("PUT", "/upload_url")
            .with_status(202)
            .with_header("etag", "string")
            .create();

        // mock finalize upload
        let finalize_mock = mock_server
            .mock("PUT", "/api/v4/nodes/files/uploads/string/s3")
            .with_status(202)
            .create();

        // mock upload status
        let status_res = include_str!("../tests/responses/upload/upload_status_ok.json");
        let status_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/uploads/string")
            .with_status(200)
            .with_body(status_res)
            .with_header("content-type", "application/json")
            .create();

        // mock missing file keys
        let missing_keys = include_str!("../tests/responses/nodes/missing_file_keys_empty_ok.json");
        let keys_mock = mock_server
            .mock("GET", "/api/v4/nodes/missingFileKeys?file_id=2&limit=50")
            .with_status(200)
            .with_body(missing_keys)
            .with_header("content-type", "application/json")
            .create();

        let node = <Dracoon<Connected> as UploadInternal<Cursor<Vec<u8>>>>::upload_to_s3_encrypted(
            &client,
            &parent_node,
            upload_options,
            reader_clone,
            None,
            None,
        )
        .await
        .unwrap();

        upload_channel_mock.assert();
        s3_urls_mock.assert();
        upload_mock.assert();
        finalize_mock.assert();
        status_mock.assert();
        keys_mock.assert();

        assert_node(&node);
    }

    #[tokio::test]
    async fn test_upload_to_s3_encrypted_no_content() {
        let (client, mut mock_server) = get_connected_client().await;

        let parent_node: Node =
            serde_json::from_str(include_str!("../tests/responses/nodes/node_ok.json")).unwrap();

        // empty file
        let mock_bytes: Vec<u8> = vec![];

        let reader = Cursor::new(mock_bytes);
        let reader_clone = BufReader::new(reader);

        let keypair =
            DracoonCrypto::create_plain_user_keypair(dco3_crypto::UserKeyPairVersion::RSA4096)
                .unwrap();
        let enc_keypair =
            DracoonCrypto::encrypt_private_key("TopSecret1234!", keypair.clone()).unwrap();
        let enc_keypair_json = serde_json::to_string(&enc_keypair).unwrap();

        let keypair_mock = mock_server
            .mock("GET", "/api/v4/user/account/keypair")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(enc_keypair_json)
            .create();

        let _kp = client
            .get_keypair(Some("TopSecret1234!".into()))
            .await
            .unwrap();

        keypair_mock.assert();

        let file_meta = FileMeta::builder()
            .with_name("test".into())
            .with_size(0)
            .build();

        let upload_options = UploadOptions::builder(file_meta).build();

        // mock upload channel
        let channel_res = include_str!("../tests/responses/upload/upload_channel_ok.json");

        let upload_channel_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads")
            .with_status(201)
            .with_body(channel_res)
            .with_header("content-type", "application/json")
            .create();

        // mock S3 urls
        let s3_urls_response =
            include_str!("../tests/responses/upload/s3_urls_ok_with_placeholder.json");
        let s3_urls_response =
            s3_urls_response.replace("$base_url/", client.get_base_url().as_str());

        let s3_urls_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads/string/s3_urls")
            .with_status(201)
            .with_body(s3_urls_response.clone())
            .with_header("content-type", "application/json")
            .create();

        let upload_res =
            serde_json::from_str::<PresignedUrlList>(s3_urls_response.as_str()).unwrap();

        // mock upload to S3
        let upload_mock = mock_server
            .mock("PUT", "/upload_url")
            .with_status(202)
            .with_header("etag", "string")
            .create();

        // mock finalize upload
        let finalize_mock = mock_server
            .mock("PUT", "/api/v4/nodes/files/uploads/string/s3")
            .with_status(202)
            .create();

        // mock upload status
        let status_res = include_str!("../tests/responses/upload/upload_status_ok.json");
        let status_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/uploads/string")
            .with_status(200)
            .with_body(status_res)
            .with_header("content-type", "application/json")
            .create();

        // mock missing file keys
        let missing_keys = include_str!("../tests/responses/nodes/missing_file_keys_empty_ok.json");
        let keys_mock = mock_server
            .mock("GET", "/api/v4/nodes/missingFileKeys?file_id=2&limit=50")
            .with_status(200)
            .with_body(missing_keys)
            .with_header("content-type", "application/json")
            .create();

        let node = <Dracoon<Connected> as UploadInternal<Cursor<Vec<u8>>>>::upload_to_s3_encrypted(
            &client,
            &parent_node,
            upload_options,
            reader_clone,
            None,
            None,
        )
        .await
        .unwrap();

        upload_channel_mock.assert();
        s3_urls_mock.assert();
        upload_mock.assert();
        finalize_mock.assert();
        status_mock.assert();
        keys_mock.assert();

        assert_node(&node);
    }

    #[tokio::test]
    async fn test_full_upload_unencrypted_s3() {
        let (client, mut mock_server) = get_connected_client().await;

        let parent_node: Node =
            serde_json::from_str(include_str!("../tests/responses/nodes/node_ok.json")).unwrap();

        let mock_bytes: Vec<u8> = vec![
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ];

        let reader = Cursor::new(mock_bytes);
        let reader_clone = BufReader::new(reader);

        let file_meta = FileMeta::builder()
            .with_name("test".into())
            .with_size(16)
            .build();

        let sys_info_res = include_str!("../tests/responses/public/system_info_ok.json");

        let system_info_mock = mock_server
            .mock("GET", "/api/v4/public/system/info")
            .with_status(200)
            .with_body(sys_info_res)
            .with_header("content-type", "application/json")
            .create();

        let upload_options = UploadOptions::builder(file_meta).build();

        // mock upload channel
        let channel_res = include_str!("../tests/responses/upload/upload_channel_ok.json");

        let upload_channel_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads")
            .with_status(201)
            .with_body(channel_res)
            .with_header("content-type", "application/json")
            .create();

        // mock S3 urls
        let s3_urls_response =
            include_str!("../tests/responses/upload/s3_urls_ok_with_placeholder.json");
        let s3_urls_response =
            s3_urls_response.replace("$base_url/", client.get_base_url().as_str());

        let s3_urls_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads/string/s3_urls")
            .with_status(201)
            .with_body(s3_urls_response.clone())
            .with_header("content-type", "application/json")
            .create();

        let upload_res =
            serde_json::from_str::<PresignedUrlList>(s3_urls_response.as_str()).unwrap();

        // mock upload to S3
        let upload_mock = mock_server
            .mock("PUT", "/upload_url")
            .with_status(202)
            .with_header("etag", "string")
            .create();

        // mock finalize upload
        let finalize_mock = mock_server
            .mock("PUT", "/api/v4/nodes/files/uploads/string/s3")
            .with_status(202)
            .create();

        // mock upload status
        let status_res = include_str!("../tests/responses/upload/upload_status_ok.json");
        let status_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/uploads/string")
            .with_status(200)
            .with_body(status_res)
            .with_header("content-type", "application/json")
            .create();

        let node = <Dracoon<Connected> as Upload<Cursor<Vec<u8>>>>::upload(
            &client,
            &parent_node,
            upload_options,
            reader_clone,
            None,
            None,
        )
        .await
        .unwrap();

        upload_channel_mock.assert();
        s3_urls_mock.assert();
        upload_mock.assert();
        finalize_mock.assert();
        status_mock.assert();

        assert_node(&node);
    }

    #[tokio::test]
    async fn test_full_upload_encrypted_s3() {
        let (client, mut mock_server) = get_connected_client().await;

        let parent_node: Node = serde_json::from_str(include_str!(
            "../tests/responses/nodes/node_encrypted_ok.json"
        ))
        .unwrap();

        let mock_bytes: Vec<u8> = vec![
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ];

        let reader = Cursor::new(mock_bytes);
        let reader_clone = BufReader::new(reader);

        let sys_info_res = include_str!("../tests/responses/public/system_info_ok.json");

        let system_info_mock = mock_server
            .mock("GET", "/api/v4/public/system/info")
            .with_status(200)
            .with_body(sys_info_res)
            .with_header("content-type", "application/json")
            .create();

        let keypair =
            DracoonCrypto::create_plain_user_keypair(dco3_crypto::UserKeyPairVersion::RSA4096)
                .unwrap();
        let enc_keypair =
            DracoonCrypto::encrypt_private_key("TopSecret1234!", keypair.clone()).unwrap();
        let enc_keypair_json = serde_json::to_string(&enc_keypair).unwrap();

        let keypair_mock = mock_server
            .mock("GET", "/api/v4/user/account/keypair")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(enc_keypair_json)
            .create();

        let _kp = client
            .get_keypair(Some("TopSecret1234!".into()))
            .await
            .unwrap();

        keypair_mock.assert();

        let file_meta = FileMeta::builder()
            .with_name("test".into())
            .with_size(16)
            .build();

        let upload_options = UploadOptions::builder(file_meta).build();

        // mock upload channel
        let channel_res = include_str!("../tests/responses/upload/upload_channel_ok.json");

        let upload_channel_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads")
            .with_status(201)
            .with_body(channel_res)
            .with_header("content-type", "application/json")
            .create();

        // mock S3 urls
        let s3_urls_response =
            include_str!("../tests/responses/upload/s3_urls_ok_with_placeholder.json");
        let s3_urls_response =
            s3_urls_response.replace("$base_url/", client.get_base_url().as_str());

        let s3_urls_mock = mock_server
            .mock("POST", "/api/v4/nodes/files/uploads/string/s3_urls")
            .with_status(201)
            .with_body(s3_urls_response.clone())
            .with_header("content-type", "application/json")
            .create();

        let upload_res =
            serde_json::from_str::<PresignedUrlList>(s3_urls_response.as_str()).unwrap();

        // mock upload to S3
        let upload_mock = mock_server
            .mock("PUT", "/upload_url")
            .with_status(202)
            .with_header("etag", "string")
            .create();

        // mock finalize upload
        let finalize_mock = mock_server
            .mock("PUT", "/api/v4/nodes/files/uploads/string/s3")
            .with_status(202)
            .create();

        // mock upload status
        let status_res = include_str!("../tests/responses/upload/upload_status_ok.json");
        let status_mock = mock_server
            .mock("GET", "/api/v4/nodes/files/uploads/string")
            .with_status(200)
            .with_body(status_res)
            .with_header("content-type", "application/json")
            .create();

        // mock missing file keys
        let missing_keys = include_str!("../tests/responses/nodes/missing_file_keys_empty_ok.json");
        let keys_mock = mock_server
            .mock("GET", "/api/v4/nodes/missingFileKeys?file_id=2&limit=50")
            .with_status(200)
            .with_body(missing_keys)
            .with_header("content-type", "application/json")
            .create();

        let node = <Dracoon<Connected> as Upload<Cursor<Vec<u8>>>>::upload(
            &client,
            &parent_node,
            upload_options,
            reader_clone,
            None,
            None,
        )
        .await
        .unwrap();

        upload_channel_mock.assert();
        s3_urls_mock.assert();
        upload_mock.assert();
        finalize_mock.assert();
        status_mock.assert();
        keys_mock.assert();

        assert_node(&node);
    }

    //TODO: test NFS upload (unencrypted and encrypted)
}
