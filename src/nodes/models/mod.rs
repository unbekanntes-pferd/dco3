#![allow(dead_code, unused_imports)]

pub mod filters;
pub mod sorts;

use dco3_crypto::DracoonCrypto;
use dco3_crypto::DracoonRSACrypto;
use dco3_crypto::PlainUserKeyPairContainer;
use dco3_derive::FromResponse;
pub use filters::*;
pub use sorts::*;
use tracing::debug;
use tracing::error;

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::sync::Arc;
use std::sync::Mutex;

use crate::{
    auth::{errors::DracoonClientError, models::DracoonErrorResponse},
    models::{ObjectExpiration, Range, RangedItems},
    utils::parse_body,
    utils::FromResponse,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dco3_crypto::FileKey;
use dco3_crypto::PublicKeyContainer;
use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};

use super::rooms::models::NodePermissionsBuilder;

/// A callback function that is called after each chunk is processed (download)
pub type DownloadProgressCallback = Box<dyn FnMut(u64, u64) + Send + Sync>;

/// A callback function that is called after each chunk is processed (upload)
pub type UploadProgressCallback = Box<dyn FnMut(u64, u64) + Send + Sync>;

/// A callback function (thread-safe) that can be cloned and called from multiple threads (upload)
pub struct CloneableUploadProgressCallback(Arc<Mutex<UploadProgressCallback>>);

impl Clone for CloneableUploadProgressCallback {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl CloneableUploadProgressCallback {
    pub fn new<F>(callback: F) -> Self
    where
        F: 'static + FnMut(u64, u64) + Send + Sync,
    {
        Self(Arc::new(Mutex::new(Box::new(callback))))
    }

    pub fn call(&self, bytes_read: u64, total_size: u64) {
        (self.0.lock().unwrap())(bytes_read, total_size);
    }
}

/// file meta information (name, size, timestamp creation, timestamp modification)
#[derive(Debug, Clone)]
pub struct FileMeta(
    pub String,
    pub u64,
    pub Option<DateTime<Utc>>,
    pub Option<DateTime<Utc>>,
);

#[derive(Default)]
pub struct FileMetaBuilder {
    name: Option<String>,
    size: Option<u64>,
    timestamp_creation: Option<DateTime<Utc>>,
    timestamp_modification: Option<DateTime<Utc>>,
}

impl FileMeta {
    pub fn builder() -> FileMetaBuilder {
        FileMetaBuilder::new()
    }
}

impl FileMetaBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            size: None,
            timestamp_creation: None,
            timestamp_modification: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_timestamp_creation(mut self, timestamp_creation: DateTime<Utc>) -> Self {
        self.timestamp_creation = Some(timestamp_creation);
        self
    }

    pub fn with_timestamp_modification(mut self, timestamp_modification: DateTime<Utc>) -> Self {
        self.timestamp_modification = Some(timestamp_modification);
        self
    }

    pub fn build(self) -> FileMeta {
        FileMeta(
            self.name.unwrap(),
            self.size.unwrap(),
            self.timestamp_creation,
            self.timestamp_modification,
        )
    }
}

/// upload options (expiration, classification, keep share links, resolution strategy)
#[derive(Debug, Clone, Default)]
pub struct UploadOptions(
    pub Option<ObjectExpiration>,
    pub Option<u8>,
    pub Option<bool>,
    pub Option<ResolutionStrategy>,
);

impl UploadOptions {
    pub fn builder() -> UploadOptionsBuilder {
        UploadOptionsBuilder::new()
    }
}

#[derive(Default)]
pub struct UploadOptionsBuilder {
    expiration: Option<ObjectExpiration>,
    classification: Option<u8>,
    keep_share_links: Option<bool>,
    resolution_strategy: Option<ResolutionStrategy>,
}

impl UploadOptionsBuilder {
    pub fn new() -> Self {
        Self {
            expiration: None,
            classification: None,
            keep_share_links: None,
            resolution_strategy: None,
        }
    }

    pub fn with_expiration(mut self, expiration: ObjectExpiration) -> Self {
        self.expiration = Some(expiration);
        self
    }

    pub fn with_classification(mut self, classification: u8) -> Self {
        self.classification = Some(classification);
        self
    }

    pub fn with_keep_share_links(mut self, keep_share_links: bool) -> Self {
        self.keep_share_links = Some(keep_share_links);
        self
    }

    pub fn with_resolution_strategy(mut self, resolution_strategy: ResolutionStrategy) -> Self {
        self.resolution_strategy = Some(resolution_strategy);
        self
    }

    pub fn build(self) -> UploadOptions {
        UploadOptions(
            self.expiration,
            self.classification,
            self.keep_share_links,
            self.resolution_strategy,
        )
    }
}

/// A list of nodes in DRACOON - GET /nodes
pub type NodeList = RangedItems<Node>;

impl NodeList {
    pub fn get_files(&self) -> Vec<Node> {
        self.items
            .iter()
            .filter(|node| node.node_type == NodeType::File)
            .cloned()
            .collect()
    }

    pub fn get_folders(&self) -> Vec<Node> {
        self.items
            .iter()
            .filter(|node| node.node_type == NodeType::Folder)
            .cloned()
            .collect()
    }

    pub fn get_rooms(&self) -> Vec<Node> {
        self.items
            .iter()
            .filter(|node| node.node_type == NodeType::Room)
            .cloned()
            .collect()
    }
}

/// A node in DRACOON - GET /nodes/{nodeId}
#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: u64,
    pub reference_id: Option<u64>,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub name: String,
    pub timestamp_creation: Option<DateTime<Utc>>,
    pub timestamp_modification: Option<DateTime<Utc>>,
    pub parent_id: Option<u64>,
    pub parent_path: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub created_by: Option<UserInfo>,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<UserInfo>,
    pub expire_at: Option<DateTime<Utc>>,
    pub hash: Option<String>,
    pub file_type: Option<String>,
    pub media_type: Option<String>,
    pub size: Option<u64>,
    pub classification: Option<u64>,
    pub notes: Option<String>,
    pub permissions: Option<NodePermissions>,
    pub inherit_permissions: Option<bool>,
    pub is_encrypted: Option<bool>,
    pub encryption_info: Option<EncryptionInfo>,
    pub cnt_deleted_versions: Option<u64>,
    pub cnt_comments: Option<u64>,
    pub cnt_upload_shares: Option<u64>,
    pub cnt_download_shares: Option<u64>,
    pub recycle_bin_retention_period: Option<u64>,
    pub has_activities_log: Option<bool>,
    pub quota: Option<u64>,
    pub is_favorite: Option<bool>,
    pub branch_version: Option<u64>,
    pub media_token: Option<String>,
    pub is_browsable: Option<bool>,
    pub cnt_rooms: Option<u64>,
    pub cnt_folders: Option<u64>,
    pub cnt_files: Option<u64>,
    pub auth_parent_id: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum NodeType {
    #[serde(rename = "room")]
    Room,
    #[serde(rename = "folder")]
    Folder,
    #[serde(rename = "file")]
    File,
}

impl From<NodeType> for String {
    fn from(node_type: NodeType) -> Self {
        match node_type {
            NodeType::Room => "room".to_string(),
            NodeType::Folder => "folder".to_string(),
            NodeType::File => "file".to_string(),
        }
    }
}

impl From<&NodeType> for String {
    fn from(node_type: &NodeType) -> Self {
        match node_type {
            NodeType::Room => "room".to_string(),
            NodeType::Folder => "folder".to_string(),
            NodeType::File => "file".to_string(),
        }
    }
}

/// DRACOOON node permissions
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct NodePermissions {
    pub manage: bool,
    pub read: bool,
    pub create: bool,
    pub change: bool,
    pub delete: bool,
    pub manage_download_share: bool,
    pub manage_upload_share: bool,
    pub read_recycle_bin: bool,
    pub restore_recycle_bin: bool,
    pub delete_recycle_bin: bool,
}

impl NodePermissions {
    pub fn builder() -> NodePermissionsBuilder {
        NodePermissionsBuilder::new()
    }

    pub fn new_with_edit_permissions() -> Self {
        Self {
            manage: false,
            read: true,
            create: true,
            change: true,
            delete: true,
            manage_download_share: true,
            manage_upload_share: true,
            read_recycle_bin: true,
            restore_recycle_bin: true,
            delete_recycle_bin: false,
        }
    }

    pub fn new_with_read_permissions() -> Self {
        Self {
            manage: false,
            read: true,
            create: false,
            change: false,
            delete: false,
            manage_download_share: false,
            manage_upload_share: false,
            read_recycle_bin: false,
            restore_recycle_bin: false,
            delete_recycle_bin: false,
        }
    }

    pub fn new_with_manage_permissions() -> Self {
        Self {
            manage: true,
            read: true,
            create: true,
            change: true,
            delete: true,
            manage_download_share: true,
            manage_upload_share: true,
            read_recycle_bin: true,
            restore_recycle_bin: true,
            delete_recycle_bin: true,
        }
    }
}

impl ToString for NodePermissions {
    fn to_string(&self) -> String {
        let mapping = [
            (self.manage, 'm'),
            (self.read, 'r'),
            (self.create, 'w'),
            (self.change, 'c'),
            (self.delete, 'd'),
            (self.manage_download_share, 'm'),
            (self.manage_upload_share, 'm'),
            (self.read_recycle_bin, 'r'),
            (self.restore_recycle_bin, 'r'),
            (self.delete_recycle_bin, 'd'),
        ];

        let mut perms = String::with_capacity(mapping.len());

        for (i, &(flag, ch)) in mapping.iter().enumerate() {
            perms.push(if flag { ch } else { '-' });

            // Add a dash after the "delete" permission
            if i == 4 {
                perms.push('-');
            }
        }

        perms
    }
}

/// DRACOOON encryption info (rescue keys)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionInfo {
    user_key_state: String,
    room_key_state: String,
    data_space_key_state: String,
}

/// DRACOON user info on nodes (`created_by`, `updated_by`)
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i64,
    pub user_type: UserType,
    pub user_name: Option<String>,
    pub avatar_uuid: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum UserType {
    #[serde(rename = "internal")]
    Internal,
    #[serde(rename = "external")]
    External,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "deleted")]
    Deleted,
}

#[async_trait]
impl FromResponse for NodeList {
    /// transforms a response into a NodeList
    async fn from_response(res: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(res).await
    }
}

/// Response for download url of a node - POST /nodes/files/{nodeId}/download
#[derive(Serialize, Deserialize, Debug, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct DownloadUrlResponse {
    pub download_url: String,
}

/// Error response for S3 requests (XML)
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct S3XmlError {
    code: Option<String>,
    request_id: Option<String>,
    host_id: Option<String>,
    message: Option<String>,
    argument_name: Option<String>,
}

/// Error response for S3 requests
#[derive(Debug, PartialEq)]
pub struct S3ErrorResponse {
    pub status: StatusCode,
    pub error: S3XmlError,
}

impl Display for S3ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error: {} ({})",
            self.error
                .message
                .as_ref()
                .unwrap_or(&String::from("Unknown S3 error")),
            self.status,
        )
    }
}

impl S3ErrorResponse {
    /// transforms a `S3XmlError` into a `S3ErrorResponse`
    pub fn from_xml_error(status: StatusCode, error: S3XmlError) -> Self {
        Self { status, error }
    }
}

#[async_trait]
impl FromResponse for FileKey {
    /// transforms a response into a `FileKey`
    async fn from_response(res: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(res).await
    }
}

#[derive(Debug, Deserialize, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileUploadResponse {
    pub upload_url: String,
    pub upload_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignedUrl {
    pub url: String,
    pub part_number: u32,
}

#[derive(Debug, Deserialize, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct PresignedUrlList {
    pub urls: Vec<PresignedUrl>,
}

#[derive(Debug, Deserialize, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct S3FileUploadStatus {
    pub status: S3UploadStatus,
    pub node: Option<Node>,
    pub error_details: Option<DracoonErrorResponse>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum S3UploadStatus {
    #[serde(rename = "transfer")]
    Transfer,
    #[serde(rename = "finishing")]
    Finishing,
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(non_snake_case)]
pub struct CreateFileUploadRequest {
    parent_id: u64,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    classification: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<ObjectExpiration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direct_S3_upload: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_creation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_modification: Option<String>,
}

impl CreateFileUploadRequest {
    pub fn builder(parent_id: u64, name: String) -> CreateFileUploadRequestBuilder {
        CreateFileUploadRequestBuilder {
            parent_id,
            name,
            size: None,
            classification: None,
            expiration: None,
            direct_s3_upload: Some(true),
            timestamp_creation: None,
            timestamp_modification: None,
        }
    }
}

pub struct CreateFileUploadRequestBuilder {
    parent_id: u64,
    name: String,
    size: Option<u64>,
    classification: Option<u8>,
    expiration: Option<ObjectExpiration>,
    direct_s3_upload: Option<bool>,
    timestamp_creation: Option<String>,
    timestamp_modification: Option<String>,
}

impl CreateFileUploadRequestBuilder {
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_classification(mut self, classification: u8) -> Self {
        self.classification = Some(classification);
        self
    }

    pub fn with_expiration(mut self, expiration: ObjectExpiration) -> Self {
        self.expiration = Some(expiration);
        self
    }
    pub fn with_timestamp_creation(mut self, timestamp_creation: DateTime<Utc>) -> Self {
        self.timestamp_creation = Some(timestamp_creation.to_rfc3339());
        self
    }
    pub fn with_timestamp_modification(mut self, timestamp_modification: DateTime<Utc>) -> Self {
        self.timestamp_modification = Some(timestamp_modification.to_rfc3339());
        self
    }

    pub fn with_direct_s3_upload(mut self, direct_s3_upload: bool) -> Self {
        if !direct_s3_upload {
            self.direct_s3_upload = None;
        } else {
            self.direct_s3_upload = Some(direct_s3_upload);
        }
        self
    }

    pub fn build(self) -> CreateFileUploadRequest {
        CreateFileUploadRequest {
            parent_id: self.parent_id,
            name: self.name,
            size: self.size,
            classification: self.classification,
            expiration: self.expiration,
            direct_S3_upload: self.direct_s3_upload,
            timestamp_creation: self.timestamp_creation,
            timestamp_modification: self.timestamp_modification,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePresignedUrlsRequest {
    size: u64,
    first_part_number: u32,
    last_part_number: u32,
}

impl GeneratePresignedUrlsRequest {
    pub fn new(size: u64, first_part_number: u32, last_part_number: u32) -> Self {
        Self {
            size,
            first_part_number,
            last_part_number,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteS3FileUploadRequest {
    parts: Vec<S3FileUploadPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolution_strategy: Option<ResolutionStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keep_share_links: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_key: Option<FileKey>,
}

pub struct CompleteS3FileUploadRequestBuilder {
    parts: Vec<S3FileUploadPart>,
    resolution_strategy: Option<ResolutionStrategy>,
    file_name: Option<String>,
    keep_share_links: Option<bool>,
    file_key: Option<FileKey>,
}

impl CompleteS3FileUploadRequest {
    pub fn builder(parts: Vec<S3FileUploadPart>) -> CompleteS3FileUploadRequestBuilder {
        CompleteS3FileUploadRequestBuilder {
            parts,
            resolution_strategy: None,
            file_name: None,
            keep_share_links: None,
            file_key: None,
        }
    }
}

impl CompleteS3FileUploadRequestBuilder {
    pub fn with_resolution_strategy(mut self, resolution_strategy: ResolutionStrategy) -> Self {
        self.resolution_strategy = Some(resolution_strategy);
        self
    }

    pub fn with_file_name(mut self, file_name: String) -> Self {
        self.file_name = Some(file_name);
        self
    }

    pub fn with_keep_share_links(mut self, keep_share_links: bool) -> Self {
        self.keep_share_links = Some(keep_share_links);
        self
    }

    pub fn with_file_key(mut self, file_key: FileKey) -> Self {
        self.file_key = Some(file_key);
        self
    }

    pub fn build(self) -> CompleteS3FileUploadRequest {
        CompleteS3FileUploadRequest {
            parts: self.parts,
            resolution_strategy: self.resolution_strategy,
            file_name: self.file_name,
            keep_share_links: self.keep_share_links,
            file_key: self.file_key,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteUploadRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    resolution_strategy: Option<ResolutionStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keep_share_links: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_key: Option<FileKey>,
}

pub struct CompleteUploadRequestBuilder {
    resolution_strategy: Option<ResolutionStrategy>,
    file_name: Option<String>,
    keep_share_links: Option<bool>,
    file_key: Option<FileKey>,
}

impl CompleteUploadRequest {
    pub fn builder() -> CompleteUploadRequestBuilder {
        CompleteUploadRequestBuilder {
            resolution_strategy: None,
            file_name: None,
            keep_share_links: None,
            file_key: None,
        }
    }
}

impl CompleteUploadRequestBuilder {
    pub fn with_resolution_strategy(mut self, resolution_strategy: ResolutionStrategy) -> Self {
        self.resolution_strategy = Some(resolution_strategy);
        self
    }

    pub fn with_file_name(mut self, file_name: String) -> Self {
        self.file_name = Some(file_name);
        self
    }

    pub fn with_keep_share_links(mut self, keep_share_links: bool) -> Self {
        self.keep_share_links = Some(keep_share_links);
        self
    }

    pub fn with_file_key(mut self, file_key: FileKey) -> Self {
        self.file_key = Some(file_key);
        self
    }

    pub fn build(self) -> CompleteUploadRequest {
        CompleteUploadRequest {
            resolution_strategy: self.resolution_strategy,
            file_name: self.file_name,
            keep_share_links: self.keep_share_links,
            file_key: self.file_key,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum ResolutionStrategy {
    #[serde(rename = "autorename")]
    AutoRename,
    #[serde(rename = "overwrite")]
    Overwrite,
    #[serde(rename = "fail")]
    Fail,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3FileUploadPart {
    part_number: u32,
    part_etag: String,
}

impl S3FileUploadPart {
    pub fn new(part_number: u32, part_etag: String) -> Self {
        Self {
            part_number,
            part_etag,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteNodesRequest {
    node_ids: Vec<u64>,
}

impl From<Vec<u64>> for DeleteNodesRequest {
    fn from(node_ids: Vec<u64>) -> Self {
        Self { node_ids }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferNodesRequest {
    items: Vec<TransferNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolution_strategy: Option<ResolutionStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keep_share_links: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferNode {
    id: u64,
    name: Option<String>,
    timestamp_creation: Option<String>,
    timestamp_modification: Option<String>,
}

impl From<u64> for TransferNode {
    fn from(node_id: u64) -> Self {
        Self {
            id: node_id,
            name: None,
            timestamp_creation: None,
            timestamp_modification: None,
        }
    }
}

impl From<Vec<u64>> for TransferNodesRequest {
    fn from(node_ids: Vec<u64>) -> Self {
        Self {
            items: node_ids.into_iter().map(std::convert::Into::into).collect(),
            resolution_strategy: None,
            keep_share_links: None,
        }
    }
}

pub struct TransferNodesRequestBuilder {
    items: Vec<TransferNode>,
    resolution_strategy: Option<ResolutionStrategy>,
    keep_share_links: Option<bool>,
}

impl TransferNodesRequest {
    pub fn builder(items: Vec<TransferNode>) -> TransferNodesRequestBuilder {
        TransferNodesRequestBuilder {
            items,
            resolution_strategy: None,
            keep_share_links: None,
        }
    }

    pub fn new_from_ids(node_ids: Vec<u64>) -> TransferNodesRequestBuilder {
        TransferNodesRequestBuilder {
            items: node_ids.into_iter().map(std::convert::Into::into).collect(),
            resolution_strategy: None,
            keep_share_links: None,
        }
    }

    pub fn with_resolution_strategy(mut self, resolution_strategy: ResolutionStrategy) -> Self {
        self.resolution_strategy = Some(resolution_strategy);
        self
    }

    pub fn with_keep_share_links(mut self, keep_share_links: bool) -> Self {
        self.keep_share_links = Some(keep_share_links);
        self
    }

    pub fn build(self) -> TransferNodesRequest {
        TransferNodesRequest {
            items: self.items,
            resolution_strategy: self.resolution_strategy,
            keep_share_links: self.keep_share_links,
        }
    }
}

pub struct TransferNodeBuilder {
    id: u64,
    name: Option<String>,
    timestamp_creation: Option<String>,
    timestamp_modification: Option<String>,
}

impl TransferNode {
    pub fn builder(id: u64) -> TransferNodeBuilder {
        TransferNodeBuilder {
            id,
            name: None,
            timestamp_creation: None,
            timestamp_modification: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_timestamp_creation(mut self, timestamp_creation: String) -> Self {
        self.timestamp_creation = Some(timestamp_creation);
        self
    }

    pub fn with_timestamp_modification(mut self, timestamp_modification: String) -> Self {
        self.timestamp_modification = Some(timestamp_modification);
        self
    }

    pub fn build(self) -> TransferNode {
        TransferNode {
            id: self.id,
            name: self.name,
            timestamp_creation: self.timestamp_creation,
            timestamp_modification: self.timestamp_modification,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderRequest {
    name: String,
    parent_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_creation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_modification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    classification: Option<u8>,
}

pub struct CreateFolderRequestBuilder {
    name: String,
    parent_id: u64,
    notes: Option<String>,
    timestamp_creation: Option<String>,
    timestamp_modification: Option<String>,
    classification: Option<u8>,
}

impl CreateFolderRequest {
    pub fn builder(name: impl Into<String>, parent_id: u64) -> CreateFolderRequestBuilder {
        CreateFolderRequestBuilder {
            name: name.into(),
            parent_id,
            notes: None,
            timestamp_creation: None,
            timestamp_modification: None,
            classification: None,
        }
    }
}

impl CreateFolderRequestBuilder {
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    pub fn with_timestamp_creation(mut self, timestamp_creation: impl Into<String>) -> Self {
        self.timestamp_creation = Some(timestamp_creation.into());
        self
    }

    pub fn with_timestamp_modification(
        mut self,
        timestamp_modification: impl Into<String>,
    ) -> Self {
        self.timestamp_modification = Some(timestamp_modification.into());
        self
    }

    pub fn with_classification(mut self, classification: u8) -> Self {
        self.classification = Some(classification);
        self
    }

    pub fn build(self) -> CreateFolderRequest {
        CreateFolderRequest {
            name: self.name,
            parent_id: self.parent_id,
            notes: self.notes,
            timestamp_creation: self.timestamp_creation,
            timestamp_modification: self.timestamp_modification,
            classification: self.classification,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFolderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_creation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_modification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    classification: Option<u8>,
}

pub struct UpdateFolderRequestBuilder {
    name: Option<String>,
    notes: Option<String>,
    timestamp_creation: Option<String>,
    timestamp_modification: Option<String>,
    classification: Option<u8>,
}

impl UpdateFolderRequest {
    pub fn builder() -> UpdateFolderRequestBuilder {
        UpdateFolderRequestBuilder {
            name: None,
            notes: None,
            timestamp_creation: None,
            timestamp_modification: None,
            classification: None,
        }
    }
}

impl UpdateFolderRequestBuilder {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    pub fn with_timestamp_creation(mut self, timestamp_creation: impl Into<String>) -> Self {
        self.timestamp_creation = Some(timestamp_creation.into());
        self
    }

    pub fn with_timestamp_modification(
        mut self,
        timestamp_modification: impl Into<String>,
    ) -> Self {
        self.timestamp_modification = Some(timestamp_modification.into());
        self
    }

    pub fn with_classification(mut self, classification: u8) -> Self {
        self.classification = Some(classification);
        self
    }

    pub fn build(self) -> UpdateFolderRequest {
        UpdateFolderRequest {
            name: self.name,
            notes: self.notes,
            timestamp_creation: self.timestamp_creation,
            timestamp_modification: self.timestamp_modification,
            classification: self.classification,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserIdFileItem {
    pub user_id: u64,
    pub file_id: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserUserPublicKey {
    pub id: u64,
    pub public_key_container: PublicKeyContainer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileFileKeys {
    pub id: u64,
    pub file_key_container: FileKey,
}

#[derive(Debug, Deserialize, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct MissingKeysResponse {
    pub range: Option<Range>,
    pub items: Vec<UserIdFileItem>,
    pub users: Vec<UserUserPublicKey>,
    pub files: Vec<FileFileKeys>,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserFileKeySetBatchRequest {
    items: Vec<UserFileKeySetRequest>,
}

impl UserFileKeySetBatchRequest {
    pub fn new() -> Self {
        UserFileKeySetBatchRequest { items: Vec::new() }
    }

    pub fn add(&mut self, user_id: u64, file_id: u64, file_key: FileKey) {
        self.items
            .push(UserFileKeySetRequest::new(user_id, file_id, file_key));
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn try_new_from_missing_keys(
        missing_keys: MissingKeysResponse,
        keypair: &PlainUserKeyPairContainer,
    ) -> Result<Self, DracoonClientError> {
        let reqs = missing_keys
            .items
            .into_iter()
            .flat_map::<Result<UserFileKeySetRequest, DracoonClientError>, _>(|item| {
                let file_id = item.file_id;
                let user_id = item.user_id;
                let public_key = missing_keys
                    .users
                    .iter()
                    .find(|u| u.id == user_id)
                    .ok_or_else(|| {
                        error!("User not found in response: {}", user_id);
                        DracoonClientError::Unknown
                    })? // this is safe because the user id is in the response
                    .public_key_container
                    .clone();
                let file_key = missing_keys
                    .files
                    .iter()
                    .find(|f| f.id == file_id)
                    .ok_or_else(|| {
                        error!("File not found in response: {}", file_id);
                        DracoonClientError::Unknown
                    })? // this is safe because the file id is in the response
                    .file_key_container
                    .clone();

                let plain_file_key =
                    DracoonCrypto::decrypt_file_key(file_key, keypair).map_err(|err| {
                        error!("Could not decrypt file key: {:?}", err);
                        DracoonClientError::CryptoError(err)
                    })?;
                let file_key = DracoonCrypto::encrypt_file_key(plain_file_key, public_key)
                    .map_err(|err| {
                        error!("Could not encrypt file key: {:?}", err);
                        DracoonClientError::CryptoError(err)
                    })?;
                let set_key_req = UserFileKeySetRequest::new(user_id, file_id, file_key);
                Ok(set_key_req)
            })
            .collect::<Vec<_>>();

        debug!("Built {} key requests", reqs.len());

        Ok(reqs.into())
    }
}

impl From<Vec<UserFileKeySetRequest>> for UserFileKeySetBatchRequest {
    fn from(items: Vec<UserFileKeySetRequest>) -> Self {
        UserFileKeySetBatchRequest { items }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFileKeySetRequest {
    user_id: u64,
    file_id: u64,
    file_key: FileKey,
}

impl UserFileKeySetRequest {
    pub fn new(user_id: u64, file_id: u64, file_key: FileKey) -> Self {
        UserFileKeySetRequest {
            user_id,
            file_id,
            file_key,
        }
    }
}

#[derive(Debug, Clone)]
pub enum UseKey {
    RoomRescueKey,
    SystemRescueKey,
    PreviousUserKey,
    PreviousRoomRescueKey,
    PreviousSystemRescueKey,
}

impl From<UseKey> for String {
    fn from(use_key: UseKey) -> Self {
        match use_key {
            UseKey::RoomRescueKey => "room_rescue_key".to_string(),
            UseKey::SystemRescueKey => "system_rescue_key".to_string(),
            UseKey::PreviousUserKey => "previous_user_key".to_string(),
            UseKey::PreviousRoomRescueKey => "previous_room_rescue_key".to_string(),
            UseKey::PreviousSystemRescueKey => "previous_system_rescue_key".to_string(),
        }
    }
}
