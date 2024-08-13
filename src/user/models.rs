use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dco3_crypto::UserKeyPairContainer;
use dco3_derive::FromResponse;
use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::{
    auth::{errors::DracoonClientError, models::DracoonErrorResponse, DracoonClient},
    utils::{parse_body, FromResponse},
};

#[derive(Clone)]
pub struct UserEndpoint<S> {
    client: Arc<DracoonClient<S>>,
    state: std::marker::PhantomData<S>,
}

impl<S> UserEndpoint<S> {
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
#[allow(non_snake_case)]
pub struct UserAccount {
    pub id: u64,
    pub first_name: String,
    pub last_name: String,
    pub user_name: String,
    pub is_locked: bool,
    pub has_manageable_rooms: bool,
    pub user_roles: RoleList,
    pub language: String,
    pub auth_data: UserAuthData,
    pub must_set_email: Option<bool>,
    pub needs_to_accept_EULA: Option<bool>,
    pub expire_at: Option<DateTime<Utc>>,
    pub is_encryption_enabled: Option<bool>,
    pub last_login_success_at: Option<String>,
    pub last_login_fail_at: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub home_room_id: Option<u64>,
    pub user_groups: Option<Vec<UserGroup>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserAuthData {
    pub method: String,
    pub login: Option<String>,
    pub password: Option<String>,
    pub must_change_password: Option<bool>,
    pub ad_config_id: Option<u64>,
    pub oid_config_id: Option<u64>,
}

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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserGroup {
    pub id: u64,
    pub is_member: bool,
    pub name: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(non_snake_case)]
pub struct UpdateUserAccountRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    user_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    accept_EULA: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
}

impl UpdateUserAccountRequest {
    pub fn builder() -> UpdateUserAccountRequestBuilder {
        UpdateUserAccountRequestBuilder::new()
    }
}

#[derive(Debug, Serialize, Default)]
#[allow(non_snake_case)]
pub struct UpdateUserAccountRequestBuilder {
    user_name: Option<String>,
    accept_EULA: Option<bool>,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    language: Option<String>,
}

#[allow(non_snake_case)]
impl UpdateUserAccountRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_user_name(mut self, user_name: impl Into<String>) -> Self {
        self.user_name = Some(user_name.into());
        self
    }

    pub fn with_accept_EULA(mut self, accept_EULA: bool) -> Self {
        self.accept_EULA = Some(accept_EULA);
        self
    }

    pub fn with_first_name(mut self, first_name: impl Into<String>) -> Self {
        self.first_name = Some(first_name.into());
        self
    }

    pub fn with_last_name(mut self, last_name: impl Into<String>) -> Self {
        self.last_name = Some(last_name.into());
        self
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn build(self) -> UpdateUserAccountRequest {
        UpdateUserAccountRequest {
            user_name: self.user_name,
            accept_EULA: self.accept_EULA,
            first_name: self.first_name,
            last_name: self.last_name,
            email: self.email,
            phone: self.phone,
            language: self.language,
        }
    }
}

#[async_trait]
impl FromResponse for UserKeyPairContainer {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct CustomerData {
    pub id: u64,
    pub name: String,
    pub is_provider_customer: bool,
    pub space_limit: u64,
    pub space_used: u64,
    pub accounts_limit: u64,
    pub accounts_used: u64,
    pub cnt_internal_user: Option<u64>,
    pub cnt_guest_user: Option<u64>,
    pub customer_encryption_enabled: bool,
}
