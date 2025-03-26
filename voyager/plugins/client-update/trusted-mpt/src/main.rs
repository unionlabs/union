#![warn(clippy::unwrap_used)]

use std::collections::VecDeque;

use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use ethereum_light_client_types::AccountProof;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};
use trusted_mpt_light_client_types::Header;
use unionlabs::{ibc::core::client::height::Height, primitives::H160, ErrorReporter};
use voyager_message::{
    call::Call,
    core::{ChainId, ClientType},
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    hook::UpdateHook,
    into_value,
    module::{PluginInfo, PluginServer},
    DefaultCmd, Plugin, PluginMessage, VoyagerMessage,
};
use voyager_vm::{pass::PassResult, BoxDynError, Op, Visit};

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

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub provider: DynProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,

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

    #[instrument(
        skip_all,
        fields(
            %block_number,
            ibc_handler_address = %self.ibc_handler_address
        )
    )]
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

        // tokio::time::sleep(std::time::Duration::from_millis(500));

        debug!(storage_hash = %account_update.storage_hash, "fetched account update");

        Ok(AccountProof {
            storage_root: account_update.storage_hash.into(),
            proof: account_update
                .account_proof
                .into_iter()
                .map(|x| x.into())
                .collect(),
        })
    }
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                // .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.rpc_url)
                .await?,
        );

        let chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        if chain_id != config.chain_id {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            )
            .into());
        }

        Ok(Self {
            chain_id,
            ibc_handler_address: config.ibc_handler_address,
            provider,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(
                &config.chain_id,
                &ClientType::new(ClientType::TRUSTED_MPT),
            ),
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
                    UpdateHook::new(
                        &self.chain_id,
                        &ClientType::new(ClientType::TRUSTED_MPT),
                        |fetch| {
                            Call::Plugin(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchUpdate {
                                    from_height: fetch.update_from,
                                    to_height: fetch.update_to,
                                    counterparty_chain_id: fetch.counterparty_chain_id.clone(),
                                    client_id: fetch.client_id.clone(),
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
        println!("FETCH  update BRO?????");
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate {
                from_height,
                to_height,
                counterparty_chain_id,
                ..
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
        if update_from_block_number == update_to_block_number {
            info!("update is for the same height, noop");
            return Ok(voyager_vm::data(OrderedHeaders { headers: vec![] }));
        }

        let header = self
            .provider
            .get_block_by_number(update_to_block_number.height().into())
            .await
            .expect("big trouble")
            .expect("big trouble")
            .header;

        let ibc_account_proof = self
            .fetch_account_update(update_to_block_number.height())
            .await?;

        let header = Header {
            state_root: header.state_root.0.into(),
            ibc_account_proof,
            height: update_to_block_number.height(),
            timestamp: header.timestamp,
        };

        Ok(voyager_vm::data(OrderedHeaders {
            headers: vec![(
                DecodedHeaderMeta {
                    height: update_to_block_number,
                },
                into_value(header),
            )],
        }))
    }
}
