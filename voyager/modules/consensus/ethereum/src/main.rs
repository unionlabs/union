use std::{collections::VecDeque, ops::Div};

use beacon_api::{client::BeaconApiClient, types::Spec};
use bitvec::{order::Msb0, vec::BitVec};
use chain_utils::{ethereum::ETHEREUM_REVISION_NUMBER, BoxDynError};
use ethers::providers::{Middleware, Provider, ProviderError, Ws, WsClientError};
use futures::{stream, StreamExt};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
};
use queue_msg::{call, data, defer, now, seq, Op};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, instrument};
use unionlabs::{
    constants::metric::NANOS_PER_SECOND,
    ethereum::{config::PresetBaseKind, IBC_HANDLER_COMMITMENTS_SLOT},
    hash::H160,
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{
            self,
            account_proof::AccountProof,
            account_update::AccountUpdate,
            header::UnboundedHeader,
            light_client_update::UnboundedLightClientUpdate,
            trusted_sync_committee::{UnboundedActiveSyncCommittee, UnboundedTrustedSyncCommittee},
        },
    },
    ErrorReporter,
};
use voyager_message::{
    call::{Call, FetchUpdateHeaders, WaitForTimestamp},
    core::{ChainId, ClientType},
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    module::{ConsensusModuleInfo, ConsensusModuleServer, ModuleInfo, QueueInteractionsServer},
    run_module_server, DefaultCmd, ModuleContext, ModuleServer, VoyagerMessage,
};

use crate::{
    call::{
        FetchAccountUpdate, FetchBeaconGenesis, FetchBeaconSpec, FetchBootstrap,
        FetchFinalityUpdate, FetchLightClientUpdate, FetchLightClientUpdates, ModuleCall,
    },
    callback::ModuleCallback,
    data::{
        AccountUpdateData, BeaconGenesis, BeaconSpec, BootstrapData, FinalityUpdate,
        LightClientUpdates, ModuleData,
    },
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server::<Module, _, _, _>().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

    pub chain_spec: PresetBaseKind,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub provider: Provider<Ws>,
    pub beacon_api_client: BeaconApiClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: ChainId<'static>,

    pub chain_spec: PresetBaseKind,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

fn plugin_name(chain_id: &ChainId<'_>) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    pub async fn fetch_account_update(&self, slot: u64) -> AccountUpdate {
        let execution_height = self
            .beacon_api_client
            .execution_height(beacon_api::client::BlockId::Slot(slot))
            .await
            .unwrap();

        let account_update = self
            .provider
            .get_proof(
                ethers::types::H160::from(self.ibc_handler_address),
                vec![],
                // NOTE: Proofs are from the execution layer, so we use execution height, not beacon slot.
                Some(execution_height.into()),
            )
            .await
            .unwrap();

        AccountUpdate {
            account_proof: AccountProof {
                storage_root: account_update.storage_hash.into(),
                proof: account_update
                    .account_proof
                    .into_iter()
                    .map(|x| x.to_vec())
                    .collect(),
            },
        }
    }
}

impl ModuleContext for Module {
    type Config = Config;
    type Cmd = DefaultCmd;
    type Info = ConsensusModuleInfo;

    async fn new(config: Self::Config) -> Result<Self, chain_utils::BoxDynError> {
        let provider = Provider::new(Ws::connect(config.eth_rpc_api).await?);

        let chain_id = ChainId::new(provider.get_chainid().await?.to_string());

        if chain_id != config.chain_id {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            )
            .into());
        }

        let beacon_api_client = BeaconApiClient::new(config.eth_beacon_rpc_api).await?;

        let spec = beacon_api_client.spec().await.unwrap().data;

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

    fn info(config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            name: plugin_name(&config.chain_id),
            kind: ConsensusModuleInfo {
                chain_id: config.chain_id,
                client_type: ClientType::new(match config.chain_spec {
                    PresetBaseKind::Minimal => ClientType::ETHEREUM_MINIMAL,
                    PresetBaseKind::Mainnet => ClientType::ETHEREUM_MAINNET,
                }),
            },
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleInitError {
    #[error("unable to connect to websocket")]
    Ws(#[from] WsClientError),
    #[error("provider error")]
    Provider(#[from] ProviderError),
    #[error("beacon error")]
    Beacon(#[from] beacon_api::client::NewError),
}

#[async_trait]
impl QueueInteractionsServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        let beacon_api_client = &self.ctx.beacon_api_client;

        match msg {
            ModuleCall::FetchFinalityUpdate(FetchFinalityUpdate {}) => {
                let finality_update = beacon_api_client.finality_update().await.unwrap().data;

                let has_supermajority = {
                    let scb = BitVec::<u8, Msb0>::try_from(
                        finality_update.sync_aggregate.sync_committee_bits.clone(),
                    )
                    .unwrap();

                    let sync_committee_size = self
                        .ctx
                        .beacon_api_client
                        .spec()
                        .await
                        .unwrap()
                        .data
                        .sync_committee_size;

                    assert_eq!(scb.len() as u64, sync_committee_size);

                    scb.count_ones() * 3 < scb.len() * 2
                };

                if has_supermajority {
                    info!(
                        signature_slot = finality_update.signature_slot,
                        "signature supermajority not hit"
                    );

                    Ok(seq([
                        defer(now() + 1),
                        call(Call::plugin(self.ctx.plugin_name(), FetchFinalityUpdate {})),
                    ]))
                } else {
                    Ok(data(Data::plugin(
                        self.ctx.plugin_name(),
                        FinalityUpdate { finality_update },
                    )))
                }
            }
            ModuleCall::FetchLightClientUpdates(FetchLightClientUpdates {
                trusted_period,
                target_period,
            }) => Ok(data(Data::plugin(
                self.ctx.plugin_name(),
                LightClientUpdates {
                    light_client_updates: beacon_api_client
                        .light_client_updates(trusted_period + 1, target_period - trusted_period)
                        .await
                        .unwrap()
                        .0
                        .into_iter()
                        .map(|x| x.data)
                        .collect(),
                },
            ))),
            ModuleCall::FetchLightClientUpdate(FetchLightClientUpdate { period }) => {
                Ok(data(Data::plugin(
                    self.ctx.plugin_name(),
                    crate::data::LightClientUpdate {
                        update: beacon_api_client
                            .light_client_updates(period, 1)
                            .await
                            .unwrap()
                            .0
                            .into_iter()
                            .map(|x| x.data)
                            .collect::<Vec<_>>()
                            .pop()
                            .unwrap(),
                    },
                )))
            }
            ModuleCall::FetchBootstrap(FetchBootstrap { slot }) => Ok(data(Data::plugin(
                self.ctx.plugin_name(),
                BootstrapData {
                    slot,
                    bootstrap: beacon_api_client
                        .bootstrap_for_slot(slot)
                        .await
                        .unwrap()
                        .data,
                },
            ))),
            ModuleCall::FetchAccountUpdate(FetchAccountUpdate { slot }) => {
                let execution_height = beacon_api_client
                    .execution_height(beacon_api::client::BlockId::Slot(slot))
                    .await
                    .unwrap();

                let account_update = self
                    .ctx
                    .provider
                    .get_proof(
                        ethers::types::H160::from(self.ctx.ibc_handler_address),
                        vec![],
                        // NOTE: Proofs are from the execution layer, so we use execution height, not beacon slot.
                        Some(execution_height.into()),
                    )
                    .await
                    .unwrap();

                Ok(data(Data::plugin(
                    self.ctx.plugin_name(),
                    AccountUpdateData {
                        slot,
                        update: AccountUpdate {
                            account_proof: AccountProof {
                                storage_root: account_update.storage_hash.into(),
                                proof: account_update
                                    .account_proof
                                    .into_iter()
                                    .map(|x| x.to_vec())
                                    .collect(),
                            },
                        },
                    },
                )))
            }
            ModuleCall::FetchBeaconGenesis(FetchBeaconGenesis {}) => Ok(data(Data::plugin(
                self.ctx.plugin_name(),
                BeaconGenesis {
                    genesis: beacon_api_client.genesis().await.unwrap().data,
                },
            ))),
            ModuleCall::FetchBeaconSpec(FetchBeaconSpec {}) => Ok(data(Data::plugin(
                self.ctx.plugin_name(),
                BeaconSpec {
                    spec: beacon_api_client.spec().await.unwrap().data,
                },
            ))),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn callback(
        &self,
        cb: ModuleCallback,
        _data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {
            // ModuleCallback::CreateUpdate(aggregate) => do_callback(aggregate, data),
            // ModuleCallback::MakeCreateUpdates(aggregate) => do_callback(aggregate, data),
            // ModuleCallback::MakeCreateUpdatesFromLightClientUpdates(aggregate) => {
            //     do_callback(aggregate, data)
            // }
            // ModuleCallback::AggregateHeaders(cb) => {
            //     cb.aggregate(&self.ctx.beacon_api_client, data).await?
            // }
        }
    }
}

#[async_trait]
impl ConsensusModuleServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn self_client_state(&self, height: Height) -> RpcResult<Value> {
        let genesis = self.ctx.beacon_api_client.genesis().await.unwrap().data;

        let spec = self.ctx.beacon_api_client.spec().await.unwrap().data;

        Ok(serde_json::to_value(ethereum::client_state::ClientState {
            chain_id: self
                .ctx
                .chain_id
                .as_str()
                .parse()
                .expect("self.chain_id is a valid u256"),
            genesis_validators_root: genesis.genesis_validators_root,
            genesis_time: genesis.genesis_time,
            fork_parameters: spec.to_fork_parameters(),
            seconds_per_slot: spec.seconds_per_slot,
            slots_per_epoch: spec.slots_per_epoch,
            epochs_per_sync_committee_period: spec.epochs_per_sync_committee_period,
            latest_slot: height.revision_height,
            min_sync_committee_participants: 0,
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            ibc_commitment_slot: IBC_HANDLER_COMMITMENTS_SLOT,
            ibc_contract_address: self.ctx.ibc_handler_address,
        })
        .expect("infallible"))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn self_consensus_state(&self, height: Height) -> RpcResult<Value> {
        let beacon_api_client = &self.ctx.beacon_api_client;

        let trusted_header = beacon_api_client
            .header(beacon_api::client::BlockId::Slot(height.revision_height))
            .await
            .unwrap()
            .data;

        let bootstrap = beacon_api_client
            .bootstrap(trusted_header.root)
            .await
            .unwrap()
            .data;

        let spec = self.ctx.beacon_api_client.spec().await.unwrap().data;

        assert!(bootstrap.header.beacon.slot == height.revision_height);

        let light_client_update = {
            let current_period = height.revision_height.div(spec.period());

            debug!(%current_period);

            let light_client_updates = beacon_api_client
                .light_client_updates(current_period, 1)
                .await
                .unwrap();

            let [light_client_update] = &*light_client_updates.0 else {
                panic!()
            };

            light_client_update.data.clone()
        };

        // Normalize to nanos in order to be compliant with cosmos
        let timestamp = bootstrap.header.execution.timestamp * 1_000_000_000;

        Ok(
            serde_json::to_value(ethereum::consensus_state::ConsensusState {
                slot: bootstrap.header.beacon.slot,
                state_root: bootstrap.header.execution.state_root,
                storage_root: self
                    .ctx
                    .provider
                    .get_proof(
                        ethers::types::H160::from(*self.ctx.ibc_handler_address.get()),
                        vec![],
                        Some(bootstrap.header.execution.block_number.into()),
                    )
                    .await
                    .unwrap()
                    .storage_hash
                    .0
                    .into(),
                timestamp,
                current_sync_committee: bootstrap.current_sync_committee.aggregate_pubkey,
                next_sync_committee: light_client_update
                    .next_sync_committee
                    .map(|nsc| nsc.aggregate_pubkey),
            })
            .expect("infallible"),
        )
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn fetch_update_headers(
        &self,
        update_from: Height,
        update_to: Height,
        counterparty_chain_id: ChainId<'static>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        // Ok(promise(
        //     [
        //         call(Call::plugin(self.ctx.plugin_name(), FetchFinalityUpdate {})),
        //         call(Call::plugin(self.ctx.plugin_name(), FetchBeaconSpec {})),
        //     ],
        //     [],
        //     Callback::plugin(
        //         self.ctx.plugin_name(),
        //         MakeCreateUpdates {
        //             update_from,
        //             update_to,
        //             counterparty_chain_id,
        //         },
        //     ),
        // ))

        self.ctx
            .fetch_update(update_from, update_to, counterparty_chain_id)
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching update: {}", ErrorReporter(&*e)),
                    None::<()>,
                )
            })
    }
}

impl Module {
    /// Fetch a client update from the provided trusted height (`update_from`) to at least the desired new height (`update_to`).
    ///
    /// Note that this will generate updates as close to the tip of the chain as possible, as long as that height is > `update_to`. Due to the nature of ethereum finality, it is not possible to update to a *specific* height in the same way as is possible in chains with single slot finality (such as tendermint or cometbls). While it would be possible to update to a height *closer* to `update_to`, the extra complexity brought by that is unlikely to be worth the slightly smaller update generated, especially since in practice the light client will likely always be up to date with the tip of the (finalized) chain.
    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %counterparty_chain_id,
            %update_from,
            %update_to
        )
    )]
    async fn fetch_update(
        &self,
        update_from: Height,
        update_to: Height,
        counterparty_chain_id: ChainId<'static>,
    ) -> Result<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>, BoxDynError> {
        let finality_update = self.beacon_api_client.finality_update().await.unwrap().data;

        // === FETCH VALID FINALITY UPDATE

        // TODO: This is named poorly?
        let has_supermajority = {
            let scb = BitVec::<u8, Msb0>::try_from(
                finality_update.sync_aggregate.sync_committee_bits.clone(),
            )
            .unwrap();

            let sync_committee_size = self
                .beacon_api_client
                .spec()
                .await
                .unwrap()
                .data
                .sync_committee_size;

            assert_eq!(scb.len() as u64, sync_committee_size);

            scb.count_ones() * 3 < scb.len() * 2
        };

        if has_supermajority {
            info!(
                signature_slot = finality_update.signature_slot,
                "signature supermajority not hit"
            );

            return Ok(seq([
                defer(now() + 1),
                call(FetchUpdateHeaders {
                    chain_id: self.chain_id.clone(),
                    counterparty_chain_id,
                    update_from,
                    update_to,
                }),
            ]));
        };

        // === FETCH LIGHT CLIENT UPDATES

        let spec = self.beacon_api_client.spec().await.unwrap().data;

        let target_period =
            sync_committee_period(finality_update.attested_header.beacon.slot, spec.period());

        let trusted_period = sync_committee_period(update_from.revision_height, spec.period());

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
            .unwrap()
            .0
            .into_iter()
            .map(|x| x.data)
            .collect::<Vec<_>>();

        info!(
            "fetched {} light client updates",
            light_client_updates.len()
        );

        let (updates, last_update_block_number) = stream::iter(light_client_updates)
            .fold((VecDeque::new(), update_from.revision_height), {
                |(mut vec, mut trusted_slot), update| {
                    let self_ = self.clone();
                    let spec = spec.clone();

                    async move {
                        let old_trusted_slot = trusted_slot;

                        // REVIEW: Assert that this is greater (i.e. increasing)?
                        trusted_slot = update.attested_header.beacon.slot;

                        vec.push_back(
                            self_
                                .make_header(old_trusted_slot, update, true, &spec)
                                .await,
                        );

                        (vec, trusted_slot)
                    }
                }
            })
            .await;

        let lc_updates = if trusted_period < target_period {
            updates
        } else {
            [].into()
        };

        let does_not_have_finality_update = last_update_block_number >= update_to.revision_height;

        debug!(last_update_block_number, update_to.revision_height);

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
                    UnboundedLightClientUpdate {
                        attested_header: finality_update.attested_header,
                        next_sync_committee: None,
                        next_sync_committee_branch: None,
                        finalized_header: finality_update.finalized_header,
                        finality_branch: finality_update.finality_branch,
                        sync_aggregate: finality_update.sync_aggregate,
                        signature_slot: finality_update.signature_slot,
                    },
                    false,
                    &spec,
                )
                .await,
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
                    format!("error fetching beacon genesis: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?
            .data;

        let last_update_signature_slot = headers
            .iter()
            .map(|h| h.consensus_update.signature_slot)
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
                .unwrap()
                    * NANOS_PER_SECOND as i64,
            }),
            queue_msg::data(OrderedHeaders {
                headers: headers
                    .into_iter()
                    .map(|header| {
                        (
                            DecodedHeaderMeta {
                                height: Height {
                                    revision_number: ETHEREUM_REVISION_NUMBER,
                                    revision_height: header
                                        .consensus_update
                                        .attested_header
                                        .beacon
                                        .slot,
                                },
                            },
                            serde_json::to_value(header).unwrap(),
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
            %currently_trusted_slot,
            signature_slot = %light_client_update.signature_slot,
            %is_next,
        )
    )]
    async fn make_header(
        &self,
        currently_trusted_slot: u64,
        light_client_update: UnboundedLightClientUpdate,
        is_next: bool,
        spec: &Spec,
    ) -> UnboundedHeader {
        // When we fetch the update at this height, the `next_sync_committee` will
        // be the current sync committee of the period that we want to update to.
        let previous_period = u64::max(
            1,
            light_client_update.attested_header.beacon.slot / spec.period(),
        ) - 1;

        let account_update = self
            .fetch_account_update(light_client_update.attested_header.beacon.slot)
            .await;

        let previous_period_light_client_update = self
            .beacon_api_client
            .light_client_updates(previous_period, 1)
            .await
            .unwrap()
            .0
            .into_iter()
            .map(|x| x.data)
            .collect::<Vec<_>>()
            .pop()
            .unwrap();

        UnboundedHeader {
            consensus_update: light_client_update,
            trusted_sync_committee: UnboundedTrustedSyncCommittee {
                trusted_height: Height {
                    revision_number: ETHEREUM_REVISION_NUMBER,
                    revision_height: currently_trusted_slot,
                },
                sync_committee: if is_next {
                    UnboundedActiveSyncCommittee::Next(
                        previous_period_light_client_update
                            .next_sync_committee
                            .unwrap(),
                    )
                } else {
                    UnboundedActiveSyncCommittee::Current(
                        previous_period_light_client_update
                            .next_sync_committee
                            .unwrap(),
                    )
                },
            },
            account_update,
        }
    }
}

// REVIEW: Does this function exist anywhere else?
fn sync_committee_period(height: u64, period: u64) -> u64 {
    height.div(period)
}
