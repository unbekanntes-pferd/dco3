#[cfg(test)]
pub mod tests {
    use crate::{
        nodes::UserType,
        roles::{RoleGroup, RoleUser, Roles},
        tests::dracoon::get_connected_client,
    };

    pub fn assert_role_group(role_group: &RoleGroup) {
        assert_eq!(role_group.id, 1);
        assert_eq!(role_group.is_member, true);
        assert_eq!(role_group.name, "group name");
    }

    pub fn assert_role_user(role_user: &RoleUser) {
        assert_eq!(role_user.id, Some(1));
        assert_eq!(role_user.display_name, Some("display name".to_string()));
        assert!(role_user.is_member);

        let user_info = role_user.user_info.clone();

        assert_eq!(user_info.id, 1);
        assert_eq!(user_info.user_name, Some("user name".to_string()));
        assert_eq!(user_info.first_name, Some("first name".to_string()));
        assert_eq!(user_info.last_name, Some("last name".to_string()));
        assert_eq!(user_info.email, Some("email".to_string()));
        assert_eq!(user_info.avatar_uuid, "avatar uuid");
        assert_eq!(user_info.user_type, UserType::Internal);
    }

    #[tokio::test]
    async fn get_roles() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let roles_res = include_str!("./responses/roles/roles_ok.json");

        let roles_mock = mock_server
            .mock("GET", "/api/v4/roles")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(roles_res)
            .create();

        let roles = dracoon.roles.get_roles().await.unwrap();

        roles_mock.assert();

        assert_eq!(roles.items.len(), 1);

        let role = roles.items.first().unwrap();

        assert_eq!(role.id, 1);
        assert_eq!(role.name, "role name");
        assert_eq!(role.description, "role description");

        let role_items = role.items.clone().unwrap();
        assert_eq!(role_items.len(), 1);
        let role_item = role_items.first().unwrap();
        assert_eq!(role_item.id, 1);
        assert_eq!(role_item.name, "item name");
        assert_eq!(role_item.description, "item description");
    }

    #[tokio::test]
    async fn get_all_groups_with_role() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let groups_res = include_str!("./responses/roles/role_group_list_ok.json");

        let groups_mock = mock_server
            .mock("GET", "/api/v4/roles/1/groups")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(groups_res)
            .create();

        let groups = dracoon
            .roles
            .get_all_groups_with_role(1, None)
            .await
            .unwrap();

        groups_mock.assert();

        assert_eq!(groups.items.len(), 1);

        let role_group = groups.items.first().unwrap();
        assert_role_group(role_group)
    }

    #[tokio::test]
    async fn assign_role_to_groups() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let groups_res = include_str!("./responses/roles/role_group_list_ok.json");

        let groups_mock = mock_server
            .mock("POST", "/api/v4/roles/1/groups")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(groups_res)
            .create();

        let groups = dracoon
            .roles
            .assign_role_to_groups(1, vec![1, 2, 3].into())
            .await
            .unwrap();

        groups_mock.assert();

        assert_eq!(groups.items.len(), 1);

        let role_group = groups.items.first().unwrap();

        assert_role_group(role_group)
    }

    #[tokio::test]
    async fn revoke_role_from_groups() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let groups_res = include_str!("./responses/roles/role_group_list_ok.json");

        let groups_mock = mock_server
            .mock("DELETE", "/api/v4/roles/1/groups")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(groups_res)
            .create();

        let groups = dracoon
            .roles
            .revoke_role_from_groups(1, vec![1, 2, 3].into())
            .await
            .unwrap();

        groups_mock.assert();

        assert_eq!(groups.items.len(), 1);

        let role_group = groups.items.first().unwrap();
        assert_role_group(role_group)
    }

    #[tokio::test]
    async fn get_all_users_with_role() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let users_res = include_str!("./responses/roles/role_user_list_ok.json");

        let users_mock = mock_server
            .mock("GET", "/api/v4/roles/1/users")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(users_res)
            .create();

        let res = dracoon
            .roles
            .get_all_users_with_role(1, None)
            .await
            .unwrap();

        users_mock.assert();

        assert_eq!(res.items.len(), 1);

        let role_user = res.items.first().unwrap();

        assert_role_user(role_user)
    }

    #[tokio::test]
    async fn assign_role_to_users() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let users_res = include_str!("./responses/roles/role_user_list_ok.json");

        let users_mock = mock_server
            .mock("POST", "/api/v4/roles/1/users")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(users_res)
            .create();

        let res = dracoon
            .roles
            .assign_role_to_users(1, vec![1, 2, 3].into())
            .await
            .unwrap();

        users_mock.assert();

        assert_eq!(res.items.len(), 1);

        let role_user = res.items.first().unwrap();

        assert_role_user(role_user)
    }

    #[tokio::test]
    async fn revoke_role_from_users() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let users_res = include_str!("./responses/roles/role_user_list_ok.json");

        let users_mock = mock_server
            .mock("DELETE", "/api/v4/roles/1/users")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(users_res)
            .create();

        let res = dracoon
            .roles
            .revoke_role_from_users(1, vec![1, 2, 3].into())
            .await
            .unwrap();

        users_mock.assert();

        assert_eq!(res.items.len(), 1);

        let role_user = res.items.first().unwrap();

        assert_role_user(role_user)
    }
}
