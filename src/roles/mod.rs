use async_trait::async_trait;

mod models;
#[allow(clippy::module_inception)]
mod roles;

pub use models::*;

use crate::{models::ListAllParams, DracoonClientError};

#[async_trait]
pub trait Roles {
    /// Get a list of all roles.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Roles};
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
    /// let roles = dracoon.roles.get_roles().await.unwrap();
    /// # }
    /// ```
    async fn get_roles(&self) -> Result<RoleList, DracoonClientError>;
    /// Get a list of all groups with a specific role.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Roles, ListAllParams};
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
    ///    .with_offset(0)
    ///   .with_limit(100)
    ///  .build();
    /// let groups = dracoon.roles.get_all_groups_with_role(1, Some(params)).await.unwrap();
    /// # }
    /// ```
    async fn get_all_groups_with_role(
        &self,
        role_id: u64,
        params: Option<ListAllParams>,
    ) -> Result<RoleGroupList, DracoonClientError>;
    /// Assign a role to a list of groups.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Roles};
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
    /// let groups = dracoon.roles.assign_role_to_groups(1, vec![1, 2, 3].into()).await.unwrap();
    /// # }
    /// ```
    async fn assign_role_to_groups(
        &self,
        role_id: u64,
        group_ids: AssignRoleBatchRequest,
    ) -> Result<RoleGroupList, DracoonClientError>;
    /// Revoke a role from a list of groups.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Roles};
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
    /// let groups = dracoon.roles.revoke_role_from_groups(1, vec![1, 2, 3].into()).await.unwrap();
    /// # }
    /// ```
    async fn revoke_role_from_groups(
        &self,
        role_id: u64,
        group_ids: RevokeRoleBatchRequest,
    ) -> Result<RoleGroupList, DracoonClientError>;
    /// Get a list of all users with a specific role.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Roles, ListAllParams};
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
    ///   .with_offset(0)
    /// .with_limit(100)
    /// .build();
    /// let users = dracoon.roles.get_all_users_with_role(1, Some(params)).await.unwrap();
    /// # }
    /// ```
    async fn get_all_users_with_role(
        &self,
        role_id: u64,
        params: Option<ListAllParams>,
    ) -> Result<RoleUserList, DracoonClientError>;
    /// Assign a role to a list of users.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Roles};
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
    /// let users = dracoon.roles.assign_role_to_users(1, vec![1, 2, 3].into()).await.unwrap();
    /// # }
    /// ```
    async fn assign_role_to_users(
        &self,
        role_id: u64,
        user_ids: AssignRoleBatchRequest,
    ) -> Result<RoleUserList, DracoonClientError>;
    /// Revoke a role from a list of users.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Roles};
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
    /// let users = dracoon.roles.revoke_role_from_users(1, vec![1, 2, 3].into()).await.unwrap();
    /// # }
    /// ```
    async fn revoke_role_from_users(
        &self,
        role_id: u64,
        user_ids: RevokeRoleBatchRequest,
    ) -> Result<RoleUserList, DracoonClientError>;
}
