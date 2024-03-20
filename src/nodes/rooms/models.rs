use async_trait::async_trait;
use dco3_crypto::{
    DracoonCrypto, DracoonCryptoError, DracoonRSACrypto, PublicKeyContainer, UserKeyPairContainer,
    UserKeyPairVersion,
};
use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::{
    auth::{errors::DracoonClientError, models::DracoonErrorResponse},
    models::RangedItems,
    nodes::models::{NodePermissions, UserInfo},
    utils::{parse_body, FromResponse},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomRequest {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recycle_bin_retention_period: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quota: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inherit_permissions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    admin_ids: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    admin_group_ids: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_group_member_acceptance: Option<GroupMemberAcceptance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_activities_log: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    classification: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_creation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_modification: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum GroupMemberAcceptance {
    #[serde(rename = "autoallow")]
    AutoAllow,
    #[serde(rename = "pending")]
    Pending,
}

impl CreateRoomRequest {
    pub fn builder(name: &str) -> CreateRoomRequestBuilder {
        CreateRoomRequestBuilder {
            name: name.to_string(),
            parent_id: None,
            recycle_bin_retention_period: None,
            quota: None,
            inherit_permissions: None,
            admin_ids: None,
            admin_group_ids: None,
            new_group_member_acceptance: None,
            notes: None,
            has_activities_log: None,
            classification: None,
            timestamp_creation: None,
            timestamp_modification: None,
        }
    }
}

pub struct CreateRoomRequestBuilder {
    name: String,
    parent_id: Option<u64>,
    recycle_bin_retention_period: Option<u32>,
    quota: Option<u64>,
    inherit_permissions: Option<bool>,
    admin_ids: Option<Vec<u64>>,
    admin_group_ids: Option<Vec<u64>>,
    new_group_member_acceptance: Option<GroupMemberAcceptance>,
    notes: Option<String>,
    has_activities_log: Option<bool>,
    classification: Option<u8>,
    timestamp_creation: Option<String>,
    timestamp_modification: Option<String>,
}

impl CreateRoomRequestBuilder {
    pub fn with_parent_id(mut self, parent_id: u64) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_recycle_bin_retention_period(mut self, recycle_bin_retention_period: u32) -> Self {
        self.recycle_bin_retention_period = Some(recycle_bin_retention_period);
        self
    }

    pub fn with_quota(mut self, quota: u64) -> Self {
        self.quota = Some(quota);
        self
    }

    pub fn with_inherit_permissions(mut self, inherit_permissions: bool) -> Self {
        self.inherit_permissions = Some(inherit_permissions);
        self
    }

    pub fn with_admin_ids(mut self, admin_ids: Vec<u64>) -> Self {
        self.admin_ids = Some(admin_ids);
        self
    }

    pub fn with_admin_group_ids(mut self, admin_group_ids: Vec<u64>) -> Self {
        self.admin_group_ids = Some(admin_group_ids);
        self
    }

    pub fn with_new_group_member_acceptance(
        mut self,
        new_group_member_acceptance: GroupMemberAcceptance,
    ) -> Self {
        self.new_group_member_acceptance = Some(new_group_member_acceptance);
        self
    }

    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    pub fn with_has_activities_log(mut self, has_activities_log: bool) -> Self {
        self.has_activities_log = Some(has_activities_log);
        self
    }

    pub fn with_classification(mut self, classification: u8) -> Self {
        self.classification = Some(classification);
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

    pub fn build(self) -> CreateRoomRequest {
        CreateRoomRequest {
            name: self.name,
            parent_id: self.parent_id,
            recycle_bin_retention_period: self.recycle_bin_retention_period,
            quota: self.quota,
            inherit_permissions: self.inherit_permissions,
            admin_ids: self.admin_ids,
            admin_group_ids: self.admin_group_ids,
            new_group_member_acceptance: self.new_group_member_acceptance,
            notes: self.notes,
            has_activities_log: self.has_activities_log,
            classification: self.classification,
            timestamp_creation: self.timestamp_creation,
            timestamp_modification: self.timestamp_modification,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoomRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quota: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_creation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_modification: Option<String>,
}

impl UpdateRoomRequest {
    pub fn builder() -> UpdateRoomRequestBuilder {
        UpdateRoomRequestBuilder {
            name: None,
            quota: None,
            notes: None,
            timestamp_creation: None,
            timestamp_modification: None,
        }
    }
}

pub struct UpdateRoomRequestBuilder {
    name: Option<String>,
    quota: Option<u64>,
    notes: Option<String>,
    timestamp_creation: Option<String>,
    timestamp_modification: Option<String>,
}

impl UpdateRoomRequestBuilder {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_quota(mut self, quota: u64) -> Self {
        self.quota = Some(quota);
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

    pub fn build(self) -> UpdateRoomRequest {
        UpdateRoomRequest {
            name: self.name,
            quota: self.quota,
            notes: self.notes,
            timestamp_creation: self.timestamp_creation,
            timestamp_modification: self.timestamp_modification,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyRoomRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    default_expiration_period: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_virus_protection_enabled: Option<bool>,
}

impl PolicyRoomRequest {
    pub fn builder() -> PolicyRoomRequestBuilder {
        PolicyRoomRequestBuilder {
            default_expiration_period: None,
            is_virus_protection_enabled: None,
        }
    }
}

pub struct PolicyRoomRequestBuilder {
    default_expiration_period: Option<u64>,
    is_virus_protection_enabled: Option<bool>,
}

impl PolicyRoomRequestBuilder {
    pub fn with_default_expiration_period(mut self, default_expiration_period: u64) -> Self {
        self.default_expiration_period = Some(default_expiration_period);
        self
    }
    
    pub fn with_is_virus_protection_enabled(mut self, enable_virus_protection: bool) -> Self {
        self.is_virus_protection_enabled = Some(enable_virus_protection);
        self
    }

    pub fn build(self) -> PolicyRoomRequest {
        PolicyRoomRequest {
            default_expiration_period: self.default_expiration_period,
            is_virus_protection_enabled: self.is_virus_protection_enabled,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyRoom {
    pub default_expiration_period: u64,
    pub is_virus_protection_enabled: bool,
}

#[async_trait]
impl FromResponse for PolicyRoom {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRoomRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    recycle_bin_retention_period: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inherit_permissions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    take_over_permissions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    admin_ids: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    admin_group_ids: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_group_member_acceptance: Option<GroupMemberAcceptance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_activities_log: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    classification: Option<u8>,
}

impl ConfigRoomRequest {
    pub fn builder() -> ConfigRoomRequestBuilder {
        ConfigRoomRequestBuilder {
            recycle_bin_retention_period: None,
            inherit_permissions: None,
            take_over_permissions: None,
            admin_ids: None,
            admin_group_ids: None,
            new_group_member_acceptance: None,
            has_activities_log: None,
            classification: None,
        }
    }
}

pub struct ConfigRoomRequestBuilder {
    recycle_bin_retention_period: Option<u32>,
    inherit_permissions: Option<bool>,
    take_over_permissions: Option<bool>,
    admin_ids: Option<Vec<u64>>,
    admin_group_ids: Option<Vec<u64>>,
    new_group_member_acceptance: Option<GroupMemberAcceptance>,
    has_activities_log: Option<bool>,
    classification: Option<u8>,
}

impl ConfigRoomRequestBuilder {
    pub fn with_recycle_bin_retention_period(mut self, recycle_bin_retention_period: u32) -> Self {
        self.recycle_bin_retention_period = Some(recycle_bin_retention_period);
        self
    }

    pub fn with_inherit_permissions(mut self, inherit_permissions: bool) -> Self {
        self.inherit_permissions = Some(inherit_permissions);
        self
    }

    pub fn with_take_over_permissions(mut self, take_over_permissions: bool) -> Self {
        self.take_over_permissions = Some(take_over_permissions);
        self
    }

    pub fn with_admin_ids(mut self, admin_ids: Vec<u64>) -> Self {
        self.admin_ids = Some(admin_ids);
        self
    }

    pub fn with_admin_group_ids(mut self, admin_group_ids: Vec<u64>) -> Self {
        self.admin_group_ids = Some(admin_group_ids);
        self
    }

    pub fn with_new_group_member_acceptance(
        mut self,
        new_group_member_acceptance: GroupMemberAcceptance,
    ) -> Self {
        self.new_group_member_acceptance = Some(new_group_member_acceptance);
        self
    }

    pub fn with_has_activities_log(mut self, has_activities_log: bool) -> Self {
        self.has_activities_log = Some(has_activities_log);
        self
    }

    pub fn with_classification(mut self, classification: u8) -> Self {
        self.classification = Some(classification);
        self
    }

    pub fn build(self) -> ConfigRoomRequest {
        ConfigRoomRequest {
            recycle_bin_retention_period: self.recycle_bin_retention_period,
            inherit_permissions: self.inherit_permissions,
            take_over_permissions: self.take_over_permissions,
            admin_ids: self.admin_ids,
            admin_group_ids: self.admin_group_ids,
            new_group_member_acceptance: self.new_group_member_acceptance,
            has_activities_log: self.has_activities_log,
            classification: self.classification,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptRoomRequest {
    is_encrypted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_data_space_rescue_key: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data_room_rescue_key: Option<UserKeyPairContainer>,
}

impl EncryptRoomRequest {
    pub fn builder(is_encrypted: bool) -> EncryptRoomRequestBuilder {
        EncryptRoomRequestBuilder {
            is_encrypted,
            use_data_space_rescue_key: None,
            data_room_rescue_key: None,
        }
    }
}

pub struct EncryptRoomRequestBuilder {
    is_encrypted: bool,
    use_data_space_rescue_key: Option<bool>,
    data_room_rescue_key: Option<UserKeyPairContainer>,
}

impl EncryptRoomRequestBuilder {
    pub fn with_use_data_space_rescue_key(mut self, use_data_space_rescue_key: bool) -> Self {
        self.use_data_space_rescue_key = Some(use_data_space_rescue_key);
        self
    }

    pub fn try_with_data_room_rescue_key(
        mut self,
        data_room_rescue_secret: &str,
    ) -> Result<Self, DracoonCryptoError> {
        let keypair = DracoonCrypto::create_plain_user_keypair(UserKeyPairVersion::RSA4096)?;
        let enc_keypair = DracoonCrypto::encrypt_private_key(data_room_rescue_secret, keypair)?;
        self.data_room_rescue_key = Some(enc_keypair);
        Ok(self)
    }

    pub fn build(self) -> EncryptRoomRequest {
        EncryptRoomRequest {
            is_encrypted: self.is_encrypted,
            use_data_space_rescue_key: self.use_data_space_rescue_key,
            data_room_rescue_key: self.data_room_rescue_key,
        }
    }
}

pub type RoomGroupList = RangedItems<RoomGroup>;

#[async_trait]
impl FromResponse for RoomGroupList {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<RoomGroupList, DracoonErrorResponse>(response).await
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomGroup {
    pub id: u64,
    pub name: String,
    pub is_granted: bool,
    pub new_group_member_acceptance: Option<GroupMemberAcceptance>,
    pub permissions: Option<NodePermissions>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomGroupsAddBatchRequest {
    items: Vec<RoomGroupsAddBatchRequestItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomGroupsAddBatchRequestItem {
    id: u64,
    new_group_member_acceptance: Option<GroupMemberAcceptance>,
    permissions: NodePermissions,
}

impl RoomGroupsAddBatchRequestItem {
    pub fn new(
        id: u64,
        permissions: NodePermissions,
        new_group_member_acceptance: Option<GroupMemberAcceptance>,
    ) -> Self {
        RoomGroupsAddBatchRequestItem {
            id,
            new_group_member_acceptance,
            permissions,
        }
    }
}

impl From<Vec<RoomGroupsAddBatchRequestItem>> for RoomGroupsAddBatchRequest {
    fn from(items: Vec<RoomGroupsAddBatchRequestItem>) -> Self {
        RoomGroupsAddBatchRequest { items }
    }
}

#[derive(Default)]
pub struct NodePermissionsBuilder {
    manage: Option<bool>,
    read: Option<bool>,
    create: Option<bool>,
    change: Option<bool>,
    delete: Option<bool>,
    manage_download_share: Option<bool>,
    manage_upload_share: Option<bool>,
    read_recycle_bin: Option<bool>,
    restore_recycle_bin: Option<bool>,
    delete_recycle_bin: Option<bool>,
}

impl NodePermissionsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_manage(mut self, manage: bool) -> Self {
        self.manage = Some(manage);
        self
    }

    pub fn with_read(mut self, read: bool) -> Self {
        self.read = Some(read);
        self
    }

    pub fn with_create(mut self, create: bool) -> Self {
        self.create = Some(create);
        self
    }

    pub fn with_change(mut self, change: bool) -> Self {
        self.change = Some(change);
        self
    }

    pub fn with_delete(mut self, delete: bool) -> Self {
        self.delete = Some(delete);
        self
    }

    pub fn with_manage_download_share(mut self, manage_download_shares: bool) -> Self {
        self.manage_download_share = Some(manage_download_shares);
        self
    }

    pub fn with_manage_upload_share(mut self, manage_upload_shares: bool) -> Self {
        self.manage_upload_share = Some(manage_upload_shares);
        self
    }

    pub fn with_read_recycle_bin(mut self, read_recycle_bin: bool) -> Self {
        self.read_recycle_bin = Some(read_recycle_bin);
        self
    }

    pub fn with_restore_recycle_bin(mut self, restore_recycle_bin: bool) -> Self {
        self.restore_recycle_bin = Some(restore_recycle_bin);
        self
    }

    pub fn with_delete_recycle_bin(mut self, delete_recycle_bin: bool) -> Self {
        self.delete_recycle_bin = Some(delete_recycle_bin);
        self
    }

    pub fn build(self) -> NodePermissions {
        NodePermissions {
            manage: self.manage.unwrap_or(false),
            read: self.read.unwrap_or(false),
            create: self.create.unwrap_or(false),
            change: self.change.unwrap_or(false),
            delete: self.delete.unwrap_or(false),
            manage_download_share: self.manage_download_share.unwrap_or(false),
            manage_upload_share: self.manage_upload_share.unwrap_or(false),
            read_recycle_bin: self.read_recycle_bin.unwrap_or(false),
            restore_recycle_bin: self.restore_recycle_bin.unwrap_or(false),
            delete_recycle_bin: self.delete_recycle_bin.unwrap_or(false),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomGroupsDeleteBatchRequest {
    ids: Vec<u64>,
}

impl From<Vec<u64>> for RoomGroupsDeleteBatchRequest {
    fn from(ids: Vec<u64>) -> Self {
        RoomGroupsDeleteBatchRequest { ids }
    }
}

pub type RoomUserList = RangedItems<RoomUser>;

#[async_trait]
impl FromResponse for RoomUserList {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomUser {
    pub user_info: UserInfo,
    pub is_granted: bool,
    pub permissions: Option<NodePermissions>,
    pub public_key_container: Option<PublicKeyContainer>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomUsersAddBatchRequest {
    items: Vec<RoomUsersAddBatchRequestItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomUsersAddBatchRequestItem {
    id: u64,
    permissions: NodePermissions,
}

impl From<Vec<RoomUsersAddBatchRequestItem>> for RoomUsersAddBatchRequest {
    fn from(items: Vec<RoomUsersAddBatchRequestItem>) -> Self {
        RoomUsersAddBatchRequest { items }
    }
}

impl RoomUsersAddBatchRequestItem {
    pub fn new(id: u64, permissions: NodePermissions) -> Self {
        RoomUsersAddBatchRequestItem { id, permissions }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomUsersDeleteBatchRequest {
    ids: Vec<u64>,
}

impl From<Vec<u64>> for RoomUsersDeleteBatchRequest {
    fn from(ids: Vec<u64>) -> Self {
        RoomUsersDeleteBatchRequest { ids }
    }
}
