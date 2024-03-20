#[cfg(test)]
mod tests {
    use dco3_crypto::UserKeyPairVersion;

    use crate::{
        nodes::{
            ConfigRoomRequest, CreateRoomRequest, EncryptRoomRequest, NodePermissions, RoomUser,
            RoomUsersAddBatchRequestItem, UpdateRoomRequest, UserType, RoomGroup, GroupMemberAcceptance,
            RoomGroupsAddBatchRequestItem, PoliciesRoomRequest
        },
        tests::{dracoon::get_connected_client, nodes::tests::assert_node},
        ListAllParams, Rooms,
    };

    fn assert_room_user(room_user: &RoomUser) {
        assert!(room_user.is_granted);
        assert!(room_user.permissions.as_ref().unwrap().manage);
        assert!(room_user.permissions.as_ref().unwrap().read);
        assert!(room_user.permissions.as_ref().unwrap().create);
        assert!(room_user.permissions.as_ref().unwrap().change);
        assert!(room_user.permissions.as_ref().unwrap().delete);
        assert!(
            room_user
                .permissions
                .as_ref()
                .unwrap()
                .manage_download_share
        );
        assert!(room_user.permissions.as_ref().unwrap().manage_upload_share);
        assert!(room_user.permissions.as_ref().unwrap().read_recycle_bin);
        assert!(room_user.permissions.as_ref().unwrap().restore_recycle_bin);
        assert!(room_user.permissions.as_ref().unwrap().delete_recycle_bin);

        assert_eq!(
            room_user.public_key_container.as_ref().unwrap().version,
            UserKeyPairVersion::RSA4096
        );
        assert_eq!(
            room_user
                .public_key_container
                .as_ref()
                .unwrap()
                .created_by
                .unwrap(),
            1
        );
        assert_eq!(
            room_user
                .public_key_container
                .as_ref()
                .unwrap()
                .created_at
                .as_ref()
                .unwrap(),
            "2020-01-00T00:00:00Z"
        );

        assert_eq!(room_user.user_info.id, 3);
        assert_eq!(room_user.user_info.user_type, UserType::Internal);
        assert_eq!(room_user.user_info.user_name.as_ref().unwrap(), "string");
        assert_eq!(room_user.user_info.first_name.as_ref().unwrap(), "string");
        assert_eq!(room_user.user_info.last_name.as_ref().unwrap(), "string");
        assert_eq!(room_user.user_info.email.as_ref().unwrap(), "string");
        assert_eq!(room_user.user_info.avatar_uuid, "string");
    }

    fn assert_room_group(room_group: &RoomGroup) {
        assert_eq!(room_group.id, 3);
        assert_eq!(room_group.name, "string");
        assert_eq!(room_group.new_group_member_acceptance.as_ref().unwrap(), &GroupMemberAcceptance::AutoAllow);
        assert!(room_group.is_granted);
        assert!(room_group.permissions.as_ref().unwrap().manage);
        assert!(room_group.permissions.as_ref().unwrap().read);
        assert!(room_group.permissions.as_ref().unwrap().create);
        assert!(room_group.permissions.as_ref().unwrap().change);
        assert!(room_group.permissions.as_ref().unwrap().delete);
        assert!(
            room_group
                .permissions
                .as_ref()
                .unwrap()
                .manage_download_share
        );
        assert!(room_group.permissions.as_ref().unwrap().manage_upload_share);
        assert!(room_group.permissions.as_ref().unwrap().read_recycle_bin);
        assert!(room_group.permissions.as_ref().unwrap().restore_recycle_bin);
        assert!(room_group.permissions.as_ref().unwrap().delete_recycle_bin);

    }

    #[tokio::test]
    async fn test_create_room() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_res = include_str!("../tests/responses/nodes/node_ok.json");

        let room_mock = mock_server
            .mock("POST", "/api/v4/nodes/rooms")
            .with_status(201)
            .with_body(room_res)
            .with_header("content-type", "application/json")
            .create();

        let room_req = CreateRoomRequest::builder("test").build();

        let room = client.create_room(room_req).await.unwrap();

        assert_node(&room);
    }

    #[tokio::test]
    async fn test_update_room() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_res = include_str!("../tests/responses/nodes/node_ok.json");

        let room_mock = mock_server
            .mock("PUT", "/api/v4/nodes/rooms/123")
            .with_status(200)
            .with_body(room_res)
            .with_header("content-type", "application/json")
            .create();

        let update = UpdateRoomRequest::builder().with_quota(1234567890).build();

        let room = client.update_room(123, update).await.unwrap();

        assert_node(&room);
    }

    #[tokio::test]
    async fn test_config_room() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_res = include_str!("../tests/responses/nodes/node_ok.json");

        let room_mock = mock_server
            .mock("PUT", "/api/v4/nodes/rooms/123/config")
            .with_status(200)
            .with_body(room_res)
            .with_header("content-type", "application/json")
            .create();

        let config = ConfigRoomRequest::builder()
            .with_inherit_permissions(false)
            .with_admin_ids(vec![1])
            .build();

        let room = client.config_room(123, config).await.unwrap();

        assert_node(&room);
    }

    #[tokio::test]
    async fn test_get_room_policies() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_policies_res = include_str!("../tests/responses/nodes/room_policies_ok.json");

        let room_policies_mock = mock_server
            .mock("GET", "/api/v4/nodes/rooms/123/policies")
            .with_status(200)
            .with_body(room_policies_res)
            .with_header("content-type", "application/json")
            .create();

        let room_policies = client.get_room_policies(123).await.unwrap();

        assert_eq!(room_policies.default_expiration_period, 0);
        assert_eq!(room_policies.is_virus_protection_enabled, false);
    }


    #[tokio::test]
    async fn test_update_room_policy() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_mock = mock_server
            .mock("PUT", "/api/v4/nodes/rooms/123/policies")
            .with_status(204)
            .with_header("content-type", "application/json")
            .create();

        let policy = PoliciesRoomRequest::builder()
            .with_default_expiration_period(60 * 60 * 24 * 7)
            .with_virus_protection_enabled(true)
            .build();

        let no_response_body = client.update_room_policies(123, policy).await.unwrap();

        assert_eq!(no_response_body, ())
    }

    #[tokio::test]
    async fn test_encrypt_room() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_res = include_str!("../tests/responses/nodes/node_ok.json");

        let room_mock = mock_server
            .mock("PUT", "/api/v4/nodes/rooms/123/encrypt")
            .with_status(200)
            .with_body(room_res)
            .with_header("content-type", "application/json")
            .create();

        let room_enc = EncryptRoomRequest::builder(true)
            .with_use_data_space_rescue_key(true)
            .build();

        let room = client.encrypt_room(123, room_enc).await.unwrap();

        assert_node(&room);
    }

    #[tokio::test]
    async fn test_encrypt_room_with_room_rescue_key() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_res = include_str!("../tests/responses/nodes/node_ok.json");

        let room_mock = mock_server
            .mock("PUT", "/api/v4/nodes/rooms/123/encrypt")
            .with_status(200)
            .with_body(room_res)
            .with_header("content-type", "application/json")
            .create();

        let room_enc = EncryptRoomRequest::builder(true)
            .try_with_data_room_rescue_key("TopSecret123!")
            .unwrap()
            .build();

        let room = client.encrypt_room(123, room_enc).await.unwrap();

        assert_node(&room);
    }

    #[tokio::test]
    async fn test_get_room_users() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_users_res = include_str!("../tests/responses/nodes/room_users_ok.json");

        let room_users_mock = mock_server
            .mock("GET", "/api/v4/nodes/rooms/123/users?offset=0")
            .with_status(200)
            .with_body(room_users_res)
            .with_header("content-type", "application/json")
            .create();

        let room_users = client.get_room_users(123, None).await.unwrap();

        assert_eq!(room_users.range.total, 1);
        assert_eq!(room_users.items.len(), 1);

        let room_user = room_users.items.get(0).unwrap();

        room_users_mock.assert();

        assert_room_user(room_user);
    }

    #[tokio::test]
    async fn test_get_room_users_with_limit() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_users_res = include_str!("../tests/responses/nodes/room_users_ok.json");

        let room_users_mock = mock_server
            .mock("GET", "/api/v4/nodes/rooms/123/users?limit=100&offset=0")
            .with_status(200)
            .with_body(room_users_res)
            .with_header("content-type", "application/json")
            .create();

        let params = ListAllParams::builder().with_limit(100).build();

        let room_users = client.get_room_users(123, Some(params)).await.unwrap();

        assert_eq!(room_users.range.total, 1);
        assert_eq!(room_users.items.len(), 1);

        let room_user = room_users.items.get(0).unwrap();

        room_users_mock.assert();

        assert_room_user(room_user);
    }

    #[tokio::test]
    async fn test_get_room_users_with_offset() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_users_res = include_str!("../tests/responses/nodes/room_users_ok.json");

        let room_users_mock = mock_server
            .mock("GET", "/api/v4/nodes/rooms/123/users?offset=500")
            .with_status(200)
            .with_body(room_users_res)
            .with_header("content-type", "application/json")
            .create();

        let params = ListAllParams::builder().with_offset(500).build();

        let room_users = client.get_room_users(123, Some(params)).await.unwrap();

        assert_eq!(room_users.range.total, 1);
        assert_eq!(room_users.items.len(), 1);

        let room_user = room_users.items.get(0).unwrap();

        room_users_mock.assert();

        assert_room_user(room_user);
    }

    #[tokio::test]
    async fn test_update_room_users() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_users_mock = mock_server
            .mock("PUT", "/api/v4/nodes/rooms/123/users")
            .with_status(204)
            .create();

        let user_updates = vec![RoomUsersAddBatchRequestItem::new(
            1,
            NodePermissions::new_with_read_permissions(),
        )];

        let room_users = client
            .update_room_users(123, user_updates.into())
            .await
            .unwrap();

        room_users_mock.assert();
    }

    #[tokio::test]
    async fn test_delete_room_users() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_users_mock = mock_server
            .mock("DELETE", "/api/v4/nodes/rooms/123/users")
            .with_status(204)
            .create();

        let user_ids = vec![1, 2, 3];

        let room_users = client
            .delete_room_users(123, user_ids.into())
            .await
            .unwrap();

        room_users_mock.assert();
    }

    #[tokio::test]
    async fn test_get_room_groups() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_groups_res = include_str!("../tests/responses/nodes/room_groups_ok.json");

        let room_groups_mock = mock_server
            .mock("GET", "/api/v4/nodes/rooms/123/groups?offset=0")
            .with_status(200)
            .with_body(room_groups_res)
            .with_header("content-type", "application/json")
            .create();

        let room_groups = client.get_room_groups(123, None).await.unwrap();

        room_groups_mock.assert();

        assert_eq!(room_groups.range.total, 1);
        assert_eq!(room_groups.items.len(), 1);
        let room_group = room_groups.items.get(0).unwrap();

        assert_room_group(room_group);
    }

    #[tokio::test]
    async fn test_get_room_groups_with_limit() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_groups_res = include_str!("../tests/responses/nodes/room_groups_ok.json");

        let room_groups_mock = mock_server
            .mock("GET", "/api/v4/nodes/rooms/123/groups?limit=100&offset=0")
            .with_status(200)
            .with_body(room_groups_res)
            .with_header("content-type", "application/json")
            .create();

        let params = ListAllParams::builder().with_limit(100).build();

        let room_groups = client.get_room_groups(123, Some(params)).await.unwrap();

        room_groups_mock.assert();

        assert_eq!(room_groups.range.total, 1);
        assert_eq!(room_groups.items.len(), 1);
        let room_group = room_groups.items.get(0).unwrap();

        assert_room_group(room_group);
    }

    #[tokio::test]
    async fn test_get_room_groups_with_offset() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_groups_res = include_str!("../tests/responses/nodes/room_groups_ok.json");

        let room_groups_mock = mock_server
            .mock("GET", "/api/v4/nodes/rooms/123/groups?offset=500")
            .with_status(200)
            .with_body(room_groups_res)
            .with_header("content-type", "application/json")
            .create();

        let params = ListAllParams::builder().with_offset(500).build();

        let room_groups = client.get_room_groups(123, Some(params)).await.unwrap();

        assert_eq!(room_groups.range.total, 1);
        assert_eq!(room_groups.items.len(), 1);

        let room_group = room_groups.items.get(0).unwrap();

        room_groups_mock.assert();

        assert_room_group(room_group);
    }

    #[tokio::test]
    async fn test_update_room_groups() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_groups_mock = mock_server
            .mock("PUT", "/api/v4/nodes/rooms/123/groups")
            .with_status(204)
            .create();

        let group_updates = vec![RoomGroupsAddBatchRequestItem::new(1, NodePermissions::new_with_edit_permissions(), Some(GroupMemberAcceptance::Pending))];

        let room_groups = client
            .update_room_groups(123, group_updates.into())
            .await
            .unwrap();

        room_groups_mock.assert();
    }

    #[tokio::test]
    async fn test_delete_room_groups() {
        let (client, mut mock_server) = get_connected_client().await;

        let room_groups_mock = mock_server
            .mock("DELETE", "/api/v4/nodes/rooms/123/groups")
            .with_status(204)
            .create();

        let group_ids = vec![1, 2, 3];

        let room_groups = client
            .delete_room_groups(123, group_ids.into())
            .await
            .unwrap();

        room_groups_mock.assert();
    }
}
