use async_trait::async_trait;
use reqwest::header;

use crate::{
    auth::{errors::DracoonClientError, Connected},
    constants::{DRACOON_API_PREFIX, FOLDERS_BASE, NODES_BASE},
    utils::FromResponse,
    Dracoon,
};

use super::{
    models::{CreateFolderRequest, Node, UpdateFolderRequest},
    Folders,
};

#[async_trait]
impl Folders for Dracoon<Connected> {
    async fn create_folder(&self, req: CreateFolderRequest) -> Result<Node, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{NODES_BASE}/{FOLDERS_BASE}");
    

        let api_url = self.build_api_url(&url_part);
        let response = self
            .client
            .http
            .post(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&req)
            .send()
            .await?;

        Node::from_response(response).await
    }

    async fn update_folder(&self, folder_id: u64, req: UpdateFolderRequest) -> Result<Node, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{NODES_BASE}/{FOLDERS_BASE}/{folder_id}");


        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .put(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&req)
            .send()
            .await?;

        Node::from_response(response).await
    }
}


#[cfg(test)]
mod tests {
    use crate::{tests::dracoon::get_connected_client, nodes::{UserType, NodeType}};

    use super::*;

    fn assert_folder(folder: &Node) {
        assert_eq!(folder.id, 2);
        assert!(folder.parent_id.is_some());
        assert_eq!(folder.parent_id.unwrap(), 1);
        assert_eq!(folder.node_type, NodeType::Folder);
        assert!(folder.reference_id.is_some());
        assert_eq!(folder.reference_id.unwrap(), 2);
        assert_eq!(folder.name, "string");
        assert_eq!(folder.clone().parent_path.unwrap(), "string");
        assert_eq!(folder.quota.unwrap(), 0);
        assert_eq!(folder.inherit_permissions.unwrap(), true);
        assert_eq!(folder.branch_version.unwrap(), 123456);
        assert_eq!(folder.cnt_rooms.unwrap(), 1);
        assert_eq!(folder.cnt_files.unwrap(), 3);
        assert_eq!(folder.cnt_folders.unwrap(), 2);
        assert_eq!(folder.auth_parent_id.unwrap(), 1);
        assert_eq!(folder.clone().media_token.unwrap(), "string");
        assert_eq!(folder.cnt_comments.unwrap(), 0);
        assert_eq!(folder.cnt_deleted_versions.unwrap(), 0);
        assert_eq!(folder.recycle_bin_retention_period.unwrap(), 9999);
        assert_eq!(folder.is_encrypted.unwrap(), false);
        assert_eq!(folder.has_activities_log.unwrap(), true);
        assert_eq!(
            folder.clone().timestamp_creation.unwrap(),
            "2020-01-01T00:00:00.000Z"
        );
        assert_eq!(
            folder.clone().timestamp_modification.unwrap(),
            "2020-01-01T00:00:00.000Z"
        );
        assert_eq!(folder.clone().updated_at.unwrap(), "2020-02-01T00:00:00.000Z");
        assert_eq!(folder.clone().created_at.unwrap(), "2020-01-01T00:00:00.000Z");
        assert_eq!(folder.clone().size.unwrap(), 123456);
        assert_eq!(folder.clone().classification.unwrap(), 4);

        let created_by = folder.clone().created_by.unwrap();
        let updated_by = folder.clone().updated_by.unwrap();

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
    async fn test_create_folder() {

        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let folder_res = include_str!("../tests/responses/nodes/folder_ok.json");

        let folder_mock = mock_server
             .mock("POST", "/api/v4/nodes/folders")
             .with_status(200)
             .with_body(folder_res)
             .with_header("content-type", "application/json")
             .create();

        let folder = CreateFolderRequest::builder("test", 123).build();
        let folder = dracoon.create_folder(folder).await.unwrap();

        assert_folder(&folder);
    }

    #[tokio::test]
    async fn test_update_folder() {

        let (dracoon, mock_server) = get_connected_client().await;

        let mut mock_server = mock_server;

        let folder_res = include_str!("../tests/responses/nodes/folder_ok.json");

        let folder_mock = mock_server
             .mock("PUT", "/api/v4/nodes/folders/123")
             .with_status(200)
             .with_body(folder_res)
             .with_header("content-type", "application/json")
             .create();

        let update = UpdateFolderRequest::builder().with_name("other test").build();
        let folder = dracoon.update_folder(123, update).await.unwrap();

        assert_folder(&folder);

    }
}