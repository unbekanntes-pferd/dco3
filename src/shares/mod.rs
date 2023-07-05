//! This module implements the shares DRACOON API.
//! Documentation can be found here: <https://download.dracoon.com/api/swagger-ui/index.html?configUrl=/api/spec_v4/swagger-config#/shares>
use async_trait::async_trait;
pub use models::*;

use crate::{models::ListAllParams, DracoonClientError};

mod download;
mod models;
mod upload;

/// This trait provides all methods to manage download shares.
#[async_trait]
pub trait DownloadShares {
    /// Get a list shares (download shares).
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, DownloadShares, shares::{DownloadSharesFilter, DownloadSharesSortBy}, models::{ListAllParams, SortOrder}};
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
    ///     .with_filter(DownloadSharesFilter::name_contains("test"))
    ///     .with_sort(DownloadSharesSortBy::name(SortOrder::Desc))
    ///     .build();
    /// // pass None if you don't want to use any params
    /// let shares = dracoon.get_download_shares(Some(params)).await.unwrap();
    ///
    /// # }
    /// ```
    async fn get_download_shares(
        &self,
        params: Option<ListAllParams>,
    ) -> Result<DownloadSharesList, DracoonClientError>;
    /// Update list shares (download shares).
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, DownloadShares, shares::{UpdateDownloadSharesBulkRequest}};
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
    /// let ids = vec![1, 2, 3];
    /// let update = UpdateDownloadSharesBulkRequest::builder(ids)
    ///     .with_show_creator_name(true)
    ///     .build();
    /// dracoon.update_download_shares(update).await.unwrap();
    /// # }
    /// ```
    async fn update_download_shares(
        &self,
        update: UpdateDownloadSharesBulkRequest,
    ) -> Result<(), DracoonClientError>;
    /// Delete a list of shares (download shares).
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, DownloadShares, shares::{DeleteDownloadSharesRequest}};
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
    /// let ids = vec![1, 2, 3];
    /// dracoon.delete_download_shares(ids.into()).await.unwrap();
    /// // you can also use the DeleteDownloadSharesRequest::new() method
    /// let ids = vec![1, 2, 3];
    /// let delete = DeleteDownloadSharesRequest::new(ids);
    /// dracoon.delete_download_shares(delete).await.unwrap();
    /// # }
    /// ```
    async fn delete_download_shares(
        &self,
        delete: DeleteDownloadSharesRequest,
    ) -> Result<(), DracoonClientError>;
    /// Create a download share (share a node).
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, DownloadShares, shares::{CreateDownloadShareRequest}};
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
    /// let share = CreateDownloadShareRequest::builder(1)
    ///     .with_name("test")
    ///     .build();
    /// dracoon.create_download_share(share).await.unwrap();
    /// # }
    /// ```
    async fn create_download_share(
        &self,
        create: CreateDownloadShareRequest,
    ) -> Result<DownloadShare, DracoonClientError>;
    /// Get download share
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, DownloadShares};
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
    /// let file_request = dracoon.get_download_share(1).await.unwrap();
    /// # }
    /// ```
    async fn get_download_share(&self, share_id: u64) -> Result<DownloadShare, DracoonClientError>;
    /// Update download share
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, DownloadShares, shares::{UpdateDownloadShareRequest}};
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
    /// let update = UpdateDownloadShareRequest::builder()
    ///    .with_name("test")
    ///    .build();
    /// let file_request = dracoon.update_download_share(123, update).await.unwrap();
    /// # }
    /// ```
    async fn update_download_share(
        &self,
        share_id: u64,
        update: UpdateDownloadShareRequest,
    ) -> Result<DownloadShare, DracoonClientError>;
    /// Delete upload share
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, UploadShares};
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
    /// dracoon.delete_upload_share(1).await.unwrap();
    /// # }
    /// ```
    async fn delete_download_share(&self, share_id: u64) -> Result<(), DracoonClientError>;
    /// Send download share via email
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, DownloadShares, shares::{DownloadShareLinkEmail}};
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
    /// let recipients = vec!["test@test.foo".to_string(), "test2@test.foo".to_string()];
    /// let send_mail = DownloadShareLinkEmail::new("Test email", recipients, None);
    /// dracoon.send_download_share_email(123, send_mail).await.unwrap();
    /// # }
    /// ```
    async fn send_download_share_email(
        &self,
        share_id: u64,
        email: DownloadShareLinkEmail,
    ) -> Result<(), DracoonClientError>;
}

/// This trait provides all methods to manage upload shares.
#[async_trait]
pub trait UploadShares {
    /// Get a list file requests (upload shares).
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, UploadShares, shares::{UploadSharesFilter, UploadSharesSortBy}, models::{ListAllParams, SortOrder}};
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
    ///     .with_filter(UploadSharesFilter::name_contains("test"))
    ///     .with_sort(UploadSharesSortBy::name(SortOrder::Desc))
    ///     .build();
    /// // pass None if you don't want to use any params
    /// let file_requests = dracoon.get_upload_shares(Some(params)).await.unwrap();
    ///
    /// # }
    /// ```
    async fn get_upload_shares(
        &self,
        params: Option<ListAllParams>,
    ) -> Result<UploadSharesList, DracoonClientError>;
    /// Update a list of file requests (upload shares).
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, UploadShares, shares::{UpdateUploadSharesBulkRequest}};
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
    /// let ids = vec![1, 2, 3];
    /// let update = UpdateUploadSharesBulkRequest::builder(ids)
    ///     .with_max_size(1024)
    ///     .with_reset_max_size(true)
    ///     .build();
    /// dracoon.update_upload_shares(update).await.unwrap();
    /// # }
    /// ```
    async fn update_upload_shares(
        &self,
        update: UpdateUploadSharesBulkRequest,
    ) -> Result<(), DracoonClientError>;
    /// Delete a list of file requests (upload shares).
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, UploadShares, shares::{DeleteUploadSharesRequest}};
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
    /// let ids = vec![1, 2, 3];
    /// dracoon.delete_upload_shares(ids.into()).await.unwrap();
    /// // you can also use the DeleteUploadSharesRequest::new() method
    /// let ids = vec![1, 2, 3];
    /// let delete = DeleteUploadSharesRequest::new(ids);
    /// dracoon.delete_upload_shares(delete).await.unwrap();
    /// # }
    /// ```
    async fn delete_upload_shares(
        &self,
        delete: DeleteUploadSharesRequest,
    ) -> Result<(), DracoonClientError>;
    /// Create an upload share (request files into a node).
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, UploadShares, shares::{CreateUploadShareRequest}};
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
    /// let share = CreateUploadShareRequest::builder(1)
    ///     .with_name("test")
    ///     .build();
    /// dracoon.create_upload_share(share).await.unwrap();
    /// # }
    /// ```
    async fn create_upload_share(
        &self,
        create: CreateUploadShareRequest,
    ) -> Result<UploadShare, DracoonClientError>;
    /// Get upload share
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, UploadShares};
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
    /// let file_request = dracoon.get_upload_share(1).await.unwrap();
    /// # }
    /// ```
    async fn get_upload_share(
        &self,
        upload_share_id: u64,
    ) -> Result<UploadShare, DracoonClientError>;
    /// Update upload share
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, UploadShares, shares::{UpdateUploadShareRequest}};
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
    /// let update = UpdateUploadShareRequest::builder()
    ///    .with_name("test")
    ///   .build();
    /// let file_request = dracoon.update_upload_share(123, update).await.unwrap();
    /// # }
    /// ```
    async fn update_upload_share(
        &self,
        upload_share_id: u64,
        update: UpdateUploadShareRequest,
    ) -> Result<UploadShare, DracoonClientError>;
    /// Delete upload share
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, UploadShares};
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
    /// dracoon.delete_upload_share(1).await.unwrap();
    /// # }
    /// ```
    async fn delete_upload_share(&self, upload_share_id: u64) -> Result<(), DracoonClientError>;
    /// Send upload share via email
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, UploadShares, shares::{UploadShareLinkEmail}};
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
    /// let recipients = vec!["test@test.foo".to_string(), "test2@test.foo".to_string()];
    /// let send_mail = UploadShareLinkEmail::new("Test email", recipients, None);
    /// dracoon.send_upload_share_email(123, send_mail).await.unwrap();
    /// # }
    /// ```
    async fn send_upload_share_email(
        &self,
        upload_share_id: u64,
        email: UploadShareLinkEmail,
    ) -> Result<(), DracoonClientError>;
}
