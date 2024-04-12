use std::sync::Arc;

use crate::auth::DracoonClient;

use self::auth::SystemAuthEndpoint;

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
