use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dco3_crypto::UserKeyPairContainer;
use dco3_derive::FromResponse;
use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::{
    client::DracoonErrorResponse,
    models::{FilterOperator, FilterQuery, ObjectExpiration, RangedItems, SortOrder, SortQuery},
    nodes::models::{NodeType, UserInfo},
    utils::{parse_body, FromResponse},
    DracoonClientError,
};

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct DownloadShare {
    pub id: u64,
    pub node_id: u64,
    pub name: String,
    pub access_key: String,
    pub cnt_downloads: u32,
    pub created_at: DateTime<Utc>,
    pub created_by: UserInfo,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<UserInfo>,
    pub notes: Option<String>,
    pub show_creator_name: Option<bool>,
    pub show_creator_username: Option<bool>,
    pub internal_notes: Option<String>,
    pub node_path: Option<String>,
    pub data_url: Option<String>,
    pub is_encrypted: Option<bool>,
    pub is_protected: Option<bool>,
    pub node_type: Option<NodeType>,
    pub max_downloads: Option<u32>,
    pub expire_at: Option<DateTime<Utc>>,
}

pub type DownloadSharesList = RangedItems<DownloadShare>;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateDownloadShareRequest {
    node_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<ObjectExpiration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    show_creator_name: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    show_creator_username: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    internal_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    receiver_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_message_recipients: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keypair: Option<UserKeyPairContainer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_downloads: Option<u32>,
}

impl CreateDownloadShareRequest {
    pub fn builder(node_id: u64) -> CreateDownloadShareRequestBuilder {
        CreateDownloadShareRequestBuilder::new(node_id)
    }
}

pub struct CreateDownloadShareRequestBuilder {
    node_id: u64,
    name: Option<String>,
    password: Option<String>,
    expiration: Option<ObjectExpiration>,
    notes: Option<String>,
    show_creator_name: Option<bool>,
    show_creator_username: Option<bool>,
    internal_notes: Option<String>,
    receiver_language: Option<String>,
    text_message_recipients: Option<Vec<String>>,
    keypair: Option<UserKeyPairContainer>,
    max_downloads: Option<u32>,
}

impl CreateDownloadShareRequestBuilder {
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            name: None,
            password: None,
            expiration: None,
            notes: None,
            show_creator_name: None,
            show_creator_username: None,
            internal_notes: None,
            receiver_language: None,
            text_message_recipients: None,
            keypair: None,
            max_downloads: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn with_expiration(mut self, expiration: impl Into<ObjectExpiration>) -> Self {
        self.expiration = Some(expiration.into());
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    pub fn with_show_creator_name(mut self, show_creator_name: bool) -> Self {
        self.show_creator_name = Some(show_creator_name);
        self
    }

    pub fn with_show_creator_username(mut self, show_creator_username: bool) -> Self {
        self.show_creator_username = Some(show_creator_username);
        self
    }

    pub fn with_internal_notes(mut self, internal_notes: impl Into<String>) -> Self {
        self.internal_notes = Some(internal_notes.into());
        self
    }

    pub fn with_receiver_language(mut self, receiver_language: impl Into<String>) -> Self {
        self.receiver_language = Some(receiver_language.into());
        self
    }

    pub fn with_text_message_recipients(mut self, text_message_recipients: Vec<String>) -> Self {
        self.text_message_recipients = Some(text_message_recipients);
        self
    }

    pub fn with_keypair(mut self, keypair: UserKeyPairContainer) -> Self {
        self.keypair = Some(keypair);
        self
    }

    pub fn with_max_downloads(mut self, max_downloads: u32) -> Self {
        self.max_downloads = Some(max_downloads);
        self
    }

    pub fn build(self) -> CreateDownloadShareRequest {
        CreateDownloadShareRequest {
            node_id: self.node_id,
            name: self.name,
            password: self.password,
            expiration: self.expiration,
            notes: self.notes,
            show_creator_name: self.show_creator_name,
            show_creator_username: self.show_creator_username,
            internal_notes: self.internal_notes,
            receiver_language: self.receiver_language,
            text_message_recipients: self.text_message_recipients,
            keypair: self.keypair,
            max_downloads: self.max_downloads,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDownloadSharesBulkRequest {
    object_ids: Vec<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<ObjectExpiration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    show_creator_name: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    show_creator_username: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_downloads: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reset_max_downloads: Option<bool>,
}

impl UpdateDownloadSharesBulkRequest {
    pub fn builder(object_ids: Vec<u64>) -> UpdateDownloadSharesBulkRequestBuilder {
        UpdateDownloadSharesBulkRequestBuilder::new(object_ids)
    }
}

#[async_trait]
impl FromResponse for DownloadSharesList {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

pub struct UpdateDownloadSharesBulkRequestBuilder {
    object_ids: Vec<u64>,
    expiration: Option<ObjectExpiration>,
    show_creator_name: Option<bool>,
    show_creator_username: Option<bool>,
    max_downloads: Option<u32>,
    reset_max_downloads: Option<bool>,
}

impl UpdateDownloadSharesBulkRequestBuilder {
    pub fn new(object_ids: Vec<u64>) -> Self {
        Self {
            object_ids,
            expiration: None,
            show_creator_name: None,
            show_creator_username: None,
            max_downloads: None,
            reset_max_downloads: None,
        }
    }

    pub fn with_expiration(mut self, expiration: impl Into<ObjectExpiration>) -> Self {
        self.expiration = Some(expiration.into());
        self
    }

    pub fn with_show_creator_name(mut self, show_creator_name: bool) -> Self {
        self.show_creator_name = Some(show_creator_name);
        self
    }

    pub fn with_show_creator_username(mut self, show_creator_username: bool) -> Self {
        self.show_creator_username = Some(show_creator_username);
        self
    }

    pub fn with_max_downloads(mut self, max_downloads: u32) -> Self {
        self.max_downloads = Some(max_downloads);
        self
    }

    pub fn with_reset_max_downloads(mut self, reset_max_downloads: bool) -> Self {
        self.reset_max_downloads = Some(reset_max_downloads);
        self
    }

    pub fn build(self) -> UpdateDownloadSharesBulkRequest {
        UpdateDownloadSharesBulkRequest {
            object_ids: self.object_ids,
            expiration: self.expiration,
            show_creator_name: self.show_creator_name,
            show_creator_username: self.show_creator_username,
            max_downloads: self.max_downloads,
            reset_max_downloads: self.reset_max_downloads,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDownloadSharesRequest {
    share_ids: Vec<u64>,
}

impl From<Vec<u64>> for DeleteDownloadSharesRequest {
    fn from(share_ids: Vec<u64>) -> Self {
        DeleteDownloadSharesRequest { share_ids }
    }
}

impl DeleteDownloadSharesRequest {
    pub fn new(share_ids: Vec<u64>) -> Self {
        share_ids.into()
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDownloadShareRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<ObjectExpiration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    internal_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    show_creator_name: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    show_creator_username: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_downloads: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_message_recipients: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    receiver_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reset_password: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reset_max_downloads: Option<bool>,
}

impl UpdateDownloadShareRequest {
    pub fn builder() -> UpdateDownloadShareRequestBuilder {
        UpdateDownloadShareRequestBuilder::new()
    }
}

#[derive(Debug, Default)]
pub struct UpdateDownloadShareRequestBuilder {
    name: Option<String>,
    password: Option<String>,
    expiration: Option<ObjectExpiration>,
    notes: Option<String>,
    internal_notes: Option<String>,
    show_creator_name: Option<bool>,
    show_creator_username: Option<bool>,
    max_downloads: Option<u32>,
    text_message_recipients: Option<Vec<String>>,
    receiver_language: Option<String>,
    default_country: Option<String>,
    reset_password: Option<bool>,
    reset_max_downloads: Option<bool>,
}

impl UpdateDownloadShareRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn with_expiration(mut self, expiration: impl Into<ObjectExpiration>) -> Self {
        self.expiration = Some(expiration.into());
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    pub fn with_internal_notes(mut self, internal_notes: impl Into<String>) -> Self {
        self.internal_notes = Some(internal_notes.into());
        self
    }

    pub fn with_show_creator_name(mut self, show_creator_name: bool) -> Self {
        self.show_creator_name = Some(show_creator_name);
        self
    }

    pub fn with_show_creator_username(mut self, show_creator_username: bool) -> Self {
        self.show_creator_username = Some(show_creator_username);
        self
    }

    pub fn with_max_downloads(mut self, max_downloads: u32) -> Self {
        self.max_downloads = Some(max_downloads);
        self
    }

    pub fn with_text_message_recipients(mut self, text_message_recipients: Vec<String>) -> Self {
        self.text_message_recipients = Some(text_message_recipients);
        self
    }

    pub fn with_receiver_language(mut self, receiver_language: impl Into<String>) -> Self {
        self.receiver_language = Some(receiver_language.into());
        self
    }

    pub fn with_default_country(mut self, default_country: impl Into<String>) -> Self {
        self.default_country = Some(default_country.into());
        self
    }

    pub fn with_reset_password(mut self, reset_password: bool) -> Self {
        self.reset_password = Some(reset_password);
        self
    }

    pub fn with_reset_max_downloads(mut self, reset_max_downloads: bool) -> Self {
        self.reset_max_downloads = Some(reset_max_downloads);
        self
    }

    pub fn build(self) -> UpdateDownloadShareRequest {
        UpdateDownloadShareRequest {
            name: self.name,
            password: self.password,
            expiration: self.expiration,
            notes: self.notes,
            internal_notes: self.internal_notes,
            show_creator_name: self.show_creator_name,
            show_creator_username: self.show_creator_username,
            max_downloads: self.max_downloads,
            text_message_recipients: self.text_message_recipients,
            receiver_language: self.receiver_language,
            default_country: self.default_country,
            reset_password: self.reset_password,
            reset_max_downloads: self.reset_max_downloads,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DownloadSharesFilter {
    Name(FilterOperator, String),
    CreatedAt(FilterOperator, String),
    CreatedBy(FilterOperator, String),
    CreatedById(FilterOperator, u64),
    AcessKey(FilterOperator, String),
    NodeId(FilterOperator, u64),
    UpdatedBy(FilterOperator, String),
    UpdatedById(FilterOperator, u64),
}

impl FilterQuery for DownloadSharesFilter {
    fn to_filter_string(&self) -> String {
        match self {
            DownloadSharesFilter::Name(op, value) => {
                let op: String = op.into();

                format!("name:{}:{}", op, value)
            }
            DownloadSharesFilter::CreatedAt(op, value) => {
                let op: String = op.into();

                format!("createdAt:{}:{}", op, value)
            }
            DownloadSharesFilter::CreatedBy(op, value) => {
                let op: String = op.into();

                format!("createdBy:{}:{}", op, value)
            }
            DownloadSharesFilter::CreatedById(op, value) => {
                let op: String = op.into();

                format!("createdById:{}:{}", op, value)
            }
            DownloadSharesFilter::AcessKey(op, value) => {
                let op: String = op.into();

                format!("accessKey:{}:{}", op, value)
            }
            DownloadSharesFilter::NodeId(op, value) => {
                let op: String = op.into();

                format!("nodeId:{}:{}", op, value)
            }
            DownloadSharesFilter::UpdatedBy(op, value) => {
                let op: String = op.into();

                format!("updatedBy:{}:{}", op, value)
            }
            DownloadSharesFilter::UpdatedById(op, value) => {
                let op: String = op.into();

                format!("updatedById:{}:{}", op, value)
            }
        }
    }
}

impl From<DownloadSharesFilter> for Box<dyn FilterQuery> {
    fn from(filter: DownloadSharesFilter) -> Self {
        Box::new(filter)
    }
}

impl DownloadSharesFilter {
    pub fn name_contains(value: impl Into<String>) -> Self {
        DownloadSharesFilter::Name(FilterOperator::Cn, value.into())
    }

    pub fn created_at_before(value: impl Into<String>) -> Self {
        DownloadSharesFilter::CreatedAt(FilterOperator::Le, value.into())
    }

    pub fn created_at_after(value: impl Into<String>) -> Self {
        DownloadSharesFilter::CreatedAt(FilterOperator::Ge, value.into())
    }

    pub fn created_by_contains(value: impl Into<String>) -> Self {
        DownloadSharesFilter::CreatedBy(FilterOperator::Cn, value.into())
    }

    pub fn created_by_equals(value: impl Into<String>) -> Self {
        DownloadSharesFilter::CreatedBy(FilterOperator::Eq, value.into())
    }

    pub fn created_by_id_equals(value: u64) -> Self {
        DownloadSharesFilter::CreatedById(FilterOperator::Eq, value)
    }

    pub fn access_key_contains(value: impl Into<String>) -> Self {
        DownloadSharesFilter::AcessKey(FilterOperator::Cn, value.into())
    }

    pub fn node_id_equals(value: u64) -> Self {
        DownloadSharesFilter::NodeId(FilterOperator::Eq, value)
    }

    pub fn updated_by_contains(value: impl Into<String>) -> Self {
        DownloadSharesFilter::UpdatedBy(FilterOperator::Cn, value.into())
    }

    pub fn updated_by_equals(value: impl Into<String>) -> Self {
        DownloadSharesFilter::UpdatedBy(FilterOperator::Eq, value.into())
    }

    pub fn updated_by_id_equals(value: u64) -> Self {
        DownloadSharesFilter::UpdatedById(FilterOperator::Eq, value)
    }
}

#[derive(Debug, Clone)]
pub enum DownloadSharesSortBy {
    Name(SortOrder),
    NotifyCreator(SortOrder),
    ExpireAt(SortOrder),
    CreatedAt(SortOrder),
    CreatedBy(SortOrder),
    Classification(SortOrder),
}

impl SortQuery for DownloadSharesSortBy {
    fn to_sort_string(&self) -> String {
        match self {
            DownloadSharesSortBy::Name(order) => {
                let order: String = order.into();

                format!("name:{}", order)
            }
            DownloadSharesSortBy::NotifyCreator(order) => {
                let order: String = order.into();

                format!("notifyCreator:{}", order)
            }
            DownloadSharesSortBy::ExpireAt(order) => {
                let order: String = order.into();

                format!("expireAt:{}", order)
            }
            DownloadSharesSortBy::CreatedAt(order) => {
                let order: String = order.into();

                format!("createdAt:{}", order)
            }
            DownloadSharesSortBy::CreatedBy(order) => {
                let order: String = order.into();

                format!("createdBy:{}", order)
            }
            DownloadSharesSortBy::Classification(order) => {
                let order: String = order.into();

                format!("classification:{}", order)
            }
        }
    }
}

impl From<DownloadSharesSortBy> for Box<dyn SortQuery> {
    fn from(sort: DownloadSharesSortBy) -> Self {
        Box::new(sort)
    }
}

impl DownloadSharesSortBy {
    pub fn name(order: SortOrder) -> Self {
        DownloadSharesSortBy::Name(order)
    }

    pub fn notify_creator(order: SortOrder) -> Self {
        DownloadSharesSortBy::NotifyCreator(order)
    }

    pub fn expire_at(order: SortOrder) -> Self {
        DownloadSharesSortBy::ExpireAt(order)
    }

    pub fn created_at(order: SortOrder) -> Self {
        DownloadSharesSortBy::CreatedAt(order)
    }

    pub fn created_by(order: SortOrder) -> Self {
        DownloadSharesSortBy::CreatedBy(order)
    }

    pub fn classification(order: SortOrder) -> Self {
        DownloadSharesSortBy::Classification(order)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DownloadShareLinkEmail {
    recipients: Vec<String>,
    body: String,
    receiver_language: Option<String>,
}

impl DownloadShareLinkEmail {
    pub fn new(
        body: impl Into<String>,
        recipients: Vec<String>,
        receiver_language: Option<String>,
    ) -> Self {
        DownloadShareLinkEmail {
            recipients,
            body: body.into(),
            receiver_language,
        }
    }
}
