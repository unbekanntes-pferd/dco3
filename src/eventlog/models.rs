use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dco3_derive::FromResponse;
use reqwest::Response;
use serde::Deserialize;

use crate::{
    client::{DracoonClient, DracoonErrorResponse},
    nodes::{NodePermissions, UserInfo},
    utils::{parse_body, FromResponse},
    DracoonClientError, FilterOperator, FilterQuery, RangedItems, SortOrder, SortQuery,
};

#[derive(Clone)]
pub struct EventlogEndpoint<S> {
    client: Arc<DracoonClient<S>>,
    state: std::marker::PhantomData<S>,
}

impl<S> EventlogEndpoint<S> {
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

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(try_from = "i64")]
pub enum EventStatus {
    Success = 0,
    Failure = 2,
}

impl From<EventStatus> for i64 {
    fn from(status: EventStatus) -> Self {
        status as i64
    }
}

impl TryFrom<i64> for EventStatus {
    type Error = DracoonClientError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EventStatus::Success),
            2 => Ok(EventStatus::Failure),
            _ => Err(DracoonClientError::Unknown),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub id: i64,
    pub time: DateTime<Utc>,
    pub user_id: i64,
    pub message: String,
    pub operation_id: Option<i64>,
    pub operation_name: Option<String>,
    pub status: Option<EventStatus>,
    pub user_client: Option<String>,
    pub customer_id: Option<i64>,
    pub user_name: Option<String>,
    pub user_ip: Option<String>,
    pub auth_parent_source: Option<String>,
    pub auth_parent_target: Option<String>,
    pub object_id1: Option<i64>,
    pub object_id2: Option<i64>,
    pub object_type1: Option<i64>,
    pub object_type2: Option<i64>,
    pub object_name1: Option<String>,
    pub object_name2: Option<String>,
    pub attribute1: Option<String>,
    pub attribute2: Option<String>,
    pub attribute3: Option<String>,
}

pub type LogEventList = RangedItems<LogEvent>;

#[async_trait]
impl FromResponse for LogEventList {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LogOperation {
    pub id: i64,
    pub name: String,
    pub is_deprecated: bool,
}

#[derive(Deserialize, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct LogOperationList {
    pub operation_list: Vec<LogOperation>,
}

#[derive(Default, Debug)]
pub struct EventlogParams {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub sort: Option<Box<dyn SortQuery>>,
    pub date_start: Option<DateTime<Utc>>,
    pub date_end: Option<DateTime<Utc>>,
    pub user_id: Option<i64>,
    pub operation_type: Option<i64>,
    pub status: Option<EventStatus>,
    pub user_client: Option<String>,
}

impl EventlogParams {
    pub fn builder() -> EventlogParamsBuilder {
        EventlogParamsBuilder::new()
    }

    pub fn is_empty(&self) -> bool {
        self.offset.is_none()
            && self.limit.is_none()
            && self.sort.is_none()
            && self.date_start.is_none()
            && self.date_end.is_none()
            && self.user_id.is_none()
            && self.operation_type.is_none()
            && self.status.is_none()
            && self.user_client.is_none()
    }
}

#[derive(Default, Debug)]
pub struct EventlogParamsBuilder {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub sort: Option<Box<dyn SortQuery>>,
    pub date_start: Option<DateTime<Utc>>,
    pub date_end: Option<DateTime<Utc>>,
    pub user_id: Option<i64>,
    pub operation_type: Option<i64>,
    pub status: Option<EventStatus>,
    pub user_client: Option<String>,
}

impl EventlogParamsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_sort(mut self, sort: impl Into<Box<dyn SortQuery>>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    pub fn with_date_start(mut self, date_start: DateTime<Utc>) -> Self {
        self.date_start = Some(date_start);
        self
    }

    pub fn with_date_end(mut self, date_end: DateTime<Utc>) -> Self {
        self.date_end = Some(date_end);
        self
    }

    pub fn with_user_id(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_operation_type(mut self, operation_type: i64) -> Self {
        self.operation_type = Some(operation_type);
        self
    }

    pub fn with_status(mut self, status: EventStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_user_client(mut self, user_client: String) -> Self {
        self.user_client = Some(user_client);
        self
    }

    pub fn build(self) -> EventlogParams {
        EventlogParams {
            offset: self.offset,
            limit: self.limit,
            sort: self.sort,
            date_start: self.date_start,
            date_end: self.date_end,
            user_id: self.user_id,
            operation_type: self.operation_type,
            status: self.status,
            user_client: self.user_client,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EventlogSortBy {
    Time(SortOrder),
}

impl SortQuery for EventlogSortBy {
    fn to_sort_string(&self) -> String {
        match self {
            EventlogSortBy::Time(order) => {
                let order: String = order.into();
                format!("time:{}", order)
            }
        }
    }
}

impl From<EventlogSortBy> for Box<dyn SortQuery> {
    fn from(sort_by: EventlogSortBy) -> Self {
        Box::new(sort_by)
    }
}

#[derive(Deserialize, FromResponse, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuditUserPermission {
    pub user_id: i64,
    pub user_login: String,
    pub user_first_name: String,
    pub user_last_name: String,
    pub permissions: NodePermissions,
}

#[derive(Deserialize, FromResponse, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuditNodeResponse {
    pub node_id: i64,
    pub node_name: String,
    pub node_parent_path: String,
    pub node_cnt_children: u64,
    pub audit_user_permission_list: Vec<AuditUserPermission>,
    pub node_parent_id: Option<i64>,
    pub node_size: Option<u64>,
    pub node_recycle_bin_retention_period: Option<u64>,
    pub node_quota: Option<u64>,
    pub node_is_encrypted: Option<bool>,
    pub node_has_activities_log: Option<bool>,
    pub node_created_at: Option<DateTime<Utc>>,
    pub node_updated_at: Option<DateTime<Utc>>,
    pub node_created_by: Option<UserInfo>,
    pub node_updated_by: Option<UserInfo>,
}

pub type AuditNodeList = Vec<AuditNodeResponse>;

#[async_trait]
impl FromResponse for AuditNodeList {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

#[derive(Debug, Clone)]
pub enum AuditNodesSortBy {
    NodeId(SortOrder),
    NodeName(SortOrder),
    ParentId(SortOrder),
    Size(SortOrder),
    Quota(SortOrder),
}

impl SortQuery for AuditNodesSortBy {
    fn to_sort_string(&self) -> String {
        match self {
            AuditNodesSortBy::NodeId(order) => {
                let order: String = order.into();
                format!("nodeId:{}", order)
            }
            AuditNodesSortBy::NodeName(order) => {
                let order: String = order.into();
                format!("nodeName:{}", order)
            }
            AuditNodesSortBy::ParentId(order) => {
                let order: String = order.into();
                format!("nodeParentId:{}", order)
            }
            AuditNodesSortBy::Size(order) => {
                let order: String = order.into();
                format!("nodeSize:{}", order)
            }
            AuditNodesSortBy::Quota(order) => {
                let order: String = order.into();
                format!("nodeQuota:{}", order)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuditNodesFilter {
    NodeId(u64, FilterOperator),
    NodeName(String, FilterOperator),
    NodeParentId(u64, FilterOperator),
    UserId(u64, FilterOperator),
    UserName(String, FilterOperator),
    PermissionsManage(bool),
    NodeIsEncrypted(bool),
}

impl FilterQuery for AuditNodesFilter {
    fn to_filter_string(&self) -> String {
        match self {
            AuditNodesFilter::NodeId(val, op) => {
                let op: String = op.into();
                format!("nodeId:{}:{}", op, val)
            }
            AuditNodesFilter::NodeName(val, op) => {
                let op: String = op.into();
                format!("nodeName:{}:{}", op, val)
            }
            AuditNodesFilter::NodeParentId(val, op) => {
                let op: String = op.into();
                format!("nodeParentId:{}:{}", op, val)
            }
            AuditNodesFilter::UserId(val, op) => {
                let op: String = op.into();
                format!("userId:{}:{}", op, val)
            }
            AuditNodesFilter::UserName(val, op) => {
                let op: String = op.into();
                format!("userName:{}:{}", op, val)
            }
            AuditNodesFilter::PermissionsManage(val) => format!("permissionsManage:eq:{}", val),
            AuditNodesFilter::NodeIsEncrypted(val) => format!("nodeIsEncrypted:eq:{}", val),
        }
    }
}

impl AuditNodesFilter {
    pub fn node_id_equals(val: u64) -> Self {
        Self::NodeId(val, FilterOperator::Eq)
    }

    pub fn node_name_contains(val: impl Into<String>) -> Self {
        Self::NodeName(val.into(), FilterOperator::Cn)
    }

    pub fn node_name_equals(val: impl Into<String>) -> Self {
        Self::NodeName(val.into(), FilterOperator::Eq)
    }

    pub fn user_id_equals(val: u64) -> Self {
        Self::UserId(val, FilterOperator::Eq)
    }

    pub fn user_name_contains(val: impl Into<String>) -> Self {
        Self::UserName(val.into(), FilterOperator::Cn)
    }

    pub fn user_name_equals(val: impl Into<String>) -> Self {
        Self::UserName(val.into(), FilterOperator::Eq)
    }

    pub fn permissions_manage(val: bool) -> Self {
        Self::PermissionsManage(val)
    }

    pub fn node_is_encrypted(val: bool) -> Self {
        Self::NodeIsEncrypted(val)
    }
}

impl AuditNodesSortBy {
    pub fn node_id(order: SortOrder) -> Self {
        Self::NodeId(order)
    }

    pub fn node_name(order: SortOrder) -> Self {
        Self::NodeName(order)
    }

    pub fn node_parent_id(order: SortOrder) -> Self {
        Self::ParentId(order)
    }

    pub fn node_size(order: SortOrder) -> Self {
        Self::Size(order)
    }

    pub fn node_quota(order: SortOrder) -> Self {
        Self::Quota(order)
    }
}

impl EventlogSortBy {
    pub fn time(order: SortOrder) -> Self {
        Self::Time(order)
    }
}

impl From<AuditNodesSortBy> for Box<dyn SortQuery> {
    fn from(sort_by: AuditNodesSortBy) -> Self {
        Box::new(sort_by)
    }
}

impl From<AuditNodesFilter> for Box<dyn FilterQuery> {
    fn from(filter: AuditNodesFilter) -> Self {
        Box::new(filter)
    }
}
