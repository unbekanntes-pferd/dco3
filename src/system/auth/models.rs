use std::sync::Arc;

use async_trait::async_trait;
use dco3_derive::FromResponse;
use reqwest::Response;
use serde::Deserialize;

use crate::{
    client::{DracoonClient, DracoonErrorResponse},
    utils::{parse_body, FromResponse},
    DracoonClientError,
};

#[derive(Clone)]
pub struct SystemAuthEndpoint<S> {
    client: Arc<DracoonClient<S>>,
    state: std::marker::PhantomData<S>,
}

impl<S> SystemAuthEndpoint<S> {
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
pub struct ActiveDirectoryConfig {
    pub id: u64,
    pub alias: String,
    pub server_ip: String,
    pub server_port: u32,
    pub server_admin_name: String,
    pub ldap_users_domain: String,
    pub user_filter: String,
    pub user_import: bool,
    pub use_ldaps: bool,
    pub ad_export_group: String,
    pub sds_import_group: Option<u64>,
    pub ssl_finger_print: Option<String>,
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
pub struct ActiveDirectoryConfigList {
    pub items: Vec<ActiveDirectoryConfig>,
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct OpenIdIdpConfig {
    pub id: u64,
    pub name: Option<String>,
    pub issuer: Option<String>,
    pub authorization_end_point_url: Option<String>,
    pub token_end_point_url: Option<String>,
    pub user_info_end_point_url: Option<String>,
    pub jwks_end_point_url: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uris: Vec<String>,
    pub scopes: Option<Vec<String>>,
    pub mapping_claim: Option<String>,
    pub flow: Option<String>,
    pub pkce_enabled: Option<bool>,
    pub pkce_challenge_method: Option<String>,
    pub fallback_mapping_claim: Option<String>,
    pub user_info_source: Option<String>,
    pub user_import_enabled: Option<bool>,
    pub user_import_group: Option<u64>,
    pub user_update_enabled: Option<bool>,
    pub user_management_url: Option<String>,
}

// this is needed because no type alias exists
#[async_trait]
impl FromResponse for Vec<OpenIdIdpConfig> {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}
