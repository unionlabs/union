use std::{ffi::OsString, str::FromStr};

use clap::{self, Parser, Subcommand};
use unionlabs::{self, bounded::BoundedI64, ibc::core::client::height::Height, result_unwrap};
use voyager_message::{
    core::{ChainId, ClientType, IbcInterface, IbcSpecId, QueryHeight},
    module::{ClientModuleInfo, ConsensusModuleInfo, ProofModuleInfo, StateModuleInfo},
    RawClientId, VoyagerMessage,
};
use voyager_vm::{BoxDynError, Op};

// use crate::cli::handshake::HandshakeCmd;

// pub mod handshake;

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
    // Handshake(HandshakeCmd),
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
    #[command(subcommand)]
    Rpc(RpcCmd),
    #[command(subcommand)]
    Msg(MsgCmd),
    // Query {
    //     #[arg(value_parser(|s: &str| Ok::<_, BoxDynError>(ChainId::new(s.to_owned()))))]
    //     on: ChainId,
    //     #[arg(long, short = 'H', default_value_t = QueryHeight::Latest)]
    //     height: QueryHeight,
    //     #[command(subcommand)]
    //     path: ics24::Path,
    // },
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
pub enum QueueCmd {
    /// Enqueue a new op to the queue of an already running voyager instance.
    #[command(alias = "e")]
    Enqueue {
        #[arg(value_parser(|s: &str| serde_json::from_str::<Op<VoyagerMessage>>(s)))]
        op: Op<VoyagerMessage>,
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
        plugin_name: Option<String>,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
        args: Vec<String>,
    },
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
        // #[arg(value_parser(|s: &str| ok(RawClientId::new(s.parse::<Value>().unwrap_or_else(|_| Value::String(s.to_owned()))))))]
        client_id: RawClientId,
        #[arg(value_parser(|s: &str| ok(IbcSpecId::new(s.to_owned()))))]
        ibc_spec_id: IbcSpecId,
        #[arg(long, default_value_t = QueryHeight::Latest)]
        height: QueryHeight,
        #[arg(long, short = 'd', default_value_t = false)]
        decode: bool,
    },
    ConsensusState {
        #[arg(value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
        on: ChainId,
        // #[arg(value_parser(|s: &str| ok(RawClientId::new(s.parse::<Value>().unwrap_or_else(|_| Value::String(s.to_owned()))))))]
        client_id: RawClientId,
        #[arg(value_parser(|s: &str| ok(IbcSpecId::new(s.to_owned()))))]
        ibc_spec_id: IbcSpecId,
        #[arg(long, default_value_t = QueryHeight::Latest)]
        height: QueryHeight,
        trusted_height: Height,
        #[arg(long, short = 'd', default_value_t = false)]
        decode: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum MsgCmd {
    CreateClient {
        #[arg(long, value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
        on: ChainId,
        #[arg(long, value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
        tracking: ChainId,
        #[arg(long, value_parser(|s: &str| ok(IbcInterface::new(s.to_owned()))))]
        ibc_interface: IbcInterface,
        #[arg(long, value_parser(|s: &str| ok(IbcSpecId::new(s.to_owned()))))]
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

        /// Automatically enqueue the op.
        #[arg(long, short = 'e', default_value_t = false)]
        enqueue: bool,
    },
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "intended as sugar to specify the error type"
)]
fn ok<T>(t: T) -> Result<T, BoxDynError> {
    Ok(t)
}
