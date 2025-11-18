use alexandria_bytes::BytesTrait;
use alexandria_encoding::sol_abi::encode::SolAbiEncodeTrait;
use core::keccak::compute_keccak_byte_array;
use crate::types::{ChannelId, ClientId, ClientIdImpl, ConnectionId, Id};

pub const CLIENT_STATE: u256 = 0;
pub const CONSENSUS_STATE: u256 = 1;
pub const CONNECTIONS: u256 = 2;
pub const CHANNELS: u256 = 3;
pub const PACKETS: u256 = 4;
pub const PACKET_ACKS: u256 = 5;
pub const MEMBERSHIP_PROOF: u256 = 6;
pub const NON_MEMBERSHIP_PROOF: u256 = 7;
pub const PACKET_TIMEOUTS: u256 = 8;

pub enum StorePath {
    ClientState: ClientStatePath,
    ConsensusState: ConsensusStatePath,
    Connection: ConnectionPath,
    Channel: ChannelPath,
    BatchReceipts: BatchReceiptsPath,
    BatchPackets: BatchPacketsPath,
}

pub trait StorePathKeyTrait<T> {
    fn key(self: @T) -> u256;
}

impl StorePathKeyImpl of StorePathKeyTrait<StorePath> {
    fn key(self: @StorePath) -> u256 {
        match self {
            StorePath::ClientState(path) => path.key(),
            StorePath::ConsensusState(path) => path.key(),
            StorePath::Connection(path) => path.key(),
            StorePath::Channel(path) => path.key(),
            StorePath::BatchReceipts(path) => path.key(),
            StorePath::BatchPackets(path) => path.key(),
        }
    }
}

#[derive(Drop)]
pub struct ClientStatePath {
    pub client_id: ClientId,
}

impl ClientStatePathKeyImpl of StorePathKeyTrait<ClientStatePath> {
    fn key(self: @ClientStatePath) -> u256 {
        compute_keccak_byte_array(
            @BytesTrait::new_empty()
                .encode(CLIENT_STATE)
                .encode(Into::<u32, u256>::into(self.client_id.raw()))
                .into(),
        )
    }
}

#[derive(Drop)]
pub struct ConsensusStatePath {
    pub client_id: ClientId,
    pub height: u64,
}

impl ConsensusStatePathKeyImpl of StorePathKeyTrait<ConsensusStatePath> {
    fn key(self: @ConsensusStatePath) -> u256 {
        compute_keccak_byte_array(
            @BytesTrait::new_empty()
                .encode(CONSENSUS_STATE)
                .encode(Into::<u32, u256>::into(self.client_id.raw()))
                .encode(Into::<u64, u256>::into(*self.height))
                .into(),
        )
    }
}

#[derive(Drop)]
pub struct ConnectionPath {
    pub connection_id: ConnectionId,
}

impl ConnectionPathKeyImpl of StorePathKeyTrait<ConnectionPath> {
    fn key(self: @ConnectionPath) -> u256 {
        compute_keccak_byte_array(
            @BytesTrait::new_empty()
                .encode(CONNECTIONS)
                .encode(Into::<u32, u256>::into(self.connection_id.raw()))
                .into(),
        )
    }
}

#[derive(Drop)]
pub struct ChannelPath {
    pub channel_id: ChannelId,
}

impl ChannelPathKeyImpl of StorePathKeyTrait<ChannelPath> {
    fn key(self: @ChannelPath) -> u256 {
        compute_keccak_byte_array(
            @BytesTrait::new_empty()
                .encode(CHANNELS)
                .encode(Into::<u32, u256>::into(self.channel_id.raw()))
                .into(),
        )
    }
}

#[derive(Drop)]
pub struct BatchReceiptsPath {
    pub batch_hash: u256,
}

impl BatchReceiptsPathKeyImpl of StorePathKeyTrait<BatchReceiptsPath> {
    fn key(self: @BatchReceiptsPath) -> u256 {
        compute_keccak_byte_array(
            @BytesTrait::new_empty().encode(PACKET_ACKS).encode(*self.batch_hash).into(),
        )
    }
}

#[derive(Drop)]
pub struct BatchPacketsPath {
    pub batch_hash: u256,
}

impl BatchPacketsPathKeyImpl of StorePathKeyTrait<BatchPacketsPath> {
    fn key(self: @BatchPacketsPath) -> u256 {
        compute_keccak_byte_array(
            @BytesTrait::new_empty().encode(PACKETS).encode(*self.batch_hash).into(),
        )
    }
}
