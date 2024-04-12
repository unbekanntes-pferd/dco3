use std::sync::Arc;

use chrono::{DateTime, Utc};
use dco3_derive::FromResponse;
use serde::Deserialize;

use crate::{auth::DracoonClient, nodes::UserInfo};

#[derive(Clone)]
pub struct ConfigEndpoint<S> {
    client: Arc<DracoonClient<S>>,
    state: std::marker::PhantomData<S>,
}

impl<S> ConfigEndpoint<S> {
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
pub struct GeneralSettingsInfo {
    pub share_password_sms_enabled: bool,
    pub crypto_enabled: bool,
    pub email_notification_button_enabled: bool,
    pub eula_enabled: bool,
    pub use_s3_storage: bool,
    pub s3_tags_enabled: bool,
    pub home_rooms_active: bool,
    pub home_room_parent_id: Option<u64>,
    pub subscription_plan: Option<u8>,
    pub auth_token_restrictions: Option<AuthTokenRestrictions>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthTokenRestrictions {
    pub restriction_enabled: Option<bool>,
    pub access_token_validity: Option<u32>,
    pub refresh_token_validity: Option<u32>,
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct SystemDefaults {
    pub language_default: Option<String>,
    pub download_share_default_expiration_period: Option<u32>,
    pub upload_share_default_expiration_period: Option<u32>,
    pub file_default_expiration_period: Option<u32>,
    pub nonmember_viewer_default: Option<bool>,
    pub hide_login_input_fields: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct InfrastructureProperties {
    pub sms_config_enabled: Option<bool>,
    pub media_server_config_enabled: Option<bool>,
    pub s3_default_region: Option<String>,
    pub s3_enforce_direct_upload: Option<bool>,
    pub is_dracoon_cloud: Option<bool>,
    pub tenant_uuid: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum AlgorithmStatus {
    #[serde(rename = "REQUIRED")]
    Required,
    #[serde(rename = "DISCOURAGED")]
    Discouraged,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AlgorithmVersionInfo {
    pub version: String,
    pub description: String,
    pub status: AlgorithmStatus,
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct AlgorithmVersionInfoList {
    pub file_key_algorithms: Vec<AlgorithmVersionInfo>,
    pub key_pair_algorithms: Vec<AlgorithmVersionInfo>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(from = "u8")]

pub enum MinimumClassification {
    NoPassword = 0,
    Public = 1,
    Internal = 2,
    Confidential = 3,
    StrictlyConfidential = 4,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareClassificationPolicies {
    pub classification_requires_share_password: MinimumClassification,
}

impl From<u8> for MinimumClassification {
    fn from(num: u8) -> Self {
        match num {
            1 => MinimumClassification::Public,
            2 => MinimumClassification::Internal,
            3 => MinimumClassification::Confidential,
            4 => MinimumClassification::StrictlyConfidential,
            _ => MinimumClassification::NoPassword,
        }
    }
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationPoliciesConfig {
    pub share_classification_policies: Option<ShareClassificationPolicies>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PasswordExpiration {
    pub enabled: bool,
    pub max_password_age: Option<u32>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum CharacterRule {
    #[serde(rename = "alpha")]
    Alpha,
    #[serde(rename = "uppercase")]
    UpperCase,
    #[serde(rename = "lowercase")]
    LowerCase,
    #[serde(rename = "numeric")]
    Numeric,
    #[serde(rename = "special")]
    Special,
    #[serde(rename = "all")]
    All,
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterRules {
    pub must_contain_characters: Vec<CharacterRule>,
    pub number_of_characteristics_to_enforce: i32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserLockout {
    pub enabled: bool,
    pub max_number_of_login_failures: Option<i32>,
    pub lockout_period: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginPasswordPolicies {
    pub character_rules: CharacterRules,
    pub min_length: u16,
    pub reject_dictionary_words: bool,
    pub reject_user_info: bool,
    pub reject_keyboard_patterns: bool,
    pub number_of_archived_passwords: u8,
    pub password_expiration: PasswordExpiration,
    pub user_lockout: UserLockout,
    pub updated_at: DateTime<Utc>,
    pub updated_by: UserInfo,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SharesPasswordPolicies {
    pub character_rules: Option<CharacterRules>,
    pub min_length: Option<i32>,
    pub reject_dictionary_words: Option<bool>,
    pub reject_user_info: Option<bool>,
    pub reject_keyboard_patterns: Option<bool>,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<UserInfo>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionPasswordPolicies {
    pub character_rules: Option<CharacterRules>,
    pub min_length: Option<u16>,
    pub reject_dictionary_words: Option<bool>,
    pub reject_user_info: Option<bool>,
    pub reject_keyboard_patterns: Option<bool>,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<UserInfo>,
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
#[serde(rename_all = "camelCase")]
pub struct PasswordPoliciesConfig {
    pub login_password_policies: Option<LoginPasswordPolicies>,
    pub shares_password_policies: Option<SharesPasswordPolicies>,
    pub encryption_password_policies: Option<EncryptionPasswordPolicies>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub feature_id: u8,
    pub feature_name: String,
    pub is_available: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeaturedOAuthClient {
    pub is_available: bool,
    pub oauth_client_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductPackagesResponse {
    pub product_package_id: u8,
    pub product_package_name: String,
    pub features: Vec<Feature>,
    pub clients: Vec<FeaturedOAuthClient>,
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
pub struct ProductPackageResponseList {
    pub packages: Vec<ProductPackagesResponse>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct S3Tag {
    pub id: Option<u64>,
    pub key: Option<String>,
    pub value: Option<String>,
    pub is_mandatory: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, FromResponse)]
pub struct S3TagList {
    pub items: Option<Vec<S3Tag>>,
}
