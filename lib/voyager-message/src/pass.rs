use std::future::Future;

use serde_json::Value;
use voyager_vm::{
    pass::{Pass, PassResult},
    Op,
};

use crate::{module::PluginClient, VoyagerMessage};

pub struct PluginOptPass<T> {
    client: T,
}

impl<T> PluginOptPass<T> {
    pub fn new(client: T) -> Self {
        Self { client }
    }
}

impl<T: PluginClient<Value, Value> + Send + Sync> Pass<VoyagerMessage> for PluginOptPass<&'_ T> {
    type Error = jsonrpsee::core::client::Error;

    fn run_pass(
        &self,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> impl Future<Output = Result<PassResult<VoyagerMessage>, Self::Error>> + Send {
        self.client.run_pass(msgs)
    }
}
