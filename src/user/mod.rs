//! This module implements a subset of the DRACOON user API. 
//! Documentation can be found here: <https://download.dracoon.com/api/swagger-ui/index.html?configUrl=/api/spec_v4/swagger-config#/user>

use async_trait::async_trait;
use dco3_crypto::{PlainUserKeyPairContainer};

pub use self::models::*;
use super::auth::errors::DracoonClientError;

pub mod models;
pub mod account;
pub mod keypairs;


#[async_trait]
pub trait User {
    /// Get the user account information.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, User};
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
    /// let account = dracoon.get_user_account().await.unwrap();
    /// # }
    /// ```
    async fn get_user_account(&self) -> Result<UserAccount, DracoonClientError>;
      /// Update the user account information.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, User, user::{UpdateUserAccountRequest}};
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
    /// let update = UpdateUserAccountRequest::builder()
    ///                      .with_first_name("Jane")
    ///                      .with_last_name("Doe")
    ///                      .with_email("jane.doe@localhost")
    ///                      .build();
    /// 
    /// let account = dracoon.update_user_account(update).await.unwrap();
    /// # }
    /// ```  
    async fn update_user_account(&self, update: UpdateUserAccountRequest) -> Result<UserAccount, DracoonClientError>;
}

#[async_trait]
#[allow(clippy::module_name_repetitions)]
pub trait UserAccountKeypairs {
    /// Get the plain user keypair container.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, UserAccountKeypairs};
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
    /// let keypair = dracoon.get_user_keypair("secret123").await.unwrap();
    /// // note: you will usually not need the plain keypair since encryption / decryption
    /// // is handled by the dracoon client for up- and downloads.
    /// # }
    /// ```
    async fn get_user_keypair(&self, secret: &str) -> Result<PlainUserKeyPairContainer, DracoonClientError>;
    /// Set the user keypair container.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, UserAccountKeypairs};
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
    /// dracoon.set_user_keypair("Secret123!").await.unwrap();
    /// // note: you need to delete the existing keypair before setting a new one.
    /// # }
    /// ```
    async fn set_user_keypair(&self, secret: &str) -> Result<(), DracoonClientError>;
    /// Delete the user keypair container.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, UserAccountKeypairs};
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
    /// dracoon.delete_user_keypair().await.unwrap();
    /// // note: you need to delete the existing keypair before setting a new one.
    /// # }
    /// ```
    async fn delete_user_keypair(&self) -> Result<(), DracoonClientError>;
}