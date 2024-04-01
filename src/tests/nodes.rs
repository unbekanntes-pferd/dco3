#[cfg(test)]
pub mod tests {
    use crate::{
        nodes::{
            Node, NodeType, NodesFilter, NodesSearchFilter, NodesSearchSortBy, NodesSortBy,
            UserType,
        },
        tests::dracoon::get_connected_client,
        *,
    };

    pub fn assert_node(node: &Node) {
        assert_eq!(node.id, 2);
        assert!(node.parent_id.is_some());
        assert_eq!(node.parent_id.unwrap(), 1);
        assert_eq!(node.node_type, NodeType::Room);
        assert!(node.reference_id.is_some());
        assert_eq!(node.reference_id.unwrap(), 2);
        assert_eq!(node.name, "string");
        assert_eq!(node.clone().parent_path.unwrap(), "string");
        assert_eq!(node.quota.unwrap(), 0);
        assert!(node.inherit_permissions.unwrap());
        assert_eq!(node.branch_version.unwrap(), 123456);
        assert_eq!(node.cnt_rooms.unwrap(), 1);
        assert_eq!(node.cnt_files.unwrap(), 3);
        assert_eq!(node.cnt_folders.unwrap(), 2);
        assert_eq!(node.auth_parent_id.unwrap(), 1);
        assert_eq!(node.clone().media_token.unwrap(), "string");
        assert_eq!(node.cnt_comments.unwrap(), 0);
        assert_eq!(node.cnt_deleted_versions.unwrap(), 0);
        assert_eq!(node.recycle_bin_retention_period.unwrap(), 9999);
        assert!(!node.is_encrypted.unwrap());
        assert!(node.has_activities_log.unwrap());
        assert_eq!(
            node.clone().timestamp_creation.unwrap(),
            "2020-01-01T00:00:00.000Z"
        );
        assert_eq!(
            node.clone().timestamp_modification.unwrap(),
            "2020-01-01T00:00:00.000Z"
        );
        assert_eq!(node.clone().updated_at.unwrap(), "2020-02-01T00:00:00.000Z");
        assert_eq!(node.clone().created_at.unwrap(), "2020-01-01T00:00:00.000Z");
        assert_eq!(node.clone().size.unwrap(), 123456);
        assert_eq!(node.clone().classification.unwrap(), 4);

        let created_by = node.clone().created_by.unwrap();
        let updated_by = node.clone().updated_by.unwrap();

        assert_eq!(created_by.id, 3);
        assert_eq!(created_by.first_name.unwrap(), "string");
        assert_eq!(created_by.last_name.unwrap(), "string");
        assert_eq!(created_by.user_name.unwrap(), "string");
        assert_eq!(created_by.email.unwrap(), "string");
        assert_eq!(created_by.avatar_uuid, "string");
        assert_eq!(created_by.user_type, UserType::Internal);

        assert_eq!(updated_by.id, 3);
        assert_eq!(updated_by.first_name.unwrap(), "string");
        assert_eq!(updated_by.last_name.unwrap(), "string");
        assert_eq!(updated_by.user_name.unwrap(), "string");
        assert_eq!(updated_by.email.unwrap(), "string");
        assert_eq!(updated_by.avatar_uuid, "string");
        assert_eq!(updated_by.user_type, UserType::Internal);
    }

    #[tokio::test]
    async fn test_get_nodes() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let nodes_mock = mock_server
            .mock("GET", "/api/v4/nodes?offset=0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let nodes = dracoon.get_nodes(None, None, None).await.unwrap();

        nodes_mock.assert();

        assert_eq!(nodes.items.len(), 1);

        let node = nodes.items.first().unwrap();

        assert_node(node);
    }

    #[tokio::test]
    async fn test_get_nodes_with_parent_id() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let nodes_mock = mock_server
            .mock("GET", "/api/v4/nodes?offset=0&parent_id=123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let _nodes = dracoon.get_nodes(Some(123), None, None).await.unwrap();

        nodes_mock.assert();
    }

    #[tokio::test]
    async fn test_get_nodes_with_offset() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let nodes_mock = mock_server
            .mock("GET", "/api/v4/nodes?offset=500")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();
        let params = ListAllParams::builder().with_offset(500).build();

        let _nodes = dracoon.get_nodes(None, None, Some(params)).await.unwrap();

        nodes_mock.assert();
    }

    #[tokio::test]
    async fn test_get_nodes_with_limit() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let nodes_mock = mock_server
            .mock("GET", "/api/v4/nodes?limit=100&offset=0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let params = ListAllParams::builder().with_limit(100).build();

        let _nodes = dracoon.get_nodes(None, None, Some(params)).await.unwrap();

        nodes_mock.assert();
    }

    #[tokio::test]
    async fn test_get_nodes_with_filter() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let nodes_mock = mock_server
            .mock("GET", "/api/v4/nodes?offset=0&filter=name%3Aeq%3Atest")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let params = ListAllParams::builder()
            .with_filter(NodesFilter::name_equals("test"))
            .build();

        let _nodes = dracoon.get_nodes(None, None, Some(params)).await.unwrap();

        nodes_mock.assert();
    }

    #[tokio::test]
    async fn test_get_nodes_with_sort() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let nodes_mock = mock_server
            .mock("GET", "/api/v4/nodes?offset=0&sort=name%3Aasc")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let params = ListAllParams::builder()
            .with_sort(NodesSortBy::Name(SortOrder::Asc))
            .build();

        let _nodes = dracoon.get_nodes(None, None, Some(params)).await.unwrap();

        nodes_mock.assert();
    }

    #[tokio::test]
    async fn test_get_node_from_path() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let path = "/some/path/test";

        let nodes_mock = mock_server
            .mock("GET", "/api/v4/nodes/search?search_string=test&depth_level=2&filter=parentPath%3Aeq%3A%2Fsome%2Fpath%2F")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let node = dracoon.get_node_from_path(path).await.unwrap();

        nodes_mock.assert();

        assert!(node.is_some());

        let node = node.unwrap();

        assert_node(&node);
    }

    #[tokio::test]
    async fn test_get_node_from_path_no_result() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_search_no_result.json");

        let path = "/some/path/test";

        let nodes_mock = mock_server
            .mock("GET", "/api/v4/nodes/search?search_string=test&depth_level=2&filter=parentPath%3Aeq%3A%2Fsome%2Fpath%2F")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let node = dracoon.get_node_from_path(path).await.unwrap();

        nodes_mock.assert();

        assert!(node.is_none());
    }

    #[tokio::test]
    async fn test_get_node() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let node_res = include_str!("./responses/nodes/node_ok.json");

        let node_mock = mock_server
            .mock("GET", "/api/v4/nodes/123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(node_res)
            .create();

        let node = dracoon.get_node(123).await.unwrap();

        assert_node(&node);
    }

    #[tokio::test]
    async fn test_delete_node() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let node_mock = mock_server
            .mock("DELETE", "/api/v4/nodes/123")
            .with_status(204)
            .with_header("content-type", "application/json")
            .create();

        dracoon.delete_node(123).await.unwrap();

        node_mock.assert();
    }

    #[tokio::test]
    async fn test_delete_nodes() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let node_ids = vec![1, 2, 3];

        let node_mock = mock_server
            .mock("DELETE", "/api/v4/nodes")
            .with_status(204)
            .with_header("content-type", "application/json")
            .create();

        dracoon.delete_nodes(node_ids.into()).await.unwrap();

        node_mock.assert();
    }

    #[tokio::test]
    async fn test_copy_nodes() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let node_res = include_str!("./responses/nodes/node_ok.json");

        let node_ids = vec![1, 2, 3];

        let copy_mock = mock_server
            .mock("POST", "/api/v4/nodes/123/copy_to")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(node_res)
            .create();

        let target_node = dracoon.copy_nodes(node_ids.into(), 123).await.unwrap();

        copy_mock.assert();

        assert_node(&target_node);
    }

    #[tokio::test]
    async fn test_move_nodes() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let node_res = include_str!("./responses/nodes/node_ok.json");

        let node_ids = vec![1, 2, 3];

        let move_mock = mock_server
            .mock("POST", "/api/v4/nodes/123/move_to")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(node_res)
            .create();

        let target_node = dracoon.move_nodes(node_ids.into(), 123).await.unwrap();

        move_mock.assert();

        assert_node(&target_node);
    }

    #[tokio::test]
    async fn test_search_nodes() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let nodes_mock = mock_server
            .mock("GET", "/api/v4/nodes/search?search_string=test&offset=0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let nodes = dracoon
            .search_nodes("test", None, None, None)
            .await
            .unwrap();

        nodes_mock.assert();

        assert_eq!(nodes.items.len(), 1);

        let node = nodes.items.first().unwrap();

        assert_node(node);
    }

    #[tokio::test]
    async fn test_search_nodes_with_parent_id() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let nodes_mock = mock_server
            .mock(
                "GET",
                "/api/v4/nodes/search?search_string=test&offset=0&parent_id=123",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let nodes = dracoon
            .search_nodes("test", Some(123), None, None)
            .await
            .unwrap();

        nodes_mock.assert();

        assert_eq!(nodes.items.len(), 1);

        let node = nodes.items.first().unwrap();

        assert_node(node);
    }

    #[tokio::test]
    async fn test_search_nodes_with_limit() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let nodes_mock = mock_server
            .mock(
                "GET",
                "/api/v4/nodes/search?search_string=test&limit=100&offset=0",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let params = ListAllParams::builder().with_limit(100).build();

        let nodes = dracoon
            .search_nodes("test", None, None, Some(params))
            .await
            .unwrap();

        nodes_mock.assert();

        assert_eq!(nodes.items.len(), 1);

        let node = nodes.items.first().unwrap();

        assert_node(node);
    }

    #[tokio::test]
    async fn test_search_nodes_with_offset() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let nodes_mock = mock_server
            .mock("GET", "/api/v4/nodes/search?search_string=test&offset=500")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let params = ListAllParams::builder().with_offset(500).build();

        let nodes = dracoon
            .search_nodes("test", None, None, Some(params))
            .await
            .unwrap();

        nodes_mock.assert();

        assert_eq!(nodes.items.len(), 1);

        let node = nodes.items.first().unwrap();

        assert_node(node);
    }

    #[tokio::test]
    async fn test_search_nodes_with_filter() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let nodes_mock = mock_server
            .mock(
                "GET",
                "/api/v4/nodes/search?search_string=test&offset=0&filter=parentPath%3Acn%3Atest",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let params = ListAllParams::builder()
            .with_filter(NodesSearchFilter::parent_path_contains("test"))
            .build();

        let nodes = dracoon
            .search_nodes("test", None, None, Some(params))
            .await
            .unwrap();

        nodes_mock.assert();

        assert_eq!(nodes.items.len(), 1);

        let node = nodes.items.first().unwrap();

        assert_node(node);
    }

    #[tokio::test]
    async fn test_search_nodes_with_sort() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let nodes_mock = mock_server
            .mock(
                "GET",
                "/api/v4/nodes/search?search_string=test&offset=0&sort=fileType%3Aasc",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let params = ListAllParams::builder()
            .with_sort(NodesSearchSortBy::FileType(SortOrder::Asc))
            .build();

        let nodes = dracoon
            .search_nodes("test", None, None, Some(params))
            .await
            .unwrap();

        nodes_mock.assert();

        assert_eq!(nodes.items.len(), 1);

        let node = nodes.items.first().unwrap();

        assert_node(node);
    }

    #[tokio::test]
    async fn test_search_nodes_with_depth_level() {
        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let nodes_res = include_str!("./responses/nodes/nodes_ok.json");

        let nodes_mock = mock_server
            .mock(
                "GET",
                "/api/v4/nodes/search?search_string=test&depth_level=-1&offset=0",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(nodes_res)
            .create();

        let nodes = dracoon
            .search_nodes("test", None, Some(-1), None)
            .await
            .unwrap();

        nodes_mock.assert();

        assert_eq!(nodes.items.len(), 1);

        let node = nodes.items.first().unwrap();

        assert_node(node);
    }
}
