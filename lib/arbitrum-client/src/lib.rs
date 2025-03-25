use std::error::Error;

use alloy::{
    eips::BlockNumberOrTag,
    network::{AnyNetwork, AnyRpcBlock},
    providers::Provider,
    rpc::types::{Filter, Log},
    sol_types::SolEvent,
};
use arbitrum_types::{NodeCreated, L1_NEXT_NODE_NUM_SLOT, L1_NEXT_NODE_NUM_SLOT_OFFSET_BYTES};
use tracing::{debug, instrument};
use unionlabs::{
    primitives::{H160, H256},
    ByteArrayExt,
};

#[instrument(skip_all, fields(%l1_height, %l1_contract_address))]
pub async fn next_node_num_at_l1_height(
    l1_provider: impl Provider,
    l1_contract_address: H160,
    l1_height: u64,
) -> Result<u64, Box<dyn Error>> {
    let raw_slot = l1_provider
        .get_storage_at(l1_contract_address.into(), L1_NEXT_NODE_NUM_SLOT)
        .block_id(l1_height.into())
        .await
        .unwrap();

    debug!(raw_slot = %<H256>::new(raw_slot.to_be_bytes()));

    let latest_confirmed = u64::from_be_bytes(
        raw_slot
            .to_be_bytes::<32>()
            .array_slice::<{ L1_NEXT_NODE_NUM_SLOT_OFFSET_BYTES as usize }, 8>(),
    );

    debug!("l1_height {l1_height} is next node num {latest_confirmed}");

    Ok(latest_confirmed)
}

#[instrument(skip_all, fields(%l1_height, %l1_contract_address))]
pub async fn finalized_execution_block_of_l1_height(
    l1_provider: impl Provider,
    l2_provider: impl Provider<AnyNetwork>,
    l1_contract_address: H160,
    l1_height: u64,
) -> Result<AnyRpcBlock, Box<dyn Error>> {
    // read the next_node_num at l1.execution_height(beacon_slot), then from there filter for `NodeCreated`
    let next_node_num =
        next_node_num_at_l1_height(&l1_provider, l1_contract_address, l1_height).await?;

    let [event]: [Log; 1] = l1_provider
        .get_logs(
            &Filter::new()
                .select(BlockNumberOrTag::Earliest..=BlockNumberOrTag::Latest)
                .address::<alloy::primitives::Address>(l1_contract_address.into())
                .event_signature(NodeCreated::SIGNATURE_HASH)
                .topic1(alloy::primitives::U256::from(next_node_num)),
        )
        .await?
        .try_into()
        .unwrap();

    let event: NodeCreated = NodeCreated::decode_log(&event.inner, true)?.data;

    debug!(next_node_num, "{event:?}");

    let block = l2_provider
        .get_block(event.assertion.afterState.globalState.bytes32Vals[0].into())
        .await?
        .expect("block must exist");

    Ok(block)
}

#[cfg(test)]
mod tests {
    use alloy::{hex, network::AnyNetwork, providers::ProviderBuilder};

    use crate::finalized_execution_block_of_l1_height;

    #[tokio::test]
    async fn block() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();

        let l1_provider = ProviderBuilder::new()
            .connect("https://eth-sepolia.g.alchemy.com/v2/MS7UF39itji9IWEiJBISExWgEGtEGbs7")
            .await
            .unwrap();
        let l2_provider = ProviderBuilder::new()
            .network::<AnyNetwork>()
            .connect("https://testnet.corn-rpc.com")
            .await
            .unwrap();

        let block = finalized_execution_block_of_l1_height(
            l1_provider,
            l2_provider,
            hex!("0xD318638594A5B17b50a1389B0c0580576226C0AE").into(),
            7993090,
        )
        .await
        .unwrap();

        dbg!(block);
    }
}
