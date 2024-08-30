use std::sync::Arc;

use crate::client::DracoonClient;

use self::auth::SystemAuthEndpoint;

pub use self::auth::AuthenticationMethods;

mod auth;

#[derive(Clone)]
pub struct SystemEndpoint<S> {
    pub auth: SystemAuthEndpoint<S>,
}

impl<S> SystemEndpoint<S> {
    pub fn new(client: Arc<DracoonClient<S>>) -> Self {
        Self {
            auth: SystemAuthEndpoint::new(client),
        }
    }
}
