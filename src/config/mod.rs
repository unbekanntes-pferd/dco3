mod models;

use async_trait::async_trait;
use reqwest::header;

use crate::constants::{
    CONFIG_ALGORITHMS, CONFIG_BASE, CONFIG_CLASSIFICATION_POLICIES, CONFIG_DEFAULTS,
    CONFIG_GENERAL, CONFIG_INFRASTRUCTURE, CONFIG_PASSWORD_POLICIES, CONFIG_POLICIES,
    CONFIG_PRODUCT_PACKAGES, CONFIG_PRODUCT_PACKAGES_CURRENT, CONFIG_S3_TAGS, DRACOON_API_PREFIX,
};
use crate::utils::FromResponse;
use crate::{auth::Connected, DracoonClientError};

pub use self::models::*;

#[async_trait]
pub trait Config {
    async fn get_defaults(&self) -> Result<SystemDefaults, DracoonClientError>;

    async fn get_general_settings(&self) -> Result<GeneralSettingsInfo, DracoonClientError>;

    async fn get_infrastructure_properties(
        &self,
    ) -> Result<InfrastructureProperties, DracoonClientError>;

    async fn get_password_policies(&self) -> Result<PasswordPoliciesConfig, DracoonClientError>;

    async fn get_classification_policies(
        &self,
    ) -> Result<ClassificationPoliciesConfig, DracoonClientError>;

    async fn get_algorithms(&self) -> Result<AlgorithmVersionInfoList, DracoonClientError>;

    async fn get_product_packages(&self) -> Result<ProductPackageResponseList, DracoonClientError>;

    async fn get_current_product_package(
        &self,
    ) -> Result<ProductPackageResponseList, DracoonClientError>;

    async fn get_s3_tags(&self) -> Result<S3TagList, DracoonClientError>;
}

#[async_trait]
impl Config for ConfigEndpoint<Connected> {
    async fn get_defaults(&self) -> Result<SystemDefaults, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{CONFIG_BASE}/{CONFIG_DEFAULTS}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        SystemDefaults::from_response(response).await
    }

    async fn get_general_settings(&self) -> Result<GeneralSettingsInfo, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{CONFIG_BASE}/{CONFIG_GENERAL}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        GeneralSettingsInfo::from_response(response).await
    }

    async fn get_infrastructure_properties(
        &self,
    ) -> Result<InfrastructureProperties, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{CONFIG_BASE}/{CONFIG_INFRASTRUCTURE}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        InfrastructureProperties::from_response(response).await
    }

    async fn get_classification_policies(
        &self,
    ) -> Result<ClassificationPoliciesConfig, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{CONFIG_BASE}/{CONFIG_POLICIES}/{CONFIG_CLASSIFICATION_POLICIES}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        ClassificationPoliciesConfig::from_response(response).await
    }

    async fn get_password_policies(&self) -> Result<PasswordPoliciesConfig, DracoonClientError> {
        let url_part = format!(
            "/{DRACOON_API_PREFIX}/{CONFIG_BASE}/{CONFIG_POLICIES}/{CONFIG_PASSWORD_POLICIES}"
        );

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        PasswordPoliciesConfig::from_response(response).await
    }

    async fn get_algorithms(&self) -> Result<AlgorithmVersionInfoList, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{CONFIG_BASE}/{CONFIG_ALGORITHMS}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        AlgorithmVersionInfoList::from_response(response).await
    }

    async fn get_product_packages(&self) -> Result<ProductPackageResponseList, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{CONFIG_BASE}/{CONFIG_PRODUCT_PACKAGES}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        ProductPackageResponseList::from_response(response).await
    }

    async fn get_current_product_package(
        &self,
    ) -> Result<ProductPackageResponseList, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{CONFIG_BASE}/{CONFIG_PRODUCT_PACKAGES}/{CONFIG_PRODUCT_PACKAGES_CURRENT}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        ProductPackageResponseList::from_response(response).await
    }

    async fn get_s3_tags(&self) -> Result<S3TagList, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{CONFIG_BASE}/{CONFIG_S3_TAGS}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        S3TagList::from_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::{
        config::{AlgorithmStatus, CharacterRule, MinimumClassification},
        nodes::UserType,
        tests::dracoon::get_connected_client,
        Config,
    };

    #[tokio::test]
    async fn test_get_defaults() {
        let (client, mut mock_server) = get_connected_client().await;

        let defaults_res = include_str!("../tests/responses/config/defaults_ok.json");

        let defaults_mock = mock_server
            .mock("GET", "/api/v4/config/info/defaults")
            .with_status(200)
            .with_body(defaults_res)
            .with_header("content-type", "application/json")
            .create();

        let defaults = client.config.get_defaults().await.unwrap();

        defaults_mock.assert();

        assert_eq!(defaults.language_default, Some("string".into()));
        assert_eq!(defaults.download_share_default_expiration_period, Some(0));
        assert_eq!(defaults.upload_share_default_expiration_period, Some(0));
        assert_eq!(defaults.file_default_expiration_period, Some(0));
        assert_eq!(defaults.nonmember_viewer_default, Some(true));
    }

    #[tokio::test]
    async fn test_get_general_settings() {
        let (client, mut mock_server) = get_connected_client().await;

        let general_settings_res =
            include_str!("../tests/responses/config/general_settings_ok.json");

        let general_settings_mock = mock_server
            .mock("GET", "/api/v4/config/info/general")
            .with_status(200)
            .with_body(general_settings_res)
            .with_header("content-type", "application/json")
            .create();

        let general_settings = client.config.get_general_settings().await.unwrap();

        general_settings_mock.assert();

        assert!(general_settings.share_password_sms_enabled);
        assert!(general_settings.crypto_enabled);
        assert!(general_settings.email_notification_button_enabled);
        assert!(general_settings.eula_enabled);
        assert!(general_settings.use_s3_storage);
        assert!(general_settings.s3_tags_enabled);
        assert!(general_settings.home_rooms_active);
        assert_eq!(general_settings.home_room_parent_id, Some(2));
        assert_eq!(general_settings.subscription_plan, Some(1));
        assert!(general_settings.auth_token_restrictions.is_some());
        assert_eq!(
            general_settings
                .auth_token_restrictions
                .as_ref()
                .unwrap()
                .restriction_enabled,
            Some(false)
        );
        assert_eq!(
            general_settings
                .auth_token_restrictions
                .as_ref()
                .unwrap()
                .access_token_validity,
            Some(0)
        );
        assert_eq!(
            general_settings
                .auth_token_restrictions
                .as_ref()
                .unwrap()
                .refresh_token_validity,
            Some(0)
        );
    }

    #[tokio::test]
    async fn test_get_infrastructure_properties() {
        let (client, mut mock_server) = get_connected_client().await;

        let infrastructure_properties_res =
            include_str!("../tests/responses/config/infrastructure_properties_ok.json");

        let infrastructure_properties_mock = mock_server
            .mock("GET", "/api/v4/config/info/infrastructure")
            .with_status(200)
            .with_body(infrastructure_properties_res)
            .with_header("content-type", "application/json")
            .create();

        let infrastructure_properties =
            client.config.get_infrastructure_properties().await.unwrap();

        infrastructure_properties_mock.assert();

        assert_eq!(infrastructure_properties.sms_config_enabled, Some(true));
        assert_eq!(
            infrastructure_properties.media_server_config_enabled,
            Some(true)
        );
        assert_eq!(
            infrastructure_properties.s3_default_region,
            Some("string".into())
        );
        assert_eq!(
            infrastructure_properties.s3_enforce_direct_upload,
            Some(true)
        );
        assert_eq!(infrastructure_properties.is_dracoon_cloud, Some(true));
        assert_eq!(infrastructure_properties.tenant_uuid, Some("string".into()));
    }

    #[tokio::test]
    async fn test_get_classification_policies() {
        let (client, mut mock_server) = get_connected_client().await;

        let classification_policies_res =
            include_str!("../tests/responses/config/classification_policies_ok.json");

        let classification_policies_mock = mock_server
            .mock("GET", "/api/v4/config/info/policies/classifications")
            .with_status(200)
            .with_body(classification_policies_res)
            .with_header("content-type", "application/json")
            .create();

        let classification_policies = client.config.get_classification_policies().await.unwrap();

        classification_policies_mock.assert();

        assert!(classification_policies
            .share_classification_policies
            .is_some());
        let share_classification_policies = classification_policies
            .share_classification_policies
            .as_ref()
            .unwrap();
        assert_eq!(
            share_classification_policies.classification_requires_share_password,
            MinimumClassification::NoPassword
        );
    }

    #[tokio::test]
    async fn test_get_password_policies() {
        let (client, mut mock_server) = get_connected_client().await;

        let password_policies_res =
            include_str!("../tests/responses/config/password_policies_ok.json");

        let password_policies_mock = mock_server
            .mock("GET", "/api/v4/config/info/policies/passwords")
            .with_status(200)
            .with_body(password_policies_res)
            .with_header("content-type", "application/json")
            .create();

        let password_policies = client.config.get_password_policies().await.unwrap();

        password_policies_mock.assert();

        assert!(password_policies.login_password_policies.is_some());
        let login_password_policies = password_policies.login_password_policies.as_ref().unwrap();
        assert!(password_policies.shares_password_policies.is_some());
        let shares_password_policies = password_policies.shares_password_policies.as_ref().unwrap();
        assert!(password_policies.encryption_password_policies.is_some());
        let encryption_password_policies = password_policies
            .encryption_password_policies
            .as_ref()
            .unwrap();

        assert_eq!(login_password_policies.min_length, 1024);
        assert!(login_password_policies.reject_dictionary_words);
        assert!(login_password_policies.reject_user_info);
        assert!(login_password_policies.reject_keyboard_patterns);
        assert_eq!(login_password_policies.number_of_archived_passwords, 10);
        assert_eq!(
            login_password_policies.password_expiration.max_password_age,
            Some(0)
        );
        assert!(login_password_policies.password_expiration.enabled);
        assert!(login_password_policies.user_lockout.enabled);
        assert_eq!(
            login_password_policies
                .user_lockout
                .max_number_of_login_failures,
            Some(0)
        );
        assert_eq!(login_password_policies.user_lockout.lockout_period, Some(0));
        assert_eq!(
            login_password_policies
                .character_rules
                .must_contain_characters
                .len(),
            1
        );
        let char_rule = login_password_policies
            .character_rules
            .must_contain_characters
            .first()
            .unwrap();
        assert_eq!(char_rule, &CharacterRule::Alpha);
        assert_eq!(
            login_password_policies
                .character_rules
                .number_of_characteristics_to_enforce,
            1
        );
        assert_eq!(
            login_password_policies.updated_at,
            DateTime::parse_from_rfc3339("2023-01-01T00:00:00.000Z").unwrap()
        );
        let updated_by = &login_password_policies.updated_by;
        assert_eq!(updated_by.id, 1);
        assert_eq!(updated_by.first_name, Some("string".into()));
        assert_eq!(updated_by.last_name, Some("string".into()));
        assert_eq!(updated_by.user_name, Some("string".into()));
        assert_eq!(updated_by.email, Some("string".into()));
        assert_eq!(updated_by.avatar_uuid, "string");
        assert_eq!(updated_by.user_type, UserType::Internal);

        assert_eq!(shares_password_policies.reject_dictionary_words, Some(true));
        assert_eq!(shares_password_policies.reject_user_info, Some(true));
        assert_eq!(
            shares_password_policies.reject_keyboard_patterns,
            Some(true)
        );
        assert_eq!(shares_password_policies.min_length, Some(1024));
        assert_eq!(
            shares_password_policies.updated_at.unwrap(),
            DateTime::parse_from_rfc3339("2023-01-01T00:00:00.000Z").unwrap()
        );
        let updated_by = shares_password_policies.updated_by.as_ref().unwrap();
        assert_eq!(updated_by.id, 1);
        assert_eq!(updated_by.first_name, Some("string".into()));
        assert_eq!(updated_by.last_name, Some("string".into()));
        assert_eq!(updated_by.user_name, Some("string".into()));
        assert_eq!(updated_by.email, Some("string".into()));
        assert_eq!(updated_by.avatar_uuid, "string");
        assert_eq!(updated_by.user_type, UserType::Internal);

        assert_eq!(encryption_password_policies.reject_dictionary_words, None);
        assert_eq!(encryption_password_policies.reject_user_info, Some(true));
        assert_eq!(
            encryption_password_policies.reject_keyboard_patterns,
            Some(true)
        );
        assert_eq!(encryption_password_policies.min_length, Some(1024));
        assert_eq!(
            encryption_password_policies.updated_at.unwrap(),
            DateTime::parse_from_rfc3339("2023-01-01T00:00:00.000Z").unwrap()
        );
        let updated_by = encryption_password_policies.updated_by.as_ref().unwrap();
        assert_eq!(updated_by.id, 1);
        assert_eq!(updated_by.first_name, Some("string".into()));
        assert_eq!(updated_by.last_name, Some("string".into()));
        assert_eq!(updated_by.user_name, Some("string".into()));
        assert_eq!(updated_by.email, Some("string".into()));
        assert_eq!(updated_by.avatar_uuid, "string");
        assert_eq!(updated_by.user_type, UserType::Internal);
    }

    #[tokio::test]
    async fn test_get_algorithms() {
        let (client, mut mock_server) = get_connected_client().await;

        let algorithms_res = include_str!("../tests/responses/config/algorithms_ok.json");

        let algorithms_mock = mock_server
            .mock("GET", "/api/v4/config/info/algorithms")
            .with_status(200)
            .with_body(algorithms_res)
            .with_header("content-type", "application/json")
            .create();

        let algorithms = client.config.get_algorithms().await.unwrap();

        algorithms_mock.assert();

        assert_eq!(algorithms.file_key_algorithms.len(), 1);
        let file_key_algorithm = algorithms.file_key_algorithms.first().unwrap();
        assert_eq!(file_key_algorithm.version, "string");
        assert_eq!(file_key_algorithm.description, "string");
        assert_eq!(file_key_algorithm.status, AlgorithmStatus::Required);
        assert_eq!(algorithms.key_pair_algorithms.len(), 1);
        let key_pair_algorithm = algorithms.key_pair_algorithms.first().unwrap();
        assert_eq!(key_pair_algorithm.version, "string");
        assert_eq!(key_pair_algorithm.description, "string");
        assert_eq!(key_pair_algorithm.status, AlgorithmStatus::Required);
    }

    #[tokio::test]
    async fn test_get_product_packages() {
        let (client, mut mock_server) = get_connected_client().await;

        let product_packages_res =
            include_str!("../tests/responses/config/product_packages_ok.json");

        let product_packages_mock = mock_server
            .mock("GET", "/api/v4/config/info/product_packages")
            .with_status(200)
            .with_body(product_packages_res)
            .with_header("content-type", "application/json")
            .create();

        let product_packages = client.config.get_product_packages().await.unwrap();

        product_packages_mock.assert();

        assert_eq!(product_packages.packages.len(), 1);
        let product_package = product_packages.packages.first().unwrap();
        assert_eq!(product_package.product_package_id, 0);
        assert_eq!(product_package.product_package_name, "string");
        assert_eq!(product_package.features.len(), 1);
        let feature = product_package.features.first().unwrap();
        assert_eq!(feature.feature_id, 0);
        assert_eq!(feature.feature_name, "string");
        assert!(feature.is_available);
        assert_eq!(product_package.clients.len(), 1);
        let client = product_package.clients.first().unwrap();
        assert_eq!(client.oauth_client_name, Some("string".into()));
        assert!(client.is_available);
    }

    #[tokio::test]
    async fn test_get_current_product_package() {
        let (client, mut mock_server) = get_connected_client().await;

        let product_packages_res =
            include_str!("../tests/responses/config/product_packages_ok.json");

        let product_packages_mock = mock_server
            .mock("GET", "/api/v4/config/info/product_packages/current")
            .with_status(200)
            .with_body(product_packages_res)
            .with_header("content-type", "application/json")
            .create();

        let product_packages = client.config.get_current_product_package().await.unwrap();

        product_packages_mock.assert();

        assert_eq!(product_packages.packages.len(), 1);
        let product_package = product_packages.packages.first().unwrap();
        assert_eq!(product_package.product_package_id, 0);
        assert_eq!(product_package.product_package_name, "string");
        assert_eq!(product_package.features.len(), 1);
        let feature = product_package.features.first().unwrap();
        assert_eq!(feature.feature_id, 0);
        assert_eq!(feature.feature_name, "string");
        assert!(feature.is_available);
        assert_eq!(product_package.clients.len(), 1);
        let client = product_package.clients.first().unwrap();
        assert_eq!(client.oauth_client_name, Some("string".into()));
        assert!(client.is_available);
    }

    #[tokio::test]
    async fn test_get_s3_tags() {
        let (client, mut mock_server) = get_connected_client().await;

        let s3_tags_res = include_str!("../tests/responses/config/s3_tags_ok.json");

        let s3_tags_mock = mock_server
            .mock("GET", "/api/v4/config/info/s3_tags")
            .with_status(200)
            .with_body(s3_tags_res)
            .with_header("content-type", "application/json")
            .create();

        let s3_tags = client.config.get_s3_tags().await.unwrap();

        s3_tags_mock.assert();

        assert_eq!(s3_tags.items.as_ref().unwrap().len(), 1);
        let s3_tag = s3_tags.items.as_ref().unwrap().first().unwrap();

        assert_eq!(s3_tag.id, Some(0));
        assert_eq!(s3_tag.key, Some("string".into()));
        assert_eq!(s3_tag.value, Some("string".into()));
        assert_eq!(s3_tag.is_mandatory, Some(false));
    }
}
