mod download;
mod upload;

use std::sync::Arc;

pub use download::*;
pub use upload::*;

use crate::client::DracoonClient;

#[derive(Clone)]
pub struct SharesEndpoint<S> {
    client: Arc<DracoonClient<S>>,
    state: std::marker::PhantomData<S>,
}

impl<S> SharesEndpoint<S> {
    pub fn new(client: Arc<DracoonClient<S>>) -> Self {
        Self {
            client,
            state: std::marker::PhantomData,
        }
    }

    pub fn client(&self) -> &Arc<DracoonClient<S>> {
        &self.client
    }
}
