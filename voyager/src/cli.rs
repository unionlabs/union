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
use voyager_message::core::ChainId;

use crate::cli::handshake::HandshakeCmd;

pub mod handshake;

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct AppArgs {
    #[arg(
        long,
        short = 'c',
        env,
        global = true,
        default_value = "voyager/devnet-config.json"
    )]
    pub config_file_path: OsString,
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
    PrintConfig,
    Handshake(HandshakeCmd),
    /// Construct a `FetchBlocks` message to send to the specified chain. The message will start at the current latest height of the chain.
    InitFetch {
        chain_id: String,
    },
    /// Run Voyager.
    Relay,
    #[command(subcommand)]
    Queue(QueueCmd),
    #[command(subcommand)]
    Util(UtilCmd),
    Module {
        plugin_name: Option<String>,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
        args: Vec<String>,
    },
    Query {
        #[arg(value_parser(|s: &str| Ok::<_, BoxDynError>(ChainId::new(s.to_owned()))))]
        on: ChainId<'static>,
        #[arg(long, default_value_t = QueryHeight::Latest)]
        height: QueryHeight,
        #[command(subcommand)]
        path: ics24::Path,
    },
}

type Pg64 = BoundedI64<1, { i64::MAX }>;

#[derive(Debug, Subcommand)]
pub enum QueueCmd {
    // History {
    //     id: PgId,
    //     #[arg(long, default_value_t = result_unwrap!(Pg32::new(10)))]
    //     max_depth: Pg32,
    // },
    Failed {
        #[arg(long, default_value_t = result_unwrap!(Pg64::new(1)))]
        page: Pg64,
        #[arg(long, default_value_t = result_unwrap!(Pg64::new(1)))]
        per_page: Pg64,
        #[arg(long, short = 'i')]
        item_filters: Vec<String>,
        #[arg(long, short = 'm')]
        message_filters: Vec<String>,
    },
    FailedById {
        id: Pg64,
    },
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
pub enum SignerCmd {
    /// Fetch the balances of all of the configured signers for all enabled chains. If --on is specified, only fetch the signers of that chain, whether the chain is enabled or not.
    Balances {
        #[arg(long)]
        on: Option<String>,
    },
}
