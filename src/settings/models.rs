use std::sync::Arc;

use crate::auth::DracoonClient;

#[derive(Clone)]
pub struct SettingsEndpoint<S> {
    client: Arc<DracoonClient<S>>,
    state: std::marker::PhantomData<S>,
}

impl<S> SettingsEndpoint<S> {
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
