/*
/// Module: ibc
module ibc::ibc;
*/
module ibc::ibc {
    use std::string::{String, utf8};
    use sui::table::{Self, Table};
    use sui::object::{Self, UID};
    use ibc::packet::{Self, Packet};
    
    use sui::transfer;
    use sui::bcs;
    use sui::event;
    const IBC_APP_SEED: vector<u8> = b"union-ibc-app-v1";
    const COMMITMENT_MAGIC: vector<u8> = x"0100000000000000000000000000000000000000000000000000000000000000";
    const COMMITMENT_NULL: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";

    const CLIENT_TYPE_COMETBLS: vector<u8> = b"cometbls";

    const CHAN_STATE_UNINITIALIZED: u8 = 0;
    const CHAN_STATE_INIT: u8 = 1;
    const CHAN_STATE_TRYOPEN: u8 = 2;
    const CHAN_STATE_OPEN: u8 = 3;
    const CHAN_STATE_CLOSED: u8 = 4;

    const CHAN_ORDERING_NONE: u8 = 0;
    const CHAN_ORDERING_UNORDERED: u8 = 1;
    const CHAN_ORDERING_ORDERED: u8 = 2;

    const CONN_STATE_UNSPECIFIED: u64 = 0;
    const CONN_STATE_INIT: u64 = 1;
    const CONN_STATE_TRYOPEN: u64 = 2;
    const CONN_STATE_OPEN: u64 = 3;

    const VAULT_SEED: vector<u8> = b"IBC_VAULT_SEED";

    const E_NOT_ENOUGH_PERMISSIONS_TO_INITIALIZE: u64 = 1001;
    const E_CLIENT_NOT_FOUND: u64 = 1002;
    const E_VERSION_MUST_BE_UNSET: u64 = 1006;
    const E_UNSUPPORTED_VERSION: u64 = 1007;
    const E_INVALID_CONNECTION_STATE: u64 = 1008;
    const E_CONNECTION_ALREADY_EXISTS: u64 = 1009;
    const E_CONN_NOT_SINGLE_HOP: u64 = 1011;
    const E_CONN_NOT_SINGLE_VERSION: u64 = 1012;
    const E_UNSUPPORTED_FEATURE: u64 = 1013;
    const E_PORT_ID_MUST_BE_LOWERCASE: u64 = 1015;
    const E_INVALID_CHANNEL_STATE: u64 = 1016;
    const E_COUNTERPARTY_CHANNEL_NOT_EMPTY: u64 = 1017;
    const E_INVALID_TIMEOUT_HEIGHT: u64 = 1018;
    const E_LATEST_TIMESTAMP_NOT_FOUND: u64 = 1019;
    const E_UNAUTHORIZED: u64 = 1020;
    const E_INVALID_TIMEOUT_TIMESTAMP: u64 = 1021;
    const E_LATEST_HEIGHT_NOT_FOUND: u64 = 1022;
    const E_SOURCE_AND_COUNTERPARTY_PORT_MISMATCH: u64 = 1023;
    const E_SOURCE_AND_COUNTERPARTY_CHANNEL_MISMATCH: u64 = 1022;
    const E_TIMESTAMP_TIMEOUT: u64 = 1023;
    const E_HEIGHT_TIMEOUT: u64 = 1024;
    const E_PACKET_ALREADY_RECEIVED: u64 = 1025;
    const E_PACKET_SEQUENCE_NEXT_SEQUENCE_MISMATCH: u64 = 1026;
    const E_UNKNOWN_CHANNEL_ORDERING: u64 = 1027;
    const E_CONNECTION_DOES_NOT_EXIST: u64 = 1028;
    const E_ACKNOWLEDGEMENT_IS_EMPTY: u64 = 1028;
    const E_ACKNOWLEDGEMENT_ALREADY_EXISTS: u64 = 1029;
    const E_DESTINATION_AND_COUNTERPARTY_PORT_MISMATCH: u64 = 1030;
    const E_DESTINATION_AND_COUNTERPARTY_CHANNEL_MISMATCH: u64 = 1031;
    const E_PACKET_COMMITMENT_NOT_FOUND: u64 = 1032;
    const E_INVALID_PACKET_COMMITMENT: u64 = 1033;
    const E_TIMESTAMP_TIMEOUT_NOT_REACHED: u64 = 1034;
    const E_TIMEOUT_HEIGHT_NOT_REACHED: u64 = 1035;
    const E_INVALID_UPDATE: u64 = 1036;
    const E_NEXT_SEQUENCE_MUST_BE_GREATER_THAN_TIMEOUT_SEQUENCE: u64 = 1037;
    const E_CLIENT_NOT_ACTIVE: u64 = 1038;
    const E_UNKNOWN_CLIENT_TYPE: u64 = 1039;
    const E_NOT_ENOUGH_PACKETS: u64 = 1040;
    const E_PACKET_NOT_RECEIVED: u64 = 1041;
    const E_ACK_ALREADY_EXIST: u64 = 1042;
    const E_CANNOT_INTENT_ORDERED: u64 = 1043;
    const E_TIMEOUT_MUST_BE_SET: u64 = 1044;
    const E_PACKET_SEQUENCE_ACK_SEQUENCE_MISMATCH: u64 = 1045;

    #[event]
    public struct ClientCreatedEvent has copy, drop, store {
        client_id: u32,
        client_type: String,
        consensus_height: u64
    }

    #[event]
    public struct ClientUpdated has copy, drop, store {
        client_id: u32,
        client_type: String,
        height: u64
    }

    #[event]
    public struct ConnectionOpenInit has copy, drop, store {
        connection_id: u32,
        client_type: String,
        client_id: u32,
        counterparty_client_type: String,
        counterparty_client_id: u32
    }

    #[event]
    public struct ChannelOpenInit has copy, drop, store {
        port_id: String,
        channel_id: u32,
        connection_id: u32,
        version: vector<u8>
    }

    #[event]
    public struct ChannelOpenTry has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_channel_id: u32,
        connection_id: u32,
        version: vector<u8>
    }

    #[event]
    public struct ChannelOpenAck has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_channel_id: u32,
        connection_id: u32
    }

    #[event]
    public struct ChannelOpenConfirm has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_channel_id: u32,
        connection_id: u32
    }

    #[event]
    public struct ConnectionOpenTry has copy, drop, store {
        connection_id: u32,
        client_type: String,
        client_id: u32,
        counterparty_client_type: String,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    }

    #[event]
    public struct ConnectionOpenAck has copy, drop, store {
        connection_id: u32,
        client_type: String,
        client_id: u32,
        counterparty_client_type: String,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    }

    #[event]
    public struct ConnectionOpenConfirm has copy, drop, store {
        connection_id: u32,
        client_type: String,
        client_id: u32,
        counterparty_client_type: String,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    }

    #[event]
    public struct SendPacket has drop, store {
        sequence: u64,
        source_channel: u32,
        destination_channel: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64
    }

    #[event]
    public struct RecvPacket has drop, store {
        packet: Packet
    }

    #[event]
    public struct RecvIntentPacket has drop, store {
        packet: Packet
    }

    #[event]
    public struct TimeoutPacket has drop, store {
        packet: Packet
    }

    #[event]
    public struct WriteAcknowledgement has drop, store {
        packet: Packet,
        acknowledgement: vector<u8>
    }

    #[event]
    public struct AcknowledgePacket has drop, store {
        packet: Packet,
        acknowledgement: vector<u8>
    }

    #[event]
    public struct SubmitMisbehaviour has drop, store {
        client_id: u32,
        client_type: String
    }

    #[test]
    fun test_sui() {
        assert!(3==3, 0);
        assert!(4==4, 0);
    }
}