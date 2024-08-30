use async_trait::async_trait;

#[allow(clippy::module_inception)]
mod groups;
mod models;

pub use models::*;

use crate::{models::ListAllParams, DracoonClientError};

/// This trait provides all methods to manage groups.
/// All sorting and filtering is implemented and can be found using respective '*Filter` or
/// '*SortBy' enums.
///
#[async_trait]
pub trait Groups {
    /// Get a list of all groups.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Groups, groups::{GroupsFilter, GroupsSortBy}, models::{ListAllParams, SortOrder}};
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
    /// // Params are optional
    /// let params = ListAllParams::builder()
    ///     .with_filter(GroupsFilter::name_contains("test"))
    ///     .with_sort(GroupsSortBy::name(SortOrder::Desc))
    ///     .build();
    /// // pass None if you don't want to use any params
    /// let groups = dracoon.groups().get_groups(Some(params)).await.unwrap();
    ///
    /// # }
    /// ```
    async fn get_groups(
        &self,
        params: Option<ListAllParams>,
    ) -> Result<GroupList, DracoonClientError>;
    /// Create a group.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Groups, groups::{CreateGroupRequest}};
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
    /// // There's no builder - use new and set the required field
    /// let group = CreateGroupRequest::new("test_group", None);
    /// let group = dracoon.groups().create_group(group).await.unwrap();
    /// # }
    /// ```
    async fn create_group(&self, group: CreateGroupRequest) -> Result<Group, DracoonClientError>;
    /// Get a group by id.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Groups};
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
    /// let group = dracoon.groups().get_group(123).await.unwrap();
    /// # }
    /// ```
    async fn get_group(&self, group_id: u64) -> Result<Group, DracoonClientError>;
    /// Update a group by id.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Groups, groups::{UpdateGroupRequest}};
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
    /// // There's no builder - use relevant methods to set the fields (name or expiration)
    /// let update = UpdateGroupRequest::name("new_name");
    /// let group = dracoon.groups().update_group(123, update).await.unwrap();
    /// # }
    /// ```
    async fn update_group(
        &self,
        group_id: u64,
        group: UpdateGroupRequest,
    ) -> Result<Group, DracoonClientError>;
    /// Delete a group by id.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Groups};
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
    /// dracoon.groups().delete_group(123).await.unwrap();
    /// # }
    /// ```
    async fn delete_group(&self, group_id: u64) -> Result<(), DracoonClientError>;
    /// Get a list of all groups.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Groups, groups::{GroupUsersFilter}, models::{ListAllParams, SortOrder}};
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
    /// // Params are optional
    /// let params = ListAllParams::builder()
    ///     .with_filter(GroupUsersFilter::user_contains("test"))
    ///     .build();
    /// // pass None if you don't want to use any params
    /// let groups = dracoon.groups().get_group_users(123, Some(params)).await.unwrap();
    ///
    /// # }
    /// ```
    async fn get_group_users(
        &self,
        group_id: u64,
        params: Option<ListAllParams>,
    ) -> Result<GroupUserList, DracoonClientError>;
    /// Add users to a group.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Groups, groups::{ChangeGroupMembersRequest}};
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
    /// // you can pass a vec and use into()
    /// let user_ids = vec![1, 2, 3];
    /// dracoon.groups().add_group_users(123, user_ids.into()).await.unwrap();
    /// // or use the ChangeGroupMembersRequest
    /// let user_ids = ChangeGroupMembersRequest::new(vec![1, 2, 3]);
    /// dracoon.groups().add_group_users(123, user_ids).await.unwrap();
    /// # }
    /// ```
    async fn add_group_users(
        &self,
        group_id: u64,
        user_ids: ChangeGroupMembersRequest,
    ) -> Result<Group, DracoonClientError>;
    /// Remove users from a group.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Groups, groups::{ChangeGroupMembersRequest}};
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
    /// // you can pass a vec and use into()
    /// let user_ids = vec![1, 2, 3];
    /// dracoon.groups().add_group_users(123, user_ids.into()).await.unwrap();
    /// // or use the ChangeGroupMembersRequest
    /// let user_ids = ChangeGroupMembersRequest::new(vec![1, 2, 3]);
    /// dracoon.groups().remove_group_users(123, user_ids).await.unwrap();
    /// # }
    /// ```
    async fn remove_group_users(
        &self,
        group_id: u64,
        user_ids: ChangeGroupMembersRequest,
    ) -> Result<Group, DracoonClientError>;
    /// Get group last admin rooms
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Groups};
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
    /// let rooms = dracoon.groups().get_group_last_admin_rooms(123).await.unwrap();
    /// # }
    /// ```
    async fn get_group_last_admin_rooms(
        &self,
        group_id: u64,
    ) -> Result<LastAdminGroupRoomList, DracoonClientError>;
}
