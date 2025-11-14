use core::hash::{Hash, HashStateTrait};

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

#[derive(Drop, Serde)]
pub struct Connection {
    pub state: ConnectionState,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    // can be None if the connection is in the init state
    pub counterparty_connection_id: Option<ConnectionId>,
}

#[derive(Drop, Serde)]
pub enum ConnectionState {
    Init,
    TryOpen,
    Open,
}

#[derive(Drop, Serde)]
pub struct Channel {
    pub state: ChannelState,
    pub connection_id: ConnectionId,
    // can be None when the channel is in the init state
    pub counterparty_channel_id: Option<ChannelId>,
    pub counterparty_port_id: ByteArray,
    pub version: ByteArray,
}

#[derive(Drop, Serde)]
pub enum ChannelState {
    Init,
    TryOpen,
    Open,
    Closed,
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
