// #![warn(clippy::unwrap_used)]

use std::{cmp::Ordering, collections::VecDeque};

use cainome_cairo_serde::CairoSerde;
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use starknet::{
    core::types::{BlockId, EventFilter, Felt},
    macros::selector,
    providers::{JsonRpcClient, Provider, Url, jsonrpc::HttpTransport},
};
use tracing::{info, instrument, warn};
use unionlabs::{ibc::core::client::height::Height, never::Never, primitives::H256};
use voyager_sdk::{
    ExtensionsExt, VoyagerClient,
    anyhow::{self, bail},
    hook::simple_take_filter,
    into_value,
    message::{
        PluginMessage, VoyagerMessage,
        call::{Call, WaitForHeight},
        data::{Data, EventProvableHeight},
    },
    plugin::Plugin,
    primitives::ChainId,
    rpc::{PluginServer, RpcError, RpcResult, types::PluginInfo},
    vm::{Op, call, conc, pass::PassResult, seq},
};

use crate::{
    call::{FetchBlock, FetchBlocks, MakeChainEvent, ModuleCall},
    ibc_events::CairoIbcEvent,
};

pub mod ibc_events;

pub mod call;

#[tokio::main]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub client: JsonRpcClient<HttpTransport>,

    pub chunk_block_fetch_size: u64,

    pub index_trivial_events: bool,

    pub ibc_host_contract_address: Felt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,

    pub rpc_url: String,

    #[serde(default = "default_chunk_block_fetch_size")]
    pub chunk_block_fetch_size: u64,

    /// Whether or not to fully index events that do not produce a counterparty action (packet_recv, packet_acknowledgement, packet_timeout, update_client).
    #[serde(default)]
    pub index_trivial_events: bool,

    #[serde(default)]
    pub ibc_host_contract_address: Felt,
}

fn default_chunk_block_fetch_size() -> u64 {
    10
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    /// Return an op to fetch the events from a single block from the chain.
    FetchSingleBlock { height: u64 },
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = Cmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        let client = JsonRpcClient::new(HttpTransport::new(Url::parse(&config.rpc_url)?));

        let chain_id = ChainId::new(client.chain_id().await?.to_string());

        if chain_id != config.chain_id {
            bail!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id,
                chain_id
            );
        }

        Ok(Self {
            client,
            chain_id,
            chunk_block_fetch_size: config.chunk_block_fetch_size,
            index_trivial_events: config.index_trivial_events,
            ibc_host_contract_address: config.ibc_host_contract_address,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: simple_take_filter(format!(
                r#"[.. | (."@type"? == "index" or ."@type"? == "index_range") and ."@value".chain_id == "{}"] | any"#,
                config.chain_id
            )),
        }
    }

    async fn cmd(config: Self::Config, cmd: Self::Cmd) {
        match cmd {
            Cmd::FetchSingleBlock { height } => {
                print!(
                    "{}",
                    into_value(call::<VoyagerMessage>(PluginMessage::new(
                        plugin_name(&config.chain_id),
                        ModuleCall::from(FetchBlock { height })
                    )))
                )
            }
        }
    }
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new(height)
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
                .map(|op| match op {
                    Op::Call(Call::Index(fetch)) if fetch.chain_id == self.chain_id => {
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchBlocks {
                                height: fetch.start_height.height(),
                                until: None,
                            }),
                        ))
                    }
                    Op::Call(Call::IndexRange(fetch)) if fetch.chain_id == self.chain_id => {
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchBlocks {
                                height: fetch.range.from_height().height(),
                                until: Some(fetch.range.to_height().height()),
                            }),
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
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchBlocks(FetchBlocks { height, until }) => {
                self.fetch_blocks(e.voyager_client()?, height, until).await
            }
            ModuleCall::FetchBlock(FetchBlock { height }) => self.fetch_block(height).await,
            ModuleCall::MakeChainEvent(MakeChainEvent {
                block_number,
                tx_hash,
                event,
            }) => {
                self.make_chain_event(e.voyager_client()?, block_number, tx_hash, event)
                    .await
            }
        }
    }
}

impl Module {
    #[instrument(skip_all, fields(%height))]
    async fn fetch_blocks(
        &self,
        voyager_client: &VoyagerClient,
        height: u64,
        until: Option<u64>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        if let Some(until) = until {
            if height > until {
                return Err(RpcError::fatal_from_message(format!(
                    "height {height} cannot be greater than the until height {until}"
                )));
            } else if height == until {
                // if this is a ranged fetch, we need to fetch the upper bound of the range individually since FetchBlocks is exclusive on the upper bound
                return Ok(call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::from(FetchBlock { height }),
                )));
            }
        }

        let latest_height = voyager_client
            .query_latest_height(self.chain_id.clone(), true)
            .await?
            .height();

        info!(%latest_height, %height, ?until, "fetching blocks");

        let continuation = |next_height: u64| {
            seq([
                // TODO: Make this a config param
                call(WaitForHeight {
                    chain_id: self.chain_id.clone(),
                    height: Height::new(next_height),
                    finalized: true,
                }),
                call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::from(FetchBlocks {
                        height: next_height,
                        until,
                    }),
                )),
            ])
        };

        match height.cmp(&latest_height) {
            // height < latest_height
            // fetch transactions on all blocks height..next_height (*exclusive* on the upper bound!)
            // and then queue the continuation starting at next_height
            Ordering::Equal | Ordering::Less => {
                let next_height =
                    (latest_height - height).clamp(1, self.chunk_block_fetch_size) + height;

                let next_height = next_height.min(until.map_or(next_height, |until| until));

                info!(
                    from_height = height,
                    to_height = next_height,
                    ?until,
                    "batch fetching blocks in range {height}..{next_height}"
                );

                Ok(conc(
                    (height..next_height)
                        .map(|h| {
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchBlock { height: h }),
                            ))
                        })
                        .chain([continuation(next_height)]),
                ))
            }
            Ordering::Greater => {
                warn!(
                    "the latest finalized height ({latest_height}) \
                    is less than the requested height ({height})"
                );

                Ok(continuation(height))
            }
        }
    }

    #[instrument(skip_all, fields(height))]
    async fn fetch_block(&self, block_number: u64) -> RpcResult<Op<VoyagerMessage>> {
        info!(%block_number, "fetching events in block");

        // list of MakeChainEvent ops that will be queued in a conc
        let mut make_chain_event_ops: Vec<Op<VoyagerMessage>> = vec![];

        let mut continuation_token = None::<String>;

        loop {
            info!(%block_number, continuation_token = %continuation_token.as_deref().unwrap_or_default(), "fetching page");

            let response = self
                .client
                .get_events(
                    EventFilter {
                        from_block: Some(BlockId::Number(block_number)),
                        to_block: Some(BlockId::Number(block_number)),
                        address: Some(self.ibc_host_contract_address),
                        keys: None,
                    },
                    continuation_token,
                    // https://github.com/eqlabs/pathfinder/blob/a34566b9a9f6ea6d7eb3889130d62c8f3fe6a499/crates/rpc/src/method/get_events.rs#L15
                    1024,
                )
                .await
                .map_err(RpcError::retryable(format_args!(
                    "error fetching events for block {block_number}"
                )))?;

            for emitted_event in response.events {
                use cainome_cairo_serde::{ByteArray, ContractAddress, NonZero};

                if emitted_event.keys[0] == selector!("ConnectionOpenInit") {
                    let (connection_id, client_id) =
                        <(NonZero<u32>, NonZero<u32>)>::cairo_deserialize(&emitted_event.keys, 1)
                            .unwrap();

                    let counterparty_client_id =
                        <NonZero<u32>>::cairo_deserialize(&emitted_event.data, 0).unwrap();

                    let event = CairoIbcEvent::ConnectionOpenInit {
                        connection_id,
                        client_id,
                        counterparty_client_id,
                    };

                    make_chain_event_ops.push(call(PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::from(MakeChainEvent {
                            block_number,
                            tx_hash: emitted_event.transaction_hash.to_bytes_be().into(),
                            event,
                        }),
                    )));
                } else if emitted_event.keys[0] == selector!("ChannelOpenTry") {
                    let (port_id, channel_id, counterparty_version) =
                        <(ContractAddress, NonZero<u32>, ByteArray)>::cairo_deserialize(
                            &emitted_event.keys,
                            1,
                        )
                        .unwrap();

                    let (counterparty_port_id, counterparty_channel_id, connection_id) =
                        <(ByteArray, NonZero<u32>, NonZero<u32>)>::cairo_deserialize(
                            &emitted_event.data,
                            0,
                        )
                        .unwrap();

                    let event = CairoIbcEvent::ChannelOpenTry {
                        port_id,
                        channel_id,
                        counterparty_port_id,
                        counterparty_channel_id,
                        connection_id,
                        counterparty_version,
                    };

                    dbg!(event, emitted_event);
                }
            }

            if response.continuation_token.is_none() {
                break;
            }

            continuation_token = response.continuation_token;
        }

        Ok(conc(make_chain_event_ops.into_iter()))
    }

    #[instrument(level = "info", skip_all, fields(%height, %tx_hash))]
    async fn make_chain_event(
        &self,
        _voyager_client: &VoyagerClient,
        height: u64,
        tx_hash: H256,
        _event: CairoIbcEvent,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let _provable_height = EventProvableHeight::Min(Height::new(height));

        unimplemented!()
    }
}
