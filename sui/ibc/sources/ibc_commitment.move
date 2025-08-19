// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's sui subdirectory                      
//                       The Licensed Work is (c) 2024 Union.fi, Labs Inc.
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
#[allow(implicit_const_copy)]
module ibc::commitment {
    use sui::hash;
    use sui::address;
    use std::bcs;
    use ibc::packet::{Self, Packet, PacketBcs};
    use ibc::ethabi::{encode_dyn_array, encode_bytes};
    use sui::hash::keccak256;

    const COMMITMENT_MAGIC: u8 = 0x01;

    const CLIENT_STATE: address = @0x00;
    const CONSENSUS_STATE: address = @0x01;
    const CONNECTIONS: address = @0x02;
    const CHANNELS: address = @0x03;
    const PACKETS: address = @0x04;
    const PACKET_ACKS: address = @0x05;
    const NEXT_SEQ_SEND: address = @0x06;
    const NEXT_SEQ_RECV: address = @0x07;
    const NEXT_SEQ_ACK: address = @0x08;

    public struct ClientStateCommitmentBcs has drop {
        prefix: address,
        client_id: address,
    }

    // Generate the path for client state
    public fun client_state_path(client_id: u32): vector<u8> {
        bcs::to_bytes(&ClientStateCommitmentBcs {
            prefix: CLIENT_STATE,
            client_id: address::from_u256(
                client_id as u256
            )
        })
    }

    public struct ConsensusStateCommitmentBcs has drop {
        prefix: address,
        client_id: address,
        height: address
    }

    // Updated function: consensus_state_path
    public fun consensus_state_path(client_id: u32, height: u64): vector<u8> {
        bcs::to_bytes(&ConsensusStateCommitmentBcs {
            prefix: CONSENSUS_STATE,
            client_id: address::from_u256(client_id as u256),
            height: address::from_u256(height as u256)
        })
    }

    public struct ConnectionCommitmentBcs has drop {
        prefix: address,
        connection_id: address
    }

    // Generate the path for connection
    public fun connection_path(connection_id: u32): vector<u8> {
        bcs::to_bytes(&ConnectionCommitmentBcs {
            prefix: CONNECTIONS,
            connection_id: address::from_u256(connection_id as u256)
        })
    }

    public struct ChannelCommitmentBcs has drop {
        prefix: address,
        channel_id: address,
    }

    // Generate the path for channel
    public fun channel_path(channel_id: u32): vector<u8> {
        bcs::to_bytes(&ChannelCommitmentBcs {
            prefix: CHANNELS,
            channel_id: address::from_u256(channel_id as u256)
        })
    }

    public struct BatchPacketsCommitmentBcs has drop {
        prefix: address,
        batch_hash: address,
    }

    // Generate the path for channel
    public fun batch_packets_commitment_path(
        batch_hash: vector<u8>
    ): vector<u8> {
        bcs::to_bytes(&BatchPacketsCommitmentBcs {
            prefix: PACKETS,
            batch_hash: address::from_bytes(batch_hash)
        })
    }

    public struct BatchReceiptsCommitmentBcs has drop {
        prefix: address,
        batch_hash: address,
    }

    // Generate the path for channel
    public fun batch_receipts_commitment_path(
        batch_hash: vector<u8>
    ): vector<u8> {
        bcs::to_bytes(&BatchReceiptsCommitmentBcs {
            prefix: PACKET_ACKS,
            batch_hash: address::from_bytes(batch_hash)
        })
    }

    public fun client_state_commitment_key(channel_id: u32): vector<u8> {
        client_state_path(channel_id)
    }

    public fun consensus_state_commitment_key(
        channel_id: u32, height: u64
    ): vector<u8> {
        consensus_state_path(channel_id, height)
    }

    public fun connection_commitment_key(connection_id: u32): vector<u8> {
        keccak256(&connection_path(connection_id))
    }

    public fun channel_commitment_key(channel_id: u32): vector<u8> {
        keccak256(&channel_path(channel_id))
    }

    public fun batch_packets_commitment_key(
        batch_hash: vector<u8>
    ): vector<u8> {
        keccak256(&batch_packets_commitment_path(batch_hash))
    }

    public fun batch_receipts_commitment_key(
        batch_hash: vector<u8>
    ): vector<u8> {
        keccak256(&batch_receipts_commitment_path(batch_hash))
    }

    public struct PacketCommitmentBcs has drop {
        offset_0x20_1: address,
        index_1: address,
        offset_0x20_2: address,
        packet: PacketBcs
    }

    // not calling `commit_packets` here because this function is optimized for a single packet
    public fun commit_packet(packet: &Packet): vector<u8> {
        keccak256(&bcs::to_bytes(&PacketCommitmentBcs {
            offset_0x20_1: address::from_u256(0x20),
            index_1: address::from_u256(1),
            offset_0x20_2: address::from_u256(0x20),
            packet: packet.to_bcs_repr()
        }))
    }

    public fun commit_packets(packets: &vector<Packet>): vector<u8> {
        // slightly optimizes the gas cost
        if (packets.length() == 1) {
            return commit_packet(&packets[0])
        };

        let mut buf = vector::empty();

        let mut rest_buf = vector::empty();

        let mut i = 0;

        ibc::ethabi::encode_uint(&mut buf, 0x20);
        let len = vector::length(packets);
        ibc::ethabi::encode_uint(&mut buf, len);

        while (i < len) {
            ibc::ethabi::encode_uint(&mut buf, len * 32 + vector::length(&rest_buf));

            let encoded = packets[i].encode();
            let encoded_len = encoded.length();

            let mut j = 0;
            while (j < encoded_len) {
                rest_buf.push_back(encoded[j]);
                j = j + 1;
            };
            i = i + 1;
        };

        vector::append(&mut buf, rest_buf);

        keccak256(&buf)
    }

    public fun commit_acks(acks: vector<vector<u8>>): vector<u8> {
        let mut buf = vector::empty();
        encode_dyn_array!(
            &mut buf,
            &acks,
            |buf, item| {
                encode_bytes(buf, item);
            }
        );
        merge_ack(keccak256(&buf))
    }

    fun merge_ack(mut ack: vector<u8>): vector<u8> {
        let magic = vector::borrow_mut(&mut ack, 0);
        *magic = COMMITMENT_MAGIC;
        ack
    }

    public fun commit_ack(ack: vector<u8>): vector<u8> {
        commit_acks(vector[ack])
    }
}
