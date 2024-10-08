module ibc::commitment{
    use std::string::{Self, String};

    use ibc::height::{Self, Height};

    // Generate the path for client state
    public fun client_state_path(client_id: String): String {
        let mut path = string::utf8(b"clients/");
        path.append(client_id);
        path.append_utf8(b"/clientState");
        path
    }

    // Generate the path for consensus state
    public fun consensus_state_path(client_id: String, revision_number: u64, revision_height: u64): String {
        let mut path = string::utf8(b"clients/");
        path.append(client_id);
        path.append_utf8(b"/consensusStates/");
        path.append(revision_number.to_string());
        path.append_utf8(b"-");
        path.append(revision_height.to_string());
        path
    }

    // Generate the path for connection
    public fun connection_path(connection_id: String): String {
        let mut path = string::utf8(b"connections/");
        path.append(connection_id);
        path
    }

    // Generate the path for channel
    public fun channel_path(port_id: String, channel_id: String): String {
        let mut path = string::utf8(b"channelEnds/ports/");
        path.append(port_id);
        path.append_utf8(b"/channels/");
        path.append(channel_id);
        path
    }
    
    // Generate the path for packet commitment
    public fun packet_path(port_id: String, channel_id: String, sequence: u64): String {
        let mut path = string::utf8(b"commitments/ports/");
        path.append(port_id);
        path.append_utf8(b"/channels/");
        path.append(channel_id);
        path.append_utf8(b"/sequences/");
        path.append(sequence.to_string());
        path
    }

    // Generate the path for packet acknowledgment commitment
    public fun packet_acknowledgement_path(port_id: String, channel_id: String, sequence: u64): String {
        let mut path = string::utf8(b"acks/ports/");
        path.append(port_id);
        path.append_utf8(b"/channels/");
        path.append(channel_id);
        path.append_utf8(b"/sequences/");
        path.append(sequence.to_string());
        path
    }

    // Generate the path for packet receipt commitment
    public fun packet_receipt_path(port_id: String, channel_id: String, sequence: u64): String {
        let mut path = string::utf8(b"receipts/ports/");
        path.append(port_id);
        path.append_utf8(b"/channels/");
        path.append(channel_id);
        path.append_utf8(b"/sequences/");
        path.append(sequence.to_string());
        path
    }

    // Generate the path for next sequence send commitment
    public fun next_sequence_send_path(port_id: String, channel_id: String): String {
        let mut path = string::utf8(b"nextSequenceSend/ports/");
        path.append(port_id);
        path.append_utf8(b"/channels/");
        path.append(channel_id);
        path
    }

    // Generate the path for next sequence receive commitment
    public fun next_sequence_recv_path(port_id: String, channel_id: String): String {
        let mut path = string::utf8(b"nextSequenceRecv/ports/");
        path.append(port_id);
        path.append_utf8(b"/channels/");
        path.append(channel_id);
        path
    }

    // Generate the path for next sequence acknowledge commitment
    public fun next_sequence_ack_path(port_id: String, channel_id: String): String {
        let mut path = string::utf8(b"nextSequenceAck/ports/");
        path.append(port_id);
        path.append_utf8(b"/channels/");
        path.append(channel_id);
        path
    }

    // Key generation functions
    public fun client_state_key(client_id: String): vector<u8> {
        *string::bytes(&client_state_path(client_id))
    }

    public fun consensus_state_key(client_id: String, height: Height): vector<u8> {
        *string::bytes(&consensus_state_path(client_id, height::get_revision_number(&height), height::get_revision_height(&height)))
    }

    public fun connection_key(connection_id: String): vector<u8> {
        *string::bytes(&connection_path(connection_id))
    }

    public fun channel_key(port_id: String, channel_id: String): vector<u8> {
        *string::bytes(&channel_path(port_id, channel_id))
    }

    public fun packet_key(port_id: String, channel_id: String, sequence: u64): vector<u8> {
        *string::bytes(&packet_path(port_id, channel_id, sequence))
    }

    public fun packet_acknowledgement_key(port_id: String, channel_id: String, sequence: u64): vector<u8> {
        *string::bytes(&packet_acknowledgement_path(port_id, channel_id, sequence))
    }

    public fun packet_receipt_key(port_id: String, channel_id: String, sequence: u64): vector<u8> {
        *string::bytes(&packet_receipt_path(port_id, channel_id, sequence))
    }

    public fun next_sequence_send_key(port_id: String, channel_id: String): vector<u8> {
        *string::bytes(&next_sequence_send_path(port_id, channel_id))
    }

    public fun next_sequence_recv_key(port_id: String, channel_id: String): vector<u8> {
        *string::bytes(&next_sequence_recv_path(port_id, channel_id))
    }

    public fun next_sequence_ack_key(port_id: String, channel_id: String): vector<u8> {
        *string::bytes(&next_sequence_ack_path(port_id, channel_id))
    }
}
