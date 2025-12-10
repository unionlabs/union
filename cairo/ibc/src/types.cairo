// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's cairo subdirectory
//                       The Licensed Work is (c) 2025 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//

// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.

// Notice

// Business Source License 1.1

// Terms

// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.

// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.

// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.

// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.

// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.

// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.

// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).

// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

use alexandria_bytes::BytesTrait;
use alexandria_bytes::byte_array_ext::{ByteArrayTraitExt, ByteArrayTraitExtImpl};
use alexandria_encoding::sol_abi::encode::SolAbiEncodeU8;
use alexandria_evm::encoder::{AbiEncodeTrait, EVMCalldata};
use alexandria_evm::evm_enum::EVMTypes;
use core::hash::{Hash, HashStateTrait};
use core::keccak::compute_keccak_byte_array;
use core::num::traits::Zero;

pub trait Id<T, +Copy<T>> {
    fn new(id: NonZero<u32>) -> T;

    fn increment(self: T) -> T;

    fn raw(self: @T) -> u32;
}

/// Chain identifier with the max length of 31.
#[derive(Debug, Drop, starknet::Store, PartialEq)]
pub struct ChainId {
    id: ByteArray,
}

impl ChainIdSerde of Serde<ChainId> {
    fn serialize(self: @ChainId, ref output: Array<felt252>) {
        self.id.serialize(ref output);
    }

    fn deserialize(ref serialized: Span<felt252>) -> Option<ChainId> {
        let id: ByteArray = Serde::deserialize(ref serialized)?;

        ChainIdImpl::try_from_bytes(id)
    }
}

#[generate_trait]
pub impl ChainIdImpl of ChainIdTrait {
    fn try_from_bytes(id: ByteArray) -> Option<ChainId> {
        if id.len() > 31 {
            None
        } else {
            Some(ChainId { id })
        }
    }
}


#[derive(Debug, Copy, Drop, Serde, starknet::Store, PartialEq)]
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

#[derive(Debug, Copy, Drop, PartialEq, Serde, starknet::Store)]
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

#[derive(Debug, Copy, Drop, PartialEq, Serde, starknet::Store)]
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

#[derive(Debug, Drop, Serde, starknet::Store, PartialEq)]
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

#[derive(Debug, Drop, PartialEq, Copy, Serde, starknet::Store)]
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

#[derive(Drop, Serde, starknet::Store, Clone)]
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

#[derive(Debug, Drop, PartialEq, Copy, Serde, starknet::Store)]
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

#[derive(Debug, Drop, PartialEq, Clone, Serde)]
pub struct Packet {
    pub source_channel_id: ChannelId,
    pub destination_channel_id: ChannelId,
    pub data: ByteArray,
    // pub timeout_height: MustBeZero,
    pub timeout_timestamp: Timestamp,
}

#[generate_trait]
pub impl PacketImpl of PacketTrait {
    fn encode(self: @Packet) -> ByteArray {
        let mut buf = ByteArrayTraitExtImpl::new_empty();
        buf.append_u256(self.source_channel_id.raw().into());
        buf.append_u256(self.destination_channel_id.raw().into());
        buf.append_u256(0x20 * 4);
        buf.append_u256((*self.timeout_timestamp.raw).into());
        buf.append_u256(self.data.len().into());
        buf.append(self.data);
        buf
    }

    fn hash(self: @Packet) -> u256 {
        compute_keccak_byte_array(@self.encode())
    }
}

#[derive(Debug, Drop, PartialEq, Clone, Serde)]
pub struct Timestamp {
    raw: u64,
}

impl TimestampZero of core::num::traits::Zero<Timestamp> {
    fn zero() -> Timestamp {
        Timestamp { raw: 0 }
    }

    fn is_zero(self: @Timestamp) -> bool {
        self.raw.is_zero()
    }

    fn is_non_zero(self: @Timestamp) -> bool {
        self.raw.is_non_zero()
    }
}
