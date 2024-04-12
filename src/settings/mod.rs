use async_trait::async_trait;

use crate::DracoonClientError;

mod keypair;
mod models;

pub use models::SettingsEndpoint;

#[async_trait]
/// This trait currently implements distributing missing keys uing the system rescue key.
pub trait RescueKeyPair {
    /// Distributes missing file keys using the rescue key.
    /// Returns the total amount missing keys.
    /// If the total amount is larger than 100, more keys need distribution
    /// and the method should be called again.
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, RescueKeyPair};
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
    /// let mut missing_keys = dracoon.settings.distribute_missing_keys("rescue_key_secret", None, None, None).await.unwrap();
    ///
    /// while missing_keys > 100 {
    /// // loop until no more keys need distribution
    /// missing_keys = dracoon.settings.distribute_missing_keys("rescue_key_secret", None, None, None).await.unwrap();
    /// }
    ///
    /// // distribute missing keys for a specific room
    /// let missing_room_keys = dracoon.settings.distribute_missing_keys("rescue_key_secret", Some(123), None, None).await.unwrap();
    ///
    /// // distribute missing keys for a specific file
    /// let missing_file_keys = dracoon.settings.distribute_missing_keys("rescue_key_secret", None, Some(123), None).await.unwrap();
    ///
    /// // distribute missing keys for a specific user
    /// let missing_user_keys = dracoon.settings.distribute_missing_keys("rescue_key_secret", None, None, Some(123)).await.unwrap();
    ///
    /// # }
    ///
    async fn distribute_missing_keys(
        &self,
        rescue_key_secret: &str,
        room_id: Option<u64>,
        file_id: Option<u64>,
        user_id: Option<u64>,
    ) -> Result<u64, DracoonClientError>;
}
