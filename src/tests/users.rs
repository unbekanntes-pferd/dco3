#[cfg(test)]
pub mod tests {
    use crate::{
        tests::dracoon::get_connected_client,
        user::UserAuthData,
        users::{
            CreateUserRequest, UpdateUserRequest, UserData, UserItem, UsersFilter, UsersSortBy,
        },
        ListAllParams, SortOrder, Users,
    };

    pub fn assert_user_item(user: &UserItem) {
        assert_eq!(user.id, 1);
        assert_eq!(user.user_name, "string");
        assert_eq!(user.first_name, "string");
        assert_eq!(user.last_name, "string");
        assert_eq!(user.email.as_ref().unwrap(), "string");
        assert!(!user.is_locked);
        assert!(user.is_encryption_enabled.unwrap());
        assert_eq!(user.avatar_uuid, "string");
    }

    fn assert_user_data(user: &UserData) {
        assert_eq!(user.id, 1);
        assert_eq!(user.user_name, "string");
        assert_eq!(user.first_name, "string");
        assert_eq!(user.last_name, "string");
        assert_eq!(user.email.as_ref().unwrap(), "string");
        assert!(!user.is_locked);
        assert!(user.is_encryption_enabled.unwrap());
        assert_eq!(user.avatar_uuid, "string");
        assert_eq!(user.auth_data.method, "basic");
        assert_eq!(user.home_room_id.unwrap(), 2);
        assert!(user.is_mfa_enabled.unwrap());
        assert!(user.is_mfa_enforced.unwrap());
    }

    #[tokio::test]
    async fn test_get_users() {
        let (client, mut mock_server) = get_connected_client().await;

        let users_res = include_str!("./responses/users/users_ok.json");

        let users_mock = mock_server
            .mock("GET", "/api/v4/users?offset=0")
            .with_status(200)
            .with_body(users_res)
            .create();

        let users = client.users.get_users(None, None, None).await.unwrap();

        users_mock.assert();
        assert_eq!(users.range.offset, 0);
        assert_eq!(users.range.limit, 0);
        assert_eq!(users.range.total, 1);
        let user = users.items.first().unwrap();
        assert_user_item(user);
    }

    #[tokio::test]
    async fn test_get_users_with_limit() {
        let (client, mut mock_server) = get_connected_client().await;

        let users_res = include_str!("./responses/users/users_ok.json");

        let users_mock = mock_server
            .mock("GET", "/api/v4/users?limit=100&offset=0")
            .with_status(200)
            .with_body(users_res)
            .create();

        let params = ListAllParams::builder().with_limit(100).build();

        let users = client
            .users
            .get_users(Some(params), None, None)
            .await
            .unwrap();

        users_mock.assert();
        assert_eq!(users.range.offset, 0);
        assert_eq!(users.range.limit, 0);
        assert_eq!(users.range.total, 1);
        let user = users.items.first().unwrap();
        assert_user_item(user);
    }

    #[tokio::test]
    async fn test_get_users_with_offset() {
        let (client, mut mock_server) = get_connected_client().await;

        let users_res = include_str!("./responses/users/users_ok.json");

        let users_mock = mock_server
            .mock("GET", "/api/v4/users?offset=500")
            .with_status(200)
            .with_body(users_res)
            .create();

        let params = ListAllParams::builder().with_offset(500).build();

        let users = client
            .users
            .get_users(Some(params), None, None)
            .await
            .unwrap();

        users_mock.assert();
        assert_eq!(users.range.offset, 0);
        assert_eq!(users.range.limit, 0);
        assert_eq!(users.range.total, 1);
        let user = users.items.first().unwrap();
        assert_user_item(user);
    }

    #[tokio::test]
    async fn test_get_users_with_sort() {
        let (client, mut mock_server) = get_connected_client().await;

        let users_res = include_str!("./responses/users/users_ok.json");

        let users_mock = mock_server
            .mock("GET", "/api/v4/users?offset=0&sort=createdAt%3Aasc")
            .with_status(200)
            .with_body(users_res)
            .create();

        let params = ListAllParams::builder()
            .with_sort(UsersSortBy::CreatedAt(SortOrder::Asc))
            .build();

        let users = client
            .users
            .get_users(Some(params), None, None)
            .await
            .unwrap();

        users_mock.assert();
        assert_eq!(users.range.offset, 0);
        assert_eq!(users.range.limit, 0);
        assert_eq!(users.range.total, 1);
        let user = users.items.first().unwrap();
        assert_user_item(user);
    }

    #[tokio::test]
    async fn test_get_users_with_filter() {
        let (client, mut mock_server) = get_connected_client().await;

        let users_res = include_str!("./responses/users/users_ok.json");

        let users_mock = mock_server
            .mock("GET", "/api/v4/users?offset=0&filter=email%3Acn%3Atest")
            .with_status(200)
            .with_body(users_res)
            .create();

        let params = ListAllParams::builder()
            .with_filter(UsersFilter::email_contains("test"))
            .build();

        let users = client
            .users
            .get_users(Some(params), None, None)
            .await
            .unwrap();

        users_mock.assert();
        assert_eq!(users.range.offset, 0);
        assert_eq!(users.range.limit, 0);
        assert_eq!(users.range.total, 1);
        let user = users.items.first().unwrap();
        assert_user_item(user);
    }

    #[tokio::test]
    async fn test_get_users_with_roles() {
        let (client, mut mock_server) = get_connected_client().await;

        let users_res = include_str!("./responses/users/users_ok.json");

        let users_mock = mock_server
            .mock("GET", "/api/v4/users?offset=0&include_roles=true")
            .with_status(200)
            .with_body(users_res)
            .create();

        let users = client
            .users
            .get_users(None, Some(true), None)
            .await
            .unwrap();

        users_mock.assert();
        assert_eq!(users.range.offset, 0);
        assert_eq!(users.range.limit, 0);
        assert_eq!(users.range.total, 1);
        let user = users.items.first().unwrap();
        assert_user_item(user);
    }

    #[tokio::test]
    async fn test_get_users_with_attributes() {
        let (client, mut mock_server) = get_connected_client().await;

        let users_res = include_str!("./responses/users/users_ok.json");

        let users_mock = mock_server
            .mock("GET", "/api/v4/users?offset=0&include_attributes=true")
            .with_status(200)
            .with_body(users_res)
            .create();

        let users = client
            .users
            .get_users(None, None, Some(true))
            .await
            .unwrap();

        users_mock.assert();
        assert_eq!(users.range.offset, 0);
        assert_eq!(users.range.limit, 0);
        assert_eq!(users.range.total, 1);
        let user = users.items.first().unwrap();
        assert_user_item(user);
    }

    #[tokio::test]
    async fn test_create_user() {
        let (client, mut mock_server) = get_connected_client().await;

        let user_res = include_str!("./responses/users/user_ok.json");

        let user_mock = mock_server
            .mock("POST", "/api/v4/users")
            .with_status(201)
            .with_body(user_res)
            .create();

        let auth = UserAuthData::new_basic(None, None);
        let user_req = CreateUserRequest::builder("test", "test")
            .with_email("test@localhost")
            .with_auth_data(auth)
            .build();

        let user = client.users.create_user(user_req).await.unwrap();

        user_mock.assert();

        assert_user_data(&user);
    }

    #[tokio::test]
    async fn test_get_user() {
        let (client, mut mock_server) = get_connected_client().await;

        let user_res = include_str!("./responses/users/user_ok.json");

        let user_mock = mock_server
            .mock("GET", "/api/v4/users/123")
            .with_status(200)
            .with_body(user_res)
            .create();

        let user = client.users.get_user(123, None).await.unwrap();

        user_mock.assert();

        assert_user_data(&user);
    }

    #[tokio::test]
    async fn test_update_user() {
        let (client, mut mock_server) = get_connected_client().await;
        let user_res = include_str!("./responses/users/user_ok.json");

        let user_mock = mock_server
            .mock("PUT", "/api/v4/users/123")
            .with_status(200)
            .with_body(user_res)
            .create();

        let user_req = UpdateUserRequest::builder()
            .with_email("foo@localhost")
            .build();

        let user = client.users.update_user(123, user_req).await.unwrap();

        user_mock.assert();

        assert_user_data(&user);
    }

    #[tokio::test]
    async fn test_delete_user() {
        let (client, mut mock_server) = get_connected_client().await;
        let user_res = include_str!("./responses/users/user_ok.json");

        let user_mock = mock_server
            .mock("DELETE", "/api/v4/users/123")
            .with_status(200)
            .with_body(user_res)
            .create();

        let res = client.users.delete_user(123).await;

        assert!(res.is_ok());

        user_mock.assert();
    }
}
