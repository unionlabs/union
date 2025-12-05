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
