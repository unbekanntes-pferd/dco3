//! This module implements a subset of the nodes DRACOON API.
//! Documentation can be found here: <https://download.dracoon.com/api/swagger-ui/index.html?configUrl=/api/spec_v4/swagger-config#/nodes>
pub use self::{models::*, rooms::models::*};
use super::{client::errors::DracoonClientError, models::ListAllParams};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

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
    /// Returns a list of nodes.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Nodes, nodes::{NodesFilter, NodesSortBy}, models::{ListAllParams, SortOrder}};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let nodes = dracoon.nodes().get_nodes(None, None, None).await.unwrap();
    ///
    /// // get all nodes for a parent
    /// let nodes = dracoon.nodes().get_nodes(Some(123), None, None).await.unwrap();
    ///
    /// // get all nodes visible as room manager / admin
    /// let nodes = dracoon.nodes().get_nodes(None, Some(true), None).await.unwrap();
    ///
    /// // use filtering and sorting
    /// let params = ListAllParams::builder()
    ///    .with_filter(NodesFilter::is_file())
    ///    .with_filter(NodesFilter::name_contains("foo"))
    ///    .with_sort(NodesSortBy::name(SortOrder::Desc))
    ///    .build();
    ///
    /// let nodes = dracoon.nodes().get_nodes(None, None, Some(params)).await.unwrap();
    /// # }
    /// ```
    async fn get_nodes(
        &self,
        parent_id: Option<u64>,
        room_manager: Option<bool>,
        params: Option<ListAllParams>,
    ) -> Result<NodeList, DracoonClientError>;
    /// Searches for a node via given path.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Nodes, nodes::{NodesFilter, NodesSortBy}, models::{ListAllParams, SortOrder}};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let node = dracoon.nodes().get_node_from_path("/foo/bar").await.unwrap();
    /// match node {
    ///    Some(node) => println!("Found node: {}", node.name),
    ///    None => println!("Node not found"),
    /// };
    /// # }
    /// ```
    async fn get_node_from_path(&self, path: &str) -> Result<Option<Node>, DracoonClientError>;
    /// Searches for nodes by search string.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Nodes, nodes::{NodesSearchFilter, NodesSearchSortBy}, models::{ListAllParams, SortOrder}};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// // search for nodes ("*" is wildcard)
    /// let nodes = dracoon.nodes().search_nodes("foo", None, None, None).await.unwrap();
    ///
    /// // search for nodes in a parent
    /// let nodes = dracoon.nodes().search_nodes("foo", Some(123), None, None).await.unwrap();
    ///
    /// // search for nodes in a parent with a depth level (-1 is full tree)
    /// let nodes = dracoon.nodes().search_nodes("foo", Some(123), Some(1), None).await.unwrap();
    ///
    /// // use filtering and sorting
    /// let params = ListAllParams::builder()
    ///                .with_filter(NodesSearchFilter::is_file())
    ///                .with_filter(NodesSearchFilter::size_greater_equals(100))
    ///                .with_sort(NodesSearchSortBy::name(SortOrder::Desc))
    ///                .build();
    /// let nodes = dracoon.nodes().search_nodes("foo", None, None, Some(params)).await.unwrap();
    /// # }
    /// ```
    async fn search_nodes(
        &self,
        search_string: &str,
        parent_id: Option<u64>,
        depth_level: Option<i8>,
        params: Option<ListAllParams>,
    ) -> Result<NodeList, DracoonClientError>;

    /// Returns a node by id.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Nodes};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let node = dracoon.nodes().get_node(123).await.unwrap();
    /// # }
    /// ```
    async fn get_node(&self, node_id: u64) -> Result<Node, DracoonClientError>;
    /// Deletes a node by id.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Nodes};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// dracoon.nodes().delete_node(123).await.unwrap();
    /// # }
    /// ```
    async fn delete_node(&self, node_id: u64) -> Result<(), DracoonClientError>;
    /// Deletes multiple nodes by ids.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Nodes};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let node_ids = vec![123, 456];
    /// dracoon.nodes().delete_nodes(node_ids.into()).await.unwrap();
    /// # }
    /// ```
    async fn delete_nodes(&self, req: DeleteNodesRequest) -> Result<(), DracoonClientError>;
    /// Move nodes to a target parent node (folder or room).
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Nodes};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let node_ids = vec![123, 456];
    /// dracoon.nodes().move_nodes(node_ids.into(), 789).await.unwrap();
    /// # }
    /// ```
    async fn move_nodes(
        &self,
        req: TransferNodesRequest,
        target_parent_id: u64,
    ) -> Result<Node, DracoonClientError>;
    /// Copy nodes to a target parent node (folder or room).
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Nodes};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let node_ids = vec![123, 456];
    /// dracoon.nodes().copy_nodes(node_ids.into(), 789).await.unwrap();
    /// # }
    /// ```
    async fn copy_nodes(
        &self,
        req: TransferNodesRequest,
        target_parent_id: u64,
    ) -> Result<Node, DracoonClientError>;
}

#[async_trait]
pub trait MissingFileKeys {
    /// Distributes missing file keys using the user keypair.
    /// Returns the total amount missing keys.
    /// If the total amount is larger than 100, more keys need distribution
    /// and the method should be called again.
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, MissingFileKeys};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let mut missing_keys = dracoon.distribute_missing_keys(None, None, None).await.unwrap();
    ///
    /// while missing_keys > 100 {
    /// // loop until no more keys need distribution
    /// missing_keys = dracoon.distribute_missing_keys(None, None, None).await.unwrap();
    /// }
    ///
    /// // distribute missing keys for a specific room
    /// let missing_room_keys = dracoon.distribute_missing_keys(Some(123), None, None).await.unwrap();
    ///
    /// // distribute missing keys for a specific file
    /// let missing_file_keys = dracoon.distribute_missing_keys(None, Some(123), None).await.unwrap();
    ///
    /// // distribute missing keys for a specific user
    /// let missing_user_keys = dracoon.distribute_missing_keys(None, None, Some(123)).await.unwrap();
    ///
    /// # }
    async fn distribute_missing_keys(
        &self,
        room_id: Option<u64>,
        file_id: Option<u64>,
        user_id: Option<u64>,
    ) -> Result<u64, DracoonClientError>;
}

#[async_trait]
pub trait Folders {
    /// Creates a folder in the provided parent room.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Folders, nodes::CreateFolderRequest};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let folder = CreateFolderRequest::builder("My Folder", 123)
    ///                                .with_classification(1)
    ///                                .with_notes("My notes")
    ///                                .build();
    /// let folder = dracoon.nodes().create_folder(folder).await.unwrap();
    /// # }
    /// ```
    async fn create_folder(&self, req: CreateFolderRequest) -> Result<Node, DracoonClientError>;
    /// Updates a folder with given params by id.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Folders, nodes::UpdateFolderRequest};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let update = UpdateFolderRequest::builder()
    ///                              .with_name("My Folder")
    ///                              .with_classification(2)
    ///                              .build();
    /// dracoon.nodes().update_folder(123, update).await.unwrap();
    /// # }
    /// ```
    async fn update_folder(
        &self,
        folder_id: u64,
        req: UpdateFolderRequest,
    ) -> Result<Node, DracoonClientError>;
}
/// This trait provides methods to manage rooms.
///
///  - Create a room
///  - Update a room
///  - Configure a room
///  - Get the policies of a room
///  - Set the policies of a room
///  - Encrypt a room
///  - Get groups of a room
///  - Add groups to a room
///  - Delete groups from a room
///  - Get users of a room
///  - Add users to a room
///  - Delete users from a room
///  
///  To delete a room, use the `delete_node` method from the `Nodes` trait.
#[async_trait]
pub trait Rooms {
    /// Creates a room.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Rooms, nodes::CreateRoomRequest};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let room = CreateRoomRequest::builder("My Room")
    ///                              .with_parent_id(123)
    ///                              .with_classification(1)
    ///                              .build();
    /// let room = dracoon.nodes().create_room(room).await.unwrap();
    /// # }
    /// ```
    async fn create_room(
        &self,
        create_room_req: CreateRoomRequest,
    ) -> Result<Node, DracoonClientError>;
    /// Updates a room by id.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Rooms, nodes::UpdateRoomRequest};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let room = UpdateRoomRequest::builder()
    ///                              .with_name("My new Room")
    ///                              .build();
    /// let room = dracoon.nodes().update_room(123, room).await.unwrap();
    /// # }
    /// ```
    async fn update_room(
        &self,
        room_id: u64,
        update_room_req: UpdateRoomRequest,
    ) -> Result<Node, DracoonClientError>;
    /// Configures a room by id.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Rooms, nodes::ConfigRoomRequest};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let room = ConfigRoomRequest::builder()
    ///                              .with_inherit_permissions(true)
    ///                              .with_recycle_bin_retention_period(30)
    ///                              .build();
    /// let room = dracoon.nodes().config_room(123, room).await.unwrap();
    /// # }
    /// ```
    async fn config_room(
        &self,
        room_id: u64,
        config_room_req: ConfigRoomRequest,
    ) -> Result<Node, DracoonClientError>;
    /// Gets the policies of a room by id.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Rooms, nodes::RoomPolicies};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    ///  let policies = dracoon.nodes().get_room_policies(123).await.unwrap();
    /// # }
    /// ```
    async fn get_room_policies(&self, room_id: u64) -> Result<RoomPolicies, DracoonClientError>;
    /// Set the policies of a room by id.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Rooms, nodes::RoomPoliciesRequest};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let new_policies = RoomPoliciesRequest::builder()
    ///                            .with_default_expiration_period(60 * 60 * 24 * 30)
    ///                            .with_virus_protection_enabled(true)
    ///                            .build();
    /// dracoon.nodes().update_room_policies(123, new_policies).await.unwrap();
    /// # }
    /// ```
    //
    async fn update_room_policies(
        &self,
        room_id: u64,
        policy_room_req: RoomPoliciesRequest,
    ) -> Result<(), DracoonClientError>;
    /// Encrypts a room by id.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Rooms, nodes::EncryptRoomRequest};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let encryption = EncryptRoomRequest::builder(true)
    ///                           .try_with_data_room_rescue_key("Secret123")
    ///                           .unwrap()
    ///                           .build();
    /// let room = dracoon.nodes().encrypt_room(123, encryption).await.unwrap();
    /// # }
    /// ```
    async fn encrypt_room(
        &self,
        room_id: u64,
        encrypt_room_req: EncryptRoomRequest,
    ) -> Result<Node, DracoonClientError>;
    /// Gets groups of a room by id with optional params.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Rooms};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let groups = dracoon.nodes().get_room_groups(123, None).await.unwrap();
    /// # }
    /// ```
    async fn get_room_groups(
        &self,
        room_id: u64,
        params: Option<ListAllParams>,
    ) -> Result<RoomGroupList, DracoonClientError>;

    /// Updates room groups by id.
    /// Gets groups of a room by id with optional params.
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, Rooms, nodes::{RoomGroupsAddBatchRequestItem, NodePermissions}};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    ///
    /// // add a a list of updates
    /// let group_updates = vec![RoomGroupsAddBatchRequestItem::new(123, NodePermissions::new_with_read_permissions(), None)];
    /// dracoon.nodes().update_room_groups(123, group_updates.into()).await.unwrap();
    /// # }
    /// ```
    async fn update_room_groups(
        &self,
        room_id: u64,
        room_groups_update_req: RoomGroupsAddBatchRequest,
    ) -> Result<(), DracoonClientError>;

    /// Deletes room groups by id.
    /// Gets groups of a room by id with optional params.
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, Rooms};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// // You can use a vec
    /// let group_ids = vec![1, 2, 3];
    /// dracoon.nodes().delete_room_groups(123, group_ids.into()).await.unwrap();
    /// # }
    /// ```
    async fn delete_room_groups(
        &self,
        room_id: u64,
        room_groups_del_req: RoomGroupsDeleteBatchRequest,
    ) -> Result<(), DracoonClientError>;

    /// Gets users of a room by id with optional params.
    /// Gets groups of a room by id with optional params.
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, Rooms};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let users = dracoon.nodes().get_room_users(123, None).await.unwrap();
    /// # }
    /// ```
    async fn get_room_users(
        &self,
        room_id: u64,
        params: Option<ListAllParams>,
    ) -> Result<RoomUserList, DracoonClientError>;

    /// Updates room users by id.
    /// Gets groups of a room by id with optional params.
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, Rooms, nodes::{RoomUsersAddBatchRequestItem, NodePermissions}};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    ///
    /// // add a a list of updates
    /// let user_updates = vec![RoomUsersAddBatchRequestItem::new(123, NodePermissions::new_with_read_permissions())];
    /// dracoon.nodes().update_room_users(123, user_updates.into()).await.unwrap();
    /// # }
    /// ```
    async fn update_room_users(
        &self,
        room_id: u64,
        room_users_update_req: RoomUsersAddBatchRequest,
    ) -> Result<(), DracoonClientError>;
    /// Deletes room users by id.
    /// Gets groups of a room by id with optional params.
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, Rooms};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// // You can use a vec
    /// let user_ids = vec![1, 2, 3];
    /// dracoon.nodes().delete_room_users(123, user_ids.into()).await.unwrap();
    /// # }
    /// ```
    async fn delete_room_users(
        &self,
        room_id: u64,
        room_users_del_req: RoomUsersDeleteBatchRequest,
    ) -> Result<(), DracoonClientError>;
    /// Deletes room users by id.
    /// Gets groups of a room by id with optional params.
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, Rooms, nodes::RoomGuestUserInvitation};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// // You can use a vec
    /// let guest = RoomGuestUserInvitation::new("email", "first name", "last name");
    /// dracoon.nodes().invite_guest_users(123, vec![guest].into()).await.unwrap();
    /// # }
    /// ```
    async fn invite_guest_users(
        &self,
        room_id: u64,
        invite_req: RoomGuestUserAddRequest,
    ) -> Result<(), DracoonClientError>;
}
/// This trait represents the download functionality and provides
/// a signle method to download a stream of bytes to a writer.
/// This rquires a mutable reference to the client because the download method
/// needs to be able to check for the secret and set it for the client if encryption is used.
#[async_trait]
pub trait Download {
    /// Downloads a file (node) to the given writer buffer
    /// Example
    /// ```no_run
    /// use dco3::{Dracoon, OAuth2Flow, Download, Nodes};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///    let client = Dracoon::builder()
    ///      .with_base_url("https://dracoon.team")
    ///      .with_client_id("client_id")
    ///      .with_client_secret("client_secret")
    ///      // if you do not pass a password, encrypted uploads will fail with an error
    ///      .with_encryption_password("encryption_password")
    ///      .build()
    ///      .unwrap()
    ///      .connect(OAuth2Flow::password_flow("username", "password"))
    ///      .await
    ///      .unwrap();
    ///
    ///   let node_id = 123u64;
    ///
    ///   let node = client.nodes().get_node(node_id).await.unwrap();
    ///
    ///   let mut writer = tokio::io::BufWriter::new(tokio::fs::File::create("test.txt").await.unwrap());
    ///
    ///   client.download(&node, &mut writer, None, None).await.unwrap();
    ///
    ///   // or with progress callback (boxed closure)
    ///   client.download(&node, &mut writer, Some(Box::new(|progress, total| {
    ///    println!("Download progress: {}", progress);
    ///    println!("File total: {}", total);
    ///  })), None).await.unwrap();
    /// // or with chunksize
    /// let  chunksize: usize = 1024 * 1024 * 10;
    /// client.download(&node, &mut writer, None, Some(chunksize)).await.unwrap();
    /// }
    /// ```
    async fn download<'w>(
        &'w self,
        node: &Node,
        writer: &'w mut (dyn AsyncWrite + Send + Unpin),
        mut callback: Option<DownloadProgressCallback>,
        chunksize: Option<usize>,
    ) -> Result<(), DracoonClientError>;
}

/// This trait represents the upload functionality and provides
/// a single method to upload a stream of bytes by passing a buffered reader.
/// This rquires a mutable reference to the client because the upload method
/// needs to be able to check for the secret and set it for the client if encryption is used.
#[async_trait]
pub trait Upload<R: AsyncRead> {
    /// Uploads a stream (buffered reader) with given file meta info to the given parent node
    /// # Example
    /// ```no_run
    /// use dco3::{Dracoon, OAuth2Flow, Upload, Nodes, nodes::{FileMeta, UploadOptions, ResolutionStrategy}};
    /// #[cfg(not(doctest))]
    /// #[tokio::main]
    /// async fn main() {
    ///    let client = Dracoon::builder()
    ///      .with_base_url("https://dracoon.team")
    ///      .with_client_id("client_id")
    ///      .with_client_secret("client_secret")
    ///      // if you do not pass a password, encrypted uploads will fail with an error
    ///      .with_encryption_password("encryption_password")
    ///      .build()
    ///      .unwrap()
    ///      .connect(OAuth2Flow::password_flow("username", "password"))
    ///      .await
    ///      .unwrap();
    ///
    /// let file = tokio::fs::File::open("test.txt").await.unwrap();
    /// let file_meta = FileMeta::builder("test.txt", 123456)
    /// .with_timestamp_modification("2020-01-01T00:00:00.000Z".parse().unwrap())
    /// .build();
    ///
    ///
    /// let parent_node_id = 123u64;
    ///
    /// let parent_node = client.nodes().get_node(parent_node_id).await.unwrap();
    ///
    /// let reader = tokio::io::BufReader::new(file);
    ///
    /// let options = UploadOptions::builder(file_meta)
    ///               .with_resolution_strategy(ResolutionStrategy::AutoRename)
    ///               .build();
    ///
    /// let chunk_size = 1024 * 1024 * 10; // 10 MB - DEFAULT is 32 MB
    ///
    /// client.upload(&parent_node, options, reader, None, Some(chunk_size)).await.unwrap();
    ///
    /// // or with progress callback (boxed closure)
    /// let file = tokio::fs::File::open("test.txt").await.unwrap();
    /// let file_meta = FileMeta::builder("test.txt", 123456)
    /// .with_timestamp_modification("2020-01-01T00:00:00.000Z".parse().unwrap())
    /// .build();
    /// let options = UploadOptions::builder(file_meta)
    ///               .with_resolution_strategy(ResolutionStrategy::AutoRename)
    ///               .build();
    /// let reader = tokio::io::BufReader::new(file);
    /// client.upload(&parent_node, options, reader, Some(Box::new(|progress, total| {  
    ///   println!("Upload progress: {}", progress);
    ///  println!("File total: {}", total);
    /// })), Some(chunk_size)).await.unwrap();
    /// }
    /// ```
    ///
    ///
    async fn upload<'r>(
        &'r self,
        parent_node: &Node,
        upload_options: UploadOptions,
        mut reader: BufReader<R>,
        mut callback: Option<UploadProgressCallback>,
        chunk_size: Option<usize>,
    ) -> Result<Node, DracoonClientError>;
}
