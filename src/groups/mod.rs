use async_trait::async_trait;


mod groups;
mod models;

pub use models::*;

use crate::{DracoonClientError, models::ListAllParams};

#[async_trait] 
pub trait Groups {

    async fn get_groups(&self, params: Option<ListAllParams>) -> Result<GroupList, DracoonClientError>;

    async fn create_group(&self, group: CreateGroupRequest) -> Result<Group, DracoonClientError>;

    async fn get_group(&self, group_id: u64) -> Result<Group, DracoonClientError>;

    async fn update_group(&self, group_id: u64, group: UpdateGroupRequest) -> Result<Group, DracoonClientError>;

    async fn delete_group(&self, group_id: u64) -> Result<(), DracoonClientError>;

    async fn get_group_users(&self, group_id: u64, params: Option<ListAllParams>) -> Result<GroupUserList, DracoonClientError>;

    async fn add_group_users(&self, group_id: u64, user_ids: ChangeGroupMembersRequest) -> Result<Group, DracoonClientError>;

    async fn remove_group_users(&self, group_id: u64, user_ids: ChangeGroupMembersRequest) -> Result<Group, DracoonClientError>;

    async fn get_group_last_admin_rooms(&self, group_id: u64) -> Result<LastAdminGroupRoomList, DracoonClientError>;

}