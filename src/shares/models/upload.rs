use async_trait::async_trait;
use chrono::{Date, DateTime, Utc};
use dco3_derive::FromResponse;
use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::{
    auth::DracoonErrorResponse,
    models::{FilterOperator, FilterQuery, ObjectExpiration, RangedItems, SortOrder, SortQuery},
    nodes::models::UserInfo,
    utils::{parse_body, FromResponse},
    DracoonClientError,
};

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct UploadShare {
    pub id: u64,
    pub name: String,
    pub target_id: u64,
    pub is_protected: bool,
    pub access_key: String,
    pub created_at: DateTime<Utc>,
    pub created_by: UserInfo,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<UserInfo>,
    pub expire_at: Option<DateTime<Utc>>,
    pub target_path: Option<String>,
    pub is_encrypted: Option<bool>,
    pub notes: Option<String>,
    pub internal_notes: Option<String>,
    pub file_expiry_period: Option<u32>,
    pub max_slots: Option<u32>,
    pub max_size: Option<u64>,
    pub show_uploaded_files: Option<bool>,
    pub cnt_files: Option<u32>,
    pub cnt_uploads: Option<u32>,
    pub data_url: Option<String>,
    pub target_type: Option<String>,
    pub show_creator_name: Option<bool>,
    pub show_creator_username: Option<bool>,
}

#[async_trait]
impl FromResponse for UploadSharesList {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

pub type UploadSharesList = RangedItems<UploadShare>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadShareLinkEmail {
    body: String,
    recipients: Vec<String>,
    receiver_language: Option<String>,
}

impl UploadShareLinkEmail {
    pub fn new(
        body: impl Into<String>,
        recipients: Vec<String>,
        receiver_language: Option<String>,
    ) -> Self {
        Self {
            body: body.into(),
            recipients,
            receiver_language,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateUploadShareRequest {
    target_id: u64,
    name: Option<String>,
    password: Option<String>,
    expiration: Option<ObjectExpiration>,
    file_expiry_period: Option<u32>,
    notes: Option<String>,
    internal_notes: Option<String>,
    max_slots: Option<u32>,
    max_size: Option<u64>,
    show_uploaded_files: Option<bool>,
    show_creator_name: Option<bool>,
    show_creator_username: Option<bool>,
    text_message_recipients: Option<Vec<String>>,
}

impl CreateUploadShareRequest {
    pub fn builder(target_id: u64) -> CreateUploadShareRequestBuilder {
        CreateUploadShareRequestBuilder::new(target_id)
    }
}

#[derive(Debug, Default)]
pub struct CreateUploadShareRequestBuilder {
    target_id: u64,
    name: Option<String>,
    password: Option<String>,
    expiration: Option<ObjectExpiration>,
    file_expiry_period: Option<u32>,
    notes: Option<String>,
    internal_notes: Option<String>,
    max_slots: Option<u32>,
    max_size: Option<u64>,
    show_uploaded_files: Option<bool>,
    show_creator_name: Option<bool>,
    show_creator_username: Option<bool>,
    text_message_recipients: Option<Vec<String>>,
}

impl CreateUploadShareRequestBuilder {
    pub fn new(target_id: u64) -> Self {
        Self {
            target_id,
            ..Default::default()
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

    pub fn with_expiration(mut self, expiration: ObjectExpiration) -> Self {
        self.expiration = Some(expiration);
        self
    }

    pub fn with_file_expiry_period(mut self, file_expiry_period: u32) -> Self {
        self.file_expiry_period = Some(file_expiry_period);
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

    pub fn with_max_slots(mut self, max_slots: u32) -> Self {
        self.max_slots = Some(max_slots);
        self
    }

    pub fn with_max_size(mut self, max_size: u64) -> Self {
        self.max_size = Some(max_size);
        self
    }

    pub fn with_show_uploaded_files(mut self, show_uploaded_files: bool) -> Self {
        self.show_uploaded_files = Some(show_uploaded_files);
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

    pub fn with_text_message_recipients(mut self, text_message_recipients: Vec<String>) -> Self {
        self.text_message_recipients = Some(text_message_recipients);
        self
    }

    pub fn build(self) -> CreateUploadShareRequest {
        CreateUploadShareRequest {
            target_id: self.target_id,
            name: self.name,
            password: self.password,
            expiration: self.expiration,
            file_expiry_period: self.file_expiry_period,
            notes: self.notes,
            internal_notes: self.internal_notes,
            max_slots: self.max_slots,
            max_size: self.max_size,
            show_uploaded_files: self.show_uploaded_files,
            show_creator_name: self.show_creator_name,
            show_creator_username: self.show_creator_username,
            text_message_recipients: self.text_message_recipients,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUploadShareRequest {
    name: Option<String>,
    password: Option<String>,
    expiration: Option<ObjectExpiration>,
    file_expiry_period: Option<u32>,
    notes: Option<String>,
    internal_notes: Option<String>,
    max_slots: Option<u32>,
    max_size: Option<u64>,
    show_uploaded_files: Option<bool>,
    show_creator_name: Option<bool>,
    show_creator_username: Option<bool>,
    text_message_recipients: Option<Vec<String>>,
    reset_max_slots: Option<bool>,
    reset_max_size: Option<bool>,
    reset_file_expiry_period: Option<bool>,
    default_country: Option<String>,
    receiver_language: Option<String>,
}

impl UpdateUploadShareRequest {
    pub fn builder() -> UpdateUploadShareRequestBuilder {
        UpdateUploadShareRequestBuilder::new()
    }
}

#[derive(Debug, Default)]
pub struct UpdateUploadShareRequestBuilder {
    name: Option<String>,
    password: Option<String>,
    expiration: Option<ObjectExpiration>,
    file_expiry_period: Option<u32>,
    notes: Option<String>,
    internal_notes: Option<String>,
    max_slots: Option<u32>,
    max_size: Option<u64>,
    show_uploaded_files: Option<bool>,
    show_creator_name: Option<bool>,
    show_creator_username: Option<bool>,
    text_message_recipients: Option<Vec<String>>,
    reset_max_slots: Option<bool>,
    reset_max_size: Option<bool>,
    reset_file_expiry_period: Option<bool>,
    default_country: Option<String>,
    receiver_language: Option<String>,
}

impl UpdateUploadShareRequestBuilder {
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

    pub fn with_expiration(mut self, expiration: ObjectExpiration) -> Self {
        self.expiration = Some(expiration);
        self
    }

    pub fn with_file_expiry_period(mut self, file_expiry_period: u32) -> Self {
        self.file_expiry_period = Some(file_expiry_period);
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

    pub fn with_max_slots(mut self, max_slots: u32) -> Self {
        self.max_slots = Some(max_slots);
        self
    }

    pub fn with_max_size(mut self, max_size: u64) -> Self {
        self.max_size = Some(max_size);
        self
    }

    pub fn with_show_uploaded_files(mut self, show_uploaded_files: bool) -> Self {
        self.show_uploaded_files = Some(show_uploaded_files);
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

    pub fn with_text_message_recipients(mut self, text_message_recipients: Vec<String>) -> Self {
        self.text_message_recipients = Some(text_message_recipients);
        self
    }

    pub fn with_reset_max_slots(mut self, reset_max_slots: bool) -> Self {
        self.reset_max_slots = Some(reset_max_slots);
        self
    }

    pub fn with_reset_max_size(mut self, reset_max_size: bool) -> Self {
        self.reset_max_size = Some(reset_max_size);
        self
    }

    pub fn with_reset_file_expiry_period(mut self, reset_file_expiry_period: bool) -> Self {
        self.reset_file_expiry_period = Some(reset_file_expiry_period);
        self
    }

    pub fn with_default_country(mut self, default_country: impl Into<String>) -> Self {
        self.default_country = Some(default_country.into());
        self
    }

    pub fn with_receiver_language(mut self, receiver_language: impl Into<String>) -> Self {
        self.receiver_language = Some(receiver_language.into());
        self
    }

    pub fn build(self) -> UpdateUploadShareRequest {
        UpdateUploadShareRequest {
            name: self.name,
            password: self.password,
            expiration: self.expiration,
            file_expiry_period: self.file_expiry_period,
            notes: self.notes,
            internal_notes: self.internal_notes,
            max_slots: self.max_slots,
            max_size: self.max_size,
            show_uploaded_files: self.show_uploaded_files,
            show_creator_name: self.show_creator_name,
            show_creator_username: self.show_creator_username,
            text_message_recipients: self.text_message_recipients,
            reset_max_slots: self.reset_max_slots,
            reset_max_size: self.reset_max_size,
            reset_file_expiry_period: self.reset_file_expiry_period,
            default_country: self.default_country,
            receiver_language: self.receiver_language,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteUploadSharesRequest {
    share_ids: Vec<u64>,
}

impl From<Vec<u64>> for DeleteUploadSharesRequest {
    fn from(share_ids: Vec<u64>) -> Self {
        DeleteUploadSharesRequest { share_ids }
    }
}

impl DeleteUploadSharesRequest {
    pub fn new(share_ids: Vec<u64>) -> Self {
        share_ids.into()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUploadSharesBulkRequest {
    object_ids: Vec<u64>,
    expiration: Option<ObjectExpiration>,
    show_creator_name: Option<bool>,
    show_creator_username: Option<bool>,
    show_uploaded_files: Option<bool>,
    max_slots: Option<u32>,
    reset_max_slots: Option<bool>,
    max_size: Option<u64>,
    reset_max_size: Option<bool>,
    file_expiry_period: Option<u32>,
    reset_file_expiry_period: Option<bool>,
}

impl UpdateUploadSharesBulkRequest {
    pub fn builder(object_ids: Vec<u64>) -> UpdateUploadSharesBulkRequestBuilder {
        UpdateUploadSharesBulkRequestBuilder::new(object_ids)
    }
}

#[derive(Debug, Default)]
pub struct UpdateUploadSharesBulkRequestBuilder {
    object_ids: Vec<u64>,
    expiration: Option<ObjectExpiration>,
    show_creator_name: Option<bool>,
    show_creator_username: Option<bool>,
    show_uploaded_files: Option<bool>,
    max_slots: Option<u32>,
    reset_max_slots: Option<bool>,
    max_size: Option<u64>,
    reset_max_size: Option<bool>,
    file_expiry_period: Option<u32>,
    reset_file_expiry_period: Option<bool>,
}

impl UpdateUploadSharesBulkRequestBuilder {
    pub fn new(object_ids: Vec<u64>) -> Self {
        UpdateUploadSharesBulkRequestBuilder {
            object_ids,
            ..Default::default()
        }
    }

    pub fn with_expiration(mut self, expiration: ObjectExpiration) -> Self {
        self.expiration = Some(expiration);
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

    pub fn with_show_uploaded_files(mut self, show_uploaded_files: bool) -> Self {
        self.show_uploaded_files = Some(show_uploaded_files);
        self
    }

    pub fn with_max_slots(mut self, max_slots: u32) -> Self {
        self.max_slots = Some(max_slots);
        self
    }

    pub fn with_reset_max_slots(mut self, reset_max_slots: bool) -> Self {
        self.reset_max_slots = Some(reset_max_slots);
        self
    }

    pub fn with_max_size(mut self, max_size: u64) -> Self {
        self.max_size = Some(max_size);
        self
    }

    pub fn with_reset_max_size(mut self, reset_max_size: bool) -> Self {
        self.reset_max_size = Some(reset_max_size);
        self
    }

    pub fn with_file_expiry_period(mut self, file_expiry_period: u32) -> Self {
        self.file_expiry_period = Some(file_expiry_period);
        self
    }

    pub fn with_reset_file_expiry_period(mut self, reset_file_expiry_period: bool) -> Self {
        self.reset_file_expiry_period = Some(reset_file_expiry_period);
        self
    }

    pub fn build(self) -> UpdateUploadSharesBulkRequest {
        UpdateUploadSharesBulkRequest {
            object_ids: self.object_ids,
            expiration: self.expiration,
            show_creator_name: self.show_creator_name,
            show_creator_username: self.show_creator_username,
            show_uploaded_files: self.show_uploaded_files,
            max_slots: self.max_slots,
            reset_max_slots: self.reset_max_slots,
            max_size: self.max_size,
            reset_max_size: self.reset_max_size,
            file_expiry_period: self.file_expiry_period,
            reset_file_expiry_period: self.reset_file_expiry_period,
        }
    }
}

#[derive(Debug)]
pub enum UploadSharesFilter {
    Name(FilterOperator, String),
    CreatedAt(FilterOperator, String),
    CreatedBy(FilterOperator, String),
    CreatedById(FilterOperator, u64),
    AccessKey(FilterOperator, String),
    TargetId(FilterOperator, u64),
    UserId(FilterOperator, u64),
    UpdatedBy(FilterOperator, String),
    UpdatedById(FilterOperator, u64),
}

impl FilterQuery for UploadSharesFilter {
    fn to_filter_string(&self) -> String {
        match self {
            UploadSharesFilter::Name(op, val) => {
                let op: String = op.into();

                format!("name:{}:{}", op, val)
            }
            UploadSharesFilter::CreatedAt(op, val) => {
                let op: String = op.into();

                format!("createdAt:{}:{}", op, val)
            }

            UploadSharesFilter::CreatedBy(op, val) => {
                let op: String = op.into();

                format!("createdBy:{}:{}", op, val)
            }

            UploadSharesFilter::CreatedById(op, val) => {
                let op: String = op.into();

                format!("createdById:{}:{}", op, val)
            }

            UploadSharesFilter::AccessKey(op, val) => {
                let op: String = op.into();

                format!("accessKey:{}:{}", op, val)
            }

            UploadSharesFilter::TargetId(op, val) => {
                let op: String = op.into();

                format!("targetId:{}:{}", op, val)
            }

            UploadSharesFilter::UserId(op, val) => {
                let op: String = op.into();

                format!("userId:{}:{}", op, val)
            }

            UploadSharesFilter::UpdatedBy(op, val) => {
                let op: String = op.into();

                format!("updatedBy:{}:{}", op, val)
            }

            UploadSharesFilter::UpdatedById(op, val) => {
                let op: String = op.into();

                format!("updatedById:{}:{}", op, val)
            }
        }
    }
}

impl From<UploadSharesFilter> for Box<dyn FilterQuery> {
    fn from(filter: UploadSharesFilter) -> Self {
        Box::new(filter)
    }
}

impl UploadSharesFilter {
    pub fn name_contains(val: impl Into<String>) -> Self {
        UploadSharesFilter::Name(FilterOperator::Cn, val.into())
    }

    pub fn created_at_before(val: impl Into<String>) -> Self {
        UploadSharesFilter::CreatedAt(FilterOperator::Le, val.into())
    }

    pub fn created_at_after(val: impl Into<String>) -> Self {
        UploadSharesFilter::CreatedAt(FilterOperator::Ge, val.into())
    }

    pub fn created_by_contains(val: impl Into<String>) -> Self {
        UploadSharesFilter::CreatedBy(FilterOperator::Cn, val.into())
    }

    pub fn created_by_equals(val: impl Into<String>) -> Self {
        UploadSharesFilter::CreatedBy(FilterOperator::Eq, val.into())
    }

    pub fn created_by_id_equals(val: u64) -> Self {
        UploadSharesFilter::CreatedById(FilterOperator::Eq, val)
    }

    pub fn access_key_contains(val: impl Into<String>) -> Self {
        UploadSharesFilter::AccessKey(FilterOperator::Cn, val.into())
    }

    pub fn target_id_equals(val: u64) -> Self {
        UploadSharesFilter::TargetId(FilterOperator::Eq, val)
    }

    pub fn user_id_equals(val: u64) -> Self {
        UploadSharesFilter::UserId(FilterOperator::Eq, val)
    }

    pub fn updated_by_contains(val: impl Into<String>) -> Self {
        UploadSharesFilter::UpdatedBy(FilterOperator::Cn, val.into())
    }

    pub fn updated_by_equals(val: impl Into<String>) -> Self {
        UploadSharesFilter::UpdatedBy(FilterOperator::Eq, val.into())
    }

    pub fn updated_by_id_equals(val: u64) -> Self {
        UploadSharesFilter::UpdatedById(FilterOperator::Eq, val)
    }
}

#[derive(Debug)]
pub enum UploadSharesSortBy {
    Name(SortOrder),
    NotifyCreator(SortOrder),
    ExpireAt(SortOrder),
    CreatedAt(SortOrder),
    CreatedBy(SortOrder),
}

impl SortQuery for UploadSharesSortBy {
    fn to_sort_string(&self) -> String {
        match self {
            UploadSharesSortBy::Name(order) => {
                let order: String = order.into();
                format!("name:{}", order)
            }
            UploadSharesSortBy::NotifyCreator(order) => {
                let order: String = order.into();
                format!("notifyCreator:{}", order)
            }
            UploadSharesSortBy::ExpireAt(order) => {
                let order: String = order.into();
                format!("expireAt:{}", order)
            }
            UploadSharesSortBy::CreatedAt(order) => {
                let order: String = order.into();
                format!("createdAt:{}", order)
            }
            UploadSharesSortBy::CreatedBy(order) => {
                let order: String = order.into();
                format!("createdBy:{}", order)
            }
        }
    }
}

impl From<UploadSharesSortBy> for Box<dyn SortQuery> {
    fn from(sort: UploadSharesSortBy) -> Self {
        Box::new(sort)
    }
}

impl UploadSharesSortBy {
    pub fn name(order: SortOrder) -> Self {
        UploadSharesSortBy::Name(order)
    }

    pub fn notify_creator(order: SortOrder) -> Self {
        UploadSharesSortBy::NotifyCreator(order)
    }

    pub fn expire_at(order: SortOrder) -> Self {
        UploadSharesSortBy::ExpireAt(order)
    }

    pub fn created_at(order: SortOrder) -> Self {
        UploadSharesSortBy::CreatedAt(order)
    }

    pub fn created_by(order: SortOrder) -> Self {
        UploadSharesSortBy::CreatedBy(order)
    }
}
