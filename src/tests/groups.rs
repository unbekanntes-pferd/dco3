#[cfg(test)]
mod tests {

    use chrono::DateTime;

    use crate::*;

    use crate::groups::{CreateGroupRequest, GroupsFilter, GroupsSortBy, UpdateGroupRequest};
    use crate::nodes::UserType;
    use crate::tests::dracoon::get_connected_client;

    #[tokio::test]
    async fn test_get_groups() {
        let (dracoon, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let groups_res = include_str!("./responses/groups/groups_ok.json");

        let groups_mock = mock_server
            .mock("GET", "/api/v4/groups?offset=0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(groups_res)
            .create();

        let groups = dracoon.groups().get_groups(None).await.unwrap();
        let group = groups.items.first().unwrap();

        groups_mock.assert();

        assert_eq!(groups.items.len(), 1);
        assert_eq!(group.id, 1);
        assert_eq!(group.name, "string");
        assert_eq!(group.created_by.id, 2);
        assert!(group.created_by.first_name.is_some());
        assert!(group.created_by.last_name.is_some());
        assert!(group.created_by.user_name.is_some());
        assert!(group.cnt_users.is_some());
        assert!(group.updated_by.is_some());
        assert_eq!(group.created_by.user_name.as_ref().unwrap(), "string");
        assert_eq!(
            group.created_by.first_name.as_ref().unwrap().clone(),
            "string"
        );
        assert_eq!(
            group.created_by.last_name.as_ref().unwrap().clone(),
            "string"
        );
        assert_eq!(
            group.created_at,
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00.000Z").unwrap()
        );
        assert_eq!(group.created_by.id, 2);
        assert_eq!(group.created_by.user_type, UserType::Internal);
        assert_eq!(group.created_by.email.as_ref().unwrap(), "string");
        assert_eq!(group.created_by.avatar_uuid, "string");
        assert_eq!(group.cnt_users.unwrap(), 1);

        assert_eq!(
            group
                .updated_by
                .as_ref()
                .unwrap()
                .user_name
                .as_ref()
                .unwrap(),
            "string"
        );
        assert_eq!(
            group
                .updated_by
                .as_ref()
                .unwrap()
                .first_name
                .as_ref()
                .unwrap()
                .clone(),
            "string"
        );
        assert_eq!(
            group
                .updated_by
                .as_ref()
                .unwrap()
                .last_name
                .as_ref()
                .unwrap()
                .clone(),
            "string"
        );
        assert_eq!(
            group.updated_at.as_ref().unwrap(),
            &DateTime::parse_from_rfc3339("2023-07-23T08:58:01.236Z").unwrap()
        );
        assert_eq!(group.updated_by.as_ref().unwrap().id, 2);
        assert_eq!(
            group.updated_by.as_ref().unwrap().user_type,
            UserType::Internal
        );
        assert_eq!(
            group.updated_by.as_ref().unwrap().email.as_ref().unwrap(),
            "string"
        );
        assert_eq!(group.updated_by.as_ref().unwrap().avatar_uuid, "string");
    }

    #[tokio::test]
    async fn test_get_groups_with_filter() {
        let (dracoon, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let groups_res = include_str!("./responses/groups/groups_ok.json");

        let groups_mock = mock_server
            .mock("GET", "/api/v4/groups?offset=0&filter=name%3Aeq%3Astring")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(groups_res)
            .create();

        let params = ListAllParams::builder()
            .with_filter(GroupsFilter::Name(FilterOperator::Eq, "string".to_string()))
            .build();

        let groups = dracoon.groups().get_groups(Some(params)).await.unwrap();

        groups_mock.assert();
    }

    #[tokio::test]
    async fn test_get_groups_with_sort() {
        let (dracoon, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let groups_res = include_str!("./responses/groups/groups_ok.json");

        let groups_mock = mock_server
            .mock("GET", "/api/v4/groups?offset=0&sort=name%3Aasc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(groups_res)
            .create();

        let params = ListAllParams::builder()
            .with_sort(GroupsSortBy::Name(SortOrder::Asc))
            .build();

        let groups = dracoon.groups().get_groups(Some(params)).await.unwrap();

        groups_mock.assert();
    }

    #[tokio::test]
    async fn test_get_groups_with_offset() {
        let (dracoon, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let groups_res = include_str!("./responses/groups/groups_ok.json");

        let groups_mock = mock_server
            .mock("GET", "/api/v4/groups?offset=0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(groups_res)
            .create();

        let params = ListAllParams::default();

        let groups = dracoon.groups().get_groups(Some(params)).await.unwrap();

        groups_mock.assert();
    }

    #[tokio::test]
    async fn test_get_groups_with_limit() {
        let (dracoon, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let groups_res = include_str!("./responses/groups/groups_ok.json");

        let groups_mock = mock_server
            .mock("GET", "/api/v4/groups?limit=10&offset=0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(groups_res)
            .create();

        let params = ListAllParams::builder().with_limit(10).build();

        let groups = dracoon.groups().get_groups(Some(params)).await.unwrap();

        groups_mock.assert();
    }

    #[tokio::test]
    async fn test_get_group() {
        let (dracoon, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let groups_res = include_str!("./responses/groups/group_ok.json");

        let group_mock = mock_server
            .mock("GET", "/api/v4/groups/123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(groups_res)
            .create();

        let group = dracoon.groups().get_group(123).await.unwrap();

        group_mock.assert();

        assert_eq!(group.id, 1);
        assert_eq!(group.name, "string");
        assert_eq!(group.created_by.id, 2);
        assert!(group.created_by.first_name.is_some());
        assert!(group.created_by.last_name.is_some());
        assert!(group.created_by.user_name.is_some());
        assert!(group.cnt_users.is_some());
        assert!(group.updated_by.is_some());
        assert_eq!(group.created_by.user_name.as_ref().unwrap(), "string");
        assert_eq!(
            group.created_by.first_name.as_ref().unwrap().clone(),
            "string"
        );
        assert_eq!(
            group.created_by.last_name.as_ref().unwrap().clone(),
            "string"
        );
        assert_eq!(
            group.created_at,
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00.000Z").unwrap()
        );
        assert_eq!(group.created_by.id, 2);
        assert_eq!(group.created_by.user_type, UserType::Internal);
        assert_eq!(group.created_by.email.as_ref().unwrap(), "string");
        assert_eq!(group.created_by.avatar_uuid, "string");
    }

    #[tokio::test]
    async fn test_create_group() {
        let (dracoon, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let group_res = include_str!("./responses/groups/group_ok.json");

        let group_mock = mock_server
            .mock("POST", "/api/v4/groups")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(group_res)
            .create();

        let group = CreateGroupRequest::new("string", None);

        let group = dracoon.groups().create_group(group).await.unwrap();

        group_mock.assert();

        assert_eq!(group.id, 1);
        assert_eq!(group.name, "string");
        assert_eq!(group.created_by.id, 2);
        assert!(group.created_by.first_name.is_some());
        assert!(group.created_by.last_name.is_some());
        assert!(group.created_by.user_name.is_some());
        assert!(group.cnt_users.is_some());
        assert!(group.updated_by.is_some());
        assert_eq!(group.created_by.user_name.as_ref().unwrap(), "string");
        assert_eq!(
            group.created_by.first_name.as_ref().unwrap().clone(),
            "string"
        );
        assert_eq!(
            group.created_by.last_name.as_ref().unwrap().clone(),
            "string"
        );
        assert_eq!(
            group.created_at,
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00.000Z").unwrap()
        );
        assert_eq!(group.created_by.id, 2);
        assert_eq!(group.created_by.user_type, UserType::Internal);
        assert_eq!(group.created_by.email.as_ref().unwrap(), "string");
        assert_eq!(group.created_by.avatar_uuid, "string");
    }

    #[tokio::test]
    async fn test_update_group() {
        let (dracoon, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let group_res = include_str!("./responses/groups/group_ok.json");

        let group_mock = mock_server
            .mock("PUT", "/api/v4/groups/123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(group_res)
            .create();

        let update = UpdateGroupRequest::name("string");

        let group = dracoon.groups().update_group(123, update).await.unwrap();

        group_mock.assert();

        assert_eq!(group.id, 1);
        assert_eq!(group.name, "string");
        assert_eq!(group.created_by.id, 2);
        assert!(group.created_by.first_name.is_some());
        assert!(group.created_by.last_name.is_some());
        assert!(group.created_by.user_name.is_some());
        assert!(group.cnt_users.is_some());
        assert!(group.updated_by.is_some());
        assert_eq!(group.created_by.user_name.as_ref().unwrap(), "string");
        assert_eq!(
            group.created_by.first_name.as_ref().unwrap().clone(),
            "string"
        );
        assert_eq!(
            group.created_by.last_name.as_ref().unwrap().clone(),
            "string"
        );
        assert_eq!(
            group.created_at,
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00.000Z").unwrap()
        );
        assert_eq!(group.created_by.id, 2);
        assert_eq!(group.created_by.user_type, UserType::Internal);
        assert_eq!(group.created_by.email.as_ref().unwrap(), "string");
        assert_eq!(group.created_by.avatar_uuid, "string");
    }

    #[tokio::test]
    async fn test_delete_group() {
        let (dracoon, mock_server) = get_connected_client().await;
        let mut mock_server = mock_server;

        let groups_mock = mock_server
            .mock("DELETE", "/api/v4/groups/123")
            .with_status(204)
            .create();

        dracoon.groups().delete_group(123).await.unwrap();

        groups_mock.assert();
    }
}
