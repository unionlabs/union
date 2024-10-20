use std::time::Duration;

use alloy::{
    eips::{BlockId, RpcBlockHash},
    primitives::{FixedBytes, B256},
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::{BlockTransactionsKind, Filter, FilterBlockOption},
    sol,
    sol_types::SolEvent,
    transports::http::{Client, Http},
};
use backon::{ConstantBuilder, ExponentialBuilder, Retryable};
use color_eyre::eyre::{eyre, ContextCompat, Result, WrapErr};
use tracing::{debug, info};
use unionlabs::{bounded::BoundedU32, hash::H160, uint::U256};

use crate::{
    beacon::Beacon,
    consensus::{Indexer, Querier},
};
sol! {

    #[derive(Debug)]
    event NodeCreated (
        uint64 indexed node_num,
        bytes32 indexed parent_node_hash,
        bytes32 indexed node_hash,
        bytes32 execution_hash,
        (((bytes32[2], uint64[2]), uint8), ((bytes32[2], uint64[2]), uint8), uint64) assertion,
        bytes32 after_inbox_batch_acc,
        bytes32 wasm_module_root,
        uint256 inbox_max_count,
    );
}

pub struct Arb {
    pub l1_client: RootProvider<Http<Client>>,
    pub l2_client: RootProvider<Http<Client>>,

    pub beacon: Beacon,

    pub rollup_finalization_config: RollupFinalizationConfig,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub label: String,

    pub l1_url: url::Url,
    pub l2_url: url::Url,
    pub beacon_url: url::Url,
    pub rollup_finalization_config: RollupFinalizationConfig,

    pub start_height: Option<i64>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct RollupFinalizationConfig {
    pub l1_contract_address: H160,
    pub l1_latest_confirmed_slot: U256,
    pub l1_next_node_num_slot_offset_bytes: BoundedU32<0, 24>,
}

impl Config {
    pub async fn indexer(self, db: sqlx::PgPool) -> Result<Indexer<Arb>> {
        let l2_client = ProviderBuilder::new().on_http(self.l2_url);

        let l2_chain_id = U256::from(
            l2_client
                .get_chain_id()
                .await
                .wrap_err("unable to fetch chain id from l2")?,
        )
        .to_string();

        info!("fetching db chain_id for chain {}", l2_chain_id);

        let chain_id = (|| async {
            let chain_id = crate::postgres::get_chain_id(&db, l2_chain_id.clone())
                .await?
                // This can reasonably fail because the other indexer is creating the chain_id. Otherwise
                // this should always succeed.
                .wrap_err("chain not found")?;
            Ok::<_, color_eyre::Report>(chain_id)
        })
        .retry(&ExponentialBuilder::default())
        .await?;

        let querier = Arb {
            l1_client: ProviderBuilder::new().on_http(self.l1_url),
            l2_client,

            beacon: Beacon::new(self.beacon_url, reqwest::Client::new()),

            rollup_finalization_config: self.rollup_finalization_config,
        };

        Ok(Indexer::new(chain_id, db, querier, self.start_height))
    }
}

impl Arb {
    // NOTE: Copied from chain_utils
    async fn execution_height_of_beacon_slot(&self, slot: u64) -> Result<u64> {
        // read the next_node_num at l1.execution_height(beacon_slot), then from there filter for `NodeCreated`
        let next_node_num = self.next_node_num_at_beacon_slot(slot).await?;
        let [event] = self
            .l1_client
            .get_logs(
                &Filter::new()
                    .select(
                        FilterBlockOption::Range {
                            from_block: Some(alloy::eips::BlockNumberOrTag::Earliest),
                            to_block: Some(alloy::eips::BlockNumberOrTag::Latest)
                        }
                    )
                    .address(alloy::primitives::Address(
                        FixedBytes::from_slice(self.rollup_finalization_config.l1_contract_address.get()),
                    ))
                    .event_signature(NodeCreated::SIGNATURE_HASH)
                    .topic1(alloy::primitives::FixedBytes(U256::from(next_node_num).to_be_bytes())),
            )
            .await
            .wrap_err("error fetching `NodeCreated` log from l1")?
            .try_into()
            .map_err(|e| eyre!("too many logs found? there should only be one `NodeCreated event`, but found: {e:?}"))?;
        debug!("next node num: {next_node_num}: {event:?}");
        let event = NodeCreated::decode_log(&event.inner, true).unwrap();
        let block_id = BlockId::Hash(RpcBlockHash {
            block_hash: FixedBytes::from_slice(event.assertion.0 .0 .0[0].as_ref()),
            require_canonical: None,
        });
        let block = self
            .l2_client
            .get_block(block_id, BlockTransactionsKind::Full)
            .await
            .wrap_err("error fetching l2 block")?
            .expect("block should exist if it is finalized on the l1");

        Ok(block.header.number)
    }

    pub async fn next_node_num_at_beacon_slot(&self, slot: u64) -> Result<u64> {
        let l1_height = self
            .beacon
            .get_height_at_skip_missing(slot.try_into().expect("negative slot?"))
            .await?
            .data
            .message
            .body
            .execution_payload
            .block_number;

        let slot_offset_bytes = self
            .rollup_finalization_config
            .l1_next_node_num_slot_offset_bytes
            .inner() as usize;
        let raw_slot = self
            .l1_client
            .get_storage_at(
                alloy::primitives::Address::new(
                    *self.rollup_finalization_config.l1_contract_address.get(),
                ),
                alloy::primitives::Uint::from_be_bytes(
                    self.rollup_finalization_config
                        .l1_latest_confirmed_slot
                        .to_be_bytes(),
                ),
            )
            .await?;

        let raw_slot: B256 = raw_slot.into();
        let latest_confirmed = u64::from_be_bytes(
            raw_slot.0[slot_offset_bytes..slot_offset_bytes + 8]
                .try_into()
                .expect("size is correct; qed;"),
        );

        debug!("l1_height {l1_height} is next node num {latest_confirmed}",);
        Ok(latest_confirmed)
    }
}

impl Querier for Arb {
    async fn get_execution_height(&self, slot: i64) -> Result<(i64, i64)> {
        let height = (|| self.execution_height_of_beacon_slot(slot as u64))
            .retry(
                &ConstantBuilder::default()
                    .with_delay(Duration::from_millis(500))
                    .with_max_times(60),
            )
            .await?;
        Ok((slot, height as i64))
    }
}
