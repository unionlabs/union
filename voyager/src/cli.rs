use std::{ffi::OsString, str::FromStr};

use clap::{self, Parser, Subcommand};
use ibc_union_spec::IbcUnion;
use unionlabs::{self, bounded::BoundedI64, ibc::core::client::height::Height, result_unwrap};
use voyager_message::{
    module::{ClientModuleInfo, ConsensusModuleInfo, ProofModuleInfo, StateModuleInfo},
    primitives::{ChainId, ClientType, IbcInterface, IbcSpec, IbcSpecId, QueryHeight},
    RawClientId, VoyagerMessage,
};
use voyager_vm::{BoxDynError, Op};

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct AppArgs {
    #[arg(
        long,
        short = 'c',
        // env = "VOYAGER_CONFIG_FILE_PATH",
        global = true,
        help_heading = "Global options"
    )]
    pub config_file_path: Option<OsString>,
    #[arg(
        long,
        short = 'l',
        // env = "VOYAGER_LOG_FORMAT",
        global = true,
        default_value_t = LogFormat::default(),
        help_heading = "Global options"
    )]
    pub log_format: LogFormat,
    #[arg(
        long,
        // env = "VOYAGER_STACK_SIZE",
        global = true,
        default_value_t = 2 * 1024 * 1024,
        help_heading = "Global options"
    )]
    pub stack_size: usize,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Default, clap::ValueEnum, derive_more::Display)]
pub enum LogFormat {
    #[default]
    #[display(fmt = "text")]
    Text,
    #[display(fmt = "json")]
    Json,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// Config related subcommands.
    #[command(subcommand)]
    Config(ConfigCmd),
    /// Construct a `FetchBlocks` op to send to the specified chain.
    InitFetch {
        #[arg(value_parser(|s: &str| Ok::<_, BoxDynError>(ChainId::new(s.to_owned()))))]
        chain_id: ChainId,
        /// The height to start fetching blocks at.
        #[arg(long, short = 'H', default_value_t = QueryHeight::Latest)]
        height: QueryHeight,
        /// Automatically enqueue the op.
        #[arg(long, short = 'e', default_value_t = false)]
        enqueue: bool,
        #[arg(long, global = true)]
        rpc_url: Option<String>,
        #[arg(long, global = true)]
        rest_url: Option<String>,
    },
    /// Run Voyager.
    Start,
    /// Query and interact with the queue.
    #[command(subcommand, alias = "q")]
    Queue(QueueCmd),
    #[command(subcommand)]
    Plugin(PluginCmd),
    #[command(subcommand)]
    Module(ModuleCmd),
    /// Call into the JSON-RPC of a running voyager instance.
    Rpc {
        #[arg(long, short = 'r', global = true)]
        rpc_url: Option<String>,
        #[command(subcommand)]
        cmd: RpcCmd,
    },
    #[command(subcommand)]
    Msg(MsgCmd),
}

#[derive(Debug, Subcommand)]
pub enum ConfigCmd {
    /// Print the config being used by voyager.
    Print,
    /// Print a default config.
    Default,
    /// Print the JSON Schema for the voyager config, to be used in the top-level `$schema` field.
    Schema,
}

type Pg64 = BoundedI64<1, { i64::MAX }>;

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum QueueCmd {
    /// Enqueue a new op to the queue of an already running voyager instance.
    #[command(alias = "e")]
    Enqueue {
        #[arg(value_parser(|s: &str| serde_json::from_str::<Op<VoyagerMessage>>(s)))]
        op: Op<VoyagerMessage>,
        #[arg(long, global = true)]
        rest_url: Option<String>,
    },

    // History {
    //     id: PgId,
    //     #[arg(long, default_value_t = result_unwrap!(Pg32::new(10)))]
    //     max_depth: Pg32,
    // },
    /// Query all failed messages.
    QueryFailed {
        #[arg(long, default_value_t = result_unwrap!(Pg64::new_const(1)))]
        page: Pg64,
        #[arg(long, default_value_t = result_unwrap!(Pg64::new_const(1)))]
        per_page: Pg64,
        /// SQL filters for the item.
        ///
        /// These will be run on the stringified item (`item::text`), which is the *almost* fully compact JSON:
        ///
        /// ```psql
        /// default=# select '{"a":{"b":"c"}}'::jsonb::text;
        ///        text        
        /// -------------------
        ///  {"a": {"b": "c"}}
        /// ````
        ///
        /// This can be specified multiple times to specify multiple filters.
        #[arg(long = "item-filter", short = 'i')]
        item_filters: Vec<String>,
        /// SQL filters for failure message.
        ///
        /// This can be specified multiple times to specify multiple filters.
        #[arg(long = "message-filter", short = 'm')]
        message_filters: Vec<String>,
    },
    /// Query a failed message by it's ID.
    QueryFailedById {
        id: Pg64,
        #[arg(long, short = 'e')]
        requeue: bool,
        #[arg(long)]
        rest_url: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum PluginCmd {
    /// Run the interest filter for the specified plugin on the provided JSON object.
    Interest {
        plugin_name: String,
        message: String,
    },
    /// Print the plugin info for a plugin.
    Info { plugin_name: String },
    /// Call a plugin directly from the CLI.
    Call {
        plugin_name: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
        args: Vec<String>,
    },
    /// List all available plugins.
    List,
}

#[derive(Debug, Subcommand)]
pub enum ModuleCmd {
    State(StateModuleInfo),
    Proof(ProofModuleInfo),
    Consensus(ConsensusModuleInfo),
    Client(ClientModuleInfo),
}

#[derive(Debug, Subcommand)]
pub enum RpcCmd {
    Info,
    ClientState {
        #[arg(value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
        on: ChainId,
        #[arg(
            long,
            short = 's',
            default_value_t = IbcUnion::ID,
            value_parser(|s: &str| ok(IbcSpecId::new(s.to_owned())))
        )]
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
        #[arg(long, default_value_t = QueryHeight::Latest)]
        height: QueryHeight,
        #[arg(long, short = 'd', default_value_t = false)]
        decode: bool,
    },
    ClientMeta {
        #[arg(value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
        on: ChainId,
        #[arg(
            long,
            short = 's',
            default_value_t = IbcUnion::ID,
            value_parser(|s: &str| ok(IbcSpecId::new(s.to_owned())))
        )]
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
        #[arg(long, default_value_t = QueryHeight::Latest)]
        height: QueryHeight,
    },
    ClientInfo {
        #[arg(value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
        on: ChainId,
        #[arg(
            long,
            short = 's',
            default_value_t = IbcUnion::ID,
            value_parser(|s: &str| ok(IbcSpecId::new(s.to_owned())))
        )]
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
    },
    ConsensusState {
        #[arg(value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
        on: ChainId,
        #[arg(
            long,
            short = 's',
            default_value_t = IbcUnion::ID,
            value_parser(|s: &str| ok(IbcSpecId::new(s.to_owned())))
        )]
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
        trusted_height: Height,
        #[arg(long, default_value_t = QueryHeight::Latest)]
        height: QueryHeight,
        #[arg(long, short = 'd', default_value_t = false)]
        decode: bool,
    },
    LatestHeight {
        #[arg(value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
        on: ChainId,
        #[arg(long, short = 'f', default_value_t = false)]
        finalized: bool,
    },
    LatestTimestamp {
        #[arg(value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
        on: ChainId,
        #[arg(long, short = 'f', default_value_t = false)]
        finalized: bool,
    },
    IbcState {
        #[arg(value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
        on: ChainId,
        #[arg(
            long,
            short = 's',
            default_value_t = IbcUnion::ID,
            value_parser(|s: &str| ok(IbcSpecId::new(s.to_owned())))
        )]
        ibc_spec_id: IbcSpecId,
        #[arg(long, default_value_t = QueryHeight::Latest)]
        height: QueryHeight,
        #[arg(
            // the autoref value parser selector chooses From<String> before FromStr, but Value's From<String> impl always returns Value::String(..), whereas FromStr actually parses the json contained within the string
            value_parser(serde_json::Value::from_str),
        )]
        path: serde_json::Value,
    },
    Plugin {
        name: String,
        method: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
        args: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum MsgCmd {
    CreateClient {
        #[arg(long, value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
        on: ChainId,
        #[arg(long, value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
        tracking: ChainId,
        #[arg(long, value_parser(|s: &str| ok(IbcInterface::new(s.to_owned()))))]
        ibc_interface: IbcInterface,
        #[arg(
            long,
            short = 's',
            default_value_t = IbcUnion::ID,
            value_parser(|s: &str| ok(IbcSpecId::new(s.to_owned())))
        )]
        ibc_spec_id: IbcSpecId,
        #[arg(long, value_parser(|s: &str| ok(ClientType::new(s.to_owned()))))]
        client_type: ClientType,
        #[arg(long, default_value_t = QueryHeight::Finalized)]
        height: QueryHeight,
        #[arg(
            long,
            // the autoref value parser selector chooses From<String> before FromStr, but Value's From<String> impl always returns Value::String(..), whereas FromStr actually parses the json contained within the string
            value_parser(serde_json::Value::from_str),
            default_value_t = serde_json::Value::Null
        )]
        metadata: serde_json::Value,

        /// Additional client state config to pass to `self_client_state()`.
        ///
        /// This is mutually exclusive with `--config`.
        #[arg(
            long,
            // the autoref value parser selector chooses From<String> before FromStr, but Value's From<String> impl always returns Value::String(..), whereas FromStr actually parses the json contained within the string
            value_parser(serde_json::Value::from_str),
            default_value_t = serde_json::Value::Null,
            conflicts_with = "config"
        )]
        client_state_config: serde_json::Value,

        /// Additional consensus state config to pass to `self_consensus_state()`.
        ///
        /// This is mutually exclusive with `--config`.
        #[arg(
            long,
            // the autoref value parser selector chooses From<String> before FromStr, but Value's From<String> impl always returns Value::String(..), whereas FromStr actually parses the json contained within the string
            value_parser(serde_json::Value::from_str),
            default_value_t = serde_json::Value::Null,
            conflicts_with = "config"
        )]
        consensus_state_config: serde_json::Value,

        /// Additional config to pass to both `self_client_state()` `self_consensus_state()`.
        ///
        /// This is mutually exclusive with `--client-state-config` and `--consensus-state-config`.
        #[arg(
            long,
            // the autoref value parser selector chooses From<String> before FromStr, but Value's From<String> impl always returns Value::String(..), whereas FromStr actually parses the json contained within the string
            value_parser(serde_json::Value::from_str),
            default_value_t = serde_json::Value::Null,
        )]
        config: serde_json::Value,

        /// Automatically enqueue the op.
        #[arg(long, short = 'e', default_value_t = false)]
        enqueue: bool,
        #[arg(long, global = true)]
        rest_url: Option<String>,
    },
    UpdateClient {
        #[arg(long, value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
        on: ChainId,
        #[arg(long)]
        client_id: RawClientId,
        #[arg(
            long,
            short = 's',
            default_value_t = IbcUnion::ID,
            value_parser(|s: &str| ok(IbcSpecId::new(s.to_owned())))
        )]
        ibc_spec_id: IbcSpecId,
        /// The height to update the client to. Defaults to the latest height of the chain being tracked.
        #[arg(long)]
        update_to: Option<Height>,

        /// Automatically enqueue the op.
        #[arg(long, short = 'e', default_value_t = false)]
        enqueue: bool,
        #[arg(long, global = true)]
        rest_url: Option<String>,
    },
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "intended as sugar to specify the error type"
)]
fn ok<T>(t: T) -> Result<T, BoxDynError> {
    Ok(t)
}
