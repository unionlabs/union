use alexandria_bytes::BytesTrait;
use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
use alexandria_encoding::sol_abi::encode::SolAbiEncodeU8;
use alexandria_evm::encoder::{AbiEncodeTrait, EVMCalldata};
use alexandria_evm::evm_enum::EVMTypes;
use core::hash::{Hash, HashStateTrait};
use core::keccak::compute_keccak_byte_array;

pub trait Id<T, +Copy<T>> {
    fn new(id: NonZero<u32>) -> T;

    fn increment(self: T) -> T;

    fn raw(self: @T) -> u32;
}

#[derive(Copy, Drop, Serde, starknet::Store)]
pub struct ClientId {
    raw: NonZero<u32>,
}

pub impl ClientIdImpl of Id<ClientId> {
    fn new(id: NonZero<u32>) -> ClientId {
        ClientId { raw: id }
    }

    fn increment(self: ClientId) -> ClientId {
        let raw: u32 = self.raw.into();
        ClientId { raw: (raw + 1).try_into().expect('raw is >= 1; qed;') }
    }

    fn raw(self: @ClientId) -> u32 {
        (*self.raw).into()
    }
}

impl ClientIdHashImpl<S, +HashStateTrait<S>, +Drop<S>> of Hash<ClientId, S> {
    fn update_state(state: S, value: ClientId) -> S {
        state.update(value.raw().into())
    }
}

#[derive(Copy, Drop, Serde, starknet::Store)]
pub struct ConnectionId {
    raw: NonZero<u32>,
}

pub impl ConnectionIdImpl of Id<ConnectionId> {
    fn new(id: NonZero<u32>) -> ConnectionId {
        ConnectionId { raw: id }
    }

    fn increment(self: ConnectionId) -> ConnectionId {
        let raw: u32 = self.raw.into();
        ConnectionId { raw: (raw + 1).try_into().expect('raw is >= 1; qed;') }
    }

    fn raw(self: @ConnectionId) -> u32 {
        (*self.raw).into()
    }
}

impl ConnectionIdHashImpl<S, +HashStateTrait<S>, +Drop<S>> of Hash<ConnectionId, S> {
    fn update_state(state: S, value: ConnectionId) -> S {
        state.update(value.raw().into())
    }
}

#[derive(Copy, Drop, Serde, starknet::Store)]
pub struct ChannelId {
    raw: NonZero<u32>,
}

pub impl ChannelIdImpl of Id<ChannelId> {
    fn new(id: NonZero<u32>) -> ChannelId {
        ChannelId { raw: id }
    }

    fn increment(self: ChannelId) -> ChannelId {
        let raw: u32 = self.raw.into();
        ChannelId { raw: (raw + 1).try_into().expect('raw is >= 1; qed;') }
    }

    fn raw(self: @ChannelId) -> u32 {
        (*self.raw).into()
    }
}

impl ChannelIdHashImpl<S, +HashStateTrait<S>, +Drop<S>> of Hash<ChannelId, S> {
    fn update_state(state: S, value: ChannelId) -> S {
        state.update(value.raw().into())
    }
}

#[derive(Drop, Serde, starknet::Store)]
pub struct Connection {
    pub state: ConnectionState,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    // can be None if the connection is in the init state
    pub counterparty_connection_id: Option<ConnectionId>,
}

#[generate_trait]
pub impl ConnectionImpl of ConnectionTrait {
    fn encode(self: @Connection) -> ByteArray {
        BytesTrait::new_empty()
            .encode(self.state.as_u8())
            .encode(self.client_id.raw())
            .encode(self.counterparty_client_id.raw())
            .encode(self.counterparty_connection_id.map_or(0, |id| id.raw()))
            .into()
    }

    fn commit(self: @Connection) -> u256 {
        compute_keccak_byte_array(@self.encode())
    }
}

#[derive(Drop, PartialEq, Clone, Copy, Serde, starknet::Store)]
#[allow(starknet::store_no_default_variant)] // uninitialized is not a valid state
pub enum ConnectionState {
    Init,
    TryOpen,
    Open,
}

#[generate_trait]
pub impl ConnectionStateImpl of ConnectionStateTrait {
    fn as_u8(self: @ConnectionState) -> u8 {
        match self {
            ConnectionState::Init => 1,
            ConnectionState::TryOpen => 2,
            ConnectionState::Open => 3,
        }
    }
}

#[derive(Drop, Serde, starknet::Store)]
pub struct Channel {
    pub state: ChannelState,
    pub connection_id: ConnectionId,
    // can be None when the channel is in the init state
    pub counterparty_channel_id: Option<ChannelId>,
    pub counterparty_port_id: ByteArray,
    pub version: ByteArray,
}

#[generate_trait]
pub impl ChannelImpl of ChannelTrait {
    fn encode(self: @Channel) -> ByteArray {
        let mut encoder = EVMCalldata {
            calldata: Default::default(),
            offset: 0,
            dynamic_data: Default::default(),
            dynamic_offset: 0,
        };

        let mut bz: Array<felt252> = array![
            self.state.as_u8().into(), self.connection_id.raw().into(),
            self.counterparty_channel_id.map_or(0, |id| id.raw()).into(),
            self.counterparty_port_id.len().into(),
        ];

        self.counterparty_port_id.serialize(ref bz);

        bz.append(self.version.len().into());

        self.version.serialize(ref bz);

        encoder
            .encode(
                array![
                    EVMTypes::Uint8, EVMTypes::Uint32, EVMTypes::Uint32, EVMTypes::Bytes,
                    EVMTypes::String,
                ]
                    .span(),
                bz.span(),
            )
    }

    fn commit(self: @Channel) -> u256 {
        compute_keccak_byte_array(@self.encode())
    }
}

#[derive(Drop, Serde, starknet::Store)]
#[allow(starknet::store_no_default_variant)] // uninitialized is not a valid state
pub enum ChannelState {
    Init,
    TryOpen,
    Open,
    Closed,
}

#[generate_trait]
pub impl ChannelStateImpl of ChannelStateTrait {
    fn as_u8(self: @ChannelState) -> u8 {
        match self {
            ChannelState::Init => 1,
            ChannelState::TryOpen => 2,
            ChannelState::Open => 3,
            ChannelState::Closed => 4,
        }
    }
}

#[derive(Drop, Serde)]
pub struct Packet {
    pub source_channel_id: ChannelId,
    pub destination_channel_id: ChannelId,
    pub data: ByteArray,
    // pub timeout_height: MustBeZero,
    pub timeout_timestamp: Timestamp,
}

#[derive(Drop, Serde)]
pub struct Timestamp {
    raw: u64,
}
