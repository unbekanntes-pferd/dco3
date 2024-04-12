use std::sync::Arc;

use chrono::{DateTime, Utc};
use dco3_derive::FromResponse;
use serde::Deserialize;

use crate::auth::DracoonClient;

#[derive(Clone)]
pub struct PublicEndpoint<S> {
    client: Arc<DracoonClient<S>>,
    state: std::marker::PhantomData<S>,
}

impl<S> PublicEndpoint<S> {
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

#[derive(Debug, Deserialize, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareVersionData {
    pub rest_api_version: String,
    pub sds_server_version: String,
    pub build_date: DateTime<Utc>,
    pub is_dracoon_cloud: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub language_default: String,
    pub s3_hosts: Vec<String>,
    pub s3_enforce_direct_upload: bool,
    #[serde(rename = "useS3Storage")]
    pub use_s3_storage: bool,
}
