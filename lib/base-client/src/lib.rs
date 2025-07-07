use alloy::{providers::Provider, sol};
use tracing::{debug, instrument};
use unionlabs::primitives::{H160, U256};

use crate::DisputeGameFactory::gameAtIndexReturn;

#[instrument(skip_all, fields(%l1_block_number, %l1_dispute_game_factory_proxy))]
pub async fn finalized_l2_block_number_of_l1_block_number(
    l1_provider: &impl Provider,
    l1_dispute_game_factory_proxy: H160,
    l1_block_number: u64,
) -> Result<u64, alloy::contract::Error> {
    let count = latest_game_of_l1_block_number(
        &l1_provider,
        l1_block_number,
        l1_dispute_game_factory_proxy,
    )
    .await?;

    finalized_l2_block_of_game_index(
        l1_provider,
        l1_block_number,
        l1_dispute_game_factory_proxy,
        count - U256::ONE,
    )
    .await
}

#[instrument(skip_all, fields(%l1_block_number, %l1_dispute_game_factory_proxy))]
pub async fn latest_game_of_l1_block_number(
    l1_provider: &impl Provider,
    l1_block_number: u64,
    l1_dispute_game_factory_proxy: H160,
) -> Result<U256, alloy::contract::Error> {
    let c = DisputeGameFactory::new(l1_dispute_game_factory_proxy.into(), &l1_provider);

    let count: U256 = c
        .gameCount()
        .block(l1_block_number.into())
        .call()
        .await?
        .into();

    debug!(%count);

    Ok(count)
}

#[instrument(skip_all, fields(%l1_block_number, %l1_dispute_game_factory_proxy))]
pub async fn finalized_l2_block_of_game_index(
    l1_provider: &impl Provider,
    l1_block_number: u64,
    l1_dispute_game_factory_proxy: H160,
    game_index: U256,
) -> Result<u64, alloy::contract::Error> {
    let c = DisputeGameFactory::new(l1_dispute_game_factory_proxy.into(), &l1_provider);

    let gameAtIndexReturn { proxy_, .. } = c
        .gameAtIndex(game_index.into())
        .block(l1_block_number.into())
        .call()
        .await?;

    debug!(%proxy_);

    let proxy = FaultDisputeGame::new(proxy_, &l1_provider);
    let block_number = proxy
        .l2BlockNumber()
        .block(l1_block_number.into())
        .call()
        .await?;

    debug!(%block_number);

    Ok(block_number
        .try_into()
        .expect("block number should be > u64::MAX"))
}

sol! {
    #![sol(rpc)]

    contract DisputeGameFactory {
        type Timestamp is uint64;
        type GameType is uint32;

        function gameCount() returns (uint256 gameCount);
        function gameAtIndex(uint256 _index)
                returns (GameType gameType_, Timestamp timestamp_, address proxy_);
    }

    interface FaultDisputeGame {
        function l2BlockNumber() returns (uint256 l2BlockNumber);
    }
}
