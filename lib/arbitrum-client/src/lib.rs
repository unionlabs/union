pub mod v1 {
    use std::error::Error;

    use alloy::{
        eips::BlockNumberOrTag,
        network::{AnyNetwork, AnyRpcBlock},
        providers::Provider,
        rpc::types::{Filter, Log},
        sol_types::SolEvent,
    };
    use arbitrum_types::v1::{
        NodeCreated, ROLLUP_CORE_LATEST_NODE_CREATED, read_latest_node_created,
    };
    use tracing::{debug, instrument, trace};
    use unionlabs::primitives::{H160, H256};

    #[instrument(skip_all, fields(%l1_height, %l1_contract_address))]
    pub async fn next_node_num_at_l1_height(
        l1_provider: impl Provider,
        l1_contract_address: H160,
        l1_height: u64,
    ) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let raw_slot = l1_provider
            .get_storage_at(
                l1_contract_address.into(),
                ROLLUP_CORE_LATEST_NODE_CREATED.slot().into(),
            )
            .block_id(l1_height.into())
            .await?;

        debug!(raw_slot = %<H256>::new(raw_slot.to_be_bytes()));

        let latest_confirmed = read_latest_node_created(raw_slot.into());

        debug!("l1_height {l1_height} is next node num {latest_confirmed}");

        Ok(latest_confirmed)
    }

    #[instrument(skip_all, fields(%l1_height, %l1_contract_address))]
    pub async fn finalized_l2_block_of_l1_height(
        l1_provider: impl Provider,
        l2_provider: impl Provider<AnyNetwork>,
        l1_contract_address: H160,
        l1_height: u64,
    ) -> Result<AnyRpcBlock, Box<dyn Error + Send + Sync>> {
        // read the next_node_num at l1_height
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

        let event = NodeCreated::decode_log(&event.inner)?.data;

        trace!(next_node_num, "{event:?}");

        let block_hash = event.assertion.afterState.globalState.bytes32Vals[0];

        debug!("next node num {next_node_num} is l2 block hash {block_hash}");

        let block = l2_provider
            .get_block(block_hash.into())
            .await?
            .expect("block must exist");

        debug!(
            "l2 block hash {block_hash} is l2 block number {}",
            block.header.number
        );

        Ok(block)
    }
}

pub mod v2 {
    use std::error::Error;

    use alloy::{
        eips::BlockNumberOrTag,
        network::{AnyNetwork, AnyRpcBlock},
        providers::Provider,
        rpc::types::Filter,
        sol_types::SolEvent,
    };
    use arbitrum_types::v2::AssertionCreated;
    use tracing::{debug, instrument, trace};
    use unionlabs::primitives::H160;

    #[instrument(skip_all, fields(%l1_height, %l1_contract_address))]
    pub async fn assertion_created_event_at_l1_block_height(
        l1_provider: impl Provider,
        l1_contract_address: H160,
        l1_height: u64,
    ) -> Result<AssertionCreated, Box<dyn Error + Send + Sync>> {
        let event = l1_provider
            .get_logs(
                &Filter::new()
                    .select(
                        // > Error: server returned an error response: error code -32602: Log response size exceeded. You can make eth_getLogs requests with up to a 10,000 block range and no limit on the response size, or you can request any block range with a cap of 10K logs in the response. Based on your parameters and the response size limit, this block range should work: [0x0, 0x79bcb3]
                        //
                        // just hardcode it to 10k blocks, assertions are posted more often than that
                        BlockNumberOrTag::Number(l1_height.saturating_sub(10_000))
                            ..=BlockNumberOrTag::Number(l1_height),
                    )
                    .address::<alloy::primitives::Address>(l1_contract_address.into())
                    .event_signature(AssertionCreated::SIGNATURE_HASH),
            )
            .await?
            .pop()
            .unwrap();

        let event = AssertionCreated::decode_log(&event.inner)?.data;

        trace!(?event);

        debug!(
            "l1_height {l1_height} is assertion_hash {}",
            event.assertionHash
        );

        Ok(event)
    }

    #[instrument(skip_all, fields(%l1_height, %l1_contract_address))]
    pub async fn finalized_l2_block_of_l1_height(
        l1_provider: impl Provider,
        l2_provider: impl Provider<AnyNetwork>,
        l1_contract_address: H160,
        l1_height: u64,
    ) -> Result<AnyRpcBlock, Box<dyn Error + Send + Sync>> {
        let assertion_created_event = assertion_created_event_at_l1_block_height(
            &l1_provider,
            l1_contract_address,
            l1_height,
        )
        .await?;

        let block_hash = assertion_created_event
            .assertion
            .afterState
            .globalState
            .bytes32Vals[0];

        debug!(
            "assertion {} is l2 block hash {block_hash}",
            assertion_created_event.assertionHash
        );

        let block = l2_provider
            .get_block(block_hash.into())
            .await?
            .expect("block must exist");

        debug!(
            "l2 block hash {block_hash} is l2 block number {}",
            block.header.number
        );

        Ok(block)
    }
}
