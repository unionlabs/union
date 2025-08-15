use std::num::NonZeroU64;

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use unionlabs::{bounded::BoundedI64, primitives::Bytes};

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
        data: Bytes,
        #[arg(long, short = 'H')]
        height: Option<BoundedI64<1>>,
        #[arg(long, short = 'p', default_value_t = false)]
        prove: bool,
    },
    /// /block?height=_
    Block {
        #[arg(long, short = 'H')]
        height: Option<BoundedI64<1>>,
    },
    /// /block_by_hash?hash=_
    BlockByHash,
    /// /block_results?height=_
    BlockResults,
    /// /block_search?query=_&page=_&per_page=_&order_by=_
    BlockSearch,
    /// /blockchain?minHeight=_&maxHeight=_
    Blockchain,
    /// /broadcast_evidence?evidence=_
    BroadcastEvidence,
    /// /broadcast_tx_async?tx=_
    BroadcastTxAsync,
    /// /broadcast_tx_commit?tx=_
    BroadcastTxCommit,
    /// /broadcast_tx_sync?tx=_
    BroadcastTxSync,
    /// /check_tx?tx=_
    CheckTx,
    /// /commit?height=_
    Commit { height: Option<NonZeroU64> },
    /// /consensus_params?height=_
    ConsensusParams,
    /// /consensus_state?
    ConsensusState,
    /// /dump_consensus_state?
    DumpConsensusState,
    /// /genesis?
    Genesis,
    /// /genesis_chunked?chunk=_
    GenesisChunked,
    /// /header?height=_
    Header,
    /// /header_by_hash?hash=_
    HeaderByHash,
    /// /health?
    Health,
    /// /net_info?
    NetInfo,
    /// /num_unconfirmed_txs?
    NumUnconfirmedTxs,
    /// /status?
    Status,
    /// /subscribe?query=_
    Subscribe,
    /// /tx?hash=_&prove=_
    Tx,
    /// /tx_search?query=_&prove=_&page=_&per_page=_&order_by=_
    TxSearch,
    /// /unconfirmed_txs?limit=_
    UnconfirmedTxs,
    /// /unsubscribe?query=_
    Unsubscribe,
    /// /unsubscribe_all?
    UnsubscribeAll,
    /// /validators?height=_&page=_&per_page=_
    Validators,
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let client = cometbft_rpc::Client::new(self.rpc_url).await?;

        match self.method {
            Method::AbciInfo => print_json(&client.abci_info().await?),
            Method::AbciQuery {
                path,
                data,
                height,
                prove,
            } => print_json(&client.abci_query(path, data, height, prove).await?),
            Method::Block { height } => print_json(&client.block(height).await?),
            Method::Status => print_json(&client.status().await?),
            Method::Commit { height } => print_json(&client.commit(height).await?),
            _ => bail!("not yet implemented"),
        }

        Ok(())
    }
}
