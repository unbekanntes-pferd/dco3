mod groups;
pub mod nodes;
mod rooms;
mod shares;
mod users;
mod user;
mod provisioning;


#[cfg(test)]
pub mod dracoon {
    use crate::*;
    //use dco3_crypto::DracoonCryptoError;

    pub fn assert_user_account(user_account: &UserAccount) {

        assert_eq!(user_account.id, 1);
        assert_eq!(user_account.first_name, "string");
        assert_eq!(user_account.last_name, "string");
        assert_eq!(user_account.user_name, "string");
        assert_eq!(user_account.is_locked, false);
        assert_eq!(user_account.has_manageable_rooms, true);
        assert_eq!(user_account.language, "string");
        assert_eq!(user_account.must_set_email, Some(false));
        assert_eq!(user_account.needs_to_accept_EULA, Some(false));
        assert_eq!(user_account.expire_at, None);
        assert_eq!(user_account.is_encryption_enabled, Some(true));
        assert!(user_account.last_login_success_at.is_some());
        assert!(user_account.last_login_fail_at.is_some());
        assert_eq!(user_account.email, Some("string".to_string()));
        assert_eq!(user_account.phone, Some("string".to_string()));
        assert_eq!(user_account.auth_data.method, "basic");

    }


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

        assert_user_account(&user_info);
        
    }

    #[tokio::test]
    async fn test_get_provisioning_token() {
        let client = Dracoon::builder()
                 .with_base_url("https://dracoon.team")
                 .with_provisioning_token("token")
                 .build_provisioning()
                 .unwrap();

        let token = client.get_service_token();

        assert_eq!(token, "token");
 
    }
}