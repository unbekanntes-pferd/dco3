use std::sync::Arc;

use chrono::{DateTime, Utc};
use dco3_crypto::{FileKey, PrivateKeyContainer};
use dco3_derive::FromResponse;
use serde::{Deserialize, Serialize};

use crate::{
    auth::{DracoonClient, DracoonErrorResponse, GetClient},
    nodes::{S3FileUploadPart, S3UploadStatus, UserUserPublicKey}
};

#[derive(Clone)]
pub struct PublicEndpoint<S> {
    client: Arc<DracoonClient<S>>,
    state: std::marker::PhantomData<S>,
}

impl<S> PublicEndpoint<S> {
    pub fn new(client: Arc<DracoonClient<S>>) -> Self {
        Self {
            client,
            state: std::marker::PhantomData,
        }
    }

    pub fn client(&self) -> &Arc<DracoonClient<S>> {
        &self.client
    }
}

impl <S> GetClient<S> for PublicEndpoint<S> {
    fn get_client(&self) -> &DracoonClient<S> {
        &self.client
    }
}

#[derive(Debug, Deserialize, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareVersionData {
    pub rest_api_version: String,
    pub sds_server_version: String,
    pub build_date: DateTime<Utc>,
    pub is_dracoon_cloud: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub language_default: String,
    pub s3_hosts: Vec<String>,
    pub s3_enforce_direct_upload: bool,
    #[serde(rename = "useS3Storage")]
    pub use_s3_storage: bool,
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
#[allow(non_snake_case)]
pub struct S3ShareUploadStatus {
    pub status: S3UploadStatus,
    pub file_name: String,
    pub size: Option<u64>,
    pub error_details: Option<DracoonErrorResponse>,
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct PublicDownloadShare {
    pub is_protected: bool,
    pub file_name: String,
    pub size: u64,
    pub limit_reached: bool,
    pub creator_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub has_download_limit: bool,
    pub media_type: String,
    pub name: Option<String>,
    pub creator_username: Option<String>,
    pub expire_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub is_encrypted: Option<bool>,
    pub file_key: Option<FileKey>,
    pub private_key_container: Option<PrivateKeyContainer>,
    //virus_protection_info: VirusProtectionInfo TODO: add VirusProtectionInfo into Nodes
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct PublicDownloadTokenGenerateRequest {
    password: Option<String>,
}

impl PublicDownloadTokenGenerateRequest {
    pub fn new(password: impl Into<String>) -> Self {
        Self {
            password: Some(password.into()),
        }
    }

    pub fn has_password(&self) -> bool {
        self.password.is_some()
    }
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct PublicDownloadTokenGenerateResponse {
    pub download_url: String,
}

pub struct PublicShareEncryption {
    pub file_key: FileKey,
    pub private_key_container: PrivateKeyContainer,
}

impl PublicShareEncryption {
    pub fn new(file_key: FileKey, private_key_container: PrivateKeyContainer) -> Self {
        Self {
            file_key,
            private_key_container,
        }
    }
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct PublicUploadShare {
    pub is_protected: bool,
    pub created_at: DateTime<Utc>,
    pub name: Option<String>,
    pub is_encrypted: Option<bool>,
    pub expire_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub uploaded_files: Option<Vec<PublicUploadedFileData>>,
    pub user_user_public_key_list: Option<UserUserPublicKeyList>,
    pub show_uploaded_files: Option<bool>,
    pub remaining_size: Option<u64>,
    pub remaining_slots: Option<u32>,

}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicUploadedFileData {
    pub name: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub hash: Option<String>
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct UserUserPublicKeyList {
    pub items: Vec<UserUserPublicKey>
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(non_snake_case)]
pub struct CreateShareUploadChannelRequest {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direct_S3_upload: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_creation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_modification: Option<String>,
}

impl CreateShareUploadChannelRequest {
    pub fn builder(name: impl Into<String>) -> CreateShareUploadChannelRequestBuilder {
        CreateShareUploadChannelRequestBuilder::new(name)
    }
}

#[allow(non_snake_case)]
pub struct CreateShareUploadChannelRequestBuilder {
    name: String,
    size: Option<u64>,
    password: Option<String>,
    direct_S3_upload: Option<bool>,
    timestamp_creation: Option<String>,
    timestamp_modification: Option<String>,
}

impl CreateShareUploadChannelRequestBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: None,
            password: None,
            direct_S3_upload: None,
            timestamp_creation: None,
            timestamp_modification: None,
        }
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn with_direct_s3_upload(mut self, is_s3_upload: bool) -> Self {
        self.direct_S3_upload = Some(is_s3_upload);
        self
    }

    pub fn with_timestamp_creation(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp_creation = Some(timestamp.to_rfc3339());
        self
    }

    pub fn with_timestamp_modification(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp_modification = Some(timestamp.to_rfc3339());
        self
    }

    pub fn build(self) -> CreateShareUploadChannelRequest {
        CreateShareUploadChannelRequest {
            name: self.name,
            size: self.size,
            password: self.password,
            direct_S3_upload: self.direct_S3_upload,
            timestamp_creation: self.timestamp_creation,
            timestamp_modification: self.timestamp_modification,
        }
    }
}


#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct CreateShareUploadChannelResponse {
    pub upload_id: String,
    pub upload_url: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserFileKey {
    user_id: u64,
    file_key: FileKey,
}

impl UserFileKey {
    pub fn new(user_id: u64, file_key: FileKey) -> Self {
        Self {
            user_id,
            file_key,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct UserFileKeyList {
    pub items: Vec<UserFileKey>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(non_snake_case)]
pub struct CompleteS3ShareUploadRequest {
    parts: Vec<S3FileUploadPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_file_key_list: Option<Vec<UserFileKey>>
}

impl CompleteS3ShareUploadRequest {
    pub fn new(parts: Vec<S3FileUploadPart>, user_file_key_list: Option<Vec<UserFileKey>>) -> Self {
        Self {
            parts,
            user_file_key_list
        }
    }
}

pub type FileName = String;