use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dco3_derive::FromResponse;
use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::{
    auth::{DracoonClient, DracoonErrorResponse},
    models::{FilterOperator, FilterQuery, ObjectExpiration, RangedItems, SortOrder, SortQuery},
    nodes::models::UserInfo,
    user::models::RoleList,
    utils::{parse_body, FromResponse},
    DracoonClientError,
};

#[derive(Clone)]
pub struct GroupsEndpoint<S> {
    client: Arc<DracoonClient<S>>,
    state: std::marker::PhantomData<S>,
}

impl <S> GroupsEndpoint<S> {
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

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: u64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub created_by: UserInfo,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<UserInfo>,
    pub cnt_users: Option<u64>,
    pub expire_at: Option<DateTime<Utc>>,
    pub group_roles: Option<RoleList>,
}

pub type GroupList = RangedItems<Group>;

#[async_trait]
impl FromResponse for GroupList {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupRequest {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<ObjectExpiration>,
}

impl CreateGroupRequest {
    pub fn new(name: impl Into<String>, expiration: Option<ObjectExpiration>) -> Self {
        Self {
            name: name.into(),
            expiration,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGroupRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<ObjectExpiration>,
}

impl UpdateGroupRequest {
    pub fn name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            expiration: None,
        }
    }

    pub fn expiration(expiration: ObjectExpiration) -> Self {
        Self {
            name: None,
            expiration: Some(expiration),
        }
    }

    pub fn new(name: impl Into<String>, expiration: ObjectExpiration) -> Self {
        Self {
            name: Some(name.into()),
            expiration: Some(expiration),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeGroupMembersRequest {
    ids: Vec<u64>,
}

impl From<Vec<u64>> for ChangeGroupMembersRequest {
    fn from(ids: Vec<u64>) -> Self {
        Self { ids }
    }
}

impl ChangeGroupMembersRequest {
    pub fn new(ids: Vec<u64>) -> Self {
        Self { ids }
    }
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct LastAdminGroupRoomList {
    pub items: Vec<LastAdminGroupRoom>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LastAdminGroupRoom {
    pub id: u64,
    pub name: String,
    pub parent_path: String,
    pub parent_id: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupUser {
    pub user_info: UserInfo,
    pub is_member: bool,
}

pub type GroupUserList = RangedItems<GroupUser>;

#[async_trait]
impl FromResponse for GroupUserList {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

#[derive(Debug)]
pub enum GroupsFilter {
    Name(FilterOperator, String),
    HasRole(FilterOperator, String),
}

impl FilterQuery for GroupsFilter {
    fn to_filter_string(&self) -> String {
        match self {
            GroupsFilter::Name(op, val) => {
                let op: String = op.into();
                format!("name:{}:{}", op, val)
            }
            GroupsFilter::HasRole(op, val) => {
                let op: String = op.into();
                format!("hasRole:{}:{}", op, val)
            }
        }
    }
}

impl GroupsFilter {
    pub fn name_contains(val: impl Into<String>) -> Self {
        Self::Name(FilterOperator::Cn, val.into())
    }

    pub fn has_role(val: impl Into<String>) -> Self {
        Self::HasRole(FilterOperator::Eq, val.into())
    }
}

#[derive(Debug)]
pub enum GroupsSortBy {
    Name(SortOrder),
    CreatedAt(SortOrder),
    ExpireAt(SortOrder),
    CntUsers(SortOrder),
}

impl SortQuery for GroupsSortBy {
    fn to_sort_string(&self) -> String {
        match self {
            GroupsSortBy::Name(order) => {
                let order: String = order.into();
                format!("name:{}", order)
            }
            GroupsSortBy::CreatedAt(order) => {
                let order: String = order.into();
                format!("createdAt:{}", order)
            }
            GroupsSortBy::ExpireAt(order) => {
                let order: String = order.into();
                format!("expireAt:{}", order)
            }
            GroupsSortBy::CntUsers(order) => {
                let order: String = order.into();
                format!("cntUsers:{}", order)
            }
        }
    }
}

impl GroupsSortBy {
    pub fn name(order: SortOrder) -> Self {
        Self::Name(order)
    }

    pub fn created_at(order: SortOrder) -> Self {
        Self::CreatedAt(order)
    }

    pub fn expire_at(order: SortOrder) -> Self {
        Self::ExpireAt(order)
    }

    pub fn cnt_users(order: SortOrder) -> Self {
        Self::CntUsers(order)
    }
}

impl From<GroupsSortBy> for Box<dyn SortQuery> {
    fn from(sort_by: GroupsSortBy) -> Self {
        Box::new(sort_by)
    }
}

impl From<GroupsFilter> for Box<dyn FilterQuery> {
    fn from(filter: GroupsFilter) -> Self {
        Box::new(filter)
    }
}

#[derive(Debug)]
pub enum GroupUsersFilter {
    User(FilterOperator, String),
    IsMember(FilterOperator, bool),
}

impl FilterQuery for GroupUsersFilter {
    fn to_filter_string(&self) -> String {
        match self {
            GroupUsersFilter::User(op, val) => {
                let op: String = op.into();
                format!("user:{}:{}", op, val)
            }
            GroupUsersFilter::IsMember(op, val) => {
                let op: String = op.into();
                format!("isMember:{}:{}", op, val)
            }
        }
    }
}

impl GroupUsersFilter {
    pub fn user_contains(val: impl Into<String>) -> Self {
        Self::User(FilterOperator::Cn, val.into())
    }

    pub fn is_member(val: bool) -> Self {
        Self::IsMember(FilterOperator::Eq, val)
    }
}

impl From<GroupUsersFilter> for Box<dyn FilterQuery> {
    fn from(filter: GroupUsersFilter) -> Self {
        Box::new(filter)
    }
}
