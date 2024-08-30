use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::{
    auth::{DracoonClient, DracoonErrorResponse},
    models::RangedItems,
    nodes::UserInfo,
    utils::{parse_body, FromResponse},
    DracoonClientError,
};

#[derive(Clone)]
pub struct RolesEndpoint<S> {
    client: Arc<DracoonClient<S>>,
    state: std::marker::PhantomData<S>,
}

impl<S> RolesEndpoint<S> {
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

pub type RoleGroupList = RangedItems<RoleGroup>;

#[async_trait]
impl FromResponse for RoleGroupList {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleGroup {
    pub id: u64,
    pub is_member: bool,
    pub name: String,
}

pub type RoleUserList = RangedItems<RoleUser>;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Right {
    pub id: u64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub items: Option<Vec<Right>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoleList {
    pub items: Vec<Role>,
}

#[async_trait]
impl FromResponse for RoleList {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

#[async_trait]
impl FromResponse for RoleUserList {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleUser {
    pub user_info: UserInfo,
    pub is_member: bool,
    pub id: Option<u64>,              //depreacted, but still returned
    pub display_name: Option<String>, // depreacted, but still returned
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RevokeRoleBatchRequest {
    ids: Vec<u64>,
}

impl From<Vec<u64>> for RevokeRoleBatchRequest {
    fn from(ids: Vec<u64>) -> Self {
        RevokeRoleBatchRequest { ids }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssignRoleBatchRequest {
    ids: Vec<u64>,
}

impl From<Vec<u64>> for AssignRoleBatchRequest {
    fn from(ids: Vec<u64>) -> Self {
        AssignRoleBatchRequest { ids }
    }
}
