#[cfg(test)]
mod tests {
    use dco3_crypto::{DracoonCrypto, DracoonRSACrypto, UserKeyPairContainer};

    use crate::{
        tests::dracoon::{get_connected_client, assert_user_account}, user::UpdateUserAccountRequest, User,
        UserAccountKeypairs,
    };

    #[tokio::test]
    async fn test_get_keypair() {
        let (client, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let keypair_res = include_str!("../tests/responses/keypair_ok.json");

        let kp_from_json = serde_json::from_str::<UserKeyPairContainer>(keypair_res).unwrap();
        let plain_kp_compare =
            DracoonCrypto::decrypt_private_key("TopSecret1234!", kp_from_json).unwrap();

        let keypair_mock = mock_server
            .mock("GET", "/api/v4/user/account/keypair")
            .with_status(200)
            .with_body(keypair_res)
            .with_header("content-type", "application/json")
            .create();

        let keypair = client.get_user_keypair("TopSecret1234!").await.unwrap();

        keypair_mock.assert();

        assert_eq!(
            keypair.private_key_container.private_key,
            plain_kp_compare.private_key_container.private_key
        );
        assert_eq!(
            keypair.public_key_container.public_key,
            plain_kp_compare.public_key_container.public_key
        );
        assert_eq!(
            keypair.private_key_container.version,
            plain_kp_compare.private_key_container.version
        );
        assert_eq!(
            keypair.public_key_container.version,
            plain_kp_compare.public_key_container.version
        );
    }

    #[tokio::test]
    async fn test_set_keypair() {
        let (client, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let keypair_mock = mock_server
            .mock("POST", "/api/v4/user/account/keypair")
            .with_status(204)
            .create();

        let res = client.set_user_keypair("TopSecret1234!").await;

        keypair_mock.assert();

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_delete_keypair() {
        let (client, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let keypair_mock = mock_server
            .mock("DELETE", "/api/v4/user/account/keypair")
            .with_status(204)
            .create();

        let res = client.delete_user_keypair().await;

        keypair_mock.assert();

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_account() {
        let (client, mut mock_server) = get_connected_client().await;

        let account_res = include_str!("../tests/responses/user_info_ok.json");
        let user_account_mock = mock_server
            .mock("GET", "/api/v4/user/account")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(account_res)
            .create();

        let user_account = client.get_user_account().await.unwrap();

        user_account_mock.assert();

        assert_user_account(&user_account);


    }

    #[tokio::test]
    async fn test_update_user_account() {
        let (client, mut mock_server) = get_connected_client().await;

        let account_res = include_str!("../tests/responses/user_info_ok.json");

        let user_account_mock = mock_server
            .mock("PUT", "/api/v4/user/account")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(account_res)
            .create();

        let update = UpdateUserAccountRequest::builder()
            .with_first_name("test")
            .with_last_name("test")
            .with_phone("test")
            .with_email("test@localhost")
            .build();

        let user_account = client.update_user_account(update).await.unwrap();

        user_account_mock.assert();

        assert_user_account(&user_account);
    }
}
