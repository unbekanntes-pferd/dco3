use async_trait::async_trait;
use dco3_crypto::{DracoonCrypto, DracoonRSACrypto};
use reqwest::StatusCode;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
use tracing::{error, warn};

use crate::{
    constants::{
        DEFAULT_UPLOAD_CHUNK_SIZE, DRACOON_API_PREFIX, FILES_S3_COMPLETE, FILES_S3_URLS,
        POLLING_START_DELAY, PUBLIC_BASE, PUBLIC_SHARES_BASE, PUBLIC_UPLOAD_SHARES,
    },
    nodes::{
        models::StreamingEncryptedUpload,
        upload::{calculate_s3_url_count, StreamUploadInternal},
        CloneableUploadProgressCallback, GeneratePresignedUrlsRequest, PresignedUrlList,
        S3FileUploadPart, S3UploadStatus, UploadOptions, UploadProgressCallback,
    },
    utils::{build_s3_protocol_error, FromResponse},
    DracoonClientError, Public,
};

use super::{
    CompleteS3ShareUploadRequest, CreateShareUploadChannelRequest,
    CreateShareUploadChannelResponse, FileName, PublicEndpoint, PublicUpload, PublicUploadShare,
    PublicUploadedFileData, S3ShareUploadStatus, UserFileKey, UserFileKeyList,
};

fn missing_presigned_url_error() -> DracoonClientError {
    build_s3_protocol_error(
        StatusCode::BAD_GATEWAY,
        "missing_presigned_url",
        "Presigned URL response contained no URLs",
    )
}

fn missing_upload_error_details_error() -> DracoonClientError {
    build_s3_protocol_error(
        StatusCode::BAD_GATEWAY,
        "missing_upload_error_details",
        "Upload status 'error' did not include error details",
    )
}

#[async_trait]
impl<S: Send + Sync, R: AsyncRead + Send + Sync + Unpin + 'static> PublicUpload<R>
    for PublicEndpoint<S>
{
    async fn upload<'r>(
        &'r self,
        access_key: impl Into<String> + Send + Sync,
        share: PublicUploadShare,
        upload_options: UploadOptions,
        reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<FileName, DracoonClientError> {
        let use_s3_storage = self.get_system_info().await?.use_s3_storage;
        let is_encrypted = share.is_encrypted.unwrap_or(false);

        let upload_fn = match (use_s3_storage, is_encrypted) {
            (true, true) => PublicUploadInternal::upload_to_s3_encrypted,
            (true, false) => PublicUploadInternal::upload_to_s3_unencrypted,
            (false, true) => PublicUploadInternalNfs::upload_to_nfs_encrypted,
            (false, false) => PublicUploadInternalNfs::upload_to_nfs_unencrypted,
        };

        upload_fn(
            self,
            access_key.into(),
            &share,
            upload_options,
            reader,
            callback,
            chunk_size,
        )
        .await
    }
}

impl<S> StreamUploadInternal<S> for PublicEndpoint<S> {}

#[async_trait]
impl<S: Send + Sync, R: AsyncRead + Send + Sync + Unpin + 'static> PublicUploadInternal<R, S>
    for PublicEndpoint<S>
{
    async fn create_upload_channel(
        &self,
        access_key: String,
        create_file_upload_req: CreateShareUploadChannelRequest,
    ) -> Result<CreateShareUploadChannelResponse, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{PUBLIC_BASE}/{PUBLIC_SHARES_BASE}/{PUBLIC_UPLOAD_SHARES}/{}",
            access_key
        );

        let url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .post(url)
            .json(&create_file_upload_req)
            .send()
            .await?;

        CreateShareUploadChannelResponse::from_response(response).await
    }

    async fn create_s3_upload_urls(
        &self,
        access_key: String,
        upload_id: String,
        generate_urls_req: GeneratePresignedUrlsRequest,
    ) -> Result<PresignedUrlList, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{PUBLIC_BASE}/{PUBLIC_SHARES_BASE}/{PUBLIC_UPLOAD_SHARES}/{}/{}/{FILES_S3_URLS}",
            access_key, upload_id
        );

        let url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .post(url)
            .json(&generate_urls_req)
            .send()
            .await?;

        PresignedUrlList::from_response(response).await
    }

    async fn upload_to_s3_unencrypted(
        &self,
        access_key: String,
        share: &PublicUploadShare,
        upload_options: UploadOptions,
        mut reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<FileName, DracoonClientError> {
        let fm = upload_options.file_meta.clone();

        let chunk_size = chunk_size.unwrap_or(DEFAULT_UPLOAD_CHUNK_SIZE);

        // create upload channel
        let file_upload_req =
            CreateShareUploadChannelRequest::from_upload_options(&upload_options, Some(true), None);

        let upload_channel =
            <PublicEndpoint<S> as PublicUploadInternal<R, S>>::create_upload_channel(
                self,
                access_key.clone(),
                file_upload_req,
            )
            .await?;

        let mut s3_parts = Vec::new();

        let (count_urls, last_chunk_size) = calculate_s3_url_count(fm.size, chunk_size as u64);
        let mut url_part: u32 = 1;

        let cloneable_callback = callback.map(CloneableUploadProgressCallback::new);

        if count_urls > 1 {
            while url_part < count_urls {
                let mut buffer = vec![0; chunk_size];
                let cb = cloneable_callback.clone();
                let fm = fm.clone();

                match reader.read_exact(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        buffer.truncate(n);
                        let chunk = bytes::Bytes::from(buffer);

                        let stream: async_stream::__private::AsyncStream<
                            Result<bytes::Bytes, std::io::Error>,
                            _,
                        > = async_stream::stream! {
                            let mut buffer = Vec::new();
                            let mut bytes_read = 0;

                            for byte in chunk.iter() {
                            buffer.push(*byte);
                            bytes_read += 1;
                            if buffer.len() == 1024 || bytes_read == chunk.len() {
                            if let Some(callback) = cb.clone() {
                                callback.call(buffer.len() as u64, fm.size);
                                        }
                                yield Ok(bytes::Bytes::from(buffer.clone()));
                                buffer.clear();
                                }
                            }
                        };

                        let url_req = GeneratePresignedUrlsRequest::new(
                            n.try_into().map_err(|_| DracoonClientError::IoError)?,
                            url_part,
                            url_part,
                        );
                        let url =
                        <PublicEndpoint<S> as PublicUploadInternal<R, S>>::
                            create_s3_upload_urls(self, access_key.clone(), upload_channel.upload_id.clone(), url_req)
                            .await?;
                        let url = url.urls.first().ok_or_else(missing_presigned_url_error)?;

                        // truncation is safe because chunk_size is 32 MB
                        #[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
                        let curr_pos: u64 = ((url_part - 1) * (chunk_size as u32)) as u64;

                        let e_tag = self
                            .upload_stream_to_s3(
                                Box::pin(stream),
                                url,
                                chunk_size
                                    .try_into()
                                    .map_err(|_| DracoonClientError::IoError)?,
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
                .map_err(|_| DracoonClientError::IoError)?
        ];
        let cb = cloneable_callback.clone();
        match reader.read_exact(&mut buffer).await {
            Ok(n) => {
                buffer.truncate(n);
                let chunk = bytes::Bytes::from(buffer);
                let stream: async_stream::__private::AsyncStream<
                    Result<bytes::Bytes, std::io::Error>,
                    _,
                > = async_stream::stream! {
                    let mut buffer = Vec::new();
                    let mut bytes_read = 0;

                    for byte in chunk.iter() {
                    buffer.push(*byte);
                    bytes_read += 1;
                    if buffer.len() == 1024 || bytes_read == chunk.len() {
                    if let Some(callback) = cb.clone() {
                        callback.call(buffer.len() as u64, fm.size);
                                }
                        yield Ok(bytes::Bytes::from(buffer.clone()));
                        buffer.clear();
                        }
                    }

                };

                let url_req = GeneratePresignedUrlsRequest::new(
                    n.try_into().map_err(|_| DracoonClientError::IoError)?,
                    url_part,
                    url_part,
                );
                let url = <PublicEndpoint<S> as PublicUploadInternal<R, S>>::create_s3_upload_urls(
                    self,
                    access_key.clone(),
                    upload_channel.upload_id.clone(),
                    url_req,
                )
                .await?;

                let url = url.urls.first().ok_or_else(missing_presigned_url_error)?;

                let curr_pos: u64 = (url_part - 1) as u64 * (DEFAULT_UPLOAD_CHUNK_SIZE as u64);

                let e_tag = self
                    .upload_stream_to_s3(
                        Box::pin(stream),
                        url,
                        n.try_into().map_err(|_| DracoonClientError::IoError)?,
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
        let complete_upload_req = CompleteS3ShareUploadRequest::new(s3_parts, None);

        <PublicEndpoint<S> as PublicUploadInternal<R, S>>::finalize_s3_upload(
            self,
            access_key.clone(),
            upload_channel.upload_id.clone(),
            complete_upload_req,
        )
        .await?;

        // get upload status
        // return node if upload is done
        // return error if upload failed
        // polling with exponential backoff
        let mut sleep_duration = POLLING_START_DELAY;
        loop {
            let status_response =
                <PublicEndpoint<S> as PublicUploadInternal<R, S>>::get_upload_status(
                    self,
                    access_key.clone(),
                    upload_channel.upload_id.clone(),
                )
                .await?;

            match status_response.status {
                S3UploadStatus::Done => {
                    return Ok(status_response.file_name);
                }
                S3UploadStatus::Error => {
                    let response = status_response
                        .error_details
                        .ok_or_else(missing_upload_error_details_error)?;
                    error!("Error uploading file: {}", response);
                    return Err(DracoonClientError::Http(response));
                }
                _ => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(sleep_duration)).await;
                    sleep_duration *= 2;
                }
            }
        }
    }
    async fn upload_to_s3_encrypted(
        &self,
        access_key: String,
        share: &PublicUploadShare,
        upload_options: UploadOptions,
        reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<FileName, DracoonClientError> {
        let chunk_size = chunk_size.unwrap_or(DEFAULT_UPLOAD_CHUNK_SIZE);
        let mut encrypted_upload = StreamingEncryptedUpload::new(reader, chunk_size)?;

        let fm = upload_options.file_meta.clone();

        // create upload channel
        let file_upload_req =
            CreateShareUploadChannelRequest::from_upload_options(&upload_options, Some(true), None);

        let upload_channel =
            <PublicEndpoint<S> as PublicUploadInternal<R, S>>::create_upload_channel(
                self,
                access_key.clone(),
                file_upload_req,
            )
            .await
            .map_err(|err| {
                error!("Error creating upload channel: {}", err);
                err
            })?;

        let mut s3_parts = Vec::new();
        let mut url_part: u32 = 1;

        let cloneable_callback = callback.map(CloneableUploadProgressCallback::new);

        while let Some(chunk) = encrypted_upload.next_chunk(chunk_size).await? {
            let cb = cloneable_callback.clone();
            let fm = fm.clone();
            let chunk_len = chunk.len();
            let stream: async_stream::__private::AsyncStream<
                Result<bytes::Bytes, std::io::Error>,
                _,
            > = async_stream::stream! {
                let mut buffer = Vec::new();
                let mut bytes_read = 0;

                for byte in chunk.iter() {
                    buffer.push(*byte);
                    bytes_read += 1;
                    if buffer.len() == 1024 || bytes_read == chunk.len() {
                        if let Some(callback) = cb.clone() {
                            callback.call(buffer.len() as u64, fm.size);
                        }
                        yield Ok(bytes::Bytes::from(buffer.clone()));
                        buffer.clear();
                    }
                }
            };

            let url_req = GeneratePresignedUrlsRequest::new(
                chunk_len
                    .try_into()
                    .map_err(|_| DracoonClientError::IoError)?,
                url_part,
                url_part,
            );
            let url =
                <PublicEndpoint<S> as PublicUploadInternal<R, S>>::create_s3_upload_urls::<'_, '_>(
                    self,
                    access_key.clone(),
                    upload_channel.upload_id.clone(),
                    url_req,
                )
                .await
                .map_err(|err| {
                    error!("Error creating S3 upload urls: {}", err);
                    err
                })?;
            let url = url.urls.first().ok_or_else(missing_presigned_url_error)?;

            let e_tag = self
                .upload_stream_to_s3(
                    Box::pin(stream),
                    url,
                    chunk_len
                        .try_into()
                        .map_err(|_| DracoonClientError::IoError)?,
                )
                .await
                .map_err(|err| {
                    error!("Error uploading stream to S3: {}", err);
                    err
                })?;

            s3_parts.push(S3FileUploadPart::new(url_part, e_tag));
            url_part += 1;
        }

        let plain_file_key = encrypted_upload.into_plain_file_key()?;
        let public_keys = share.user_user_public_key_list.clone().unwrap_or_default();

        let mut user_file_keys = Vec::new();
        for key in public_keys.items {
            let user_id = key.id;
            match DracoonCrypto::encrypt_file_key(
                plain_file_key.clone(),
                key.public_key_container.clone(),
            ) {
                Ok(file_key) => user_file_keys.push(UserFileKey::new(user_id, file_key)),
                Err(err) => warn!(
                    user_id,
                    access_key = %access_key,
                    file_name = %upload_options.file_meta.name,
                    error = ?err,
                    "Skipping public upload recipient key distribution",
                ),
            }
        }

        // finalize upload
        let complete_upload_req = CompleteS3ShareUploadRequest::new(s3_parts, Some(user_file_keys));

        <PublicEndpoint<S> as PublicUploadInternal<R, S>>::finalize_s3_upload::<'_, '_>(
            self,
            access_key.clone(),
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
            let status_response =
                <PublicEndpoint<S> as PublicUploadInternal<R, S>>::get_upload_status(
                    self,
                    access_key.clone(),
                    upload_channel.upload_id.clone(),
                )
                .await
                .map_err(|err| {
                    error!("Error getting upload status: {}", err);
                    err
                })?;

            match status_response.status {
                S3UploadStatus::Done => {
                    return Ok(status_response.file_name);
                }
                S3UploadStatus::Error => {
                    return Err(DracoonClientError::Http(
                        status_response
                            .error_details
                            .ok_or_else(missing_upload_error_details_error)?,
                    ));
                }
                _ => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(sleep_duration)).await;
                    sleep_duration *= 2;
                }
            }
        }
    }

    async fn finalize_s3_upload(
        &self,
        access_key: String,
        upload_id: String,
        complete_file_upload_req: CompleteS3ShareUploadRequest,
    ) -> Result<(), DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{PUBLIC_BASE}/{PUBLIC_SHARES_BASE}/{PUBLIC_UPLOAD_SHARES}/{}/{}/{FILES_S3_COMPLETE}",
            access_key, upload_id
        );

        let url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .put(url)
            .json(&complete_file_upload_req)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(DracoonClientError::from_response(response).await?)
        }
    }

    async fn get_upload_status(
        &self,
        access_key: String,
        upload_id: String,
    ) -> Result<S3ShareUploadStatus, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{PUBLIC_BASE}/{PUBLIC_SHARES_BASE}/{PUBLIC_UPLOAD_SHARES}/{}/{}",
            access_key, upload_id
        );

        let url = self.client().build_api_url(&url_part);

        let response = self.client().http.get(url).send().await?;

        S3ShareUploadStatus::from_response(response).await
    }
}

#[async_trait]
trait PublicUploadInternal<R: AsyncRead, S>: StreamUploadInternal<S> {
    async fn create_upload_channel(
        &self,
        access_key: String,
        create_file_upload_req: CreateShareUploadChannelRequest,
    ) -> Result<CreateShareUploadChannelResponse, DracoonClientError>;

    async fn create_s3_upload_urls(
        &self,
        access_key: String,
        upload_id: String,
        generate_urls_req: GeneratePresignedUrlsRequest,
    ) -> Result<PresignedUrlList, DracoonClientError>;

    async fn upload_to_s3_unencrypted(
        &self,
        access_key: String,
        share: &PublicUploadShare,
        upload_options: UploadOptions,
        reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<FileName, DracoonClientError>;
    async fn upload_to_s3_encrypted(
        &self,
        access_key: String,
        share: &PublicUploadShare,
        upload_options: UploadOptions,
        reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<FileName, DracoonClientError>;

    async fn finalize_s3_upload(
        &self,
        access_key: String,
        upload_id: String,
        complete_file_upload_req: CompleteS3ShareUploadRequest,
    ) -> Result<(), DracoonClientError>;

    async fn get_upload_status(
        &self,
        access_key: String,
        upload_id: String,
    ) -> Result<S3ShareUploadStatus, DracoonClientError>;
}

#[async_trait]
trait PublicUploadInternalNfs<R: AsyncRead, S>:
    StreamUploadInternal<S> + PublicUploadInternal<R, S>
{
    async fn upload_to_nfs_unencrypted(
        &self,
        access_key: String,
        share: &PublicUploadShare,
        upload_options: UploadOptions,
        reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<FileName, DracoonClientError>;
    async fn upload_to_nfs_encrypted(
        &self,
        access_key: String,
        share: &PublicUploadShare,
        upload_options: UploadOptions,
        reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<FileName, DracoonClientError>;
    async fn finalize_nfs_upload(
        &self,
        access_key: String,
        upload_id: String,
        user_file_key_list: Option<UserFileKeyList>,
    ) -> Result<PublicUploadedFileData, DracoonClientError>;
}

#[async_trait]
impl<R: AsyncRead + Send + Sync + Unpin + 'static, S: Send + Sync> PublicUploadInternalNfs<R, S>
    for PublicEndpoint<S>
{
    async fn upload_to_nfs_unencrypted(
        &self,
        access_key: String,
        share: &PublicUploadShare,
        upload_options: UploadOptions,
        mut reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<FileName, DracoonClientError> {
        let fm = upload_options.file_meta.clone();

        let chunk_size = chunk_size.unwrap_or(DEFAULT_UPLOAD_CHUNK_SIZE);

        // create upload channel
        let file_upload_req =
            CreateShareUploadChannelRequest::from_upload_options(&upload_options, None, None);

        let upload_channel =
            <PublicEndpoint<S> as PublicUploadInternal<R, S>>::create_upload_channel::<'_, '_>(
                self,
                access_key.clone(),
                file_upload_req,
            )
            .await
            .map_err(|err| {
                error!("Error creating upload channel: {}", err);
                err
            })?;

        let (count_chunks, last_chunk_size) = calculate_s3_url_count(fm.size, chunk_size as u64);
        let mut chunk_part: u32 = 1;

        let cloneable_callback = callback.map(CloneableUploadProgressCallback::new);

        if count_chunks > 1 {
            while chunk_part < count_chunks {
                let mut buffer = vec![0; chunk_size];
                let cb = cloneable_callback.clone();
                let fm = fm.clone();

                match reader.read_exact(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        buffer.truncate(n);
                        let chunk = bytes::Bytes::from(buffer);

                        let stream: async_stream::__private::AsyncStream<
                            Result<bytes::Bytes, std::io::Error>,
                            _,
                        > = async_stream::stream! {
                            let mut buffer = Vec::new();
                            let mut bytes_read = 0;

                            for byte in chunk.iter() {
                            buffer.push(*byte);
                            bytes_read += 1;
                            if buffer.len() == 1024 || bytes_read == chunk.len() {
                            if let Some(callback) = cb.clone() {
                                callback.call(buffer.len() as u64, fm.size);
                                        }
                                yield Ok(bytes::Bytes::from(buffer.clone()));
                                buffer.clear();
                                }
                            }
                        };

                        let url = upload_channel.upload_url.clone();

                        // truncation is safe because chunk_size is 32 MB
                        #[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
                        let curr_pos: u64 = ((chunk_part - 1) * (chunk_size as u32)) as u64;

                        self.upload_stream_to_nfs(
                            Box::pin(stream),
                            &url,
                            upload_options.file_meta.size,
                            n,
                            Some(curr_pos),
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
                .map_err(|_| DracoonClientError::IoError)?
        ];
        let cb = cloneable_callback.clone();
        match reader.read_exact(&mut buffer).await {
            Ok(n) => {
                buffer.truncate(n);
                let chunk = bytes::Bytes::from(buffer);
                let stream: async_stream::__private::AsyncStream<
                    Result<bytes::Bytes, std::io::Error>,
                    _,
                > = async_stream::stream! {
                    let mut buffer = Vec::new();
                    let mut bytes_read = 0;

                    for byte in chunk.iter() {
                    buffer.push(*byte);
                    bytes_read += 1;
                    if buffer.len() == 1024 || bytes_read == chunk.len() {
                    if let Some(callback) = cb.clone() {
                        callback.call(buffer.len() as u64, fm.size);
                                }
                        yield Ok(bytes::Bytes::from(buffer.clone()));
                        buffer.clear();
                        }
                    }

                };

                let url = upload_channel.upload_url.clone();

                let curr_pos: u64 = (chunk_part - 1) as u64 * (DEFAULT_UPLOAD_CHUNK_SIZE as u64);

                let e_tag = self
                    .upload_stream_to_nfs(
                        Box::pin(stream),
                        &url,
                        upload_options.file_meta.size,
                        n,
                        Some(curr_pos),
                    )
                    .await?;
            }
            Err(err) => {
                error!("Error reading file: {}", err);
                return Err(DracoonClientError::IoError);
            }
        }

        let public_upload =
            <PublicEndpoint<S> as PublicUploadInternalNfs<R, S>>::finalize_nfs_upload::<'_, '_>(
                self,
                access_key.clone(),
                upload_channel.upload_id.clone(),
                None,
            )
            .await
            .map_err(|err| {
                error!("Error finalizing upload: {}", err);
                err
            })?;

        Ok(public_upload.name)
    }
    async fn upload_to_nfs_encrypted(
        &self,
        access_key: String,
        share: &PublicUploadShare,
        upload_options: UploadOptions,
        reader: BufReader<R>,
        callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<FileName, DracoonClientError> {
        let chunk_size = chunk_size.unwrap_or(DEFAULT_UPLOAD_CHUNK_SIZE);
        let mut encrypted_upload = StreamingEncryptedUpload::new(reader, chunk_size)?;

        let fm = upload_options.file_meta.clone();

        // create upload channel
        let file_upload_req =
            CreateShareUploadChannelRequest::from_upload_options(&upload_options, None, None);

        let upload_channel =
            <PublicEndpoint<S> as PublicUploadInternal<R, S>>::create_upload_channel::<'_, '_>(
                self,
                access_key.clone(),
                file_upload_req,
            )
            .await
            .map_err(|err| {
                error!("Error creating upload channel: {}", err);
                err
            })?;

        let cloneable_callback = callback.map(CloneableUploadProgressCallback::new);
        let mut curr_pos = 0u64;

        while let Some(chunk) = encrypted_upload.next_chunk(chunk_size).await? {
            let cb = cloneable_callback.clone();
            let fm = fm.clone();
            let chunk_len = chunk.len();
            let stream: async_stream::__private::AsyncStream<
                Result<bytes::Bytes, std::io::Error>,
                _,
            > = async_stream::stream! {
                let mut buffer = Vec::new();
                let mut bytes_read = 0;

                for byte in chunk.iter() {
                    buffer.push(*byte);
                    bytes_read += 1;
                    if buffer.len() == 1024 || bytes_read == chunk.len() {
                        if let Some(callback) = cb.clone() {
                            callback.call(buffer.len() as u64, fm.size);
                        }
                        yield Ok(bytes::Bytes::from(buffer.clone()));
                        buffer.clear();
                    }
                }
            };

            let url = upload_channel.upload_url.clone();

            self.upload_stream_to_nfs(
                Box::pin(stream),
                &url,
                upload_options.file_meta.size,
                chunk_len,
                Some(curr_pos),
            )
            .await
            .map_err(|err| {
                error!("Error uploading stream to NFS: {}", err);
                err
            })?;

            curr_pos += chunk_len as u64;
        }

        let plain_file_key = encrypted_upload.into_plain_file_key()?;
        let public_keys = share.user_user_public_key_list.clone().unwrap_or_default();

        let mut user_file_keys = Vec::new();
        for key in public_keys.items {
            let user_id = key.id;
            match DracoonCrypto::encrypt_file_key(
                plain_file_key.clone(),
                key.public_key_container.clone(),
            ) {
                Ok(file_key) => user_file_keys.push(UserFileKey::new(user_id, file_key)),
                Err(err) => warn!(
                    user_id,
                    access_key = %access_key,
                    file_name = %upload_options.file_meta.name,
                    error = ?err,
                    "Skipping public upload recipient key distribution",
                ),
            }
        }

        let user_file_keys = UserFileKeyList::from(user_file_keys);

        let public_upload =
            <PublicEndpoint<S> as PublicUploadInternalNfs<R, S>>::finalize_nfs_upload::<'_, '_>(
                self,
                access_key.clone(),
                upload_channel.upload_id.clone(),
                Some(user_file_keys),
            )
            .await
            .map_err(|err| {
                error!("Error finalizing upload: {}", err);
                err
            })?;

        Ok(public_upload.name)
    }
    async fn finalize_nfs_upload(
        &self,
        access_key: String,
        upload_id: String,
        user_file_key_list: Option<UserFileKeyList>,
    ) -> Result<PublicUploadedFileData, DracoonClientError> {
        let url_part = format!(
            "{DRACOON_API_PREFIX}/{PUBLIC_BASE}/{PUBLIC_SHARES_BASE}/{PUBLIC_UPLOAD_SHARES}/{}/{}",
            access_key, upload_id
        );

        let url = self.client().build_api_url(&url_part);

        let response = match user_file_key_list {
            Some(user_file_keys) => {
                self.client()
                    .http
                    .put(url)
                    .json(&user_file_keys)
                    .send()
                    .await?
            }
            None => self.client().http.put(url).send().await?,
        };

        PublicUploadedFileData::from_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use dco3_crypto::{DracoonCrypto, DracoonRSACrypto, UserKeyPairVersion};
    use mockito::Matcher;

    use crate::{
        nodes::{FileMeta, UploadOptions, UserUserPublicKey},
        public::UserUserPublicKeyList,
        Dracoon,
    };

    use super::*;

    fn public_upload_share(is_encrypted: bool) -> PublicUploadShare {
        let mut share: PublicUploadShare = serde_json::from_str(include_str!(
            "../tests/responses/public/upload_share_ok.json"
        ))
        .unwrap();
        share.is_encrypted = Some(is_encrypted);
        share
    }

    fn encrypted_public_upload_share() -> PublicUploadShare {
        let mut share = public_upload_share(true);
        let keypair = DracoonCrypto::create_plain_user_keypair(UserKeyPairVersion::RSA4096)
            .expect("public key generation should succeed");

        share.user_user_public_key_list = Some(UserUserPublicKeyList {
            items: vec![UserUserPublicKey {
                id: 7,
                public_key_container: keypair.public_key_container.clone(),
            }],
        });

        share
    }

    fn s3_urls_response(base_url: &str) -> String {
        include_str!("../tests/responses/upload/s3_urls_ok_with_placeholder.json")
            .replace("$base_url", base_url)
    }

    fn upload_status_done(file_name: &str) -> String {
        format!(r#"{{"status":"done","fileName":"{file_name}"}}"#)
    }

    fn nfs_upload_channel_response(base_url: &str) -> String {
        let base_url = base_url.trim_end_matches('/');
        format!(r#"{{"uploadUrl":"{base_url}/upload_url","uploadId":"string"}}"#)
    }

    fn uploaded_file_response(name: &str, size: u64) -> String {
        format!(r#"{{"name":"{name}","size":{size},"createdAt":"2021-01-01T00:00:00.000Z"}}"#)
    }

    #[tokio::test]
    async fn test_upload_to_s3_unencrypted() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let client = Dracoon::builder()
            .with_base_url(base_url.clone())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();
        let public = client.public();

        let reader = BufReader::new(Cursor::new(vec![
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ]));
        let upload_options =
            UploadOptions::builder(FileMeta::builder("test.txt", 16).build()).build();
        let share = public_upload_share(false);
        let access_key = "test-access-key";
        let upload_path = format!("/api/v4/public/shares/uploads/{access_key}");
        let s3_urls_path = format!("/api/v4/public/shares/uploads/{access_key}/string/s3_urls");
        let finalize_path = format!("/api/v4/public/shares/uploads/{access_key}/string/s3");
        let status_path = format!("/api/v4/public/shares/uploads/{access_key}/string");

        let upload_channel_mock = mock_server
            .mock("POST", upload_path.as_str())
            .with_status(201)
            .with_body(include_str!(
                "../tests/responses/upload/upload_channel_ok.json"
            ))
            .with_header("content-type", "application/json")
            .create();

        let s3_urls_mock = mock_server
            .mock("POST", s3_urls_path.as_str())
            .with_status(201)
            .with_body(s3_urls_response(&base_url))
            .with_header("content-type", "application/json")
            .create();

        let upload_mock = mock_server
            .mock("PUT", "/upload_url")
            .with_status(202)
            .with_header("etag", "string")
            .create();

        let finalize_mock = mock_server
            .mock("PUT", finalize_path.as_str())
            .with_status(202)
            .create();

        let status_mock = mock_server
            .mock("GET", status_path.as_str())
            .with_status(200)
            .with_body(upload_status_done("test.txt"))
            .with_header("content-type", "application/json")
            .create();

        let file_name = public
            .upload_to_s3_unencrypted(
                access_key.into(),
                &share,
                upload_options,
                reader,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(file_name, "test.txt");
        upload_channel_mock.assert();
        s3_urls_mock.assert();
        upload_mock.assert();
        finalize_mock.assert();
        status_mock.assert();
    }

    #[tokio::test]
    async fn test_upload_to_s3_encrypted() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let client = Dracoon::builder()
            .with_base_url(base_url.clone())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();
        let public = client.public();

        let reader = BufReader::new(Cursor::new(vec![
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ]));
        let upload_options =
            UploadOptions::builder(FileMeta::builder("test.txt", 16).build()).build();
        let share = encrypted_public_upload_share();
        let access_key = "test-access-key";
        let upload_path = format!("/api/v4/public/shares/uploads/{access_key}");
        let s3_urls_path = format!("/api/v4/public/shares/uploads/{access_key}/string/s3_urls");
        let finalize_path = format!("/api/v4/public/shares/uploads/{access_key}/string/s3");
        let status_path = format!("/api/v4/public/shares/uploads/{access_key}/string");

        let upload_channel_mock = mock_server
            .mock("POST", upload_path.as_str())
            .with_status(201)
            .with_body(include_str!(
                "../tests/responses/upload/upload_channel_ok.json"
            ))
            .with_header("content-type", "application/json")
            .create();

        let s3_urls_mock = mock_server
            .mock("POST", s3_urls_path.as_str())
            .with_status(201)
            .with_body(s3_urls_response(&base_url))
            .with_header("content-type", "application/json")
            .create();

        let upload_mock = mock_server
            .mock("PUT", "/upload_url")
            .with_status(202)
            .with_header("etag", "string")
            .create();

        let finalize_mock = mock_server
            .mock("PUT", finalize_path.as_str())
            .match_body(Matcher::Regex(
                r#"(?s).*userFileKeyList.*"userId":7.*"#.to_string(),
            ))
            .with_status(202)
            .create();

        let status_mock = mock_server
            .mock("GET", status_path.as_str())
            .with_status(200)
            .with_body(upload_status_done("test.txt"))
            .with_header("content-type", "application/json")
            .create();

        let file_name = public
            .upload_to_s3_encrypted(
                access_key.into(),
                &share,
                upload_options,
                reader,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(file_name, "test.txt");
        upload_channel_mock.assert();
        s3_urls_mock.assert();
        upload_mock.assert();
        finalize_mock.assert();
        status_mock.assert();
    }

    #[tokio::test]
    async fn test_upload_to_s3_encrypted_streams_multiple_parts() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let client = Dracoon::builder()
            .with_base_url(base_url.clone())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();
        let public = client.public();

        let reader = BufReader::new(Cursor::new(vec![
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255,
        ]));
        let upload_options =
            UploadOptions::builder(FileMeta::builder("test.txt", 12).build()).build();
        let share = encrypted_public_upload_share();
        let access_key = "test-access-key";
        let upload_path = format!("/api/v4/public/shares/uploads/{access_key}");
        let s3_urls_path = format!("/api/v4/public/shares/uploads/{access_key}/string/s3_urls");
        let finalize_path = format!("/api/v4/public/shares/uploads/{access_key}/string/s3");
        let status_path = format!("/api/v4/public/shares/uploads/{access_key}/string");

        let upload_channel_mock = mock_server
            .mock("POST", upload_path.as_str())
            .with_status(201)
            .with_body(include_str!(
                "../tests/responses/upload/upload_channel_ok.json"
            ))
            .with_header("content-type", "application/json")
            .create();

        let s3_urls_mock = mock_server
            .mock("POST", s3_urls_path.as_str())
            .with_status(201)
            .with_body(s3_urls_response(&base_url))
            .with_header("content-type", "application/json")
            .expect(3)
            .create();

        let upload_mock = mock_server
            .mock("PUT", "/upload_url")
            .with_status(202)
            .with_header("etag", "string")
            .expect(3)
            .create();

        let finalize_mock = mock_server
            .mock("PUT", finalize_path.as_str())
            .match_body(Matcher::Regex(
                r#"(?s).*"partNumber":1.*"partNumber":2.*"partNumber":3.*userFileKeyList.*"userId":7.*"#
                    .to_string(),
            ))
            .with_status(202)
            .create();

        let status_mock = mock_server
            .mock("GET", status_path.as_str())
            .with_status(200)
            .with_body(upload_status_done("test.txt"))
            .with_header("content-type", "application/json")
            .create();

        let file_name = public
            .upload_to_s3_encrypted(
                access_key.into(),
                &share,
                upload_options,
                reader,
                None,
                Some(4),
            )
            .await
            .unwrap();

        assert_eq!(file_name, "test.txt");
        upload_channel_mock.assert();
        s3_urls_mock.assert();
        upload_mock.assert();
        finalize_mock.assert();
        status_mock.assert();
    }

    #[tokio::test]
    async fn test_upload_to_nfs_unencrypted() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let client = Dracoon::builder()
            .with_base_url(base_url.clone())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();
        let public = client.public();

        let reader = BufReader::new(Cursor::new(vec![
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ]));
        let upload_options =
            UploadOptions::builder(FileMeta::builder("test.txt", 16).build()).build();
        let share = public_upload_share(false);
        let access_key = "test-access-key";
        let upload_path = format!("/api/v4/public/shares/uploads/{access_key}");
        let finalize_path = format!("/api/v4/public/shares/uploads/{access_key}/string");

        let upload_channel_mock = mock_server
            .mock("POST", upload_path.as_str())
            .with_status(201)
            .with_body(nfs_upload_channel_response(&base_url))
            .with_header("content-type", "application/json")
            .create();

        let upload_mock = mock_server
            .mock("POST", "/upload_url")
            .with_status(202)
            .create();

        let finalize_mock = mock_server
            .mock("PUT", finalize_path.as_str())
            .with_status(200)
            .with_body(uploaded_file_response("test.txt", 16))
            .with_header("content-type", "application/json")
            .create();

        let file_name = public
            .upload_to_nfs_unencrypted(
                access_key.into(),
                &share,
                upload_options,
                reader,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(file_name, "test.txt");
        upload_channel_mock.assert();
        upload_mock.assert();
        finalize_mock.assert();
    }

    #[tokio::test]
    async fn test_upload_to_nfs_encrypted() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let client = Dracoon::builder()
            .with_base_url(base_url.clone())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();
        let public = client.public();

        let reader = BufReader::new(Cursor::new(vec![
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255, 0, 12, 33, 44,
        ]));
        let upload_options =
            UploadOptions::builder(FileMeta::builder("test.txt", 16).build()).build();
        let share = encrypted_public_upload_share();
        let access_key = "test-access-key";
        let upload_path = format!("/api/v4/public/shares/uploads/{access_key}");
        let finalize_path = format!("/api/v4/public/shares/uploads/{access_key}/string");

        let upload_channel_mock = mock_server
            .mock("POST", upload_path.as_str())
            .with_status(201)
            .with_body(nfs_upload_channel_response(&base_url))
            .with_header("content-type", "application/json")
            .create();

        let upload_mock = mock_server
            .mock("POST", "/upload_url")
            .with_status(202)
            .create();

        let finalize_mock = mock_server
            .mock("PUT", finalize_path.as_str())
            .match_body(Matcher::Regex(
                r#"(?s).*"items":\[.*"userId":7.*"#.to_string(),
            ))
            .with_status(200)
            .with_body(uploaded_file_response("test.txt", 16))
            .with_header("content-type", "application/json")
            .create();

        let file_name = public
            .upload_to_nfs_encrypted(
                access_key.into(),
                &share,
                upload_options,
                reader,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(file_name, "test.txt");
        upload_channel_mock.assert();
        upload_mock.assert();
        finalize_mock.assert();
    }

    #[tokio::test]
    async fn test_upload_to_nfs_encrypted_streams_multiple_chunks_with_offsets() {
        let mut mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let client = Dracoon::builder()
            .with_base_url(base_url.clone())
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .build()
            .unwrap();
        let public = client.public();

        let reader = BufReader::new(Cursor::new(vec![
            0, 12, 33, 44, 55, 66, 77, 88, 99, 111, 222, 255,
        ]));
        let upload_options =
            UploadOptions::builder(FileMeta::builder("test.txt", 12).build()).build();
        let share = encrypted_public_upload_share();
        let access_key = "test-access-key";
        let upload_path = format!("/api/v4/public/shares/uploads/{access_key}");
        let finalize_path = format!("/api/v4/public/shares/uploads/{access_key}/string");

        let upload_channel_mock = mock_server
            .mock("POST", upload_path.as_str())
            .with_status(201)
            .with_body(nfs_upload_channel_response(&base_url))
            .with_header("content-type", "application/json")
            .create();

        let upload_mock_1 = mock_server
            .mock("POST", "/upload_url")
            .match_header("content-range", "bytes 0-4/12")
            .with_status(202)
            .expect(1)
            .create();

        let upload_mock_2 = mock_server
            .mock("POST", "/upload_url")
            .match_header("content-range", "bytes 4-8/12")
            .with_status(202)
            .expect(1)
            .create();

        let upload_mock_3 = mock_server
            .mock("POST", "/upload_url")
            .match_header("content-range", "bytes 8-12/12")
            .with_status(202)
            .expect(1)
            .create();

        let finalize_mock = mock_server
            .mock("PUT", finalize_path.as_str())
            .match_body(Matcher::Regex(
                r#"(?s).*"items":\[.*"userId":7.*"#.to_string(),
            ))
            .with_status(200)
            .with_body(uploaded_file_response("test.txt", 12))
            .with_header("content-type", "application/json")
            .create();

        let file_name = public
            .upload_to_nfs_encrypted(
                access_key.into(),
                &share,
                upload_options,
                reader,
                None,
                Some(4),
            )
            .await
            .unwrap();

        assert_eq!(file_name, "test.txt");
        upload_channel_mock.assert();
        upload_mock_1.assert();
        upload_mock_2.assert();
        upload_mock_3.assert();
        finalize_mock.assert();
    }
}
