use std::sync::Arc;

use ethers::{
    prelude::abigen,
    providers::{Http, Provider},
    types::{Address, H160},
};

pub async fn update_contract() {
    const RPC_URL: &str = "https://eth.llamarpc.com";
    const WETH_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

    let provider = Provider::<Http>::try_from(RPC_URL).unwrap();
    let client = Arc::new(provider);
    let address: Address = WETH_ADDRESS.parse().unwrap();

    let contract = contracts::ibc_handler::IBCHandler::new(address, client);

    contract
        .bind_port("".into(), H160::zero())
        .call()
        .await
        .unwrap();
}
