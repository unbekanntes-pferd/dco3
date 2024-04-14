use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dco3_crypto::PublicKeyContainer;
use dco3_derive::FromResponse;
use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::{
    auth::{DracoonClient, DracoonErrorResponse},
    models::{ObjectExpiration, RangedItems},
    user::RoleList,
    utils::{parse_body, FromResponse},
    DracoonClientError, FilterOperator, FilterQuery, SortOrder, SortQuery,
};

pub use crate::user::UserAuthData;

#[derive(Clone)]
pub struct UsersEndpoint<S> {
    client: Arc<DracoonClient<S>>,
    state: std::marker::PhantomData<S>,
}

impl<S> UsersEndpoint<S> {
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

#[derive(Debug, Clone, Deserialize, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub id: u64,
    pub user_name: String,
    pub first_name: String,
    pub last_name: String,
    pub is_locked: bool,
    pub avatar_uuid: String,
    pub auth_data: UserAuthData,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub expire_at: Option<String>,
    pub has_manageable_rooms: Option<bool>,
    pub is_encryption_enabled: Option<bool>,
    pub last_login_success_at: Option<String>,
    pub home_room_id: Option<u64>,
    pub public_key_container: Option<PublicKeyContainer>,
    pub user_roles: Option<RoleList>,
    pub is_mfa_enabled: Option<bool>,
    pub is_mfa_enforced: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserItem {
    pub id: u64,
    pub user_name: String,
    pub first_name: String,
    pub last_name: String,
    pub is_locked: bool,
    pub avatar_uuid: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub expire_at: Option<DateTime<Utc>>,
    pub has_manageable_rooms: Option<bool>,
    pub is_encryption_enabled: Option<bool>,
    pub last_login_success_at: Option<String>,
    pub home_room_id: Option<u64>,
    pub public_key_container: Option<PublicKeyContainer>,
    pub user_roles: Option<RoleList>,
}

pub type UserList = RangedItems<UserItem>;

#[async_trait]
impl FromResponse for UserList {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastAdminUserRoom {
    pub id: u64,
    pub name: String,
    pub parent_path: String,
    pub last_admin_in_group: bool,
    pub parent_id: Option<u64>,
    pub last_admin_in_group_id: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, FromResponse)]
pub struct LastAdminUserRoomList {
    items: Vec<LastAdminUserRoom>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    first_name: String,
    last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<ObjectExpiration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    receiver_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_data: Option<UserAuthData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notify_user: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_nonmember_viewer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mfa_config: Option<MfaConfig>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MfaConfig {
    mfa_enforced: bool,
}

impl From<bool> for MfaConfig {
    fn from(mfa_enforced: bool) -> Self {
        Self { mfa_enforced }
    }
}

impl CreateUserRequest {
    pub fn builder(
        first_name: impl Into<String>,
        last_name: impl Into<String>,
    ) -> CreateUserRequestBuilder {
        CreateUserRequestBuilder::new(first_name, last_name)
    }
}

pub struct CreateUserRequestBuilder {
    first_name: String,
    last_name: String,
    user_name: Option<String>,
    phone: Option<String>,
    expiration: Option<ObjectExpiration>,
    receiver_language: Option<String>,
    auth_data: Option<UserAuthData>,
    email: Option<String>,
    notify_user: Option<bool>,
    is_nonmember_viewer: Option<bool>,
    mfa_config: Option<MfaConfig>,
}

impl CreateUserRequestBuilder {
    pub fn new(first_name: impl Into<String>, last_name: impl Into<String>) -> Self {
        Self {
            first_name: first_name.into(),
            last_name: last_name.into(),
            user_name: None,
            phone: None,
            expiration: None,
            receiver_language: None,
            auth_data: None,
            email: None,
            notify_user: None,
            is_nonmember_viewer: None,
            mfa_config: None,
        }
    }

    pub fn with_user_name(mut self, user_name: impl Into<String>) -> Self {
        self.user_name = Some(user_name.into());
        self
    }

    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    pub fn with_expiration(mut self, expiration: impl Into<ObjectExpiration>) -> Self {
        self.expiration = Some(expiration.into());
        self
    }

    pub fn with_receiver_language(mut self, receiver_language: impl Into<String>) -> Self {
        self.receiver_language = Some(receiver_language.into());
        self
    }

    pub fn with_auth_data(mut self, auth_data: UserAuthData) -> Self {
        self.auth_data = Some(auth_data);
        self
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_notify_user(mut self, notify_user: bool) -> Self {
        self.notify_user = Some(notify_user);
        self
    }

    pub fn with_is_nonmember_viewer(mut self, is_nonmember_viewer: bool) -> Self {
        self.is_nonmember_viewer = Some(is_nonmember_viewer);
        self
    }

    pub fn with_mfa_enforced(mut self, mfa_enforced: bool) -> Self {
        let mfa_config = MfaConfig::from(mfa_enforced);
        self.mfa_config = Some(mfa_config);
        self
    }

    pub fn build(self) -> CreateUserRequest {
        CreateUserRequest {
            first_name: self.first_name,
            last_name: self.last_name,
            user_name: self.user_name,
            phone: self.phone,
            expiration: self.expiration,
            receiver_language: self.receiver_language,
            auth_data: self.auth_data,
            email: self.email,
            notify_user: self.notify_user,
            is_nonmember_viewer: self.is_nonmember_viewer,
            mfa_config: self.mfa_config,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<ObjectExpiration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    receiver_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_data: Option<UserAuthDataUpdateRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mfa_config: Option<MfaConfig>,
}

impl UpdateUserRequest {
    pub fn builder() -> UpdateUserRequestBuilder {
        UpdateUserRequestBuilder::new()
    }
}

#[derive(Default)]
pub struct UpdateUserRequestBuilder {
    first_name: Option<String>,
    last_name: Option<String>,
    user_name: Option<String>,
    phone: Option<String>,
    expiration: Option<ObjectExpiration>,
    receiver_language: Option<String>,
    auth_data: Option<UserAuthDataUpdateRequest>,
    email: Option<String>,
    mf_config: Option<MfaConfig>,
}

impl UpdateUserRequestBuilder {
    pub fn new() -> Self {
        Self {
            first_name: None,
            last_name: None,
            user_name: None,
            phone: None,
            expiration: None,
            receiver_language: None,
            auth_data: None,
            email: None,
            mf_config: None,
        }
    }

    pub fn with_first_name(mut self, first_name: impl Into<String>) -> Self {
        self.first_name = Some(first_name.into());
        self
    }

    pub fn with_last_name(mut self, last_name: impl Into<String>) -> Self {
        self.last_name = Some(last_name.into());
        self
    }

    pub fn with_user_name(mut self, user_name: impl Into<String>) -> Self {
        self.user_name = Some(user_name.into());
        self
    }

    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    pub fn with_expiration(mut self, expiration: impl Into<ObjectExpiration>) -> Self {
        self.expiration = Some(expiration.into());
        self
    }

    pub fn with_receiver_language(mut self, receiver_language: impl Into<String>) -> Self {
        self.receiver_language = Some(receiver_language.into());
        self
    }

    pub fn with_auth_data(mut self, auth_data: UserAuthDataUpdateRequest) -> Self {
        self.auth_data = Some(auth_data);
        self
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_mfa_enforced(mut self, mfa_enforced: bool) -> Self {
        let mfa_config = MfaConfig::from(mfa_enforced);
        self.mf_config = Some(mfa_config);
        self
    }

    pub fn build(self) -> UpdateUserRequest {
        UpdateUserRequest {
            first_name: self.first_name,
            last_name: self.last_name,
            user_name: self.user_name,
            phone: self.phone,
            expiration: self.expiration,
            receiver_language: self.receiver_language,
            auth_data: self.auth_data,
            email: self.email,
            mfa_config: self.mf_config,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAuthDataUpdateRequest {
    auth_method: Option<String>,
    login: Option<String>,
    ad_config_id: Option<u64>,
    oid_config_id: Option<u64>,
}

impl UserAuthDataUpdateRequest {
    pub fn auth_method(auth_method: AuthMethod) -> Self {
        Self {
            auth_method: Some(auth_method.into()),
            login: None,
            ad_config_id: None,
            oid_config_id: None,
        }
    }

    pub fn login(login: impl Into<String>) -> Self {
        Self {
            auth_method: None,
            login: Some(login.into()),
            ad_config_id: None,
            oid_config_id: None,
        }
    }

    pub fn ad_config_id(ad_config_id: u64) -> Self {
        Self {
            auth_method: None,
            login: None,
            ad_config_id: Some(ad_config_id),
            oid_config_id: None,
        }
    }

    pub fn oid_config_id(oid_config_id: u64) -> Self {
        Self {
            auth_method: None,
            login: None,
            ad_config_id: None,
            oid_config_id: Some(oid_config_id),
        }
    }

    pub fn new(
        login: Option<String>,
        auth_method: Option<AuthMethod>,
        ad_config_id: Option<u64>,
        oid_config_id: Option<u64>,
    ) -> Self {
        Self {
            auth_method: None,
            login,
            ad_config_id: None,
            oid_config_id: None,
        }
    }
}

impl UserAuthData {
    pub fn builder(auth_method: AuthMethod) -> UserAuthDataBuilder {
        UserAuthDataBuilder::new(auth_method)
    }

    pub fn new_basic(password: Option<String>, must_change_password: Option<bool>) -> Self {
        // password change is default (notify user is also default)
        let mut must_change_password = must_change_password.unwrap_or(true);

        // password must be changed if set on creation
        if password.is_some() && !must_change_password {
            must_change_password = true;
        }

        Self {
            method: AuthMethod::Basic.into(),
            login: None,
            ad_config_id: None,
            oid_config_id: None,
            password,
            must_change_password: Some(must_change_password),
        }
    }

    pub fn new_oidc(login: impl Into<String>, oid_config_id: u64) -> Self {
        let login: String = login.into();
        Self {
            method: AuthMethod::OpenIdConnect {
                login: login.clone(),
                oid_config_id,
            }
            .into(),
            login: Some(login),
            ad_config_id: None,
            oid_config_id: Some(oid_config_id),
            password: None,
            must_change_password: None,
        }
    }

    pub fn new_ad(login: impl Into<String>, ad_config_id: u64) -> Self {
        let login: String = login.into();
        Self {
            method: AuthMethod::ActiveDirectory {
                login: login.clone(),
                ad_config_id,
            }
            .into(),
            login: Some(login),
            ad_config_id: Some(ad_config_id),
            oid_config_id: None,
            password: None,
            must_change_password: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuthMethod {
    Basic,
    ActiveDirectory { ad_config_id: u64, login: String },
    OpenIdConnect { oid_config_id: u64, login: String },
}

impl From<AuthMethod> for String {
    fn from(auth_method: AuthMethod) -> Self {
        match auth_method {
            AuthMethod::Basic => "basic".to_string(),
            AuthMethod::ActiveDirectory {
                ad_config_id,
                login,
            } => "active_directory".to_string(),
            AuthMethod::OpenIdConnect {
                oid_config_id,
                login,
            } => "openid".to_string(),
        }
    }
}

impl AuthMethod {
    pub fn new_active_directory(ad_config_id: u64, login: impl Into<String>) -> Self {
        Self::ActiveDirectory {
            ad_config_id,
            login: login.into(),
        }
    }

    pub fn new_open_id_connect(oid_config_id: u64, login: impl Into<String>) -> Self {
        Self::OpenIdConnect {
            oid_config_id,
            login: login.into(),
        }
    }

    pub fn new_basic() -> Self {
        Self::Basic
    }
}

pub struct UserAuthDataBuilder {
    method: AuthMethod,
    password: Option<String>,
    must_change_password: Option<bool>,
    ad_config_id: Option<u64>,
    login: Option<String>,
    oidc_config_id: Option<u64>,
}

impl UserAuthDataBuilder {
    pub fn new(method: AuthMethod) -> Self {
        match method {
            AuthMethod::Basic => Self {
                method: AuthMethod::Basic,
                password: None,
                must_change_password: None,
                ad_config_id: None,
                login: None,
                oidc_config_id: None,
            },
            AuthMethod::ActiveDirectory {
                ad_config_id,
                login,
            } => Self {
                method: AuthMethod::ActiveDirectory {
                    ad_config_id,
                    login: login.clone(),
                },
                password: None,
                must_change_password: None,
                ad_config_id: Some(ad_config_id),
                login: Some(login),
                oidc_config_id: None,
            },
            AuthMethod::OpenIdConnect {
                oid_config_id,
                login,
            } => Self {
                method: AuthMethod::OpenIdConnect {
                    oid_config_id,
                    login: login.clone(),
                },
                password: None,
                must_change_password: None,
                ad_config_id: None,
                login: Some(login),
                oidc_config_id: Some(oid_config_id),
            },
        }
    }

    pub fn new_active_directory(ad_config_id: u64, login: impl Into<String>) -> Self {
        Self::new(AuthMethod::new_active_directory(ad_config_id, login))
    }

    pub fn new_open_id_connect(oidc_config_id: u64, login: impl Into<String>) -> Self {
        Self::new(AuthMethod::new_open_id_connect(oidc_config_id, login))
    }

    pub fn new_basic() -> Self {
        Self::new(AuthMethod::new_basic())
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn with_must_change_password(mut self, must_change_password: bool) -> Self {
        self.must_change_password = Some(must_change_password);
        self
    }

    pub fn build(self) -> UserAuthData {
        UserAuthData {
            method: self.method.into(),
            password: self.password,
            must_change_password: self.must_change_password,
            ad_config_id: self.ad_config_id,
            login: self.login,
            oid_config_id: self.oidc_config_id,
        }
    }
}

#[derive(Debug)]
pub enum UsersFilter {
    Email(FilterOperator, String),
    UserName(FilterOperator, String),
    FirstName(FilterOperator, String),
    LastName(FilterOperator, String),
    IsLocked(FilterOperator, bool),
    EffectiveRoles(FilterOperator, bool),
    CreatedAt(FilterOperator, String),
    Phone(FilterOperator, String),
    IsEncryptionEnabled(FilterOperator, bool),
    HasRole(FilterOperator, String),
}

impl FilterQuery for UsersFilter {
    fn to_filter_string(&self) -> String {
        match self {
            Self::Email(op, value) => {
                let op: String = op.into();
                format!("email:{}:{}", op, value)
            }
            Self::UserName(op, value) => {
                let op: String = op.into();
                format!("userName:{}:{}", op, value)
            }
            Self::FirstName(op, value) => {
                let op: String = op.into();
                format!("firstName:{}:{}", op, value)
            }
            Self::LastName(op, value) => {
                let op: String = op.into();
                format!("lastName:{}:{}", op, value)
            }
            Self::IsLocked(op, value) => {
                let op: String = op.into();
                format!("isLocked:{}:{}", op, value)
            }
            Self::EffectiveRoles(op, value) => {
                let op: String = op.into();
                format!("effectiveRoles:{}:{}", op, value)
            }
            Self::CreatedAt(op, value) => {
                let op: String = op.into();
                format!("createdAt:{}:{}", op, value)
            }
            Self::Phone(op, value) => {
                let op: String = op.into();
                format!("phone:{}:{}", op, value)
            }
            Self::IsEncryptionEnabled(op, value) => {
                let op: String = op.into();
                format!("isEncryptionEnabled:{}:{}", op, value)
            }
            Self::HasRole(op, value) => {
                let op: String = op.into();
                format!("hasRole:{}:{}", op, value)
            }
        }
    }
}

impl UsersFilter {
    pub fn email_equals(value: impl Into<String>) -> Self {
        Self::Email(FilterOperator::Eq, value.into())
    }

    pub fn email_contains(value: impl Into<String>) -> Self {
        Self::Email(FilterOperator::Cn, value.into())
    }

    pub fn username_equals(value: impl Into<String>) -> Self {
        Self::UserName(FilterOperator::Eq, value.into())
    }

    pub fn username_contains(value: impl Into<String>) -> Self {
        Self::UserName(FilterOperator::Cn, value.into())
    }

    pub fn first_name_contains(value: impl Into<String>) -> Self {
        Self::FirstName(FilterOperator::Cn, value.into())
    }

    pub fn last_name_contains(value: impl Into<String>) -> Self {
        Self::LastName(FilterOperator::Cn, value.into())
    }

    pub fn is_locked(value: bool) -> Self {
        Self::IsLocked(FilterOperator::Eq, value)
    }

    pub fn effective_roles(value: bool) -> Self {
        Self::EffectiveRoles(FilterOperator::Eq, value)
    }

    pub fn created_at_before(value: impl Into<String>) -> Self {
        Self::CreatedAt(FilterOperator::Le, value.into())
    }

    pub fn created_at_after(value: impl Into<String>) -> Self {
        Self::CreatedAt(FilterOperator::Ge, value.into())
    }

    pub fn phone_equals(value: impl Into<String>) -> Self {
        Self::Phone(FilterOperator::Eq, value.into())
    }

    pub fn is_encryption_enabled(value: bool) -> Self {
        Self::IsEncryptionEnabled(FilterOperator::Eq, value)
    }

    pub fn has_role(value: impl Into<String>) -> Self {
        Self::HasRole(FilterOperator::Eq, value.into())
    }
}

#[derive(Debug)]
pub enum UsersSortBy {
    UserName(SortOrder),
    Email(SortOrder),
    FirstName(SortOrder),
    LastName(SortOrder),
    IsLocked(SortOrder),
    ExpireAt(SortOrder),
    CreatedAt(SortOrder),
}

impl SortQuery for UsersSortBy {
    fn to_sort_string(&self) -> String {
        match self {
            Self::UserName(order) => {
                let order: String = order.into();
                format!("userName:{}", order)
            }
            Self::Email(order) => {
                let order: String = order.into();
                format!("email:{}", order)
            }
            Self::FirstName(order) => {
                let order: String = order.into();
                format!("firstName:{}", order)
            }
            Self::LastName(order) => {
                let order: String = order.into();
                format!("lastName:{}", order)
            }
            Self::IsLocked(order) => {
                let order: String = order.into();
                format!("isLocked:{}", order)
            }
            Self::ExpireAt(order) => {
                let order: String = order.into();
                format!("expireAt:{}", order)
            }
            Self::CreatedAt(order) => {
                let order: String = order.into();
                format!("createdAt:{}", order)
            }
        }
    }
}

impl UsersSortBy {
    pub fn user_name(order: SortOrder) -> Self {
        Self::UserName(order)
    }

    pub fn email(order: SortOrder) -> Self {
        Self::Email(order)
    }

    pub fn first_name(order: SortOrder) -> Self {
        Self::FirstName(order)
    }

    pub fn last_name(order: SortOrder) -> Self {
        Self::LastName(order)
    }

    pub fn is_locked(order: SortOrder) -> Self {
        Self::IsLocked(order)
    }

    pub fn expire_at(order: SortOrder) -> Self {
        Self::ExpireAt(order)
    }

    pub fn created_at(order: SortOrder) -> Self {
        Self::CreatedAt(order)
    }
}

impl From<UsersSortBy> for Box<dyn SortQuery> {
    fn from(f: UsersSortBy) -> Self {
        Box::new(f)
    }
}

impl From<UsersFilter> for Box<dyn FilterQuery> {
    fn from(f: UsersFilter) -> Self {
        Box::new(f)
    }
}
