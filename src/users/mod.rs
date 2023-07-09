use async_trait::async_trait;

mod models;
#[allow(clippy::module_inception)]
mod users;

pub use models::*;

use crate::{models::ListAllParams, DracoonClientError};


#[async_trait]
pub trait Users {
    /// Get a list of all users.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Users, users::{UsersSortBy, UsersFilter}, SortOrder, ListAllParams};
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
    ///     .with_filter(UsersFilter::email_contains("foo"))
    ///     .with_sort(UsersSortBy::user_name(SortOrder::Desc))
    ///     .build();
    /// // optionally include roles and attributes
    /// let users = dracoon.get_users(Some(params), None, None).await.unwrap();
    /// 
    /// # }
    /// ```
    async fn get_users(&self, params: Option<ListAllParams>, include_roles: Option<bool>, include_attributes: Option<bool>) -> Result<UserList, DracoonClientError>;
    /// Create a new user.
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, Users, users::{CreateUserRequest, UserAuthData}};
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
    /// // optionally a password can be set for local users
    /// let auth = UserAuthData::new_basic(None);
    /// let user = CreateUserRequest::builder("Jane", "Doe")
    ///    .with_user_name("jane.doe")
    ///    .with_email("jane.doe@localhost")
    ///    .with_auth_data(auth)
    ///    .build();
    /// let user = dracoon.create_user(user).await.unwrap();
    /// 
    /// // creating an OIDC user
    /// let auth = UserAuthData::new_oidc("jane.doe", 4);
    /// let user = CreateUserRequest::builder("Jane", "Doe")
    ///    .with_email("jane.doe@localhost")
    ///    .with_auth_data(auth)
    ///    .build();
    /// let user = dracoon.create_user(user).await.unwrap();
    /// # }
    /// ```
    async fn create_user(&self, req: CreateUserRequest) -> Result<UserData, DracoonClientError>;
    /// Get a user by id.
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, Users};
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
    /// // optionally you can include effective roles
    /// let user = dracoon.get_user(123, None).await.unwrap();
    /// # }
    /// ```
    async fn get_user(&self, user_id: u64, effective_roles: Option<bool>) -> Result<UserData, DracoonClientError>;
    /// Update a user by id.
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, Users, users::UpdateUserRequest};
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
    /// let update = UpdateUserRequest::builder()
    ///    .with_first_name("Jane")
    ///    .with_last_name("Doe") 
    ///    .build();
    /// let user = dracoon.update_user(123, update).await.unwrap();
    /// # }
    /// ```
    async fn update_user(&self, user_id: u64, req: UpdateUserRequest) -> Result<UserData, DracoonClientError>;
    /// Deletes a user by id.
    /// ```no_run
    /// # use dco3::{Dracoon, Users, OAuth2Flow};
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
    /// dracoon.delete_user(123).await.unwrap();
    /// 
    /// # }
    /// ```
    async fn delete_user(&self, user_id: u64) -> Result<(), DracoonClientError>;
    /// Gets last admin rooms for a user.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Users};
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
    /// let rooms = dracoon.get_user_last_admin_rooms(123).await.unwrap();
    /// # }
    /// ``` 
    async fn get_user_last_admin_rooms(&self, user_id: u64) -> Result<LastAdminUserRoomList, DracoonClientError>;

}