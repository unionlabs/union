// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's aptos subdirectory
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

module ibc::commitment {

    use aptos_std::aptos_hash::keccak256;

    use std::vector;
    use std::bcs;

    use ibc::packet::{Self, Packet};
    use ibc::ethabi;

    const COMMITMENT_MAGIC: u8 = 0x01;

    const CLIENT_STATE: u256 = 0x00;
    const CONSENSUS_STATE: u256 = 0x01;
    const CONNECTIONS: u256 = 0x02;
    const CHANNELS: u256 = 0x03;
    const PACKETS: u256 = 0x04;
    const PACKET_ACKS: u256 = 0x05;
    const NEXT_SEQ_SEND: u256 = 0x06;
    const NEXT_SEQ_RECV: u256 = 0x07;
    const NEXT_SEQ_ACK: u256 = 0x08;

    // Generate the path for client state
    public fun client_state_path(client_id: u32): vector<u8> {
        let state_path = vector::empty();
        let client_State = bcs::to_bytes<u256>(&CLIENT_STATE);
        vector::reverse(&mut client_State);
        vector::append(&mut state_path, client_State);
        let client_id_bytes = bcs::to_bytes<u256>(&(client_id as u256));
        vector::reverse(&mut client_id_bytes);
        vector::append(&mut state_path, client_id_bytes);
        state_path
    }

    // Updated function: consensus_state_path
    public fun consensus_state_path(client_id: u32, revision_height: u64): vector<u8> {
        let state_path = vector::empty<u8>();

        let consensus_state_bytes = bcs::to_bytes<u256>(&CONSENSUS_STATE);
        vector::reverse(&mut consensus_state_bytes);
        vector::append(&mut state_path, consensus_state_bytes);

        let client_id_bytes = bcs::to_bytes<u256>(&(client_id as u256));
        vector::reverse(&mut client_id_bytes);
        vector::append(&mut state_path, client_id_bytes);

        let revision_height_bytes = bcs::to_bytes<u256>(&(revision_height as u256));
        vector::reverse(&mut revision_height_bytes);
        vector::append(&mut state_path, revision_height_bytes);

        state_path
    }

    // Generate the path for connection
    public fun connection_path(connection_id: u32): vector<u8> {
        let connection_path = vector::empty<u8>();

        let connections_bytes = bcs::to_bytes<u256>(&CONNECTIONS);
        vector::reverse(&mut connections_bytes);
        vector::append(&mut connection_path, connections_bytes);

        let connection_id_bytes = bcs::to_bytes<u256>(&(connection_id as u256));
        vector::reverse(&mut connection_id_bytes);
        vector::append(&mut connection_path, connection_id_bytes);
        connection_path
    }

    // Generate the path for channel
    public fun channel_path(channel_id: u32): vector<u8> {
        let channel_path = vector::empty<u8>();

        let channels_bytes = bcs::to_bytes<u256>(&CHANNELS);
        vector::reverse(&mut channels_bytes);
        vector::append(&mut channel_path, channels_bytes);

        let channel_id_bytes = bcs::to_bytes<u256>(&(channel_id as u256));
        vector::reverse(&mut channel_id_bytes);
        vector::append(&mut channel_path, channel_id_bytes);
        channel_path
    }

    // Generate the path for channel
    public fun packet_commitment_path(channel_id: u32, sequence: u64): vector<u8> {
        let path_vec = vector::empty<u8>();

        let channels_bytes = bcs::to_bytes<u256>(&PACKETS);
        vector::reverse(&mut channels_bytes);
        vector::append(&mut path_vec, channels_bytes);

        let param_bytes = bcs::to_bytes<u256>(&(channel_id as u256));
        vector::reverse(&mut param_bytes);
        vector::append(&mut path_vec, param_bytes);

        let param_bytes2 = bcs::to_bytes<u256>(&(sequence as u256));
        vector::reverse(&mut param_bytes2);
        vector::append(&mut path_vec, param_bytes2);
        path_vec
    }

    // Generate the path for channel
    public fun batch_packets_commitment_path(
        channel_id: u32, batchHash: vector<u8>
    ): vector<u8> {
        let path_vec = vector::empty<u8>();

        let channels_bytes = bcs::to_bytes<u256>(&PACKETS);
        vector::reverse(&mut channels_bytes);
        vector::append(&mut path_vec, channels_bytes);

        let param_bytes = bcs::to_bytes<u256>(&(channel_id as u256));
        vector::reverse(&mut param_bytes);
        vector::append(&mut path_vec, param_bytes);

        vector::append(&mut path_vec, batchHash);
        path_vec
    }

    // Generate the path for channel
    public fun batch_receipts_commitment_path(
        channel_id: u32, batchHash: vector<u8>
    ): vector<u8> {
        let path_vec = vector::empty<u8>();

        let channels_bytes = bcs::to_bytes<u256>(&PACKET_ACKS);
        vector::reverse(&mut channels_bytes);
        vector::append(&mut path_vec, channels_bytes);

        let param_bytes = bcs::to_bytes<u256>(&(channel_id as u256));
        vector::reverse(&mut param_bytes);
        vector::append(&mut path_vec, param_bytes);

        vector::append(&mut path_vec, batchHash);
        path_vec
    }

    // Generate the path for channel
    public fun next_sequence_send_commitment_path(channel_id: u32): vector<u8> {
        let path_vec = vector::empty<u8>();

        let channels_bytes = bcs::to_bytes<u256>(&NEXT_SEQ_SEND);
        vector::reverse(&mut channels_bytes);
        vector::append(&mut path_vec, channels_bytes);

        let param_bytes = bcs::to_bytes<u256>(&(channel_id as u256));
        vector::reverse(&mut param_bytes);
        vector::append(&mut path_vec, param_bytes);

        path_vec
    }

    // Generate the path for channel
    public fun next_sequence_recv_commitment_path(channel_id: u32): vector<u8> {
        let path_vec = vector::empty<u8>();

        let channels_bytes = bcs::to_bytes<u256>(&NEXT_SEQ_RECV);
        vector::reverse(&mut channels_bytes);
        vector::append(&mut path_vec, channels_bytes);

        let param_bytes = bcs::to_bytes<u256>(&(channel_id as u256));
        vector::reverse(&mut param_bytes);
        vector::append(&mut path_vec, param_bytes);

        path_vec
    }

    // Generate the path for channel
    public fun next_sequence_ack_commitment_path(channel_id: u32): vector<u8> {
        let path_vec = vector::empty<u8>();

        let channels_bytes = bcs::to_bytes<u256>(&NEXT_SEQ_ACK);
        vector::reverse(&mut channels_bytes);
        vector::append(&mut path_vec, channels_bytes);

        let param_bytes = bcs::to_bytes<u256>(&(channel_id as u256));
        vector::reverse(&mut param_bytes);
        vector::append(&mut path_vec, param_bytes);

        path_vec
    }

    public fun client_state_commitment_key(channel_id: u32): vector<u8> {
        keccak256(client_state_path(channel_id))
    }

    public fun consensus_state_commitment_key(
        channel_id: u32, height: u64
    ): vector<u8> {
        keccak256(consensus_state_path(channel_id, height))
    }

    public fun connection_commitment_key(channel_id: u32): vector<u8> {
        keccak256(connection_path(channel_id))
    }

    public fun channel_commitment_key(channel_id: u32): vector<u8> {
        keccak256(channel_path(channel_id))
    }

    public fun packet_commitment_key(channel_id: u32, sequence: u64): vector<u8> {
        keccak256(packet_commitment_path(channel_id, sequence))
    }

    public fun batch_packets_commitment_key(
        channel_id: u32, batch_hash: vector<u8>
    ): vector<u8> {
        keccak256(batch_packets_commitment_path(channel_id, batch_hash))
    }

    public fun batch_receipts_commitment_key(
        channel_id: u32, batch_hash: vector<u8>
    ): vector<u8> {
        keccak256(batch_receipts_commitment_path(channel_id, batch_hash))
    }

    public fun next_sequence_send_commitment_key(channel_id: u32): vector<u8> {
        keccak256(next_sequence_send_commitment_path(channel_id))
    }

    public fun next_sequence_recv_commitment_key(channel_id: u32): vector<u8> {
        keccak256(next_sequence_recv_commitment_path(channel_id))
    }

    public fun next_sequence_ack_commitment_key(channel_id: u32): vector<u8> {
        keccak256(next_sequence_ack_commitment_path(channel_id))
    }

    public fun commit_packet(packet: &Packet): vector<u8> {
        keccak256(packet::encode(packet))
    }

    public fun commit_packets(packets: &vector<Packet>): vector<u8> {
        let buf = vector::empty();

        ethabi::encode_dyn_array(
            &mut buf,
            packets,
            |buf, item| {
                vector::append(buf, packet::encode(item));
            }
        );

        buf
    }

    public fun commit_acks(acks: vector<vector<u8>>): vector<u8> {
        let buf = vector::empty();
        ethabi::encode_dyn_array(
            &mut buf,
            &acks,
            |buf, item| {
                ethabi::encode_bytes(buf, item);
            }
        );
        buf
    }

    fun merge_ack(ack: vector<u8>): vector<u8> {
        let magic = vector::borrow_mut(&mut ack, 0);
        *magic = COMMITMENT_MAGIC;
        ack
    }

    public fun commit_ack(ack: vector<u8>): vector<u8> {
        merge_ack(keccak256(ack))
    }

    #[test]
    fun test_commit_ack() {
        let buf =
            x"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000";
        let val = commit_ack(buf);
        assert!(
            val == x"01773c7d3e6e60a7ccaa29208f2ef3aa86fe273271dec70f60866a6c8c908762",
            13
        );
    }

    #[test]
    fun test_commit_acks() {
        let buf =
            x"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000";
        let vect = vector::empty();
        vector::push_back(&mut vect, buf);
        let val = commit_acks(vect);
        std::debug::print(&val);

    }

    // #[test]
    // fun test_commit_packets() {
    //     let buf =
    //         x"000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c80000000000000000000000000000000000000000000000000000000000000003010203000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c80000000000000000000000000000000000000000000000000000000000000003010203000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c800000000000000000000000000000000000000000000000000000000000000030102030000000000000000000000000000000000000000000000000000000000";

    //     let packet = packet::new(1, 2, 3, x"010203", 100, 200);
    //     let packets = vector::empty();
    //     vector::push_back(&mut packets, packet);
    //     vector::push_back(&mut packets, packet);
    //     vector::push_back(&mut packets, packet);

    //     assert!(commit_packets(&packets) == buf, 1);
    // }

    // // Generate the path for packet commitment
    // public fun packet_path(
    //     port_id: String, channel_id: String, sequence: u64
    // ): String {
    //     let path = string::utf8(b"commitments/ports/");
    //     string::append(&mut path, port_id);
    //     string::append_utf8(&mut path, b"/channels/");
    //     string::append(&mut path, channel_id);
    //     string::append_utf8(&mut path, b"/sequences/");
    //     string::append(&mut path, string_utils::to_string(&sequence));
    //     path
    // }

    // // Generate the path for packet acknowledgment commitment
    // public fun packet_acknowledgement_path(
    //     port_id: String, channel_id: String, sequence: u64
    // ): String {
    //     let path = string::utf8(b"acks/ports/");
    //     string::append(&mut path, port_id);
    //     string::append_utf8(&mut path, b"/channels/");
    //     string::append(&mut path, channel_id);
    //     string::append_utf8(&mut path, b"/sequences/");
    //     string::append(&mut path, string_utils::to_string(&sequence));
    //     path
    // }

    // // Generate the path for packet receipt commitment
    // public fun packet_receipt_path(
    //     port_id: String, channel_id: String, sequence: u64
    // ): String {
    //     let path = string::utf8(b"receipts/ports/");
    //     string::append(&mut path, port_id);
    //     string::append_utf8(&mut path, b"/channels/");
    //     string::append(&mut path, channel_id);
    //     string::append_utf8(&mut path, b"/sequences/");
    //     string::append(&mut path, string_utils::to_string(&sequence));
    //     path
    // }

    // // Generate the path for next sequence send commitment
    // public fun next_sequence_send_path(
    //     port_id: String, channel_id: String
    // ): String {
    //     let path = string::utf8(b"nextSequenceSend/ports/");
    //     string::append(&mut path, port_id);
    //     string::append_utf8(&mut path, b"/channels/");
    //     string::append(&mut path, channel_id);
    //     path
    // }

    // // Generate the path for next sequence receive commitment
    // public fun next_sequence_recv_path(
    //     port_id: String, channel_id: String
    // ): String {
    //     let path = string::utf8(b"nextSequenceRecv/ports/");
    //     string::append(&mut path, port_id);
    //     string::append_utf8(&mut path, b"/channels/");
    //     string::append(&mut path, channel_id);
    //     path
    // }

    // // Generate the path for next sequence acknowledge commitment
    // public fun next_sequence_ack_path(port_id: String, channel_id: String): String {
    //     let path = string::utf8(b"nextSequenceAck/ports/");
    //     string::append(&mut path, port_id);
    //     string::append_utf8(&mut path, b"/channels/");
    //     string::append(&mut path, channel_id);
    //     path
    // }

    // // Key generation functions
    // public fun client_state_key(client_id: u32): vector<u8> {
    //     client_state_path(client_id)
    // }

    // public fun consensus_state_key(client_id: u32, height: Height): vector<u8> {
    //     consensus_state_path(
    //         client_id,
    //         height::get_revision_height(&height)
    //     )
    // }

    // public fun connection_key(connection_id: u32): vector<u8> {
    //     connection_path(connection_id)
    // }

    // public fun channel_key(port_id: String, channel_id: String): vector<u8> {
    //     *string::bytes(&channel_path(port_id, channel_id))
    // }

    // public fun packet_key(
    //     port_id: String, channel_id: String, sequence: u64
    // ): vector<u8> {
    //     *string::bytes(&packet_path(port_id, channel_id, sequence))
    // }

    // public fun packet_acknowledgement_key(
    //     port_id: String, channel_id: String, sequence: u64
    // ): vector<u8> {
    //     *string::bytes(&packet_acknowledgement_path(port_id, channel_id, sequence))
    // }

    // public fun packet_receipt_key(
    //     port_id: String, channel_id: String, sequence: u64
    // ): vector<u8> {
    //     *string::bytes(&packet_receipt_path(port_id, channel_id, sequence))
    // }

    // public fun next_sequence_send_key(port_id: String, channel_id: String): vector<u8> {
    //     *string::bytes(&next_sequence_send_path(port_id, channel_id))
    // }

    // public fun next_sequence_recv_key(port_id: String, channel_id: String): vector<u8> {
    //     *string::bytes(&next_sequence_recv_path(port_id, channel_id))
    // }

    // public fun next_sequence_ack_key(port_id: String, channel_id: String): vector<u8> {
    //     *string::bytes(&next_sequence_ack_path(port_id, channel_id))
    // }

    // #[test]
    // public fun test_client_state_path(){
    //     let test = client_state_path(54);
    //     std::debug::print(&test);
    //     assert!(test == x"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000036", 1000);
    // }
}
