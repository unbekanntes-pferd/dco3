#[cfg(test)]
mod download_share_tests {
    use chrono::DateTime;

    use crate::{
        nodes::NodeType,
        shares::{
            CreateDownloadShareRequest, DeleteDownloadSharesRequest, DownloadShare,
            DownloadShareLinkEmail, DownloadSharesFilter, DownloadSharesSortBy,
            UpdateDownloadShareRequest, UpdateDownloadSharesBulkRequest,
        },
        tests::dracoon::get_connected_client,
        DownloadShares, ListAllParams, SortOrder,
    };

    fn assert_download_share(share: &DownloadShare) {
        assert_eq!(share.id, 1);
        assert_eq!(share.name, "string");
        assert_eq!(share.node_id, 2);
        assert_eq!(share.access_key, "string");
        assert_eq!(share.cnt_downloads, 10);
        assert_eq!(
            share.created_at,
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00.000Z").unwrap()
        );
        assert_eq!(
            share.updated_at.as_ref().unwrap(),
            &DateTime::parse_from_rfc3339("2020-01-01T00:00:00.000Z").unwrap()
        );
        assert_eq!(
            share.expire_at.as_ref().unwrap(),
            &DateTime::parse_from_rfc3339("2020-01-01T00:00:00.000Z").unwrap()
        );
        assert_eq!(share.notes.as_ref().unwrap(), "string");
        assert_eq!(share.internal_notes.as_ref().unwrap(), "string");
        assert!(share.show_creator_name.as_ref().unwrap());
        assert!(share.show_creator_username.as_ref().unwrap());
        assert!(share.is_protected.as_ref().unwrap());
        assert!(share.is_encrypted.as_ref().unwrap());
        assert_eq!(share.max_downloads.as_ref().unwrap(), &2);
        assert_eq!(share.node_path.as_ref().unwrap(), "string");
        assert_eq!(share.node_type.as_ref().unwrap(), &NodeType::File);
        assert_eq!(share.data_url.as_ref().unwrap(), "string");

        assert_eq!(share.created_by.first_name.as_ref().unwrap(), "string");
        assert_eq!(share.created_by.last_name.as_ref().unwrap(), "string");
        assert_eq!(share.created_by.email.as_ref().unwrap(), "string");
        assert_eq!(share.created_by.user_name.as_ref().unwrap(), "string");
        assert_eq!(share.created_by.avatar_uuid, "string");

        assert_eq!(
            share
                .updated_by
                .as_ref()
                .unwrap()
                .first_name
                .as_ref()
                .unwrap(),
            "string"
        );
        assert_eq!(
            share
                .updated_by
                .as_ref()
                .unwrap()
                .last_name
                .as_ref()
                .unwrap(),
            "string"
        );
        assert_eq!(
            share.updated_by.as_ref().unwrap().email.as_ref().unwrap(),
            "string"
        );
        assert_eq!(
            share
                .updated_by
                .as_ref()
                .unwrap()
                .user_name
                .as_ref()
                .unwrap(),
            "string"
        );
        assert_eq!(share.updated_by.as_ref().unwrap().avatar_uuid, "string");
    }

    #[tokio::test]
    async fn test_get_download_shares() {
        let (client, mut mock_server) = get_connected_client().await;

        let shares_res = include_str!("./responses/shares/download_shares_ok.json");
        let shares_mock = mock_server
            .mock("GET", "/api/v4/shares/downloads?offset=0")
            .with_status(200)
            .with_body(shares_res)
            .create();

        let shares = client.get_download_shares(None).await.unwrap();

        shares_mock.assert();

        assert_eq!(shares.range.offset, 0);
        assert_eq!(shares.range.limit, 0);
        assert_eq!(shares.range.total, 1);

        assert_eq!(shares.items.len(), 1);
        let share = shares.items.first().unwrap();

        assert_download_share(share);
    }

    #[tokio::test]
    async fn test_get_download_shares_with_limit() {
        let (client, mut mock_server) = get_connected_client().await;

        let shares_res = include_str!("./responses/shares/download_shares_ok.json");
        let shares_mock = mock_server
            .mock("GET", "/api/v4/shares/downloads?limit=100&offset=0")
            .with_status(200)
            .with_body(shares_res)
            .create();

        let params = ListAllParams::builder().with_limit(100).build();

        let shares = client.get_download_shares(Some(params)).await.unwrap();

        shares_mock.assert();

        assert_eq!(shares.range.offset, 0);
        assert_eq!(shares.range.limit, 0);
        assert_eq!(shares.range.total, 1);

        assert_eq!(shares.items.len(), 1);
        let share = shares.items.first().unwrap();

        assert_download_share(share);
    }

    #[tokio::test]
    async fn test_get_download_shares_with_offset() {
        let (client, mut mock_server) = get_connected_client().await;

        let shares_res = include_str!("./responses/shares/download_shares_ok.json");
        let shares_mock = mock_server
            .mock("GET", "/api/v4/shares/downloads?offset=500")
            .with_status(200)
            .with_body(shares_res)
            .create();

        let params = ListAllParams::builder().with_offset(500).build();

        let shares = client.get_download_shares(Some(params)).await.unwrap();

        shares_mock.assert();

        assert_eq!(shares.range.offset, 0);
        assert_eq!(shares.range.limit, 0);
        assert_eq!(shares.range.total, 1);

        assert_eq!(shares.items.len(), 1);
        let share = shares.items.first().unwrap();

        assert_download_share(share);
    }

    #[tokio::test]
    async fn test_get_download_shares_with_filter() {
        let (client, mut mock_server) = get_connected_client().await;

        let shares_res = include_str!("./responses/shares/download_shares_ok.json");
        let shares_mock = mock_server
            .mock(
                "GET",
                "/api/v4/shares/downloads?offset=0&filter=name%3Acn%3Atest",
            )
            .with_status(200)
            .with_body(shares_res)
            .create();

        let params = ListAllParams::builder()
            .with_filter(DownloadSharesFilter::name_contains("test"))
            .build();

        let shares = client.get_download_shares(Some(params)).await.unwrap();

        shares_mock.assert();

        assert_eq!(shares.range.offset, 0);
        assert_eq!(shares.range.limit, 0);
        assert_eq!(shares.range.total, 1);

        assert_eq!(shares.items.len(), 1);
        let share = shares.items.first().unwrap();

        assert_download_share(share);
    }

    #[tokio::test]
    async fn test_get_download_shares_with_sort() {
        let (client, mut mock_server) = get_connected_client().await;

        let shares_res = include_str!("./responses/shares/download_shares_ok.json");
        let shares_mock = mock_server
            .mock("GET", "/api/v4/shares/downloads?offset=0&sort=name%3Aasc")
            .with_status(200)
            .with_body(shares_res)
            .create();

        let params = ListAllParams::builder()
            .with_sort(DownloadSharesSortBy::Name(SortOrder::Asc))
            .build();

        let shares = client.get_download_shares(Some(params)).await.unwrap();

        shares_mock.assert();

        assert_eq!(shares.range.offset, 0);
        assert_eq!(shares.range.limit, 0);
        assert_eq!(shares.range.total, 1);

        assert_eq!(shares.items.len(), 1);
        let share = shares.items.first().unwrap();

        assert_download_share(share);
    }

    #[tokio::test]
    async fn test_update_download_shares() {
        let (client, mut mock_server) = get_connected_client().await;

        let shares_mock = mock_server
            .mock("PUT", "/api/v4/shares/downloads")
            .with_status(204)
            .create();

        let share_ids = vec![1, 2, 3];

        let update = UpdateDownloadSharesBulkRequest::builder(share_ids)
            .with_max_downloads(2)
            .build();

        let res = client.update_download_shares(update).await;

        shares_mock.assert();
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_delete_download_shares() {
        let (client, mut mock_server) = get_connected_client().await;

        let shares_mock = mock_server
            .mock("DELETE", "/api/v4/shares/downloads")
            .with_status(204)
            .create();

        let share_ids = vec![1, 2, 3];

        let update = DeleteDownloadSharesRequest::new(share_ids);

        let res = client.delete_download_shares(update).await;

        shares_mock.assert();
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_create_download_share() {
        let (client, mut mock_server) = get_connected_client().await;

        let share_res = include_str!("./responses/shares/download_share_ok.json");

        let share_mock = mock_server
            .mock("POST", "/api/v4/shares/downloads")
            .with_status(201)
            .with_body(share_res)
            .with_header("content-type", "application/json")
            .create();

        let share = CreateDownloadShareRequest::builder(1)
            .with_name("test")
            .with_max_downloads(2)
            .build();

        let share = client.create_download_share(share).await.unwrap();

        share_mock.assert();

        assert_download_share(&share);
    }

    #[tokio::test]
    async fn test_get_download_share() {
        let (client, mut mock_server) = get_connected_client().await;

        let share_res = include_str!("./responses/shares/download_share_ok.json");

        let share_mock = mock_server
            .mock("GET", "/api/v4/shares/downloads/123")
            .with_status(201)
            .with_body(share_res)
            .with_header("content-type", "application/json")
            .create();

        let share = client.get_download_share(123).await.unwrap();

        share_mock.assert();

        assert_download_share(&share);
    }

    #[tokio::test]
    async fn test_update_download_share() {
        let (client, mut mock_server) = get_connected_client().await;

        let share_res = include_str!("./responses/shares/download_share_ok.json");

        let share_mock = mock_server
            .mock("PUT", "/api/v4/shares/downloads/123")
            .with_status(201)
            .with_body(share_res)
            .with_header("content-type", "application/json")
            .create();

        let update = UpdateDownloadShareRequest::builder()
            .with_name("test")
            .with_max_downloads(2)
            .build();

        let share = client.update_download_share(123, update).await.unwrap();

        share_mock.assert();

        assert_download_share(&share);
    }

    #[tokio::test]
    async fn test_delete_download_share() {
        let (client, mut mock_server) = get_connected_client().await;

        let share_res = include_str!("./responses/shares/download_share_ok.json");

        let share_mock = mock_server
            .mock("DELETE", "/api/v4/shares/downloads/123")
            .with_status(204)
            .create();

        let res = client.delete_download_share(123).await;

        share_mock.assert();

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_send_download_share_email() {
        let (client, mut mock_server) = get_connected_client().await;

        let share_res = include_str!("./responses/shares/download_share_ok.json");

        let share_mock = mock_server
            .mock("POST", "/api/v4/shares/downloads/123/email")
            .with_status(204)
            .create();

        let email = DownloadShareLinkEmail::new("test", vec!["foo@localhost".into()], None);

        let res = client.send_download_share_email(123, email).await;

        share_mock.assert();

        assert!(res.is_ok());
    }
}

#[cfg(test)]
mod upload_share_tests {
    use chrono::DateTime;

    use crate::{
        shares::{
            CreateUploadShareRequest, UpdateUploadShareRequest, UpdateUploadSharesBulkRequest,
            UploadShare, UploadShareLinkEmail, UploadSharesFilter, UploadSharesSortBy,
        },
        tests::dracoon::get_connected_client,
        ListAllParams, SortOrder, UploadShares,
    };

    fn assert_upload_share(share: &UploadShare) {
        assert_eq!(share.id, 1);
        assert_eq!(share.name, "string");
        assert_eq!(share.target_id, 2);
        assert_eq!(share.access_key, "string");
        assert_eq!(
            share.created_at,
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00.000Z").unwrap()
        );
        assert_eq!(
            share.updated_at.as_ref().unwrap(),
            &DateTime::parse_from_rfc3339("2020-01-01T00:00:00.000Z").unwrap()
        );
        assert_eq!(
            share.expire_at.as_ref().unwrap(),
            &DateTime::parse_from_rfc3339("2020-01-01T00:00:00.000Z").unwrap()
        );
        assert_eq!(share.notes.as_ref().unwrap(), "string");
        assert_eq!(share.internal_notes.as_ref().unwrap(), "string");
        assert_eq!(share.target_path.as_ref().unwrap(), "string");
        assert!(share.show_creator_name.as_ref().unwrap());
        assert!(share.show_creator_username.as_ref().unwrap());
        assert!(share.show_uploaded_files.as_ref().unwrap());
        assert!(share.is_encrypted.as_ref().unwrap());
        assert!(share.is_protected);
        assert_eq!(share.data_url.as_ref().unwrap(), "string");
        assert_eq!(share.max_slots.as_ref().unwrap(), &1);
        assert_eq!(share.max_size.as_ref().unwrap(), &123456);

        assert_eq!(share.created_by.first_name.as_ref().unwrap(), "string");
        assert_eq!(share.created_by.last_name.as_ref().unwrap(), "string");
        assert_eq!(share.created_by.email.as_ref().unwrap(), "string");
        assert_eq!(share.created_by.user_name.as_ref().unwrap(), "string");
        assert_eq!(share.created_by.avatar_uuid, "string");

        assert_eq!(
            share
                .updated_by
                .as_ref()
                .unwrap()
                .first_name
                .as_ref()
                .unwrap(),
            "string"
        );
        assert_eq!(
            share
                .updated_by
                .as_ref()
                .unwrap()
                .last_name
                .as_ref()
                .unwrap(),
            "string"
        );
        assert_eq!(
            share.updated_by.as_ref().unwrap().email.as_ref().unwrap(),
            "string"
        );
        assert_eq!(
            share
                .updated_by
                .as_ref()
                .unwrap()
                .user_name
                .as_ref()
                .unwrap(),
            "string"
        );
        assert_eq!(share.updated_by.as_ref().unwrap().avatar_uuid, "string");
    }

    #[tokio::test]
    async fn test_get_upload_shares() {
        let (client, mut mock_server) = get_connected_client().await;

        let share_res = include_str!("./responses/shares/upload_shares_ok.json");

        let share_mock = mock_server
            .mock("GET", "/api/v4/shares/uploads?offset=0")
            .with_status(200)
            .with_body(share_res)
            .with_header("content-type", "application/json")
            .create();

        let shares = client.get_upload_shares(None).await.unwrap();

        share_mock.assert();

        assert_eq!(shares.range.total, 1);
        assert_eq!(shares.range.offset, 0);
        assert_eq!(shares.range.limit, 0);

        assert_eq!(shares.items.len(), 1);

        let share = shares.items.first().unwrap();

        assert_upload_share(share);
    }

    #[tokio::test]
    async fn test_get_upload_shares_with_limit() {
        let (client, mut mock_server) = get_connected_client().await;

        let share_res = include_str!("./responses/shares/upload_shares_ok.json");

        let share_mock = mock_server
            .mock("GET", "/api/v4/shares/uploads?limit=100&offset=0")
            .with_status(200)
            .with_body(share_res)
            .with_header("content-type", "application/json")
            .create();

        let params = ListAllParams::builder().with_limit(100).build();

        let shares = client.get_upload_shares(Some(params)).await.unwrap();

        share_mock.assert();

        assert_eq!(shares.range.total, 1);
        assert_eq!(shares.range.offset, 0);
        assert_eq!(shares.range.limit, 0);

        assert_eq!(shares.items.len(), 1);

        let share = shares.items.first().unwrap();

        assert_upload_share(share);
    }

    #[tokio::test]
    async fn test_get_upload_shares_with_offset() {
        let (client, mut mock_server) = get_connected_client().await;

        let share_res = include_str!("./responses/shares/upload_shares_ok.json");

        let share_mock = mock_server
            .mock("GET", "/api/v4/shares/uploads?offset=500")
            .with_status(200)
            .with_body(share_res)
            .with_header("content-type", "application/json")
            .create();

        let params = ListAllParams::builder().with_offset(500).build();

        let shares = client.get_upload_shares(Some(params)).await.unwrap();

        share_mock.assert();

        assert_eq!(shares.range.total, 1);
        assert_eq!(shares.range.offset, 0);
        assert_eq!(shares.range.limit, 0);

        assert_eq!(shares.items.len(), 1);

        let share = shares.items.first().unwrap();

        assert_upload_share(share);
    }

    #[tokio::test]
    async fn test_get_upload_shares_with_filter() {
        let (client, mut mock_server) = get_connected_client().await;

        let share_res = include_str!("./responses/shares/upload_shares_ok.json");

        let share_mock = mock_server
            .mock(
                "GET",
                "/api/v4/shares/uploads?offset=0&filter=userId%3Aeq%3A2",
            )
            .with_status(200)
            .with_body(share_res)
            .with_header("content-type", "application/json")
            .create();

        let params = ListAllParams::builder()
            .with_filter(UploadSharesFilter::user_id_equals(2))
            .build();

        let shares = client.get_upload_shares(Some(params)).await.unwrap();

        share_mock.assert();

        assert_eq!(shares.range.total, 1);
        assert_eq!(shares.range.offset, 0);
        assert_eq!(shares.range.limit, 0);

        assert_eq!(shares.items.len(), 1);

        let share = shares.items.first().unwrap();

        assert_upload_share(share);
    }

    #[tokio::test]
    async fn test_get_upload_shares_with_sort() {
        let (client, mut mock_server) = get_connected_client().await;

        let share_res = include_str!("./responses/shares/upload_shares_ok.json");

        let share_mock = mock_server
            .mock("GET", "/api/v4/shares/uploads?offset=0&sort=name%3Aasc")
            .with_status(200)
            .with_body(share_res)
            .with_header("content-type", "application/json")
            .create();

        let params = ListAllParams::builder()
            .with_sort(UploadSharesSortBy::Name(SortOrder::Asc))
            .build();

        let shares = client.get_upload_shares(Some(params)).await.unwrap();

        share_mock.assert();

        assert_eq!(shares.range.total, 1);
        assert_eq!(shares.range.offset, 0);
        assert_eq!(shares.range.limit, 0);

        assert_eq!(shares.items.len(), 1);

        let share = shares.items.first().unwrap();

        assert_upload_share(share);
    }

    #[tokio::test]
    async fn test_update_upload_shares() {
        let (client, mut mock_server) = get_connected_client().await;

        let shares_mock = mock_server
            .mock("PUT", "/api/v4/shares/uploads")
            .with_status(204)
            .create();

        let share_ids = vec![1, 2, 3];

        let update = UpdateUploadSharesBulkRequest::builder(share_ids)
            .with_show_creator_name(true)
            .with_show_creator_username(true)
            .with_show_uploaded_files(true)
            .build();

        let res = client.update_upload_shares(update).await;

        shares_mock.assert();

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_delete_upload_shares() {
        let (client, mut mock_server) = get_connected_client().await;

        let shares_mock = mock_server
            .mock("DELETE", "/api/v4/shares/uploads")
            .with_status(204)
            .create();

        let share_ids = vec![1, 2, 3];

        let res = client.delete_upload_shares(share_ids.into()).await;

        shares_mock.assert();

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_create_upload_share() {
        let (client, mut mock_server) = get_connected_client().await;

        let share_res = include_str!("./responses/shares/upload_share_ok.json");

        let share_mock = mock_server
            .mock("POST", "/api/v4/shares/uploads")
            .with_status(201)
            .with_body(share_res)
            .with_header("content-type", "application/json")
            .create();

        let share = CreateUploadShareRequest::builder(1)
            .with_name("test")
            .with_notes("test")
            .with_show_uploaded_files(true)
            .with_show_creator_name(true)
            .with_show_creator_username(true)
            .build();

        let share = client.create_upload_share(share).await.unwrap();

        share_mock.assert();

        assert_upload_share(&share);
    }

    #[tokio::test]
    async fn test_get_upload_share() {
        let (client, mut mock_server) = get_connected_client().await;

        let share_res = include_str!("./responses/shares/upload_share_ok.json");

        let share_mock = mock_server
            .mock("GET", "/api/v4/shares/uploads/123")
            .with_status(201)
            .with_body(share_res)
            .with_header("content-type", "application/json")
            .create();

        let share = client.get_upload_share(123).await.unwrap();

        share_mock.assert();

        assert_upload_share(&share);
    }

    #[tokio::test]
    async fn test_update_upload_share() {
        let (client, mut mock_server) = get_connected_client().await;

        let share_res = include_str!("./responses/shares/upload_share_ok.json");

        let share_mock = mock_server
            .mock("PUT", "/api/v4/shares/uploads/123")
            .with_status(201)
            .with_body(share_res)
            .with_header("content-type", "application/json")
            .create();

        let update = UpdateUploadShareRequest::builder()
            .with_name("test")
            .with_notes("test")
            .build();

        let share = client.update_upload_share(123, update).await.unwrap();

        share_mock.assert();

        assert_upload_share(&share);
    }

    #[tokio::test]
    async fn test_delete_upload_share() {
        let (client, mut mock_server) = get_connected_client().await;

        let shares_mock = mock_server
            .mock("DELETE", "/api/v4/shares/uploads/123")
            .with_status(204)
            .create();

        let res = client.delete_upload_share(123).await;

        shares_mock.assert();

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_send_upload_share_email() {
        let (client, mut mock_server) = get_connected_client().await;

        let shares_mock = mock_server
            .mock("POST", "/api/v4/shares/uploads/123/email")
            .with_status(204)
            .create();

        let email = UploadShareLinkEmail::new("test", vec!["foo@localhost".into()], None);

        let res = client.send_upload_share_email(123, email).await;

        shares_mock.assert();

        assert!(res.is_ok());
    }
}
