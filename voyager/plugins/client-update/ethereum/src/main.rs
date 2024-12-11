#![warn(clippy::unwrap_used)]

use std::{collections::VecDeque, ops::Div};

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::BlockTransactionsKind,
    transports::BoxTransport,
};
use beacon_api::{client::BeaconApiClient, types::Spec};
use beacon_api_types::{
    light_client_update::NextSyncCommitteeBranch, PresetBaseKind, SyncCommittee,
};
use bitvec::{order::Msb0, vec::BitVec};
use ethereum_light_client_types::{
    AccountProof, EpochChangeUpdate, Header, LightClientUpdate, LightClientUpdateData,
    WithinEpochUpdate,
};
use futures::{stream, StreamExt, TryStreamExt};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};
use unionlabs::{
    constants::metric::NANOS_PER_SECOND,
    hash::{H160, H256},
    ibc::core::client::height::Height,
    ErrorReporter,
};
use voyager_message::{
    call::{Call, FetchUpdateHeaders, WaitForTimestamp},
    core::ChainId,
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    hook::UpdateHook,
    into_value,
    module::{PluginInfo, PluginServer},
    DefaultCmd, Plugin, PluginMessage, VoyagerMessage,
};
use voyager_vm::{call, defer, now, pass::PassResult, seq, BoxDynError, Op, Visit};

use crate::{
    call::{FetchUpdate, ModuleCall},
    callback::ModuleCallback,
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub chain_spec: PresetBaseKind,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub provider: RootProvider<BoxTransport>,
    pub beacon_api_client: BeaconApiClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: ChainId,

    pub chain_spec: PresetBaseKind,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    pub async fn fetch_account_update(&self, block_number: u64) -> RpcResult<AccountProof> {
        let account_update = self
            .provider
            .get_proof(self.ibc_handler_address.into(), vec![])
            .block_id(
                // NOTE: Proofs are from the execution layer, so we use execution height, not beacon slot.
                block_number.into(),
            )
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching account update"),
                    None::<()>,
                )
            })?;

        Ok(AccountProof {
            storage_root: account_update.storage_hash.into(),
            proof: account_update
                .account_proof
                .into_iter()
                .map(|x| x.to_vec())
                .collect(),
        })
    }
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, chain_utils::BoxDynError> {
        let provider = ProviderBuilder::new()
            .on_builtin(&config.eth_rpc_api)
            .await?;

        let chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        if chain_id != config.chain_id {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            )
            .into());
        }

        let beacon_api_client = BeaconApiClient::new(config.eth_beacon_rpc_api).await?;

        let spec = beacon_api_client
            .spec()
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching beacon spec"),
                    None::<()>,
                )
            })?
            .data;

        if spec.preset_base != config.chain_spec {
            return Err(format!(
                "incorrect chain spec: expected `{}`, but found `{}`",
                config.chain_spec, spec.preset_base
            )
            .into());
        }

        Ok(Self {
            chain_id,
            chain_spec: spec.preset_base,
            ibc_handler_address: config.ibc_handler_address,
            provider,
            beacon_api_client,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(&config.chain_id),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[async_trait]
impl PluginServer<ModuleCall, ModuleCallback> for Module {
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
                    UpdateHook::new(&self.chain_id, |fetch| {
                        Call::Plugin(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchUpdate {
                                from_height: fetch.update_from,
                                to_height: fetch.update_to,
                                counterparty_chain_id: fetch.counterparty_chain_id.clone(),
                            }),
                        ))
                    })
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
            }) => self
                .fetch_update(from_height, to_height, counterparty_chain_id)
                .await
                .map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        format!("error fetching update: {}", ErrorReporter(&*e)),
                        None::<()>,
                    )
                }),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        cb: ModuleCallback,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

impl Module {
    async fn beacon_slot_of_execution_block_number(&self, block_number: u64) -> RpcResult<u64> {
        let block = self
            .provider
            .get_block((block_number + 1).into(), BlockTransactionsKind::Hashes)
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching block: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?
            .expect("block should exist");

        let beacon_slot = self
            .beacon_api_client
            .block(
                <H256>::from(
                    block
                        .header
                        .parent_beacon_block_root
                        .expect("parent beacon block root should exist"),
                )
                .into(),
            )
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching block: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?;

        Ok(beacon_slot.data.message.slot)
    }

    /// Fetch a client update from the provided trusted height (`update_from`) to at least the
    /// desired new height (`update_to`).
    ///
    /// Note that this will generate updates as close to the tip of the chain as possible, as long
    /// as that height is > `update_to`. Due to the nature of ethereum finality, it is not possible
    /// to update to a *specific* height in the same way as is possible in chains with single slot
    /// finality (such as tendermint or cometbls). While it would be possible to update to a height
    /// *closer* to `update_to`, the extra complexity brought by that is unlikely to be worth the
    /// slightly smaller update generated, especially since in practice the light client will likely
    /// always be up to date with the tip of the (finalized) chain.
    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %counterparty_chain_id,
            %update_from_block_number,
            %update_to_block_number
        )
    )]
    async fn fetch_update(
        &self,
        update_from_block_number: Height,
        update_to_block_number: Height,
        counterparty_chain_id: ChainId,
    ) -> Result<Op<VoyagerMessage>, BoxDynError> {
        let finality_update = self
            .beacon_api_client
            .finality_update()
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching finality update"),
                    None::<()>,
                )
            })?
            .data;

        let spec = self
            .beacon_api_client
            .spec()
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching beacon spec"),
                    None::<()>,
                )
            })?
            .data;

        // === FETCH VALID FINALITY UPDATE

        let does_not_have_has_supermajority = {
            let sync_committee_bits = BitVec::<u8, Msb0>::try_from(
                finality_update.sync_aggregate.sync_committee_bits.clone(),
            )
            .expect("sync committee bits should be valid");

            let sync_committee_size = spec.sync_committee_size;

            assert_eq!(sync_committee_bits.len() as u64, sync_committee_size);

            sync_committee_bits.count_ones() * 3 < sync_committee_bits.len() * 2
        };

        if does_not_have_has_supermajority {
            info!(
                signature_slot = finality_update.signature_slot,
                "signature supermajority not hit"
            );

            return Ok(seq([
                defer(now() + 1),
                call(FetchUpdateHeaders {
                    chain_id: self.chain_id.clone(),
                    counterparty_chain_id,
                    update_from: update_from_block_number,
                    update_to: update_to_block_number,
                }),
            ]));
        };

        // === FETCH LIGHT CLIENT UPDATES

        let target_period =
            sync_committee_period(finality_update.finalized_header.beacon.slot, spec.period());

        let update_from_beacon_slot = self
            .beacon_slot_of_execution_block_number(update_from_block_number.height())
            .await?;

        let trusted_period = sync_committee_period(update_from_beacon_slot, spec.period());

        info!("target period: {target_period}, trusted period: {trusted_period}");

        assert!(
            trusted_period <= target_period,
            "trusted period {trusted_period} is behind target \
            period {target_period}, something is wrong!",
        );

        // Eth chain is more than 1 signature period ahead of us. We need to do sync committee
        // updates until we reach the `target_period - 1`.

        // let target_period = sync_committee_period(finality_update.signature_slot, spec.period());

        let light_client_updates = self
            .beacon_api_client
            .light_client_updates(trusted_period + 1, target_period - trusted_period)
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching light client updates"),
                    None::<()>,
                )
            })?
            .0
            .into_iter()
            .map(|x| x.data)
            .collect::<Vec<_>>();

        info!(
            "fetched {} light client updates",
            light_client_updates.len()
        );

        let (updates, last_update_block_number) = stream::iter(light_client_updates)
            .map(<RpcResult<_>>::Ok)
            .try_fold((VecDeque::new(), update_from_block_number.height()), {
                |(mut vec, mut trusted_block_number), update| {
                    let self_ = self.clone();
                    let spec = spec.clone();

                    async move {
                        let old_trusted_block_number = trusted_block_number;

                        // REVIEW: Assert that this is greater (i.e. increasing)?
                        trusted_block_number = update.finalized_header.execution.block_number;

                        vec.push_back(
                            self_
                                .make_header(
                                    old_trusted_block_number,
                                    LightClientUpdateData {
                                        attested_header: update.attested_header,
                                        finalized_header: update.finalized_header,
                                        finality_branch: update.finality_branch,
                                        sync_aggregate: update.sync_aggregate,
                                        signature_slot: update.signature_slot,
                                    },
                                    Some((
                                        update
                                            .next_sync_committee
                                            .expect("next_sync_committee should exist"),
                                        update
                                            .next_sync_committee_branch
                                            .expect("next_sync_committee_branch should exist"),
                                    )),
                                    &spec,
                                )
                                .await?,
                        );

                        Ok((vec, trusted_block_number))
                    }
                }
            })
            .await?;

        let lc_updates = if trusted_period < target_period {
            updates
        } else {
            [].into()
        };

        let does_not_have_finality_update =
            last_update_block_number >= update_to_block_number.height();

        debug!(last_update_block_number, %update_to_block_number);

        let finality_update_msg = if does_not_have_finality_update {
            info!("does not have finality update");
            // do nothing
            None
        } else {
            info!("has finality update");
            // do finality update
            Some(
                self.make_header(
                    last_update_block_number,
                    LightClientUpdateData {
                        attested_header: finality_update.attested_header,
                        finalized_header: finality_update.finalized_header,
                        finality_branch: finality_update.finality_branch,
                        sync_aggregate: finality_update.sync_aggregate,
                        signature_slot: finality_update.signature_slot,
                    },
                    None,
                    &spec,
                )
                .await?,
            )
        };

        let headers = lc_updates
            .into_iter()
            .chain(finality_update_msg)
            .collect::<Vec<_>>();

        // header.sort_by_key(|header| header.consensus_update.attested_header.beacon.slot);

        let genesis = self
            .beacon_api_client
            .genesis()
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching beacon genesis"),
                    None::<()>,
                )
            })?
            .data;

        let last_update_signature_slot = headers
            .iter()
            .map(|header| match &header.consensus_update {
                LightClientUpdate::EpochChange(update) => update.update_data.signature_slot,
                LightClientUpdate::WithinEpoch(update) => update.update_data.signature_slot,
            })
            .max()
            .expect("expected at least one update");

        Ok(seq([
            call(WaitForTimestamp {
                chain_id: counterparty_chain_id.clone(),
                // we wait for one more block just to be sure the counterparty's block time has caught up
                timestamp: i64::try_from(
                    (genesis.genesis_time + (last_update_signature_slot * spec.seconds_per_slot))
                        + spec.seconds_per_slot,
                )
                .expect("if this fails good luck")
                    * NANOS_PER_SECOND as i64,
                finalized: false,
            }),
            voyager_vm::data(OrderedHeaders {
                headers: headers
                    .into_iter()
                    .map(|header| {
                        (
                            DecodedHeaderMeta {
                                height: Height::new(match &header.consensus_update {
                                    LightClientUpdate::EpochChange(update) => {
                                        update.update_data.finalized_header.execution.block_number
                                    }
                                    LightClientUpdate::WithinEpoch(update) => {
                                        update.update_data.finalized_header.execution.block_number
                                    }
                                }),
                            },
                            into_value(header),
                        )
                    })
                    .collect(),
            }),
        ]))
    }

    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %currently_trusted_block_number,
            signature_slot = %light_client_update_data.signature_slot,
        )
    )]
    async fn make_header(
        &self,
        // the block number that this update is *from*
        currently_trusted_block_number: u64,
        light_client_update_data: LightClientUpdateData,
        // if this is an epoch change update, provide the next sync committee for the target epoch
        next_sync_committee: Option<(SyncCommittee, NextSyncCommitteeBranch)>,
        spec: &Spec,
    ) -> RpcResult<Header> {
        // When we fetch the update at this height, the `next_sync_committee` will
        // be the current sync committee of the period that we want to update to.
        let previous_period = u64::max(
            1,
            light_client_update_data.finalized_header.beacon.slot / spec.period(),
        ) - 1;

        let ibc_account_proof = self
            .fetch_account_update(
                light_client_update_data
                    .finalized_header
                    .execution
                    .block_number,
            )
            .await?;

        let previous_period_light_client_update = self
            .beacon_api_client
            .light_client_updates(previous_period, 1)
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e)
                        .with_message("error fetching previous period light client update"),
                    None::<()>,
                )
            })?
            .0
            .into_iter()
            .map(|x| x.data)
            .collect::<Vec<_>>()
            .pop()
            .expect("one update was requested, if the rpc returns a value it should be valid here");

        Ok(Header {
            consensus_update: match next_sync_committee {
                Some((next_sync_committee, next_sync_committee_branch)) => {
                    LightClientUpdate::EpochChange(Box::new(EpochChangeUpdate {
                        sync_committee: previous_period_light_client_update
                            .next_sync_committee
                            .expect("next_sync_committee should exist"),
                        next_sync_committee,
                        next_sync_committee_branch,
                        update_data: light_client_update_data,
                    }))
                }
                None => LightClientUpdate::WithinEpoch(Box::new(WithinEpochUpdate {
                    sync_committee: previous_period_light_client_update
                        .next_sync_committee
                        .expect("next_sync_committee should exist"),
                    update_data: light_client_update_data,
                })),
            },
            trusted_height: Height::new(currently_trusted_block_number),
            ibc_account_proof,
        })
    }
}

// REVIEW: Does this function exist anywhere else?
fn sync_committee_period(slot: u64, period: u64) -> u64 {
    slot.div(period)
}
