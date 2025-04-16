use std::time::{SystemTime, UNIX_EPOCH};

use alloy::{
    eips::BlockId,
    network::{AnyNetwork, AnyRpcBlock},
    primitives::U256,
    providers::Provider,
};
use bob_types::L2OutputOracle;
use tracing::instrument;
use unionlabs::primitives::H160;

#[instrument(skip_all, fields(%l1_height, %l2_oracle_address))]
pub async fn latest_committed_l2_block_number(
    l1_provider: impl Provider,
    l2_oracle_address: H160,
    l1_height: u64,
) -> Result<u64, alloy::contract::Error> {
    let oracle = L2OutputOracle::new(l2_oracle_address.into(), &l1_provider);
    Ok(oracle
        .latestBlockNumber()
        .call()
        .block(l1_height.into())
        .await?
        ._0
        .try_into()
        .unwrap())
}

#[instrument(skip_all, fields(%l1_height, %l2_height, %l2_oracle_address))]
pub async fn output_index_of_l2_block_on_l1_block(
    l1_provider: impl Provider,
    l2_oracle_address: H160,
    l2_height: u64,
    l1_height: u64,
) -> Result<u32, alloy::contract::Error> {
    let oracle = L2OutputOracle::new(l2_oracle_address.into(), &l1_provider);
    Ok(oracle
        .getL2OutputIndexAfter(l2_height.try_into().unwrap())
        .call()
        .block(l1_height.into())
        .await?
        ._0
        .try_into()
        .unwrap())
}

#[instrument(skip_all, fields(%l1_height, %l2_oracle_address))]
pub async fn finalized_execution_block_of_l1_height(
    l1_provider: impl Provider,
    l2_provider: impl Provider<AnyNetwork>,
    l2_oracle_address: H160,
    l2_finalization_period_seconds: u64,
    l1_height: u64,
) -> Result<AnyRpcBlock, alloy::contract::Error> {
    assert!(l2_finalization_period_seconds != 0);
    let oracle = L2OutputOracle::new(l2_oracle_address.into(), &l1_provider);
    let start = SystemTime::now();
    let current_timestamp =
        U256::try_from(start.duration_since(UNIX_EPOCH).expect("qed").as_secs()).unwrap();
    let latest_output_index = oracle
        .latestOutputIndex()
        .call()
        .block(l1_height.into())
        .await?
        ._0;
    // The period until the L2 block is considered to be final.
    let finalization_period_seconds = U256::try_from(l2_finalization_period_seconds).unwrap();
    let mut finalized_output_index = latest_output_index;
    let finalized_output = loop {
        let current_output = oracle
            .getL2Output(finalized_output_index)
            .call()
            .block(l1_height.into())
            .await?
            ._0;
        let current_output_timestamp = U256::try_from(current_output.timestamp).unwrap();
        if current_output_timestamp + finalization_period_seconds <= current_timestamp {
            break current_output;
        }
        finalized_output_index = finalized_output_index
            .checked_sub(U256::ONE)
            .expect("impossible");
    };
    Ok(l2_provider
        .get_block(BlockId::Number(
            u64::try_from(finalized_output.l2BlockNumber)
                .expect("l2 block number > 2^64???")
                .into(),
        ))
        .await?
        .expect("impossible"))
}
