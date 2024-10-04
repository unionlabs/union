use std::ffi::OsString;

use chain_utils::BoxDynError;
use clap::{self, Parser, Subcommand};
use unionlabs::{
    self,
    bounded::BoundedI64,
    ics24::{self, Path},
    result_unwrap,
    uint::U256,
    QueryHeight,
};
use voyager_message::{
    core::ChainId,
    module::{ChainModuleInfo, ClientModuleInfo, ConsensusModuleInfo},
};

use crate::cli::handshake::HandshakeCmd;

pub mod handshake;

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct AppArgs {
    #[arg(long, short = 'c', env, global = true)]
    pub config_file_path: Option<OsString>,
    #[arg(long, short = 'l', env, global = true, default_value_t = LogFormat::default())]
    pub log_format: LogFormat,
    #[arg(long, global = true, default_value_t = 2 * 1024 * 1024)]
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
    Handshake(HandshakeCmd),
    /// Construct a `FetchBlocks` message to send to the specified chain.
    ///
    /// The message will start at the current latest height of the chain.
    InitFetch {
        // TODO: Use chain id here directly
        chain_id: String,
    },
    /// Run Voyager.
    Start,
    /// Query and interact with the queue.
    #[command(subcommand, alias = "q")]
    Queue(QueueCmd),
    #[command(subcommand)]
    Util(UtilCmd),
    #[command(subcommand)]
    Plugin(PluginCmd),
    #[command(subcommand)]
    Module(ModuleCmd),
    Query {
        #[arg(value_parser(|s: &str| Ok::<_, BoxDynError>(ChainId::new(s.to_owned()))))]
        on: ChainId<'static>,
        #[arg(long, short = 'H', default_value_t = QueryHeight::Latest)]
        height: QueryHeight,
        #[command(subcommand)]
        path: ics24::Path,
    },
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
    // History {
    //     id: PgId,
    //     #[arg(long, default_value_t = result_unwrap!(Pg32::new(10)))]
    //     max_depth: Pg32,
    // },
    /// Query all failed messages.
    QueryFailed {
        #[arg(long, default_value_t = result_unwrap!(Pg64::new(1)))]
        page: Pg64,
        #[arg(long, default_value_t = result_unwrap!(Pg64::new(1)))]
        per_page: Pg64,
        /// SQL filters for the item.
        ///
        /// These will be run on the stringified item, (item::text), which is the *almost* fully compact JSON:
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
    QueryFailedById { id: Pg64 },
}

#[derive(Debug, Subcommand)]
pub enum UtilCmd {
    /// Compute the EVM IBC commitment key for the given IBC commitment path.
    IbcCommitmentKey {
        #[command(subcommand)]
        path: Path,
        #[arg(long, default_value_t = U256::ZERO)]
        commitment_slot: U256,
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
    Chain(ChainModuleInfo),
    Consensus(ConsensusModuleInfo),
    Client(ClientModuleInfo),
}
