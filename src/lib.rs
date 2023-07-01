#![allow(dead_code)]
#![allow(unused_variables)]

//! # dco3
//!
//! `dco3` is an async wrapper around API calls in DRACOON.
//! DRACOON is a cloud service provider - more information can be found on <https://dracoon.com>
//! 
//! The
//!
//! ## Usage
//! All API calls are implemented by the `Dracoon` struct. It can be created by using the `DracoonBuilder` struct.
//! 
//! In order to access specific API calls, the `Dracoon` struct needs to be in the `Connected` state. 
//! This can be achieved by calling the `connect` method.
//! To use specific endpoints, you need to import relevant traits.
//! Currently, the following traits are implemented:
//! 
//! * [User] - for user management
//! * [UserAccountKeypairs] - for user keypair management
//! * [Nodes] - for node operations (folders, rooms, upload and download are excluded)
//! * [Download] - for downloading files
//! * [Upload] - for uploading files
//! * [Folders] - for folder operations
//! * [Rooms] - for room operations
//! 
//! 
//! ### Example
//! ```no_run
//! use dco3::{Dracoon, auth::OAuth2Flow, user::User};
//! 
//! #[tokio::main]
//! async fn main() {
//!    let dracoon = Dracoon::builder()
//!       .with_base_url("https://dracoon.team")
//!       .with_client_id("client_id")
//!       .with_client_secret("client_secret")
//!       .build()
//!      .unwrap()
//!     .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
//!    .await
//!   .unwrap();
//! 
//! 
//!   let user_info = dracoon.get_user_account().await.unwrap();
//!   println!("User info: {:?}", user_info);
//! }
//!```
//! 
//! 


use std::marker::PhantomData;

use dco3_crypto::PlainUserKeyPairContainer;
use reqwest::Url;

use self::{
    auth::{errors::DracoonClientError, Connected, Disconnected, OAuth2Flow},
    auth::{DracoonClient, DracoonClientBuilder},
    user::{models::UserAccount},
};

// re-export traits
pub use self::{
    nodes::{Download, Folders, Nodes, Rooms, Upload},
    user::User,
    user::UserAccountKeypairs,
};

pub mod auth;
pub mod constants;
pub mod models;
pub mod nodes;
pub mod user;
pub mod utils;


#[derive(Clone)]
pub struct Dracoon<State = Disconnected> {
    client: DracoonClient<State>,
    state: PhantomData<State>,
    user_info: Option<UserAccount>,
    keypair: Option<PlainUserKeyPairContainer>,
}

/// Builder for the `Dracoon` struct.
/// Requires a base url, client id and client secret.
/// Optionally, a redirect uri can be provided.
#[derive(Default)]
pub struct DracoonBuilder {
    client_builder: DracoonClientBuilder,
}

impl DracoonBuilder {
    /// Creates a new `DracoonBuilder`
    pub fn new() -> Self {
        let client_builder = DracoonClientBuilder::new();
        Self {
            client_builder,
        }
    }

    /// Sets the base url for the DRACOON instance
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.client_builder = self.client_builder.with_base_url(base_url);
        self
    }

    /// Sets the client id for the DRACOON instance
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_builder = self.client_builder.with_client_id(client_id);
        self
    }

    /// Sets the client secret for the DRACOON instance
    pub fn with_client_secret(mut self, client_secret: impl Into<String>) -> Self {
        self.client_builder = self.client_builder.with_client_secret(client_secret);
        self
    }

    /// Sets the redirect uri for the DRACOON instance
    pub fn with_redirect_uri(mut self, redirect_uri: impl Into<String>) -> Self {
        self.client_builder = self.client_builder.with_redirect_uri(redirect_uri);
        self
    }

    /// Builds the `Dracoon` struct - fails, if any of the required fields are missing
    pub fn build(self) -> Result<Dracoon<Disconnected>, DracoonClientError> {
        let dracoon = self.client_builder.build()?;

        Ok(Dracoon {
            client: dracoon,
            state: PhantomData,
            user_info: None,
            keypair: None,
        })
    }
}

impl Dracoon<Disconnected> {

    pub fn builder() -> DracoonBuilder {
        DracoonBuilder::new()
    }

    pub async fn connect(
        self,
        oauth_flow: OAuth2Flow,
    ) -> Result<Dracoon<Connected>, DracoonClientError> {
        let client = self.client.connect(oauth_flow).await?;

        Ok(Dracoon {
            client,
            state: PhantomData,
            user_info: None,
            keypair: None,
        })
    }

    pub fn get_authorize_url(&mut self) -> String {
        self.client.get_authorize_url()
    }
}

impl Dracoon<Connected> {
    pub fn build_api_url(&self, url_part: &str) -> Url {
        self.client
            .get_base_url()
            .join(url_part)
            .expect("Correct base url")
    }

    pub async fn get_auth_header(&self) -> Result<String, DracoonClientError> {
        self.client.get_auth_header().await
    }

    pub fn get_base_url(&self) -> &Url {
        self.client.get_base_url()
    }

    pub fn get_refresh_token(&self) -> &str {
        self.client.get_refresh_token()
    }

    pub async fn get_user_info(&mut self) -> Result<&UserAccount, DracoonClientError> {
        if let Some(ref user_info) = self.user_info {
            return Ok(user_info);
        }

        let user_info = self.get_user_account().await?;
        self.user_info = Some(user_info);
        Ok(self.user_info.as_ref().expect("Just set user info"))
    }

    pub async fn get_keypair(
        &mut self,
        secret: Option<&str>,
    ) -> Result<&PlainUserKeyPairContainer, DracoonClientError> {
        if let Some(ref keypair) = self.keypair {
            return Ok(keypair);
        }

        let secret = secret.ok_or(DracoonClientError::MissingEncryptionSecret)?;
        let keypair = self.get_user_keypair(secret).await?;
        self.keypair = Some(keypair);
        Ok(self.keypair.as_ref().expect("Just set keypair"))
    }
}
