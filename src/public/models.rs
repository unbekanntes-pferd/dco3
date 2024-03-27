use chrono::{DateTime, Utc};
use dco3_derive::FromResponse;
use serde::Deserialize;

#[derive(Debug, Deserialize, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareVersionData {
    pub rest_api_version: String,
    pub sds_server_version: String,
    pub build_date: DateTime<Utc>,
    pub is_dracoon_cloud: Option<bool>

}

#[derive(Debug, Deserialize, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub language_default: String,
    pub s3_hosts: Vec<String>,
    pub s3_enforce_direct_upload: bool,
    #[serde(rename = "useS3Storage")]
    pub use_s3_storage: bool,
}