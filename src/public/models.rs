use std::sync::Arc;

use chrono::{DateTime, Utc};
use dco3_derive::FromResponse;
use dco3_crypto::{FileKey, PrivateKeyContainer};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{DracoonClient, DracoonErrorResponse},
    nodes::S3UploadStatus,
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

#[derive(Debug, Deserialize, Clone)]
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
    pub creator_name: String,
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
    password: Option<String>
}

impl PublicDownloadTokenGenerateRequest {
    pub fn new(password: impl Into<String>) -> Self {
        Self {
            password: Some(password.into())
        }
    }

    pub fn has_password(&self) -> bool {
        self.password.is_some()
    }
}


#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct PublicDownloadTokenGenerateResponse {
    pub download_url: String
}
