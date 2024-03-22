#![allow(clippy::module_inception)]

use async_trait::async_trait;
use reqwest::header;
use tracing::{debug, error};

use crate::{
    auth::{errors::DracoonClientError, Connected},
    constants::{
        DRACOON_API_PREFIX, FILES_BASE, FILES_KEYS, MISSING_FILE_KEYS, NODES_BASE, NODES_COPY,
        NODES_MOVE, NODES_SEARCH,
    },
    models::ListAllParams,
    utils::FromResponse,
    Dracoon,
};

use super::{
    models::{DeleteNodesRequest, Node, NodeList, TransferNodesRequest},
    MissingKeysResponse, Nodes, UserFileKeySetBatchRequest,
};

#[async_trait]
impl Nodes for Dracoon<Connected> {
    async fn get_nodes(
        &self,
        parent_id: Option<u64>,
        room_manager: Option<bool>,
        params: Option<ListAllParams>,
    ) -> Result<NodeList, DracoonClientError> {
        let params = params.unwrap_or_default();
        let url_part = format!("/{DRACOON_API_PREFIX}/{NODES_BASE}");

        let mut api_url = self.build_api_url(&url_part);

        let filters = params.filter_to_string();
        let sorts = params.sort_to_string();

        api_url
            .query_pairs_mut()
            .extend_pairs(params.limit.map(|v| ("limit", v.to_string())))
            .extend_pairs(params.offset.map(|v| ("offset", v.to_string())))
            .extend_pairs(params.sort.map(|_| ("sort", sorts)))
            .extend_pairs(params.filter.map(|_| ("filter", filters)))
            .extend_pairs(room_manager.map(|v| ("room_manager", v.to_string())))
            .extend_pairs(parent_id.map(|v| ("parent_id", v.to_string())))
            .finish();

        let response = self
            .client
            .http
            .get(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        NodeList::from_response(response).await
    }

    async fn get_node_from_path(&self, path: &str) -> Result<Option<Node>, DracoonClientError> {
        // TODO: refactor and make use of search_nodes
        let url_part = format!("/{DRACOON_API_PREFIX}/{NODES_BASE}/{NODES_SEARCH}");

        let (parent_path, name, depth) = parse_node_path(path).map_err(|_| {
            error!("Failed to parse path: {}", path);
            DracoonClientError::InvalidPath(path.to_string())
        })?;

        let mut api_url = self.build_api_url(&url_part);

        api_url
            .query_pairs_mut()
            .append_pair("search_string", &name)
            .append_pair("depth_level", &depth.to_string())
            .append_pair("filter", &format!("parentPath:eq:{parent_path}"))
            .finish();

        let response = self
            .client
            .http
            .get(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        let nodes = NodeList::from_response(response).await?;

        debug!("Found {} nodes", nodes.items.len());

        match nodes.items.len() {
            1 => Ok(nodes.items.into_iter().next()),
            _ => Ok(None),
        }
    }

    async fn get_node(&self, node_id: u64) -> Result<Node, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{NODES_BASE}/{node_id}");

        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .get(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        Node::from_response(response).await
    }

    async fn search_nodes(
        &self,
        search_string: &str,
        parent_id: Option<u64>,
        depth_level: Option<i8>,
        params: Option<ListAllParams>,
    ) -> Result<NodeList, DracoonClientError> {
        let params = params.unwrap_or_default();
        let url_part = format!("/{DRACOON_API_PREFIX}/{NODES_BASE}/{NODES_SEARCH}");

        let mut api_url = self.build_api_url(&url_part);

        let filters = params.filter_to_string();
        let sorts = params.sort_to_string();

        api_url
            .query_pairs_mut()
            .append_pair("search_string", search_string)
            .extend_pairs(depth_level.map(|v| ("depth_level", v.to_string())))
            .extend_pairs(params.limit.map(|v| ("limit", v.to_string())))
            .extend_pairs(params.offset.map(|v| ("offset", v.to_string())))
            .extend_pairs(params.sort.map(|_| ("sort", sorts)))
            .extend_pairs(params.filter.map(|_| ("filter", filters)))
            .extend_pairs(parent_id.map(|v| ("parent_id", v.to_string())))
            .finish();

        let response = self
            .client
            .http
            .get(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        NodeList::from_response(response).await
    }

    async fn delete_node(&self, node_id: u64) -> Result<(), DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{NODES_BASE}/{node_id}");

        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .delete(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        if response.status().is_server_error() || response.status().is_client_error() {
            return Err(DracoonClientError::from_response(response)
                .await
                .expect("Could not parse error response"));
        }

        Ok(())
    }

    async fn delete_nodes(&self, req: DeleteNodesRequest) -> Result<(), DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{NODES_BASE}");

        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .delete(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&req)
            .send()
            .await?;

        if response.status().is_server_error() || response.status().is_client_error() {
            return Err(DracoonClientError::from_response(response)
                .await
                .expect("Could not parse error response"));
        }

        Ok(())
    }

    async fn move_nodes(
        &self,
        req: TransferNodesRequest,
        target_parent_id: u64,
    ) -> Result<Node, DracoonClientError> {
        let url_part =
            format!("/{DRACOON_API_PREFIX}/{NODES_BASE}/{target_parent_id}/{NODES_MOVE}");

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

    async fn copy_nodes(
        &self,
        req: TransferNodesRequest,
        target_parent_id: u64,
    ) -> Result<Node, DracoonClientError> {
        let url_part =
            format!("/{DRACOON_API_PREFIX}/{NODES_BASE}/{target_parent_id}/{NODES_COPY}");

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

    async fn distribute_missing_keys(
        &self,
        room_id: Option<u64>,
        file_id: Option<u64>,
        user_id: Option<u64>,
    ) -> Result<u64, DracoonClientError> {
        let keypair = self.get_keypair(None).await?;

        let missing_keys = self
            .get_missing_file_keys(room_id, file_id, user_id, None)
            .await?;

        let remaining_keys = if missing_keys.range.is_none() {
            0
        } else {
            missing_keys.range.as_ref().unwrap().total
        };

        let key_reqs =
            UserFileKeySetBatchRequest::try_new_from_missing_keys(missing_keys, &keypair)?;

        if !key_reqs.is_empty() {
            self.set_file_keys(key_reqs).await?;
        }

        Ok(remaining_keys)
    }
}

#[async_trait]
trait NodesMissingFileKeysInternal {
    async fn get_missing_file_keys(
        &self,
        room_id: Option<u64>,
        file_id: Option<u64>,
        user_id: Option<u64>,
        params: Option<ListAllParams>,
    ) -> Result<MissingKeysResponse, DracoonClientError>;

    async fn set_file_keys(
        &self,
        req: UserFileKeySetBatchRequest,
    ) -> Result<(), DracoonClientError>;
}

#[async_trait]
impl NodesMissingFileKeysInternal for Dracoon<Connected> {
    async fn get_missing_file_keys(
        &self,
        room_id: Option<u64>,
        file_id: Option<u64>,
        user_id: Option<u64>,
        params: Option<ListAllParams>,
    ) -> Result<MissingKeysResponse, DracoonClientError> {
        let params = params.unwrap_or_default();
        let url_part = format!("{DRACOON_API_PREFIX}/{NODES_BASE}/{MISSING_FILE_KEYS}");

        let limit = params.limit.unwrap_or(100);

        let mut api_url = self.build_api_url(&url_part);

        let sorts = params.sort_to_string();

        api_url
            .query_pairs_mut()
            .extend_pairs(Some(("limit", limit.to_string())))
            .extend_pairs(params.offset.map(|v| ("offset", v.to_string())))
            .extend_pairs(params.sort.map(|_| ("sort", sorts)))
            .extend_pairs(user_id.map(|id| ("user_id", id.to_string())))
            .extend_pairs(room_id.map(|id| ("room_id", id.to_string())))
            .extend_pairs(file_id.map(|id| ("file_id", id.to_string())))
            .finish();

        let response = self
            .client
            .http
            .get(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .send()
            .await?;

        MissingKeysResponse::from_response(response).await
    }

    async fn set_file_keys(
        &self,
        req: UserFileKeySetBatchRequest,
    ) -> Result<(), DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{NODES_BASE}/{FILES_BASE}/{FILES_KEYS}");

        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .post(api_url)
            .header(header::AUTHORIZATION, self.get_auth_header().await?)
            .json(&req)
            .send()
            .await?;

        if response.status().is_server_error() || response.status().is_client_error() {
            return Err(DracoonClientError::from_response(response)
                .await
                .expect("Could not parse error response"));
        }

        Ok(())
    }
}

type ParsedPath = (String, String, u64);
pub fn parse_node_path(path: &str) -> Result<ParsedPath, DracoonClientError> {
    if path == "/" {
        return Ok((String::from("/"), String::new(), 0));
    }

    let path_parts: Vec<&str> = path.trim_end_matches('/').split('/').collect();
    let name = String::from(
        *path_parts
            .last()
            .ok_or(DracoonClientError::InvalidPath(path.to_string()))?,
    );
    let parent_path = format!("{}/", path_parts[..path_parts.len() - 1].join("/"));
    let depth = path_parts.len().saturating_sub(2) as u64;

    Ok((parent_path, name, depth))
}

#[cfg(test)]
mod tests {
    use dco3_crypto::{FileKeyVersion, UserKeyPairVersion};

    use crate::tests::dracoon::get_connected_client;

    use super::*;

    #[test]
    fn test_parse_folder_path() {
        let path = "/test/folder/";
        let (parent_path, name, depth) = parse_node_path(path).unwrap();
        assert_eq!("/test/", parent_path);
        assert_eq!("folder", name);
        assert_eq!(1, depth);
    }

    #[test]
    fn test_parse_folder_path_deep() {
        let path = "/test/folder/sub1/";
        let (parent_path, name, depth) = parse_node_path(path).unwrap();
        assert_eq!("/test/folder/", parent_path);
        assert_eq!("sub1", name);
        assert_eq!(2, depth);
    }

    #[test]
    fn test_parse_folder_path_deeper() {
        let path = "/test/folder/sub1/sub2/sub3/";
        let (parent_path, name, depth) = parse_node_path(path).unwrap();
        assert_eq!("/test/folder/sub1/sub2/", parent_path);
        assert_eq!("sub3", name);
        assert_eq!(4, depth);
    }

    #[test]
    fn test_parse_folder_path_no_trail_slash() {
        let path = "/test/folder";
        let (parent_path, name, depth) = parse_node_path(path).unwrap();
        assert_eq!("/test/", parent_path);
        assert_eq!("folder", name);
        assert_eq!(1, depth);
    }

    #[test]
    fn test_file_path() {
        let path = "/test/folder/file.txt";
        let (parent_path, name, depth) = parse_node_path(path).unwrap();
        assert_eq!("/test/folder/", parent_path);
        assert_eq!("file.txt", name);
        assert_eq!(2, depth);
    }

    #[test]
    fn test_root_path() {
        let path = "/";
        let (parent_path, name, depth) = parse_node_path(path).unwrap();
        assert_eq!("/", parent_path);
        assert_eq!("", name);
        assert_eq!(0, depth);
    }

    #[tokio::test]
    async fn test_get_missing_file_keys() {
        let (client, mut mock_server) = get_connected_client().await;

        let missing_keys_res = include_str!("../tests/responses/nodes/missing_file_keys_ok.json");

        let missing_keys_mock = mock_server
            .mock("GET", "/api/v4/nodes/missingFileKeys?limit=100&offset=0")
            .with_body(missing_keys_res)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let missing_keys = client
            .get_missing_file_keys(None, None, None, None)
            .await
            .unwrap();

        missing_keys_mock.assert();

        assert_eq!(missing_keys.range.unwrap().total, 1);
        assert_eq!(missing_keys.items.len(), 1);
        assert_eq!(missing_keys.users.len(), 1);
        assert_eq!(missing_keys.files.len(), 1);

        let item = missing_keys.items.first().unwrap();
        let user = missing_keys.users.first().unwrap();
        let file = missing_keys.files.first().unwrap();

        assert_eq!(item.file_id, 3);
        assert_eq!(item.user_id, 2);
        assert_eq!(user.id, 2);
        assert_eq!(file.id, 3);
        assert_eq!(
            file.file_key_container.version,
            FileKeyVersion::RSA4096_AES256GCM
        );
        assert_eq!(file.file_key_container.key, "string");
        assert_eq!(file.file_key_container.iv, "string");
        assert_eq!(file.file_key_container.tag.as_ref().unwrap(), "string");
        assert_eq!(
            user.public_key_container.version,
            UserKeyPairVersion::RSA4096
        );

        assert_eq!(item.user_id, user.id);
        assert_eq!(item.file_id, file.id);
    }

    #[tokio::test]
    async fn test_get_missing_file_keys_with_file_id() {
        let (client, mut mock_server) = get_connected_client().await;

        let missing_keys_res = include_str!("../tests/responses/nodes/missing_file_keys_ok.json");

        let missing_keys_mock = mock_server
            .mock(
                "GET",
                "/api/v4/nodes/missingFileKeys?limit=100&offset=0&file_id=123",
            )
            .with_body(missing_keys_res)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let missing_keys = client
            .get_missing_file_keys(None, Some(123), None, None)
            .await
            .unwrap();

        missing_keys_mock.assert();

        assert_eq!(missing_keys.range.unwrap().total, 1);
        assert_eq!(missing_keys.items.len(), 1);
        assert_eq!(missing_keys.users.len(), 1);
        assert_eq!(missing_keys.files.len(), 1);

        let item = missing_keys.items.first().unwrap();
        let user = missing_keys.users.first().unwrap();
        let file = missing_keys.files.first().unwrap();

        assert_eq!(item.file_id, 3);
        assert_eq!(item.user_id, 2);
        assert_eq!(user.id, 2);
        assert_eq!(file.id, 3);
        assert_eq!(
            file.file_key_container.version,
            FileKeyVersion::RSA4096_AES256GCM
        );
        assert_eq!(file.file_key_container.key, "string");
        assert_eq!(file.file_key_container.iv, "string");
        assert_eq!(file.file_key_container.tag.as_ref().unwrap(), "string");
        assert_eq!(
            user.public_key_container.version,
            UserKeyPairVersion::RSA4096
        );

        assert_eq!(item.user_id, user.id);
        assert_eq!(item.file_id, file.id);
    }

    #[tokio::test]
    async fn test_get_missing_file_keys_with_room_id() {
        let (client, mut mock_server) = get_connected_client().await;

        let missing_keys_res = include_str!("../tests/responses/nodes/missing_file_keys_ok.json");

        let missing_keys_mock = mock_server
            .mock(
                "GET",
                "/api/v4/nodes/missingFileKeys?limit=100&offset=0&room_id=123",
            )
            .with_body(missing_keys_res)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let missing_keys = client
            .get_missing_file_keys(Some(123), None, None, None)
            .await
            .unwrap();

        missing_keys_mock.assert();

        assert_eq!(missing_keys.range.unwrap().total, 1);
        assert_eq!(missing_keys.items.len(), 1);
        assert_eq!(missing_keys.users.len(), 1);
        assert_eq!(missing_keys.files.len(), 1);

        let item = missing_keys.items.first().unwrap();
        let user = missing_keys.users.first().unwrap();
        let file = missing_keys.files.first().unwrap();

        assert_eq!(item.file_id, 3);
        assert_eq!(item.user_id, 2);
        assert_eq!(user.id, 2);
        assert_eq!(file.id, 3);
        assert_eq!(
            file.file_key_container.version,
            FileKeyVersion::RSA4096_AES256GCM
        );
        assert_eq!(file.file_key_container.key, "string");
        assert_eq!(file.file_key_container.iv, "string");
        assert_eq!(file.file_key_container.tag.as_ref().unwrap(), "string");
        assert_eq!(
            user.public_key_container.version,
            UserKeyPairVersion::RSA4096
        );

        assert_eq!(item.user_id, user.id);
        assert_eq!(item.file_id, file.id);
    }

    #[tokio::test]
    async fn test_get_missing_file_keys_with_user_id() {
        let (client, mut mock_server) = get_connected_client().await;

        let missing_keys_res = include_str!("../tests/responses/nodes/missing_file_keys_ok.json");

        let missing_keys_mock = mock_server
            .mock(
                "GET",
                "/api/v4/nodes/missingFileKeys?limit=100&offset=0&user_id=123",
            )
            .with_body(missing_keys_res)
            .with_header("content-type", "application/json")
            .with_status(200)
            .create();

        let missing_keys = client
            .get_missing_file_keys(None, None, Some(123), None)
            .await
            .unwrap();

        missing_keys_mock.assert();

        assert_eq!(missing_keys.range.unwrap().total, 1);
        assert_eq!(missing_keys.items.len(), 1);
        assert_eq!(missing_keys.users.len(), 1);
        assert_eq!(missing_keys.files.len(), 1);

        let item = missing_keys.items.first().unwrap();
        let user = missing_keys.users.first().unwrap();
        let file = missing_keys.files.first().unwrap();

        assert_eq!(item.file_id, 3);
        assert_eq!(item.user_id, 2);
        assert_eq!(user.id, 2);
        assert_eq!(file.id, 3);
        assert_eq!(
            file.file_key_container.version,
            FileKeyVersion::RSA4096_AES256GCM
        );
        assert_eq!(file.file_key_container.key, "string");
        assert_eq!(file.file_key_container.iv, "string");
        assert_eq!(file.file_key_container.tag.as_ref().unwrap(), "string");
        assert_eq!(
            user.public_key_container.version,
            UserKeyPairVersion::RSA4096
        );

        assert_eq!(item.user_id, user.id);
        assert_eq!(item.file_id, file.id);
    }
}
