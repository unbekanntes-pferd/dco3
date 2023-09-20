use dco3_derive::FromResponse;
use serde::Deserialize;

use crate::nodes::UserInfo;

#[derive(Debug, Deserialize, Clone, FromResponse)]
pub struct GeneralSettingsInfo {
    pub share_password_sms_enabled: bool,
    pub crypto_enabled: bool,
    pub email_notification_button_enabled: bool,
    pub eula_enabled: bool,
    pub weak_password_enabled: bool,
    pub use_s3_storage: bool,
    pub s3_tags_enabled: bool,
    pub home_rooms_active: bool,
    pub home_room_parent_id: Option<u64>,
    pub subscription_plan: Option<u8>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SystemDefaults {
    pub language_default: Option<String>,
    pub download_share_default_expiration_period: Option<u32>,
    pub upload_share_default_expiration_period: Option<u32>,
    pub file_default_expiration_period: Option<u32>,
    pub nonmember_view_default: Option<bool>,
    pub hide_login_input_fields: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InfrastructureProperties {
    pub sms_config_enabled: Option<bool>,
    pub media_server_config_enabled: Option<bool>,
    pub s3_default_region: Option<String>,
    pub s3_enforce_direct_upload: Option<bool>,
    pub is_dracoon_cloud: Option<bool>,
    pub tenant_uuid: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum AlgorithmStatus {
    Required,
    Discouraged,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AlgorithmVersionInfo {
    pub version: String,
    pub description: String,
    pub status: AlgorithmStatus,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AlgorithmVersionInfoList {
    pub file_key_algorithms: Vec<AlgorithmVersionInfo>,
    pub key_pair_algorithms: Vec<AlgorithmVersionInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum MinimumClassification {
    NoPassword,
    Public,
    Internal,
    Confidential,
    StrictlyConfidential,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ShareClassificationPolicies {
    pub classification_requires_share_password: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClassificationPoliciesConfig {
    pub share_classification_policies: Option<ShareClassificationPolicies>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PasswordExpiration {
    pub enabled: bool,
    pub max_password_age: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum CharacterRule {
    Alpha,
    UpperCase,
    LowerCase,
    Numeric,
    Special,
    All,
    None
}

#[derive(Debug, Deserialize, Clone)]
pub struct CharacterRules {
    pub must_contain_characters: Vec<CharacterRule>,
    pub number_of_characteristics_to_enforce: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserLockout {
    pub enabled: bool,
    pub max_number_of_login_failures: Option<i32>,
    pub lockout_period: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoginPasswordPolicies {
    pub character_rules: CharacterRules,
    pub min_length: u16,
    pub reject_dictionary_words: bool,
    pub reject_user_info: bool,
    pub reject_keyboard_patterns: bool,
    pub number_of_archived_passwords: u8,
    pub password_expiration: PasswordExpiration,
    pub user_lockout: UserLockout,
    pub updated_at: String,
    pub updated_by: UserInfo,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SharesPasswordPolicies {
    pub character_rules: Option<CharacterRules>,
    pub min_length: Option<i32>,
    pub reject_dictionary_words: Option<bool>,
    pub reject_user_info: Option<bool>,
    pub reject_keyboard_patterns: Option<bool>,
    pub updated_at: Option<String>,
    pub updated_by: Option<UserInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EncryptionPasswordPolicies {
    pub character_rules: Option<CharacterRules>,
    pub min_length: Option<u16>,
    pub reject_dictionary_words: Option<bool>,
    pub reject_user_info: Option<bool>,
    pub reject_keyboard_patterns: Option<bool>,
    pub updated_at: Option<String>,
    pub updated_by: Option<UserInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PasswordPoliciesConfig {
    pub login_password_policies: Option<LoginPasswordPolicies>,
    pub shares_password_policies: Option<SharesPasswordPolicies>,
    pub encryption_password_policies: Option<EncryptionPasswordPolicies>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Feature {
    pub feature_id: u8,
    pub feature_name: String,
    pub is_available: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FeaturedOAuthClient {
    pub is_available: bool,
    pub oauth_client_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProductPackagesResponse {
    pub product_package_id: u8,
    pub product_package_name: String,
    pub features: Vec<Feature>,
    pub clients: Vec<FeaturedOAuthClient>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProductPackageResponseList {
    pub packages: Vec<ProductPackagesResponse>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct S3Tag {
    pub id: Option<u64>,
    pub key: Option<String>,
    pub value: Option<String>,
    pub is_mandatory: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct S3TagList {
    pub items: Option<Vec<S3Tag>>,
}

