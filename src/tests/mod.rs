mod groups;
mod nodes;
mod rooms;
mod shares;
mod users;
mod user;


#[cfg(test)]
pub mod dracoon {
    use crate::*;
    //use dco3_crypto::DracoonCryptoError;


    pub async fn get_connected_client() -> (Dracoon<Connected>, mockito::ServerGuard) {

        let mut mock_server = mockito::Server::new();
        let base_url = mock_server.url();


        let auth_res = include_str!("../auth/tests/auth_ok.json");

        let auth_mock = mock_server
        .mock("POST", "/oauth/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(auth_res)
        .create();

        let dracoon = Dracoon::builder()
            .with_base_url(base_url)
            .with_client_id("client_id")
            .with_client_secret("client_secret")
            .with_user_agent("test_client")
            .build()
            .unwrap()
            .connect(OAuth2Flow::authorization_code("auth_code"))
            .await
            .unwrap();


        (dracoon, mock_server)

    }

    #[tokio::test]
    async fn test_get_keypair() {
        let (dracoon, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let kp_res = include_str!("./responses/keypair_ok.json");

        let kp_mock = mock_server
        .mock("GET", "/api/v4/user/account/keypair")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(kp_res)
        .create();

        let kp = dracoon.get_keypair(Some("TopSecret1234!".to_string())).await.unwrap();

        kp_mock.assert();

    }

    #[tokio::test]
    async fn test_get_keypair_wrong_secret() {
        let (dracoon, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let kp_res = include_str!("./responses/keypair_ok.json");

        let kp_mock = mock_server
        .mock("GET", "/api/v4/user/account/keypair")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(kp_res)
        .create();

        let kp = dracoon.get_keypair(Some("WrongSecret".to_string())).await;

        kp_mock.assert();
        assert!(kp.is_err());

        // TODO: implement PartialEq for DracoonCryptoError and DracoonClientError 
        // let err = kp.unwrap_err();
        // assert_eq!(err, DracoonClientError::CryptoError(DracoonCryptoError::RsaOperationFailed));
    }

    #[tokio::test]
    async fn test_get_keypair_no_secret() {
        let (dracoon, _mock_server) = get_connected_client().await;

        let kp = dracoon.get_keypair(None).await;

        assert!(kp.is_err());

        // TODO: implement PartialEq for DracoonClientError 
        // let err = kp.unwrap_err();
        // assert_eq!(err, DracoonClientError::MissingEncryptionSecret);

    }

    #[tokio::test]
    async fn test_get_user_info() {
        let (dracoon, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let user_info_res = include_str!("./responses/user_info_ok.json");

        let user_info_mock = mock_server
        .mock("GET", "/api/v4/user/account")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(user_info_res)
        .create();

        let user_info = dracoon.get_user_info().await.unwrap();

        user_info_mock.assert();
        
        assert_eq!(user_info.id, 1);
        assert_eq!(user_info.first_name, "string");
        assert_eq!(user_info.last_name, "string");
        assert_eq!(user_info.user_name, "string");
        assert_eq!(user_info.is_locked, false);
        assert_eq!(user_info.has_manageable_rooms, true);
        assert_eq!(user_info.language, "string");
        assert_eq!(user_info.must_set_email, Some(false));
        assert_eq!(user_info.needs_to_accept_EULA, Some(false));
        assert_eq!(user_info.expire_at, None);
        assert_eq!(user_info.is_encryption_enabled, Some(true));
        assert!(user_info.last_login_success_at.is_some());
        assert!(user_info.last_login_fail_at.is_some());
        assert_eq!(user_info.email, Some("string".to_string()));
        assert_eq!(user_info.phone, Some("string".to_string()));
        assert_eq!(user_info.auth_data.method, "basic");
        
    }
}