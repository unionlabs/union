use anyhow::Result;
use clap::{Args, Subcommand};
use ibc_union_spec::{
    ChannelId, ClientId, ConnectionId,
    path::{
        BatchPacketsPath, BatchReceiptsPath, ChannelPath, ClientStatePath, ConnectionPath,
        ConsensusStatePath, MembershipProofPath, NonMembershipProofPath,
    },
};
use unionlabs::{
    ethereum::ibc_commitment_key,
    ibc::core::client::height::Height,
    primitives::{Bytes, H256},
};

#[derive(Debug, Args)]
pub struct Cmd {
    /// Calculate the EVM commitment slot for this store key.
    #[arg(long, short = 'e', visible_alias = "slot", global = true)]
    evm_commitment_slot: bool,
    #[command(subcommand)]
    store_path: StorePath,
}

#[derive(Debug, Subcommand)]
pub enum StorePath {
    #[command(visible_alias = "cli")]
    ClientState { client_id: ClientId },
    #[command(visible_alias = "cons")]
    ConsensusState { client_id: ClientId, height: Height },
    #[command(visible_alias = "con")]
    Connection { connection_id: ConnectionId },
    #[command(visible_alias = "chan")]
    Channel { channel_id: ChannelId },
    #[command(visible_alias = "br")]
    BatchReceipts { batch_hash: H256 },
    #[command(visible_alias = "bp")]
    BatchPackets { batch_hash: H256 },
    #[command(visible_alias = "mp")]
    MembershipProof {
        client_id: ClientId,
        proof_height: u64,
        path: Bytes,
    },
    #[command(visible_alias = "nmp")]
    NonMembershipProof {
        client_id: ClientId,
        proof_height: u64,
        path: Bytes,
    },
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let mut key = match self.store_path {
            StorePath::ClientState { client_id } => ClientStatePath { client_id }.key(),
            StorePath::ConsensusState { client_id, height } => ConsensusStatePath {
                client_id,
                height: height.height(),
            }
            .key(),
            StorePath::Connection { connection_id } => ConnectionPath { connection_id }.key(),
            StorePath::Channel { channel_id } => ChannelPath { channel_id }.key(),
            StorePath::BatchReceipts { batch_hash } => BatchReceiptsPath { batch_hash }.key(),
            StorePath::BatchPackets { batch_hash } => BatchPacketsPath { batch_hash }.key(),
            StorePath::MembershipProof {
                client_id,
                proof_height,
                path,
            } => MembershipProofPath {
                client_id,
                proof_height,
                path,
            }
            .key(),
            StorePath::NonMembershipProof {
                client_id,
                proof_height,
                path,
            } => NonMembershipProofPath {
                client_id,
                proof_height,
                path,
            }
            .key(),
        };

        if self.evm_commitment_slot {
            key = ibc_commitment_key(key).to_be_bytes().into();
        }

        println!("{key}");

        Ok(())
    }
}
