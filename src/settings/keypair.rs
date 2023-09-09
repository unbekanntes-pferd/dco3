use async_trait::async_trait;
use dco3_crypto::{
    DracoonCrypto, DracoonRSACrypto, PlainUserKeyPairContainer, UserKeyPairContainer,
};
use reqwest::header;

use crate::{
    auth::Connected,
    constants::{
        DRACOON_API_PREFIX, FILES_BASE, FILES_KEYS, MISSING_FILE_KEYS, NODES_BASE, SETTINGS_BASE,
        SETTINGS_KEYPAIR,
    },
    nodes::{MissingKeysResponse, UserFileKeySetBatchRequest, UseKey},
    utils::FromResponse,
    Dracoon, DracoonClientError, ListAllParams,
};

use super::RescueKeyPair;

const MISSING_KEYS_LIMIT: u64 = 100;

#[async_trait]
impl RescueKeyPair for Dracoon<Connected> {
    async fn distribute_missing_keys(
        &self,
        rescue_key_secret: &str,
        room_id: Option<u64>,
        file_id: Option<u64>,
        user_id: Option<u64>,
    ) -> Result<u64, DracoonClientError> {
        let keypair = self.get_system_rescue_keypair(rescue_key_secret).await?;

        let missing_keys = self
            .get_missing_file_keys(room_id, file_id, user_id, None)
            .await?;

        let remaining_keys = if missing_keys.range.is_none() {
            0
        } else {
            missing_keys.range.as_ref().unwrap().total
        };

        let key_reqs = UserFileKeySetBatchRequest::try_new_from_missing_keys(missing_keys, &keypair)?;

        if !key_reqs.is_empty() {
            self.set_file_keys(key_reqs).await?;
        }

        Ok(remaining_keys)
    }
}

#[async_trait]
trait RescueKeypairInternal {
    async fn get_missing_file_keys(
        &self,
        room_id: Option<u64>,
        file_id: Option<u64>,
        user_id: Option<u64>,
        params: Option<ListAllParams>,
    ) -> Result<MissingKeysResponse, DracoonClientError>;

    async fn set_file_keys(
        &self,
        req: UserFileKeySetBatchRequest,
    ) -> Result<(), DracoonClientError>;

    async fn get_system_rescue_keypair(
        &self,
        secret: &str,
    ) -> Result<PlainUserKeyPairContainer, DracoonClientError>;
}

#[async_trait]
impl RescueKeypairInternal for Dracoon<Connected> {
    async fn get_missing_file_keys(
        &self,
        room_id: Option<u64>,
        file_id: Option<u64>,
        user_id: Option<u64>,
        params: Option<ListAllParams>,
    ) -> Result<MissingKeysResponse, DracoonClientError> {
        let params = params.unwrap_or_default();
        let url_part = format!("{DRACOON_API_PREFIX}/{NODES_BASE}/{MISSING_FILE_KEYS}");

        let limit = params.limit.unwrap_or(100);

        let mut api_url = self.build_api_url(&url_part);

        let sorts = params.sort_to_string();

        let rescue_key: String = UseKey::SystemRescueKey.into();

        api_url
            .query_pairs_mut()
            .extend_pairs(Some(("use_key", rescue_key)))
            .extend_pairs(Some(("limit", limit.to_string())))
            .extend_pairs(params.offset.map(|v| ("offset", v.to_string())))
            .extend_pairs(params.sort.map(|_| ("sort", sorts)))
            .extend_pairs(user_id.map(|id| ("user_id", id.to_string())))
            .extend_pairs(room_id.map(|id| ("room_id", id.to_string())))
            .extend_pairs(file_id.map(|id| ("file_id", id.to_string())))
            .finish();

        let response = self
            .client
            .http
            .get(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .send()
            .await?;

        MissingKeysResponse::from_response(response).await
    }

    async fn set_file_keys(
        &self,
        req: UserFileKeySetBatchRequest,
    ) -> Result<(), DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{NODES_BASE}/{FILES_BASE}/{FILES_KEYS}");

        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .post(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .json(&req)
            .send()
            .await?;

        if response.status().is_server_error() || response.status().is_client_error() {
            return Err(DracoonClientError::from_response(response)
                .await
                .expect("Could not parse error response"));
        }

        Ok(())
    }

    async fn get_system_rescue_keypair(
        &self,
        secret: &str,
    ) -> Result<PlainUserKeyPairContainer, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{SETTINGS_BASE}/{SETTINGS_KEYPAIR}",);

        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .get(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .send()
            .await?;

        let keypair = UserKeyPairContainer::from_response(response).await?;

        let keypair = DracoonCrypto::decrypt_private_key(secret, keypair)?;

        Ok(keypair)
    }
}

#[cfg(test)]
mod tests {
    use dco3_crypto::{
        DracoonCryptoError, FileKeyVersion,
        UserKeyPairVersion,
    };

    use crate::{
        settings::{keypair::RescueKeypairInternal, RescueKeyPair},
        tests::dracoon::get_connected_client,
        DracoonClientError,
    };

    #[tokio::test]
    async fn test_get_missing_file_keys() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/nodes/missing_file_keys_ok.json");

        let missing_keys_mock = mock_server
            .mock("GET", "/api/v4/nodes/missingFileKeys?use_key=system_rescue_key&limit=100&offset=0")
            .with_body(response)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let missing_keys = client
            .get_missing_file_keys(None, None, None, None)
            .await
            .unwrap();

        missing_keys_mock.assert();

        assert_eq!(missing_keys.range.unwrap().total, 1);
        assert_eq!(missing_keys.items.len(), 1);
        assert_eq!(missing_keys.users.len(), 1);
        assert_eq!(missing_keys.files.len(), 1);

        let item = missing_keys.items.first().unwrap();
        let user = missing_keys.users.first().unwrap();
        let file = missing_keys.files.first().unwrap();

        assert_eq!(item.file_id, 3);
        assert_eq!(item.user_id, 2);
        assert_eq!(user.id, 2);
        assert_eq!(file.id, 3);
        assert_eq!(
            file.file_key_container.version,
            FileKeyVersion::RSA4096_AES256GCM
        );
        assert_eq!(file.file_key_container.key, "string");
        assert_eq!(file.file_key_container.iv, "string");
        assert_eq!(file.file_key_container.tag.as_ref().unwrap(), "string");
        assert_eq!(
            user.public_key_container.version,
            UserKeyPairVersion::RSA4096
        );

        assert_eq!(item.user_id, user.id);
        assert_eq!(item.file_id, file.id);
    }

    #[ignore = "todo - not implemented yet"]
    #[tokio::test]
    async fn test_set_file_keys() {
        todo!()
    }

    #[tokio::test]
    async fn test_get_system_rescue_keypair() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/keypair_ok.json");

        let keypair_mock = mock_server
            .mock("GET", "/api/v4/settings/keypair")
            .with_body(response)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let keypair = client
            .get_system_rescue_keypair("TopSecret1234!")
            .await
            .unwrap();

        keypair_mock.assert();
    }


    #[tokio::test]
    async fn test_distribute_missing_keys() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/nodes/missing_file_keys_ok.json");
        let keypair_response = include_str!("../tests/responses/keypair_ok.json");

        let missing_keys_mock = mock_server
            .mock("GET", "/api/v4/nodes/missingFileKeys?use_key=system_rescue_key&limit=100&offset=0")
            .with_body(response)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let keypair_mock = mock_server
            .mock("GET", "/api/v4/settings/keypair")
            .with_body(keypair_response)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let res = client
            .distribute_missing_keys("TopSecret1234!", None, None, None)
            .await;

        assert!(res.is_ok());

        missing_keys_mock.assert();
        keypair_mock.assert();
    }

    #[tokio::test]
    async fn test_distribute_missing_keys_wrong_secret() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/nodes/missing_file_keys_ok.json");
        let keypair_response = include_str!("../tests/responses/keypair_ok.json");

        let missing_keys_mock = mock_server
            .mock("GET", "/api/v4/nodes/missingFileKeys?use_key=system_rescue_key&limit=100&offset=0")
            .with_body(response)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let keypair_mock = mock_server
            .mock("GET", "/api/v4/settings/keypair")
            .with_body(keypair_response)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let res = client
            .distribute_missing_keys("wrongsecret", None, None, None)
            .await;

        assert!(res.is_err());

        keypair_mock.assert();

        assert_eq!(
            res.unwrap_err(),
            DracoonClientError::CryptoError(DracoonCryptoError::RsaOperationFailed)
        );
    }

    #[tokio::test]
    async fn test_distribute_missing_keys_with_room_id() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/nodes/missing_file_keys_ok.json");
        let keypair_response = include_str!("../tests/responses/keypair_ok.json");

        let missing_keys_mock = mock_server
            .mock(
                "GET",
                "/api/v4/nodes/missingFileKeys?use_key=system_rescue_key&limit=100&offset=0&room_id=1",
            )
            .with_body(response)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let keypair_mock = mock_server
            .mock("GET", "/api/v4/settings/keypair")
            .with_body(keypair_response)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let res = client
            .distribute_missing_keys("TopSecret1234!", Some(1), None, None)
            .await;

        assert!(res.is_ok());

        missing_keys_mock.assert();
        keypair_mock.assert();
    }

    #[tokio::test]
    async fn test_distribute_missing_keys_with_file_id() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/nodes/missing_file_keys_ok.json");
        let keypair_response = include_str!("../tests/responses/keypair_ok.json");

        let missing_keys_mock = mock_server
            .mock(
                "GET",
                "/api/v4/nodes/missingFileKeys?use_key=system_rescue_key&limit=100&offset=0&file_id=3",
            )
            .with_body(response)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let keypair_mock = mock_server
            .mock("GET", "/api/v4/settings/keypair")
            .with_body(keypair_response)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let res = client
            .distribute_missing_keys("TopSecret1234!", None, Some(3), None)
            .await;

        assert!(res.is_ok());

        missing_keys_mock.assert();
        keypair_mock.assert();
    }

    #[tokio::test]
    async fn test_distribute_missing_keys_with_user_id() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/nodes/missing_file_keys_ok.json");
        let keypair_response = include_str!("../tests/responses/keypair_ok.json");

        let missing_keys_mock = mock_server
            .mock(
                "GET",
                "/api/v4/nodes/missingFileKeys?use_key=system_rescue_key&limit=100&offset=0&user_id=2",
            )
            .with_body(response)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let keypair_mock = mock_server
            .mock("GET", "/api/v4/settings/keypair")
            .with_body(keypair_response)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let res = client
            .distribute_missing_keys("TopSecret1234!", None, None, Some(2))
            .await;

        assert!(res.is_ok());

        missing_keys_mock.assert();
        keypair_mock.assert();
    }
}
