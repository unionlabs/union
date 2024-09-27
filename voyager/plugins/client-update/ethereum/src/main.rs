use std::{collections::VecDeque, ops::Div};

use beacon_api::{client::BeaconApiClient, types::Spec};
use bitvec::{order::Msb0, vec::BitVec};
use chain_utils::{ethereum::ETHEREUM_REVISION_NUMBER, BoxDynError};
use ethers::providers::{Middleware, Provider, ProviderError, Ws, WsClientError};
use futures::{stream, StreamExt};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use queue_msg::{call, defer, now, optimize::OptimizationResult, seq, Op};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};
use unionlabs::{
    constants::metric::NANOS_PER_SECOND,
    ethereum::config::PresetBaseKind,
    hash::H160,
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{
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
    core::ChainId,
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    module::{ModuleInfo, PluginInfo, PluginServer, PluginTypes},
    run_module_server, DefaultCmd, ModuleContext, VoyagerMessage,
};

use crate::{
    call::{FetchUpdate, ModuleCall},
    callback::ModuleCallback,
    data::ModuleData,
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server::<Module>().await
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

impl PluginTypes for Module {
    type D = ModuleData;
    type C = ModuleCall;
    type Cb = ModuleCallback;
}

impl ModuleContext for Module {
    type Config = Config;
    type Cmd = DefaultCmd;
    type Info = PluginInfo;

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
            kind: PluginInfo {
                name: plugin_name(&config.chain_id),
                interest_filter: format!(
                    r#"[.. | ."@type"? == "fetch_update_headers" and ."@value".chain_id == "{}"] | any"#,
                    config.chain_id
                ),
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
impl PluginServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>>,
    ) -> RpcResult<OptimizationResult<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(OptimizationResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .map(|op| match op {
                    Op::Call(Call::FetchUpdateHeaders(fetch))
                        if fetch.chain_id == self.chain_id =>
                    {
                        call(Call::plugin(
                            self.plugin_name(),
                            FetchUpdate {
                                from_height: fetch.update_from,
                                to_height: fetch.update_to,
                                counterparty_chain_id: fetch.counterparty_chain_id,
                            },
                        ))
                    }
                    op => op,
                })
                .enumerate()
                .map(|(i, op)| (vec![i], op))
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(
        &self,
        _: &Extensions,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
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
        _data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {
            // ModuleCallback::CreateUpdate(aggregate) => do_callback(aggregate, data),
            // ModuleCallback::MakeCreateUpdates(aggregate) => do_callback(aggregate, data),
            // ModuleCallback::MakeCreateUpdatesFromLightClientUpdates(aggregate) => {
            //     do_callback(aggregate, data)
            // }
            // ModuleCallback::AggregateHeaders(cb) => {
            //     cb.aggregate(&self.beacon_api_client, data).await?
            // }
        }
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
