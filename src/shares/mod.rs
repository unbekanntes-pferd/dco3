use async_trait::async_trait;
pub use models::*;

use crate::{DracoonClientError, models::ListAllParams};

mod models;
mod download;
mod upload;



#[async_trait]
pub trait DownloadShares {
    async fn get_download_shares(&self, params: Option<ListAllParams>) -> Result<DownloadSharesList, DracoonClientError>;

    async fn update_download_shares(&self, update: UpdateDownloadSharesBulkRequest) -> Result<(), DracoonClientError>;

    async fn delete_download_shares(&self, delete: DeleteDownloadSharesRequest) -> Result<(), DracoonClientError>;

    async fn create_download_share(&self, create: CreateDownloadShareRequest) -> Result<DownloadShare, DracoonClientError>;

    async fn get_download_share(&self, share_id: u64) -> Result<DownloadShare, DracoonClientError>;

    async fn update_download_share(&self, share_id: u64, update: UpdateDownloadShareRequest) -> Result<DownloadShare, DracoonClientError>;

    async fn delete_download_share(&self, share_id: u64) -> Result<(), DracoonClientError>;

    async fn send_download_share_email(&self, share_id: u64, email: DownloadShareLinkEmail) -> Result<(), DracoonClientError>;

}

#[async_trait]
pub trait UploadShares {
    async fn get_upload_shares(&self, params: Option<ListAllParams>) -> Result<UploadSharesList, DracoonClientError>;

    async fn update_upload_shares(&self, update: UpdateUploadSharesBulkRequest) -> Result<(), DracoonClientError>;

    async fn delete_upload_shares(&self, delete: DeleteUploadSharesRequest) -> Result<(), DracoonClientError>;

    async fn create_share(&self, create: CreateUploadShareRequest) -> Result<UploadShare, DracoonClientError>;

    async fn get_upload_share(&self, upload_share_id: u64) -> Result<UploadShare, DracoonClientError>;

    async fn update_upload_share(&self, upload_share_id: u64, update: UpdateUploadShareRequest) -> Result<UploadShare, DracoonClientError>;

    async fn delete_upload_share(&self, upload_share_id: u64) -> Result<(), DracoonClientError>;

    async fn send_upload_share_email(&self, upload_share_id: u64, email: UploadShareLinkEmail) -> Result<(), DracoonClientError>;
}

