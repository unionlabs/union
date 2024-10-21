module ibc::commitment {

    use aptos_std::string::{Self, String};
    use std::vector;
    use aptos_std::string_utils;
    use aptos_std::hash;
    use ibc::height::{Self, Height};
    use std::bcs;


    const CLIENT_STATE: u256 = 0x00;
    const CONSENSUS_STATE: u256 = 0x01;
    const CONNECTIONS: u256 = 0x02;
    const CHANNELS: u256 = 0x03;
    const PACKETS: u256 = 0x04;
    const PACKET_ACKS: u256 = 0x05;
    const NEXT_SEQ_SEND: u256 = 0x06;
    const NEXT_SEQ_RECV: u256 = 0x07;
    const NEXT_SEQ_ACK: u256 = 0x08;

    // Function to convert a String to vector<u8>
    public inline fun keccak256(s: String): vector<u8> {
        let vec_val = *string::bytes(&s);
        hash::sha2_256(vec_val)
    }

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
    public fun batch_packets_commitment_path(channel_id: u32, batchHash: vector<u8>): vector<u8> {
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
    public fun batch_receipts_commitment_path(channel_id: u32, batchHash: vector<u8>): vector<u8> {
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
        client_state_path(channel_id)
    }

    public fun consenseus_state_commitment_key(channel_id: u32): vector<u8> {
        consenseus_state_commitment_key(channel_id)
    }

    public fun connection_commitment_key(channel_id: u32): vector<u8> {
        connection_path(channel_id)
    }

    public fun channel_commitment_key(channel_id: u32): vector<u8> {
        channel_path(channel_id)
    }

    public fun packet_commitment_key(channel_id: u32, sequence: u64): vector<u8> {
        packet_commitment_path(channel_id, sequence)
    }

    public fun batch_packets_commitment_key(channel_id: u32, batch_hash: vector<u8>): vector<u8> {
        batch_packets_commitment_path(channel_id, batch_hash)
    }

    
    public fun batch_receipts_commitment_key(channel_id: u32, batch_hash: vector<u8>): vector<u8> {
        batch_receipts_commitment_path(channel_id, batch_hash)
    }

    public fun next_sequence_send_commitment_key(channel_id: u32): vector<u8> {
        next_sequence_send_commitment_path(channel_id)
    }

    public fun next_sequence_recv_commitment_key(channel_id: u32): vector<u8> {
        next_sequence_recv_commitment_path(channel_id)
    }

    public fun next_sequence_ack_commitment_key(channel_id: u32): vector<u8> {
        next_sequence_ack_commitment_path(channel_id)
    }
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
