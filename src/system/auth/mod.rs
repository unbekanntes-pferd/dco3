use async_trait::async_trait;
use reqwest::header;

use crate::utils::FromResponse;
use crate::{client::Connected, DracoonClientError};

use crate::constants::{
    DRACOON_API_PREFIX, SYSTEM_AUTH_ADS, SYSTEM_AUTH_BASE, SYSTEM_AUTH_OPENID,
    SYSTEM_AUTH_OPENID_IDPS, SYSTEM_BASE, SYSTEM_CONFIG_BASE,
};

mod models;

pub use self::models::*;

#[async_trait]
pub trait AuthenticationMethods {
    async fn get_active_directory_configurations(
        &self,
    ) -> Result<ActiveDirectoryConfigList, DracoonClientError>;

    async fn get_openid_idp_configurations(
        &self,
    ) -> Result<Vec<OpenIdIdpConfig>, DracoonClientError>;
}

#[async_trait]
impl AuthenticationMethods for SystemAuthEndpoint<Connected> {
    async fn get_active_directory_configurations(
        &self,
    ) -> Result<ActiveDirectoryConfigList, DracoonClientError> {
        let url_part =
            format!("{DRACOON_API_PREFIX}/{SYSTEM_BASE}/{SYSTEM_CONFIG_BASE}/{SYSTEM_AUTH_BASE}/{SYSTEM_AUTH_ADS}");
        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        ActiveDirectoryConfigList::from_response(response).await
    }

    async fn get_openid_idp_configurations(
        &self,
    ) -> Result<Vec<OpenIdIdpConfig>, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{SYSTEM_BASE}/{SYSTEM_CONFIG_BASE}/{SYSTEM_AUTH_BASE}/{SYSTEM_AUTH_OPENID}/{SYSTEM_AUTH_OPENID_IDPS}");
        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        Vec::<OpenIdIdpConfig>::from_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use crate::{system::auth::AuthenticationMethods, tests::dracoon::get_connected_client};

    #[tokio::test]
    async fn test_get_ad_configs() {
        let (client, mut mock_server) = get_connected_client().await;

        let response =
            include_str!("../../tests/responses/system/auth/active_directory_list_ok.json");

        let ad_config_mock = mock_server
            .mock("GET", "/api/v4/system/config/auth/ads")
            .with_status(200)
            .with_body(response)
            .with_header("content-type", "application/json")
            .create();

        let ad_configs = client
            .system()
            .auth
            .get_active_directory_configurations()
            .await
            .unwrap();

        ad_config_mock.assert();

        assert_eq!(ad_configs.items.len(), 1);
        let ad_config = &ad_configs.items.first().unwrap();
        assert_eq!(ad_config.id, 1);
        assert_eq!(ad_config.alias, "string");
        assert_eq!(ad_config.server_ip, "string");
        assert_eq!(ad_config.server_port, 65535);
        assert_eq!(ad_config.server_admin_name, "string");
        assert_eq!(ad_config.ldap_users_domain, "string");
        assert_eq!(ad_config.user_filter, "string");
        assert!(ad_config.user_import);
        assert!(ad_config.use_ldaps);
        assert_eq!(ad_config.ad_export_group, "string");
        assert_eq!(ad_config.sds_import_group, Some(2));
        assert_eq!(ad_config.ssl_finger_print, Some("string".to_string()));
    }

    #[tokio::test]
    async fn test_get_openid_configs() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../../tests/responses/system/auth/openid_config_list_ok.json");

        let openid_config_mock = mock_server
            .mock("GET", "/api/v4/system/config/auth/openid/idps")
            .with_status(200)
            .with_body(response)
            .with_header("content-type", "application/json")
            .create();

        let openid_configs = client
            .system()
            .auth
            .get_openid_idp_configurations()
            .await
            .unwrap();

        openid_config_mock.assert();

        assert_eq!(openid_configs.len(), 1);
        let openid_config = &openid_configs.first().unwrap();

        assert_eq!(openid_config.id, 1);
        assert_eq!(openid_config.name.as_ref().unwrap(), "string");
        assert_eq!(openid_config.issuer.as_ref().unwrap(), "string");
        assert_eq!(openid_config.authorization_end_point_url.as_ref().unwrap(), "string");
        assert_eq!(openid_config.token_end_point_url.as_ref().unwrap(), "string");
        assert_eq!(openid_config.user_info_end_point_url.as_ref().unwrap(), "string");
        assert_eq!(openid_config.jwks_end_point_url.as_ref().unwrap(), "string");
        assert_eq!(openid_config.client_id.as_ref().unwrap(), "string");
        assert_eq!(openid_config.client_secret.as_ref().unwrap(), "string");
        assert_eq!(openid_config.redirect_uris.len(), 1);
        assert_eq!(openid_config.redirect_uris.first().as_ref().unwrap(), &"string");
        assert_eq!(openid_config.scopes.as_ref().unwrap().len(), 1);
        assert_eq!(openid_config.scopes.as_ref().unwrap().first().unwrap(), "string");
        assert_eq!(openid_config.mapping_claim.as_ref().unwrap(), "string");
        assert_eq!(openid_config.flow, Some("authorization_code".to_string()));
        assert_eq!(openid_config.pkce_enabled, Some(true));
        assert_eq!(
            openid_config.pkce_challenge_method,
            Some("string".to_string())
        );
        assert_eq!(
            openid_config.fallback_mapping_claim,
            Some("string".to_string())
        );
        assert_eq!(
            openid_config.user_info_source,
            Some("user_info_endpoint".to_string())
        );
        assert_eq!(openid_config.user_import_enabled, Some(true));
        assert_eq!(openid_config.user_import_group, Some(2));
        assert_eq!(openid_config.user_update_enabled, Some(true));
        assert_eq!(
            openid_config.user_management_url,
            Some("string".to_string())
        );
    }
}
