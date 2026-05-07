use std::num::NonZeroU64;

use anyhow::{Result, bail};
use clap::{Args, Subcommand};
use unionlabs::{
    bounded::BoundedI64,
    primitives::{Bytes, encoding::Base64},
};

use crate::print_json;

#[derive(Debug, Args)]
pub struct Cmd {
    #[arg(global = true, short = 'r', default_value = "http://localhost:26657")]
    pub rpc_url: String,
    #[command(subcommand)]
    pub method: Method,
}

#[derive(Debug, Subcommand)]
pub enum Method {
    /// /abci_info?
    AbciInfo,
    /// /abci_query?path=_&data=_&height=_&prove=_
    AbciQuery {
        path: String,
        data: Bytes<Base64>,
        #[arg(long, short = 'H')]
        height: Option<BoundedI64<1>>,
        #[arg(long, short = 'p', default_value_t = false)]
        prove: bool,
    },
    /// /block?height=_
    Block { height: BoundedI64<1> },
    /// /block_results?height=_
    BlockResults { height: BoundedI64<0> },
    /// /blockchain?minHeight=_&maxHeight=_
    Blockchain,
    /// /broadcast_evidence?evidence=_
    BroadcastTxAsync,
    /// /broadcast_tx_commit?tx=_
    BroadcastTxCommit,
    /// /broadcast_tx_sync?tx=_
    BroadcastTxSync,
    /// /commit?height=_
    Commit { height: NonZeroU64 },
    /// /consensus_params?height=_
    ConsensusParams,
    /// /consensus_state?
    ConsensusState,
    /// /dump_consensus_state?
    DumpConsensusState,
    /// /genesis?
    Genesis,
    /// /health?
    Health,
    /// /net_info?
    NetInfo,
    /// /num_unconfirmed_txs?
    NumUnconfirmedTxs,
    /// /status?heightGte=
    Status { height_gte: Option<NonZeroU64> },
    /// /tx?hash=_&prove=_
    Tx,
    /// /unconfirmed_txs?limit=_
    UnconfirmedTxs,
    /// /validators?height=
    Validators { height: NonZeroU64 },
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let client = gno_rpc::Client::new(self.rpc_url).await?;

        match self.method {
            Method::AbciInfo => print_json(&client.abci_info().await?),
            Method::AbciQuery {
                path,
                data,
                height,
                prove,
            } => print_json(&client.abci_query(path, data, height, prove).await?),
            // Method::Block { height } => print_json(&client.block(height).await?),
            // Method::Status => print_json(&client.status().await?),
            Method::Commit { height } => print_json(&client.commit(height).await?),
            Method::Block { height } => print_json(&client.block(height).await?),
            Method::BlockResults { height } => print_json(&client.block_results(height).await?),
            Method::Status { height_gte } => print_json(&client.status(height_gte).await?),
            Method::Validators { height } => print_json(&client.validators(height).await?),
            _ => bail!("not yet implemented"),
        }

        Ok(())
    }
}
