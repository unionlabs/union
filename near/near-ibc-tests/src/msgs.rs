use borsh::{BorshDeserialize, BorshSerialize};
use ibc_vm_rs::states::connection_handshake;
use near_primitives_core::hash::CryptoHash;
use near_sdk::AccountId;
use unionlabs::{
    ibc::core::{channel, client::height::Height, connection::version::Version},
    id::{ChannelId, ConnectionId, PortId},
    near::types::{self, BlockHeaderInnerLiteView},
};

#[derive(serde::Serialize)]
pub struct RegisterClient {
    pub client_type: String,
    pub account: String,
}

#[derive(Clone, serde::Serialize)]
pub struct UpdateClient {
    pub client_id: String,
    pub client_msg: Vec<u8>,
}

#[derive(serde::Serialize)]
pub struct CreateClient {
    pub client_type: String,
    pub client_state: Vec<u8>,
    pub consensus_state: Vec<u8>,
}

#[derive(serde::Serialize)]
pub struct ConnectionOpenInit {
    pub client_id: String,
    pub counterparty: connection_handshake::Counterparty,
    pub version: Version,
    pub delay_period: u64,
}

#[derive(serde::Serialize)]
pub struct ConnectionOpenTry {
    pub client_id: String,
    pub counterparty: connection_handshake::Counterparty,
    pub counterparty_versions: Vec<Version>,
    pub connection_end_proof: Vec<u8>,
    pub proof_height: Height,
    pub delay_period: u64,
}

#[derive(serde::Serialize)]
pub struct ConnectionOpenAck {
    pub connection_id: String,
    pub version: Version,
    pub counterparty_connection_id: String,
    pub connection_end_proof: Vec<u8>,
    pub proof_height: Height,
}

#[derive(serde::Serialize)]
pub struct ConnectionOpenConfirm {
    pub connection_id: String,
    pub connection_end_proof: Vec<u8>,
    pub proof_height: Height,
}

#[derive(serde::Serialize)]
pub struct ChannelOpenInit {
    pub connection_hops: Vec<ConnectionId>,
    pub port_id: PortId,
    pub counterparty: channel::counterparty::Counterparty,
    pub version: String,
}

#[derive(serde::Serialize)]
pub struct ChannelOpenTry {
    pub connection_hops: Vec<ConnectionId>,
    pub port_id: PortId,
    pub counterparty: channel::counterparty::Counterparty,
    pub counterparty_version: String,
    pub version: String,
    pub proof_init: Vec<u8>,
    pub proof_height: Height,
}

#[derive(serde::Serialize)]
pub struct ChannelOpenAck {
    pub channel_id: ChannelId,
    pub port_id: PortId,
    pub counterparty_channel_id: String,
    pub counterparty_version: String,
    pub proof_try: Vec<u8>,
    pub proof_height: Height,
}

#[derive(serde::Serialize)]
pub struct ChannelOpenConfirm {
    pub channel_id: ChannelId,
    pub port_id: PortId,
    pub proof_ack: Vec<u8>,
    pub proof_height: Height,
}

#[derive(serde::Serialize)]
pub struct GetCommitment {
    pub key: String,
}

#[derive(serde::Serialize)]
pub struct GetAccountId {
    pub client_type: String,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ConsensusState {
    pub state: BlockHeaderInnerLiteView,
    pub chunk_prev_state_root: CryptoHash,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ClientState {
    pub latest_height: u64,
    pub ibc_account_id: AccountId,
    pub initial_block_producers: Option<Vec<types::ValidatorStakeView>>,
}
