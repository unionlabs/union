// #![warn(clippy::unwrap_used)]

use std::collections::VecDeque;

use alloy::{
    network::{AnyNetwork, AnyRpcBlock},
    providers::{DynProvider, Provider, ProviderBuilder, layers::CacheLayer},
};
use futures::future::try_join_all;
use ibc_union_spec::{ClientId, IbcUnion};
use jsonrpsee::{Extensions, core::async_trait};
use parlia_light_client_types::Header;
use parlia_types::ParliaHeader;
use parlia_verifier::{
    EPOCH_LENGTH, K_ANCESTOR_GENERATION_DEPTH, calculate_signing_valset_epoch_block_number,
    check_supermajority, get_vote_attestation_from_header_extra_data,
    parse_epoch_rotation_header_extra_data,
};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument, warn};
use unionlabs::{ibc::core::client::height::Height, never::Never, primitives::H160};
use voyager_sdk::{
    DefaultCmd, anyhow,
    hook::UpdateHook,
    into_value,
    message::{
        PluginMessage, VoyagerMessage,
        call::Call,
        data::{Data, DecodedHeaderMeta, OrderedHeaders},
    },
    plugin::Plugin,
    primitives::{ChainId, ClientType},
    rpc::{PluginServer, RpcError, RpcResult, types::PluginInfo},
    vm::{self, Op, Visit, pass::PassResult},
};

use crate::call::{FetchUpdate, ModuleCall};

pub mod call;

#[tokio::main]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub provider: DynProvider<AnyNetwork>,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,

    pub rpc_url: String,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    #[serde(default)]
    pub max_cache_size: u32,
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.rpc_url)
                .await?,
        );

        let chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        assert_eq!(chain_id, config.chain_id);

        let latest_block_number = provider.get_block_number().await?;

        let latest_epoch_block_number = latest_block_number - (latest_block_number % EPOCH_LENGTH);
        let (_, valset) = parse_epoch_rotation_header_extra_data(
            &provider
                .get_block_by_number(latest_epoch_block_number.into())
                .await?
                .unwrap()
                .header
                .extra_data,
        )?;

        let valset_size = valset.len();

        info!("valset size is {valset_size} at block {latest_epoch_block_number}");

        Ok(Self {
            chain_id,
            provider,
            ibc_handler_address: config.ibc_handler_address,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(
                &config.chain_id,
                &ClientType::new(ClientType::PARLIA),
            ),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[async_trait]
impl PluginServer<ModuleCall, Never> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .map(|mut op| {
                    UpdateHook::new(
                        &self.chain_id,
                        &ClientType::new(ClientType::PARLIA),
                        |fetch| {
                            Call::Plugin(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchUpdate {
                                    from_height: fetch.update_from,
                                    to_height: fetch.update_to,
                                    counterparty_chain_id: fetch.counterparty_chain_id.clone(),
                                    client_id: fetch
                                        .client_id
                                        .clone()
                                        .decode_spec::<IbcUnion>()
                                        .unwrap(),
                                    already_fetched_updates: vec![],
                                }),
                            ))
                        },
                    )
                    .visit_op(&mut op);

                    op
                })
                .enumerate()
                .map(|(i, op)| (vec![i], op))
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate {
                from_height,
                to_height,
                counterparty_chain_id,
                client_id,
                already_fetched_updates,
            }) => {
                self.fetch_update(
                    already_fetched_updates,
                    from_height,
                    to_height,
                    counterparty_chain_id,
                    client_id,
                )
                .await
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

impl Module {
    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %counterparty_chain_id,
            %update_from,
            %update_to,
        )
    )]
    async fn fetch_update(
        &self,
        already_fetched_updates: Vec<Header>,
        update_from: Height,
        update_to: Height,
        counterparty_chain_id: ChainId,
        client_id: ClientId,
    ) -> RpcResult<Op<VoyagerMessage>> {
        // fetch intermediate updates for every epoch

        // #[derive(Debug, Deserialize)]
        // struct Snapshot {
        //     validators: Vec<Value>,
        // }

        let mut headers: Vec<Header> = vec![];

        for source_block_number in windows(update_from.height(), update_to.height()) {
            if headers.len() >= 10 {
                let last = headers.last().unwrap().chain.first().unwrap().number;

                info!(
                    "fetched updates between {first} to {last}, continuing from {last} to {update_to}",
                    first = headers.first().unwrap().chain.first().unwrap().number,
                );

                return Ok(vm::call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::from(FetchUpdate {
                        from_height: Height::new(last.try_into().unwrap()),
                        to_height: update_to,
                        counterparty_chain_id,
                        client_id,
                        already_fetched_updates: already_fetched_updates
                            .into_iter()
                            .chain(headers)
                            .collect(),
                    }),
                )));
            }

            info!("fetching update to {source_block_number}");

            let fetch_block = async |number: u64| {
                self.provider
                    .get_block(number.into())
                    .await
                    .map_err(RpcError::retryable("error fetching block"))?
                    .ok_or_else(|| RpcError::missing_state("error fetching block: block not found"))
                    .map(convert_header)
            };

            let desired_source = fetch_block(source_block_number).await?;

            let mut current_ancestor_depth = 0;

            let mut is_ancestry_proof = false;

            let (attestation, vote_attestation) = loop {
                let attestation_block_number = source_block_number + 2 + current_ancestor_depth;

                info!("checking for attestations in block {attestation_block_number}");

                let attestation = fetch_block(attestation_block_number).await?;

                match get_vote_attestation_from_header_extra_data(&attestation).map_err(
                    RpcError::fatal(format!(
                        "unable to parse extra data from block {attestation_block_number}"
                    )),
                )? {
                    Some(vote_attestation) => {
                        info!(
                            current_ancestor_depth,
                            is_ancestry_proof,
                            "vote attestation found in block {attestation_block_number}"
                        );

                        let previous_epoch_block_number =
                            calculate_previous_epoch_block_number(attestation_block_number);

                        info!(previous_epoch_block_number);

                        let previous_valset = parse_epoch_rotation_header_extra_data(
                            &fetch_block(previous_epoch_block_number).await?.extra_data,
                        )
                        .map_err(RpcError::fatal(format!(
                            "error parsing epoch rotation header extra data for \
                            previous epoch ({previous_epoch_block_number})"
                        )))?
                        .1;

                        info!(previous_valset_len = previous_valset.len());

                        let signing_valset_epoch_block_number =
                            calculate_signing_valset_epoch_block_number(
                                attestation_block_number,
                                previous_valset.len().try_into().unwrap(),
                            );

                        info!(signing_valset_epoch_block_number);

                        let signing_valset = parse_epoch_rotation_header_extra_data(
                            &fetch_block(signing_valset_epoch_block_number)
                                .await?
                                .extra_data,
                        )
                        .map_err(RpcError::fatal(format!(
                            "error parsing epoch rotation header extra data for \
                            signing epoch ({signing_valset_epoch_block_number})"
                        )))?
                        .1;

                        info!(signing_valset_len = signing_valset.len());

                        if check_supermajority(&vote_attestation, signing_valset.len()) {
                            // if we're now looking for an ancestry proof, we just need any attestation, otherwise we need the source of the attestation to be the source block we're looking for
                            if is_ancestry_proof {
                                info!(
                                    "found attestation for block {} at block {attestation_block_number} (desired source block is {source_block_number})",
                                    vote_attestation.data.source_number
                                );

                                break (attestation, vote_attestation);
                            }

                            if vote_attestation.data.source_number == source_block_number {
                                info!(
                                    "found attestation for block {source_block_number} at block {attestation_block_number}"
                                );

                                break (attestation, vote_attestation);
                            }
                        } else {
                            warn!("supermajority not reached on block {attestation_block_number}");
                        }
                    }
                    None => {
                        info!(
                            current_ancestor_depth,
                            "no vote attestation in block {attestation_block_number}"
                        );
                    }
                };

                if current_ancestor_depth == K_ANCESTOR_GENERATION_DEPTH {
                    info!(
                        "no attestation found for block {source_block_number}, client update will be an ancestry proof"
                    );
                    is_ancestry_proof = true;
                }

                current_ancestor_depth += 1;
            };

            let target = fetch_block(vote_attestation.data.target_number).await?;

            info!(
                is_ancestry_proof,
                desired_source_hash = %desired_source.hash(),
                target_hash = %target.hash(),
                attestation_hash = %attestation.hash(),

                desired_source_number = %desired_source.number,
                target_number = %target.number,
                attestation_number = %attestation.number
            );

            let attestation_block_number: u64 = attestation
                .number
                .try_into()
                .expect("block number is < u64::MAX");

            let previous_epoch_block_number =
                calculate_previous_epoch_block_number(attestation_block_number);

            let previous_valset = parse_epoch_rotation_header_extra_data(
                &fetch_block(previous_epoch_block_number).await?.extra_data,
            )
            .map_err(RpcError::fatal(
                "error parsing epoch rotation header extra data",
            ))?
            .1;

            let trusted_valset_epoch_number = calculate_signing_valset_epoch_block_number(
                attestation_block_number,
                previous_valset.len().try_into().unwrap(),
            );

            info!(%trusted_valset_epoch_number);

            headers.push(Header {
                trusted_valset_epoch_number,
                chain: try_join_all(
                    (u64::try_from(desired_source.number).unwrap()
                        ..=u64::try_from(attestation.number).unwrap())
                        .map(fetch_block),
                )
                .await?,
            });
        }

        Ok(vm::data(OrderedHeaders {
            headers: already_fetched_updates
                .into_iter()
                .chain(headers)
                .map(|h| {
                    (
                        DecodedHeaderMeta {
                            height: Height::new(
                                h.chain.first().unwrap().number.try_into().unwrap(),
                            ),
                        },
                        into_value(h),
                    )
                })
                .collect(),
        }))
    }
}

// previous multiple of epoch length for the previous epoch
fn calculate_previous_epoch_block_number(attestation_block_number: u64) -> u64 {
    ((attestation_block_number - EPOCH_LENGTH) / (EPOCH_LENGTH)) * EPOCH_LENGTH
}

fn convert_header(block: AnyRpcBlock) -> ParliaHeader {
    let block = block.0.into_inner();

    ParliaHeader {
        parent_hash: block.header.inner.parent_hash.into(),
        sha3_uncles: block.header.inner.ommers_hash.into(),
        miner: block.header.inner.beneficiary.into(),
        state_root: block.header.inner.state_root.into(),
        transactions_root: block.header.inner.transactions_root.into(),
        receipts_root: block.header.inner.receipts_root.into(),
        logs_bloom: Box::new(block.header.inner.logs_bloom.0.into()),
        difficulty: block.header.inner.difficulty.into(),
        number: block.header.inner.number.into(),
        gas_limit: block.header.inner.gas_limit,
        gas_used: block.header.inner.gas_used,
        timestamp: block.header.inner.timestamp,
        extra_data: block.header.inner.extra_data.into(),
        // always present in a parlia header, contains the timestamp millisecond portion
        mix_hash: block.header.inner.mix_hash.unwrap().into(),
        nonce: block.header.inner.nonce.unwrap().into(),
        base_fee_per_gas: block.header.inner.base_fee_per_gas.unwrap().into(),
        withdrawals_root: block.header.inner.withdrawals_root.unwrap().into(),
        blob_gas_used: block.header.inner.blob_gas_used.unwrap(),
        excess_blob_gas: block.header.inner.excess_blob_gas.unwrap(),
        parent_beacon_block_root: block.header.inner.parent_beacon_block_root.unwrap().into(),
        requests_hash: block.header.inner.requests_hash.map(Into::into).into(),
    }
}

fn windows(from: u64, to: u64) -> impl Iterator<Item = u64> {
    std::iter::successors(Some(from), move |from| {
        if from >= &to {
            None
        } else {
            let next_rotation = from + (EPOCH_LENGTH - (from % EPOCH_LENGTH));

            if next_rotation < to {
                Some(next_rotation)
            } else {
                Some(to)
            }
        }
    })
    // skip initial `from`
    .skip(1)
}

#[test]
fn test_windows() {
    assert_eq!(
        windows(1, 9999).collect::<Vec<_>>(),
        [1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 9999]
    );
}

#[test]
fn test_calculate_previous_epoch_block_number() {
    for (b, eb) in [(8001, 7000), (8000, 7000), (8999, 7000), (9000, 8000)] {
        assert_eq!(eb, calculate_previous_epoch_block_number(b))
    }
}
