use self::{
    models::{
        CreateFolderRequest, DownloadProgressCallback, FileMeta, Node, NodeList,
        TransferNodesRequest, UpdateFolderRequest, UploadOptions, UploadProgressCallback,
    },
    rooms::models::{
        ConfigRoomRequest, CreateRoomRequest, EncryptRoomRequest, RoomGroupList,
        RoomGroupsAddBatchRequest, RoomGroupsDeleteBatchRequest, RoomUserList,
        RoomUsersAddBatchRequest, RoomUsersDeleteBatchRequest, UpdateRoomRequest,
    },
};
use super::{auth::errors::DracoonClientError, models::ListAllParams};
use async_trait::async_trait;
use std::io::Write;
use tokio::io::{AsyncRead, BufReader};

pub mod download;
pub mod folders;
pub mod models;
pub mod nodes;
pub mod rooms;
pub mod upload;

/// This trait provides methods to manage nodes.
/// Specifically, there's a method to obtain a node for a given path and 
/// all relevant methods to list nodes (get, search), move, copy and deleted nodes.
/// 
/// To download a node, use the [Download] trait.
/// To upload a node, use the [Upload] trait.
/// To manage rooms, use the [Rooms] trait.
/// To manage folders, use the [Folders] trait.
#[async_trait]
pub trait Nodes {
    /// Returns a list of nodes
    async fn get_nodes(
        &self,
        parent_id: Option<u64>,
        room_manager: Option<bool>,
        params: Option<ListAllParams>,
    ) -> Result<NodeList, DracoonClientError>;

    /// Searches for a node by path
    /// Returns the node if found (or None if not found)
    async fn get_node_from_path(&self, path: &str) -> Result<Option<Node>, DracoonClientError>;

    /// Searches for nodes by search string
    async fn search_nodes(
        &self,
        search_string: &str,
        parent_id: Option<u64>,
        depth_level: Option<i8>,
        params: Option<ListAllParams>,
    ) -> Result<NodeList, DracoonClientError>;

    /// Returns a node by id
    async fn get_node(&self, node_id: u64) -> Result<Node, DracoonClientError>;

    /// Deletes a node by id
    async fn delete_node(&self, node_id: u64) -> Result<(), DracoonClientError>;

    /// Deletes multiple nodes by ids
    async fn delete_nodes(&self, node_ids: Vec<u64>) -> Result<(), DracoonClientError>;

    /// Move nodes to a target parent node (folder or room)
    async fn move_nodes(
        &self,
        req: TransferNodesRequest,
        target_parent_id: u64,
    ) -> Result<Node, DracoonClientError>;

    /// Copy nodes to a target parent node (folder or room)
    async fn copy_nodes(
        &self,
        req: TransferNodesRequest,
        target_parent_id: u64,
    ) -> Result<Node, DracoonClientError>;
}

#[async_trait]
pub trait Folders {
    /// Creates a folder in the provided parent room
    async fn create_folder(&self, req: CreateFolderRequest) -> Result<Node, DracoonClientError>;

    /// Updates a folder with given params by id
    async fn update_folder(
        &self,
        folder_id: u64,
        req: UpdateFolderRequest,
    ) -> Result<Node, DracoonClientError>;
}

/// This trait provides methods to manage rooms
/// 
///  - Create a room
///  - Update a room
///  - Configure a room
///  - Encrypt a room
///  - Get groups of a room
///  - Add groups to a room
///  - Delete groups from a room
///  - Get users of a room
///  - Add users to a room
///  - Delete users from a room
///  
///  To delete a room, use the `delete_node` method from the `Nodes` trait
#[async_trait]
pub trait Rooms {
    /// Creates a room
    async fn create_room(
        &self,
        create_room_req: CreateRoomRequest,
    ) -> Result<Node, DracoonClientError>;

    /// Updates a room by id
    async fn update_room(
        &self,
        room_id: u64,
        update_room_req: UpdateRoomRequest,
    ) -> Result<Node, DracoonClientError>;

    /// Configures a room by id
    async fn config_room(
        &self,
        room_id: u64,
        config_room_req: ConfigRoomRequest,
    ) -> Result<Node, DracoonClientError>;

    /// Encrypts a room by id
    async fn encrypt_room(
        &self,
        room_id: u64,
        encrypt_room_req: EncryptRoomRequest,
    ) -> Result<Node, DracoonClientError>;

    /// Gets groups of a room by id with optional params
    async fn get_room_groups(
        &self,
        room_id: u64,
        params: Option<ListAllParams>,
    ) -> Result<RoomGroupList, DracoonClientError>;

    /// Updates room groups by id
    async fn update_room_groups(
        &self,
        room_id: u64,
        room_groups_update_req: RoomGroupsAddBatchRequest,
    ) -> Result<(), DracoonClientError>;

    /// Deletes room groups by id
    async fn delete_room_groups(
        &self,
        room_id: u64,
        room_groups_del_req: RoomGroupsDeleteBatchRequest,
    ) -> Result<(), DracoonClientError>;

    /// Gets users of a room by id with optional params
    async fn get_room_users(
        &self,
        room_id: u64,
        params: Option<ListAllParams>,
    ) -> Result<RoomUserList, DracoonClientError>;

    /// Updates room users by id
    async fn update_room_users(
        &self,
        room_id: u64,
        room_users_update_req: RoomUsersAddBatchRequest,
    ) -> Result<(), DracoonClientError>;

    /// Deletes room users by id
    async fn delete_room_users(
        &self,
        room_id: u64,
        room_users_del_req: RoomUsersDeleteBatchRequest,
    ) -> Result<(), DracoonClientError>;
}

/// This trait represents the download functionality and provides
/// a signle method to download a stream of bytes to a writer
#[async_trait]
pub trait Download {
    /// Downloads a file (node) to the given writer buffer
    /// Example
    /// ```no_run
    /// use dco3::{Dracoon, auth::OAuth2Flow, nodes::{Download, Nodes}};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///    let mut client = Dracoon::builder()
    ///      .with_base_url("https://dracoon.team")
    ///      .with_client_id("client_id")
    ///      .with_client_secret("client_secret")
    ///      .build()
    ///      .unwrap()
    ///      .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    ///      .await
    ///      .unwrap();
    /// 
    ///   let node_id = 123u64;
    ///
    ///   let node = client.get_node(node_id).await.unwrap();
    ///
    ///   let mut writer = std::io::BufWriter::new(std::fs::File::create("test.txt").unwrap());
    ///
    ///   client.download(&node, &mut writer, None).await.unwrap();
    ///
    ///   // or with progress callback (boxed closure)
    ///   client.download(&node, &mut writer, Some(Box::new(|progress, total| {
    ///    println!("Download progress: {}", progress);
    ///    println!("File total: {}", total);
    ///  }))).await.unwrap();
    /// }
    /// ```
    ///
    ///
    async fn download<'w>(
        &'w mut self,
        node: &Node,
        writer: &'w mut (dyn Write + Send),
        mut callback: Option<DownloadProgressCallback>,
    ) -> Result<(), DracoonClientError>;
}

/// This trait represents the upload functionality and provides
/// a single method to upload a stream of bytes by passing a buffered reader
#[async_trait]
pub trait Upload<R: AsyncRead> {
    /// Uploads a stream (buffered reader) with given file meta info to the given parent node
    /// # Example
    /// ```no_run
    /// use dco3::{Dracoon, auth::OAuth2Flow, nodes::{Upload, Nodes, models::{FileMeta, UploadOptions, ResolutionStrategy}}};
    /// #[cfg(not(doctest))]
    /// #[tokio::main]
    /// async fn main() {
    ///    let mut client = Dracoon::builder()
    ///      .with_base_url("https://dracoon.team")
    ///      .with_client_id("client_id")
    ///      .with_client_secret("client_secret")
    ///      .build()
    ///      .unwrap()
    ///      .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    ///      .await
    ///      .unwrap();
    /// 
    /// let file = tokio::fs::File::open("test.txt").await.unwrap();
    /// let file_meta = FileMeta::builder()
    /// .with_name("test.txt".into())
    /// .with_size(123456)
    /// .with_timestamp_modification("2020-01-01T00:00:00.000Z".parse().unwrap())
    /// .build();
    ///
    ///
    /// let parent_node_id = 123u64;
    /// 
    /// let parent_node = client.get_node(parent_node_id).await.unwrap();
    /// 
    /// let reader = tokio::io::BufReader::new(file);
    /// 
    /// let options = UploadOptions::builder()
    ///               .with_resolution_strategy(ResolutionStrategy::AutoRename)
    ///               .build();
    /// 
    /// let chunk_size = 1024 * 1024 * 10; // 10 MB - DEFAULT is 32 MB
    /// 
    /// client.upload(file_meta, &parent_node, options, reader, None, Some(chunk_size)).await.unwrap();
    /// 
    /// // or with progress callback (boxed closure)
    /// let file = tokio::fs::File::open("test.txt").await.unwrap();
    /// let file_meta = FileMeta::builder()
    /// .with_name("test.txt".into())
    /// .with_size(123456)
    /// .with_timestamp_modification("2020-01-01T00:00:00.000Z".parse().unwrap())
    /// .build();
    /// let options = UploadOptions::builder()
    ///               .with_resolution_strategy(ResolutionStrategy::AutoRename)
    ///               .build();
    /// let reader = tokio::io::BufReader::new(file);
    /// client.upload(file_meta, &parent_node, options, reader, Some(Box::new(|progress, total| {  
    ///   println!("Upload progress: {}", progress);
    ///  println!("File total: {}", total);
    /// })), Some(chunk_size)).await.unwrap();
    /// }
    /// ```
    ///
    ///
    async fn upload<'r>(
        &'r mut self,
        file_meta: FileMeta,
        parent_node: &Node,
        upload_options: UploadOptions,
        mut reader: BufReader<R>,
        mut callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<Node, DracoonClientError>;
}
