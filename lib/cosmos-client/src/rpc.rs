pub trait RpcT {
    fn client(&self) -> &cometbft_rpc::Client;

    // TODO: Better type here
    fn chain_id(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct Rpc {
    client: cometbft_rpc::Client,
    chain_id: String,
}

impl Rpc {
    pub async fn new(rpc_url: String) -> Result<Self, cometbft_rpc::JsonRpcError> {
        let client = cometbft_rpc::Client::new(rpc_url).await?;

        let chain_id = client.status().await?.node_info.network;

        Ok(Self { client, chain_id })
    }
}

impl RpcT for Rpc {
    fn client(&self) -> &cometbft_rpc::Client {
        &self.client
    }

    fn chain_id(&self) -> &str {
        &self.chain_id
    }
}

impl<T: RpcT> RpcT for &T {
    fn client(&self) -> &cometbft_rpc::Client {
        (*self).client()
    }

    fn chain_id(&self) -> &str {
        (*self).chain_id()
    }
}
