#[cfg(test)]
mod tests {
    use dco3_crypto::{DracoonCrypto, DracoonRSACrypto, UserKeyPairContainer};

    use crate::{
        tests::dracoon::{assert_user_account, get_connected_client},
        user::UpdateUserAccountRequest,
        User, UserAccountKeyPairs,
    };

    #[tokio::test]
    async fn test_get_keypair() {
        let (client, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let keypair_res = include_str!("../tests/responses/keypair_ok.json");

        let kp_from_json = serde_json::from_str::<UserKeyPairContainer>(keypair_res).unwrap();
        let plain_kp_compare =
            DracoonCrypto::decrypt_keypair("TopSecret1234!", kp_from_json).unwrap();

        let keypair_mock = mock_server
            .mock("GET", "/api/v4/user/account/keypair")
            .with_status(200)
            .with_body(keypair_res)
            .with_header("content-type", "application/json")
            .create();

        let keypair = client
            .user()
            .get_user_keypair("TopSecret1234!")
            .await
            .unwrap();

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

        let res = client.user().set_user_keypair("TopSecret1234!").await;

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

        let res = client.user().delete_user_keypair().await;

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

        let user_account = client.user().get_user_account().await.unwrap();

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

        let user_account = client.user().update_user_account(update).await.unwrap();

        user_account_mock.assert();

        assert_user_account(&user_account);
    }

    #[tokio::test]
    async fn test_get_customer_info() {
        let (client, mut mock_server) = get_connected_client().await;

        let customer_res = include_str!("../tests/responses/customer_info_ok.json");

        let customer_mock = mock_server
            .mock("GET", "/api/v4/user/account/customer")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(customer_res)
            .create();

        let customer = client.user().get_customer_info().await.unwrap();

        customer_mock.assert();

        assert_eq!(customer.name, "string");
        assert_eq!(customer.id, 1);
        assert_eq!(customer.is_provider_customer, true);
        assert_eq!(customer.accounts_limit, 100);
        assert_eq!(customer.space_limit, 100);
        assert_eq!(customer.space_used, 10);
        assert_eq!(customer.accounts_used, 10);
        assert_eq!(customer.cnt_guest_user.unwrap(), 1);
        assert_eq!(customer.cnt_internal_user.unwrap(), 9);
        assert_eq!(customer.customer_encryption_enabled, true);
    }
}
