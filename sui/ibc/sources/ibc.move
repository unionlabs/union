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


module ibc::ibc {
    use std::string::{String, utf8};
    use sui::table::{Self, Table};
    use sui::object::{Self, UID};
    use ibc::packet::{Self, Packet};
    use ibc::connection_end::{Self, ConnectionEnd};
    use ibc::channel::{Self, Channel}; 
    use ibc::light_client::{Self, Client};
    use ibc::commitment;
    use sui::hash;
    use sui::clock;
    use sui::transfer;
    #[test_only]
    use sui::test_scenario;

    use std::debug;
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
        client_id: u32,
        counterparty_client_id: u32
    }

    #[event]
    public struct ChannelOpenInit has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_port_id: vector<u8>,
        connection_id: u32,
        version: String
    }

    #[event]
    public struct ChannelOpenTry has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_port_id: vector<u8>,
        counterparty_channel_id: u32,
        connection_id: u32,
        version: String
    }

    #[event]
    public struct ChannelOpenAck has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_port_id: vector<u8>,
        counterparty_channel_id: u32,
        connection_id: u32
    }

    #[event]
    public struct ChannelOpenConfirm has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_port_id: vector<u8>,
        counterparty_channel_id: u32,
        connection_id: u32
    }

    #[event]
    public struct ConnectionOpenTry has copy, drop, store {
        connection_id: u32,
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    }

    #[event]
    public struct ConnectionOpenAck has copy, drop, store {
        connection_id: u32,
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    }

    #[event]
    public struct ConnectionOpenConfirm has copy, drop, store {
        connection_id: u32,
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    }


    #[event]
    public struct SendPacket has drop, copy, store {
        source_channel: u32,
        destination_channel: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64
    }

    #[event]
    public struct RecvPacket has drop, store, copy {
        packet: Packet
    }

    #[event]
    public struct RecvIntentPacket has drop, store, copy {
        packet: Packet
    }

    #[event]
    public struct TimeoutPacket has drop, store, copy {
        packet: Packet
    }

    #[event]
    public struct WriteAcknowledgement has drop, store, copy {
        packet: Packet,
        acknowledgement: vector<u8>
    }

    #[event]
    public struct AcknowledgePacket has drop, store, copy {
        packet: Packet,
        acknowledgement: vector<u8>
    }

    #[event]
    public struct SubmitMisbehaviour has drop, store, copy {
        client_id: u32,
        client_type: String
    }

    // Resource to hold the global state
    public struct IBCStore has key {
        id: UID,
        client_impls: Table<String, address>,
        client_registry: Table<String, address>,
        commitments: Table<vector<u8>, vector<u8>>,
        connections: Table<u32, ConnectionEnd>,
        channels: Table<u32, Channel>,
        clients: Table<u32, Client>
    }

    fun init(ctx: &mut TxContext) {
        // transfer::transfer(CreatorCapability {
        //     id: object::new(ctx),
        // }, tx_context::sender(ctx))
        let id = object::new(ctx);

        transfer::share_object(IBCStore {
            id: id,
            client_impls: table::new(ctx),
            client_registry: table::new(ctx),
            commitments: table::new(ctx),
            connections: table::new(ctx),
            channels: table::new(ctx),
            clients: table::new(ctx),
        });
    }

    /// Create a client with an initial client and consensus state
    public entry fun create_client(
        ibc_store: &mut IBCStore,
        client_type: String, 
        client_state_bytes: vector<u8>, 
        consensus_state_bytes: vector<u8>,
        ctx: &mut TxContext,
    ) {
        assert!(client_type.bytes() == &b"cometbls", E_UNKNOWN_CLIENT_TYPE);

        let client_id = ibc_store.generate_client_identifier();
        
        let (client, client_state_bytes, consensus_state_bytes) = light_client::create_client(
            client_id,
            client_state_bytes,
            consensus_state_bytes,
            ctx,
        );

        assert!(client.status() == 0, E_CLIENT_NOT_ACTIVE);

        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments, commitment::client_state_commitment_key(client_id), client_state_bytes);

        let latest_height = client.latest_height();

        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments, commitment::consensus_state_commitment_key(client_id, latest_height), consensus_state_bytes);
        ibc_store.clients.add(client_id, client);

        event::emit(
            ClientCreatedEvent {
                client_id,
                client_type,
                consensus_height: latest_height,
            },
        )
    }

    public entry fun update_client(
        ibc_store: &mut IBCStore,
        client_id: u32,
        client_message: vector<u8>
    ) {
        // Check if the client exists in the commitments table
        assert!(
            ibc_store.commitments.contains(commitment::client_state_commitment_key(client_id)),
            E_CLIENT_NOT_FOUND
        );

        let client = ibc_store.clients.borrow(client_id);

        // Update the client and consensus states using the client message
        let (client_state, consensus_state, height) =
            client.update_client(client_message);

        // Update the client state commitment
        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments,
            commitment::client_state_commitment_key(client_id),
            client_state
        );

        // Update the consensus state commitment
        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments,
            commitment::consensus_state_commitment_key(client_id, height),
            hash::keccak256(&consensus_state)
        );

        // Emit a ClientUpdated event for each updated height
        event::emit(
            ClientUpdated {
                client_id,
                client_type: utf8(CLIENT_TYPE_COMETBLS),
                height
            }
        );
    }

    public entry fun submit_misbehaviour(
        ibc_store: &mut IBCStore,
        client_id: u32,
        misbehaviour: vector<u8>
    ) {
        // Check if the client exists in the commitments table
        assert!(
            ibc_store.commitments.contains(commitment::client_state_commitment_key(client_id)),
            E_CLIENT_NOT_FOUND
        );

        // Report the misbehavior
        let light_client = ibc_store.clients.borrow(client_id);
        light_client.report_misbehaviour(misbehaviour);

        // Emit a misbehavior event
        event::emit(
            SubmitMisbehaviour {
                client_id: client_id,
                client_type: utf8(CLIENT_TYPE_COMETBLS)
            }
        );
    }    

    public entry fun connection_open_init(
        ibc_store: &mut IBCStore,
        client_id: u32,
        counterparty_client_id: u32
    ) {
        assert!(ibc_store.clients.borrow(client_id).status() == 0, E_CLIENT_NOT_ACTIVE);

        let connection_id = ibc_store.generate_connection_identifier();

        let connection =
            connection_end::new(
                CONN_STATE_INIT,
                client_id,
                counterparty_client_id,
                0
            );

        add_or_update_table<u32, ConnectionEnd>(&mut ibc_store.connections, connection_id, connection);

        ibc_store.commit_connection(connection_id, connection);

        event::emit(
            ConnectionOpenInit {
                connection_id,
                client_id,
                counterparty_client_id,
            }
        )
    }

    public entry fun connection_open_try(
        ibc_store: &mut IBCStore,
        counterparty_client_id: u32,
        counterparty_connection_id: u32,
        client_id: u32,
        proof_init: vector<u8>,
        proof_height: u64
    ) {
        let connection_id = ibc_store.generate_connection_identifier();

        let mut connection = &connection_end::new(
            CONN_STATE_TRYOPEN,
            client_id,
            counterparty_client_id,
            counterparty_connection_id
        );

        // Construct the expected connection state to verify against the proof
        let expected_connection = connection_end::new(
            CONN_STATE_INIT,
            counterparty_client_id,
            client_id,
            0 // counterparty_connection_id
        );

        let client = ibc_store.clients.borrow(client_id);
        // Verify the connection state using the provided proof and expected state
        let res =
            verify_connection_state(
                client,
                proof_height,
                proof_init,
                counterparty_connection_id,
                expected_connection
        );

        assert!(res == 0, res);

        // Emit an event for the connection try event
        event::emit(
            ConnectionOpenTry {
                connection_id,
                client_id,
                counterparty_client_id,
                counterparty_connection_id,
            }
        );

        // Commit the updated connection to storage
        ibc_store.commit_connection(connection_id, *connection);
    }

    public entry fun connection_open_ack(
        ibc_store: &mut IBCStore,
        connection_id: u32,
        counterparty_connection_id: u32,
        proof_try: vector<u8>,
        proof_height: u64
    ) {
        // Borrow the connection from the table
        let mut connection = ibc_store.connections.borrow_mut(connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_INIT,
            E_INVALID_CONNECTION_STATE
        );

        // Create the expected connection state to verify against the proof
        let expected_connection = connection_end::new(
            CONN_STATE_TRYOPEN,
            connection_end::counterparty_client_id(connection),
            connection_end::client_id(connection),
            connection_id
        );

        // Verify the connection state using the provided proof and expected state
        let client = ibc_store.clients.borrow(connection_end::client_id(connection));

        let res = verify_connection_state(
            client,
            proof_height,
            proof_try,
            counterparty_connection_id,
            expected_connection
        );
        assert!(res == 0, res);

        // Update the connection state to TRYOPEN and set the counterparty connection ID
        connection_end::set_state(connection, CONN_STATE_OPEN);
        connection_end::set_counterparty_connection_id(connection, counterparty_connection_id);

        // Emit an event for connection acknowledgment
        event::emit(
            ConnectionOpenAck {
                connection_id,
                client_id: connection_end::client_id(connection),
                counterparty_client_id: connection_end::counterparty_client_id(connection),
                counterparty_connection_id: connection_end::counterparty_connection_id(connection)
            }
        );

        // Commit the updated connection to storage
        ibc_store.commit_connection(connection_id, *connection);
    }

    public entry fun connection_open_confirm(
        ibc_store: &mut IBCStore,
        connection_id: u32,
        proof_ack: vector<u8>,
        proof_height: u64
    ) {
        let connection = ibc_store.connections.borrow_mut(connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_TRYOPEN,
            E_INVALID_CONNECTION_STATE
        );

        // Create the expected connection state in the `OPEN` state to verify against the proof
        let expected_connection = connection_end::new(
            CONN_STATE_OPEN,
            connection_end::counterparty_client_id(connection),
            connection_end::client_id(connection),
            connection_id
        );

        let counterparty_connection_id = connection_end::counterparty_connection_id(connection);

        // Verify the connection state using the provided proof and expected state
        let client = ibc_store.clients.borrow(connection_end::client_id(connection));
        let res = verify_connection_state(
            client,
            proof_height,
            proof_ack,
            counterparty_connection_id,
            expected_connection
        );
        assert!(res == 0, res);

        // Update the connection state to OPEN
        connection_end::set_state(connection, CONN_STATE_OPEN);

        // Emit an event for connection confirmation
        event::emit(
            ConnectionOpenConfirm {
                connection_id,
                client_id: connection_end::client_id(connection),
                counterparty_client_id: connection_end::counterparty_client_id(connection),
                counterparty_connection_id
            }
        );

        // Commit the final state of the connection to storage
        ibc_store.commit_connection(connection_id, *connection);
    }


    // Function to generate a client identifier
    fun generate_client_identifier(ibc_store: &mut IBCStore): u32 {
        let next_sequence = if (ibc_store.commitments.contains(b"nextClientSequence")) {
            let seq_bytes = ibc_store.commitments.borrow(b"nextClientSequence");
            bcs::new(*seq_bytes).peel_u32()
        } else {
            1u32
        };

        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments, b"nextClientSequence", bcs::to_bytes<u64>(&((next_sequence as u64) + 1)));
        next_sequence
    }

    fun generate_packet_sequence(ibc_store: &mut IBCStore, channel_id: u32): u64 {
        let commitment_key = commitment::next_sequence_send_commitment_key(channel_id);
        let data = ibc_store.commitments.borrow(commitment_key);
        let seq = bcs::new(*data).peel_u64();
        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments, commitment_key, bcs::to_bytes<u64>(&((seq as u64) + 1)));
        seq
    }

    fun generate_connection_identifier(ibc_store: &mut IBCStore): u32 {
        let next_sequence = if (ibc_store.commitments.contains(b"nextConnectionSequence")) {
            let seq_bytes = ibc_store.commitments.borrow(b"nextConnectionSequence");
            bcs::new(*seq_bytes).peel_u32()
        } else {
            1u32
        };

        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments, b"nextConnectionSequence", bcs::to_bytes<u64>(&((next_sequence as u64) + 1)));
        
        next_sequence
    }

    fun add_or_update_table<T: drop + store + copy, P: drop + store>(table: &mut Table<T, P>, key: T, mut value: P) {
        if (table.contains(key)) {
            let mut val = table.borrow_mut(key);
            debug::print(&utf8(b"Updating value under add_or_update_table"));
            debug::print(&value);
            *val = value;
        } else {
            table.add(key, value);
        }
    }

    fun generate_channel_identifier(ibc_store: &mut IBCStore): u32 {
        // Check if 'nextChannelSequence' exists in commitments; if not, initialize to 0
        let next_sequence = if (ibc_store.commitments.contains(b"nextChannelSequence")) {
            let seq_bytes = ibc_store.commitments.borrow(b"nextChannelSequence");
            bcs::new(*seq_bytes).peel_u32()
        } else {
            1u32
        };
        
        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments, b"nextChannelSequence", bcs::to_bytes<u64>(&((next_sequence as u64) + 1)));
        
        // Return the current sequence as the channel identifier
        next_sequence
    }


    fun commit_connection(ibc_store: &mut IBCStore, connection_id: u32, connection: ConnectionEnd) {
        let key = commitment::connection_commitment_key(connection_id);

        let encoded = encode_connection(connection);

        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments, key, encoded);

    }

    fun commit_channel(ibc_store: &mut IBCStore, channel_id: u32, channel: Channel) {
        let key = commitment::channel_commitment_key(channel_id);

        let encoded = encode_channel(channel);

        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments, key, encoded);

    }

    fun encode_channel(channel: Channel): vector<u8> {
        channel::encode(&channel)
    }

    fun encode_connection(connection: ConnectionEnd): vector<u8> {
        connection_end::encode(&connection)
    }

    fun verify_connection_state(
        client: &Client,
        height: u64,
        proof: vector<u8>,
        connection_id: u32,
        counterparty_connection: ConnectionEnd
    ): u64 {
        client.verify_membership(
            height,
            proof,
            commitment::connection_commitment_key(connection_id),
            hash::keccak256(&connection_end::encode(&counterparty_connection))
        )
    }

    fun verify_absent_commitment(
        light_client: &Client,
        height: u64,
        proof: vector<u8>,
        path: vector<u8>
    ): u64 {
        light_client.verify_non_membership(height, proof, path)
    }

    public fun verify_commitment(
        light_client: &Client,
        height: u64,
        proof: vector<u8>,
        path: vector<u8>,
        commitment: vector<u8>
    ): u64 {
        light_client.verify_membership(height, proof, path, commitment)
    }

    fun verify_channel_state(
        light_client: &Client,
        height: u64,
        proof: vector<u8>,
        channel_id: u32,
        channel: Channel
    ): u64 {
        light_client.verify_membership(
            height,
            proof,
            commitment::channel_commitment_key(channel_id),
            hash::keccak256(&channel::encode(&channel))
        )
    }


    public fun channel_open_init(
        ibc_store: &mut IBCStore,
        port_id: String,
        counterparty_port_id: vector<u8>,
        connection_id: u32,
        version: String
    ) {
        // Ensure the connection exists and is in the OPEN state
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );

        // Generate a new channel ID
        let channel_id = ibc_store.generate_channel_identifier();

        // Create a new channel and set its properties
        let mut channel = channel::new(
                CHAN_STATE_INIT,
                connection_id,
                0,
                counterparty_port_id,
                version
        );

        // Add initial sequence values for send, receive, and acknowledgment

        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments,
            commitment::next_sequence_send_commitment_key(channel_id),
            bcs::to_bytes(&1)
        );
        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments,
            commitment::next_sequence_recv_commitment_key(channel_id),
            bcs::to_bytes(&1)
        );
        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments,
            commitment::next_sequence_ack_commitment_key(channel_id),
            bcs::to_bytes(&1)
        );




        // Store the new channel in the IBCStore
        add_or_update_table<u32, Channel>(&mut ibc_store.channels,
            channel_id,
            channel
        );

        // Emit an event for the channel initialization
        event::emit(
            ChannelOpenInit {
                port_id: port_id,
                counterparty_port_id: counterparty_port_id,
                channel_id: channel_id,
                connection_id: connection_id,
                version: version
            }
        );

        ibc_store.commit_channel(channel_id, channel);
    }
    public fun channel_open_try(
        ibc_store: &mut IBCStore,
        port_id: String,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String,
        counterparty_version: String,
        proof_init: vector<u8>,
        proof_height: u64
    ) {
        // Ensure the connection exists and is in the OPEN state
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );
        let client_id = connection_end::client_id(connection);

        // Construct the expected channel state to verify against the proof
        let expected_channel = channel::new(
            CHAN_STATE_INIT,
            get_counterparty_connection(ibc_store, connection_id),
            counterparty_channel_id,
            counterparty_port_id,
            counterparty_version
        );

        let light_client = ibc_store.clients.borrow(client_id);
        // Verify the channel state using the provided proof and expected state
        let verification_result = verify_channel_state(
            light_client,
            proof_height,
            proof_init,
            counterparty_channel_id,
            expected_channel
        );
        assert!(verification_result == 0, verification_result);

        // Generate a new channel ID
        let channel_id = ibc_store.generate_channel_identifier();

        // Create a new channel and set its properties
        let mut channel = channel::new(
                CHAN_STATE_TRYOPEN,
                connection_id,
                counterparty_channel_id,
                counterparty_port_id,
                version
        );

        // Add initial sequence values for send, receive, and acknowledgment

        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments,
            commitment::next_sequence_send_commitment_key(channel_id),
            bcs::to_bytes(&1)
        );
        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments,
            commitment::next_sequence_recv_commitment_key(channel_id),
            bcs::to_bytes(&1)
        );
        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments,
            commitment::next_sequence_ack_commitment_key(channel_id),
            bcs::to_bytes(&1)
        );
    
        // Store the new channel in the IBCStore
        add_or_update_table<u32, Channel>(&mut ibc_store.channels,
            channel_id,
            channel
        );

        // Emit an event for the channel open try
        event::emit(
            ChannelOpenTry {
                port_id,
                channel_id,
                counterparty_port_id,
                counterparty_channel_id,
                connection_id,
                version
            }
        );

        // Commit the updated channel to storage
        ibc_store.commit_channel(channel_id, channel);
    }

    public fun channel_open_ack(
        ibc_store: &mut IBCStore,
        port_id: String,
        channel_id: u32,
        counterparty_version: String,
        counterparty_channel_id: u32,
        proof_try: vector<u8>,
        proof_height: u64,
    ) {
        // Ensure the channel exists and is in the TRYOPEN state
        let channel = ibc_store.channels.borrow_mut(channel_id);
        assert!(
            channel::state(channel) == CHAN_STATE_TRYOPEN,
            E_INVALID_CHANNEL_STATE
        );

        // Ensure the associated connection is in the OPEN state
        let connection_id = channel::connection_id(channel);
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );

        let connection = ibc_store.connections.borrow(connection_id);
        

        // Construct the expected channel state to verify against the proof
        let expected_channel =
            channel::new(
                CHAN_STATE_TRYOPEN,
                connection_end::counterparty_connection_id(connection),
                channel_id,
                bcs::to_bytes(&port_id),
                counterparty_version
            );

        // Verify the channel state using the provided proof and expected state
        let client_id = connection_end::client_id(connection);
        let light_client = ibc_store.clients.borrow(client_id);
        let verification_result = verify_channel_state(
            light_client,
            proof_height,
            proof_try,
            counterparty_channel_id,
            expected_channel
        );
        assert!(verification_result == 0, verification_result);

        // Update the channel state to OPEN and set the counterparty channel ID
        channel::set_state(channel, CHAN_STATE_OPEN);
        channel::set_counterparty_channel_id(channel, counterparty_channel_id);
        channel::set_version(channel, counterparty_version);

        // Emit an event for the channel open acknowledgment
        event::emit(
            ChannelOpenAck {
                port_id: port_id,
                channel_id: channel_id,
                counterparty_channel_id: counterparty_channel_id,
                counterparty_port_id: *channel::counterparty_port_id(channel),
                connection_id: connection_id
            }
        );

        // Commit the updated channel to storage
        ibc_store.commit_channel(channel_id, *channel);
    }

    public fun get_counterparty_connection(
        ibc_store: &mut IBCStore,
        connection_id: u32
    ): u32 {
        let connection = ibc_store.connections.borrow(connection_id);
        connection_end::counterparty_connection_id(connection)
    }

    public fun channel_open_confirm(
        ibc_store: &mut IBCStore,
        port_id: String,
        channel_id: u32,
        proof_ack: vector<u8>,
        proof_height: u64
    ) {
        // Ensure the channel exists and is in the TRYOPEN state
        let channel = ibc_store.channels.borrow_mut(channel_id);
        assert!(
            channel::state(channel) == CHAN_STATE_TRYOPEN,
            E_INVALID_CHANNEL_STATE
        );

        // Ensure the associated connection is in the OPEN state
        let connection_id = channel::connection_id(channel);
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );

        let connection = ibc_store.connections.borrow(connection_id);

        // Construct the expected channel state in the OPEN state to verify against the proof
        let expected_channel =
            channel::new(
                CHAN_STATE_OPEN,
                connection_end::counterparty_connection_id(connection),
                channel_id,
                *channel::counterparty_port_id(channel),
                *channel::version(channel)
            );

        // Verify the channel state using the provided proof and expected state
        let client_id = connection_end::client_id(connection);
        let light_client = ibc_store.clients.borrow(client_id);
        let verification_result = verify_channel_state(
            light_client,
            proof_height,
            proof_ack,
            channel::counterparty_channel_id(channel),
            expected_channel
        );
        assert!(verification_result == 0, verification_result);

        // Update the channel state to OPEN
        channel::set_state(channel, CHAN_STATE_OPEN);

        // Emit an event for the channel open confirmation
        event::emit(
            ChannelOpenConfirm {
                port_id,
                channel_id,
                counterparty_channel_id: channel::counterparty_channel_id(channel),
                counterparty_port_id: *channel::counterparty_port_id(channel),
                connection_id: channel::connection_id(channel)
            }
        );

        // Commit the final state of the channel to storage
        ibc_store.commit_channel(channel_id, *channel);
    }

    /// Function to send a packet through an open channel
    public fun send_packet(
        ibc_store: &mut IBCStore,
        source_channel: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        data: vector<u8>
    ): packet::Packet {
        // Check if the channel exists in the store
        let channel = *ibc_store.channels.borrow(source_channel);
        assert!(channel::state(&channel) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        // Validate timeout values
        assert!(
            timeout_height > 0 || timeout_timestamp > 0,
            E_TIMEOUT_MUST_BE_SET
        );        

        // Prepare packet for commitment
        let packet =
            packet::new(
                source_channel,
                channel::counterparty_channel_id(&channel),
                data,
                timeout_height,
                timeout_timestamp
            );

        // Commit packet details
        let commitment_key =
            commitment::batch_packets_commitment_key(
                source_channel, commitment::commit_packet(&packet)
            );

        add_or_update_table(&mut ibc_store.commitments, commitment_key, COMMITMENT_MAGIC);

        // Emit a SendPacket event
        event::emit(
            SendPacket {
                source_channel: source_channel,
                destination_channel: channel::counterparty_channel_id(&channel),
                data: data,
                timeout_height: timeout_height,
                timeout_timestamp: timeout_timestamp
            },
        );
        packet
    }

        /// Function to send a packet through an open channel
    public fun recv_packet(
        ibc_store: &mut IBCStore,
        clock: &clock::Clock,
        packets: vector<Packet>,
        proof: vector<u8>,
        proof_height: u64,
        acknowledgement: vector<u8>
    ) {
        process_receive(
            ibc_store,
            clock,
            packets,
            proof_height,
            proof,
            false,
            acknowledgement
        );
    }

    fun set_packet_receive(ibc_store: &mut IBCStore, commitment_key: vector<u8>): bool {
        if (get_commitment(ibc_store, commitment_key) != vector::empty()) { true }
        else {
            set_commitment(ibc_store, commitment_key, COMMITMENT_MAGIC);
            false
        }
    }


    public fun process_receive(
        ibc_store: &mut IBCStore,
        clock: &clock::Clock,
        packets: vector<Packet>,
        proof_height: u64,
        proof: vector<u8>,
        intent: bool,
        acknowledgement: vector<u8>
    ) {
        let l = vector::length(&packets);
        assert!(l > 0, E_NOT_ENOUGH_PACKETS);

        let first_packet = packets[0];

        let source_channel = packet::source_channel_id(&first_packet);
        let destination_channel = packet::destination_channel_id(&first_packet);

        let channel = ibc_store.channels.borrow(source_channel);
        assert!(channel::state(channel) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        let connection_id = channel::connection_id(channel);

        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );
        let client_id = connection_end::client_id(connection);

        let light_client = ibc_store.clients.borrow(client_id);
        if (!intent) {
            let commitment_key;
            if (l == 1) {
                commitment_key = commitment::batch_receipts_commitment_key(
                    destination_channel,
                    commitment::commit_packet(&first_packet)
                )
            } else {
                commitment_key = commitment::batch_receipts_commitment_key(
                    destination_channel,
                    commitment::commit_packets(&packets)
                )
            };

            let err =
                verify_commitment(
                    light_client,
                    proof_height,
                    proof,
                    commitment_key,
                    COMMITMENT_MAGIC
                );

            if (err != 0) {
                abort err
            };
        };
        let mut i = 0;
        while (i < l) {
            let packet = packets[i];

            // TODO: Fix here, there is not block library.
            // if (packet::timeout_height(&packet) != 0) {
            //     assert!(
            //         block::get_current_block_height() < packet::timeout_height(&packet),
            //         E_HEIGHT_TIMEOUT
            //     );
            // };

            let current_timestamp = clock::timestamp_ms(clock); 

            let current_timestamp = current_timestamp * 1_000_000; // 1e6
            if (packet::timeout_timestamp(&packet) != 0) {
                assert!(
                    current_timestamp < packet::timeout_timestamp(&packet),
                    E_TIMESTAMP_TIMEOUT
                );
            };

            let commitment_key =
                commitment::batch_receipts_commitment_key(
                    destination_channel,
                    commitment::commit_packet(&packet)
                );

            // TODO: verify if its all good
            if(!set_packet_receive(ibc_store, commitment_key)) {
                if (intent) {
                    event::emit(RecvIntentPacket { packet: packet });
                } else {
                    event::emit(RecvPacket { packet: packet });
                };
                if (vector::length(&acknowledgement) > 0) {
                    inner_write_acknowledgement(ibc_store, commitment_key, acknowledgement);
                    event::emit(WriteAcknowledgement { packet, acknowledgement });
                };
            };
            // let mut already_received = false;

            // if (ordering == CHAN_ORDERING_UNORDERED) {
            //     assert!(ibc_store.commitments.contains(commitment_key), E_CLIENT_NOT_FOUND);
            //     already_received = ibc_store.commitments.borrow(commitment_key) != COMMITMENT_NULL;
            //     if (!already_received) {
            //         add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments, commitment_key, COMMITMENT_MAGIC);
            //     };

            // } else if (ordering == CHAN_ORDERING_ORDERED) {
            //     if (intent) {
            //         abort E_CANNOT_INTENT_ORDERED
            //     };
            //     // set_next_sequence_recv(destination_channel, packet::sequence(&packet));
            //     let next_sequence_recv_key = commitment::next_sequence_recv_commitment_key(destination_channel);
            //     let expected_recv_sequence =
            //         bcs::new(*ibc_store.commitments.borrow(next_sequence_recv_key)).peel_u64();
            //     if (expected_recv_sequence != packet::sequence(&packet)) {
            //         abort E_PACKET_SEQUENCE_NEXT_SEQUENCE_MISMATCH
            //     };
            //     add_or_update_table<vector<u8>, vector<u8>>(
            //         &mut ibc_store.commitments,
            //         next_sequence_recv_key,
            //         bcs::to_bytes<u64>(&(expected_recv_sequence + 1))
            //     );

            // };

            // if (!already_received) {
            //     if (intent) {
            //         event::emit(RecvIntentPacket { packet: packet });
            //     } else {
            //         event::emit(RecvPacket { packet: packet });
            //     };
            //     if (vector::length(&acknowledgement) > 0) {
            //         inner_write_acknowledgement(ibc_store, commitment_key, acknowledgement);

            //         event::emit(WriteAcknowledgement { packet, acknowledgement });
            //     };
            // };
            i = i + 1;
        }
    }

    public fun get_commitment(ibc_store: &mut IBCStore, key: vector<u8>): vector<u8> {
        if (ibc_store.commitments.contains(key)) {
            return *ibc_store.commitments.borrow(key)
        } else {
            return vector::empty<u8>()
        }
    }

    public fun set_commitment(ibc_store: &mut IBCStore,key: vector<u8>, value: vector<u8>) {
        ibc_store.commitments.add(key, value);
    }

    fun inner_write_acknowledgement(
        ibc_store: &mut IBCStore,
        commitment_key: vector<u8>, acknowledgement: vector<u8>
    ) {
        if (!ibc_store.commitments.contains(commitment_key)) {
            abort E_PACKET_NOT_RECEIVED
        };
        let commitment = ibc_store.commitments.borrow(commitment_key);
        assert!(
            *commitment == COMMITMENT_MAGIC,
            E_ACK_ALREADY_EXIST
        );

        add_or_update_table<vector<u8>, vector<u8>>(
            &mut ibc_store.commitments,
            commitment_key,
            commitment::commit_ack(acknowledgement)
        );
    }

    public fun write_acknowledgement(
        ibc_store: &mut IBCStore,
        packet: packet::Packet, acknowledgement: vector<u8>
    ) {
        assert!(!vector::is_empty(&acknowledgement), E_ACKNOWLEDGEMENT_IS_EMPTY);

        let channel = *ibc_store.channels.borrow(packet::destination_channel_id(&packet));
        assert!(channel::state(&channel) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        let commitment_key =
            commitment::batch_receipts_commitment_key(
                packet::destination_channel_id(&packet),
                commitment::commit_packet(&packet)
            );

        inner_write_acknowledgement(ibc_store, commitment_key, acknowledgement);

        event::emit(WriteAcknowledgement { packet, acknowledgement });
    }


    public fun timeout_packet(
        ibc_store: &mut IBCStore,
        packet: Packet,
        proof: vector<u8>,
        proof_height: u64,
        next_sequence_recv: u64
    ) {
        let source_channel = packet::source_channel_id(&packet);
        let destination_channel = packet::destination_channel_id(&packet);

        let channel = ibc_store.channels.borrow(source_channel);
        assert!(channel::state(channel) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        let connection_id = channel::connection_id(channel);

        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );
        let client_id = connection_end::client_id(connection);

        let light_client = ibc_store.clients.borrow(client_id);
        let proof_timestamp =
            light_client.get_timestamp_at_height(proof_height);
        assert!(proof_timestamp != 0, E_LATEST_TIMESTAMP_NOT_FOUND);


        let commitment_key =
                commitment::batch_receipts_commitment_key(
                    destination_channel, commitment::commit_packet(&packet)
                );
        let err =
                verify_absent_commitment(light_client, proof_height, proof, commitment_key);
        assert!(err == 0, err);

        if (packet::timeout_timestamp(&packet) != 0) {
            assert!(
                packet::timeout_timestamp(&packet) < proof_timestamp,
                E_TIMESTAMP_TIMEOUT_NOT_REACHED
            );
        };
        let height = packet::timeout_height(&packet);
        if (height != 0) {
            assert!(
                height < proof_height,
                E_TIMEOUT_HEIGHT_NOT_REACHED
            );
        };

        let commitment_key =
            commitment::batch_packets_commitment_key(
                source_channel, commitment::commit_packet(&packet)
            );
        
        ibc_store.commitments.remove(commitment_key);

        event::emit(TimeoutPacket { packet });
    }

    public fun acknowledge_packet(
        ibc_store: &mut IBCStore,
        packets: vector<packet::Packet>,
        acknowledgements: vector<vector<u8>>,
        proof: vector<u8>,
        proof_height: u64
    )  {
        let l = vector::length(&packets);
        assert!(l > 0, E_NOT_ENOUGH_PACKETS);

        let first_packet = packets[0];

        let source_channel = packet::source_channel_id(&first_packet);
        let destination_channel = packet::destination_channel_id(&first_packet);

        let channel = ibc_store.channels.borrow(source_channel);
        assert!(channel::state(channel) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        let connection_id = channel::connection_id(channel);

        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );
        let client_id = connection_end::client_id(connection);


        let commitment_key;
        if (l == 1) {
            commitment_key = commitment::batch_receipts_commitment_key(
                destination_channel,
                commitment::commit_packet(&first_packet)
            )
        } else {
            commitment_key = commitment::batch_receipts_commitment_key(
                destination_channel,
                commitment::commit_packets(&packets)
            )
        };
        let light_client = ibc_store.clients.borrow(client_id);

        let err =
            verify_commitment(
                light_client,
                proof_height,
                proof,
                commitment_key,
                commitment::commit_acks(acknowledgements)
            );

        if (err != 0) {
            abort err
        };

        let mut i = 0;
        while (i < l) {
            let packet = packets[i];
            let commitment_key =
                commitment::batch_packets_commitment_key(
                    source_channel, commitment::commit_packet(&packet)
                );

            ibc_store.commitments.remove(commitment_key);
            
            let acknowledgement = *vector::borrow(&acknowledgements, i);

            // TODO: verify if its all good
            // onAcknowledgementPacket(...)
            // if (ordering == CHAN_ORDERING_ORDERED) {
            //     let commitment_key = commitment::next_sequence_ack_commitment_key(source_channel);
            //     let expected_ack_sequence = bcs::new(*ibc_store.commitments.borrow(commitment_key)).peel_u64();

            //     if (expected_ack_sequence != packet::sequence(&packet)) {
            //         abort E_PACKET_SEQUENCE_ACK_SEQUENCE_MISMATCH
            //     };
                
            //     add_or_update_table<vector<u8>, vector<u8>>(
            //         &mut ibc_store.commitments,
            //         commitment_key,
            //         bcs::to_bytes<u64>(&(expected_ack_sequence + 1))
            //     );
            // };
            event::emit(AcknowledgePacket { packet, acknowledgement });

            i = i + 1;
        }
    }

    #[test]
    fun test_generate_channel_identifier() {
        let mut ctx = tx_context::dummy();

        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();
        
        let channel_id1 = ibc_store.generate_channel_identifier();
        let channel_id2 = ibc_store.generate_channel_identifier();

        assert!(channel_id1 == 1, 0);
        assert!(channel_id2 == 2, 0);
    
        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_generate_client_identifier() {
        let mut ctx = tx_context::dummy();

        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();
        
        let client_id1 = ibc_store.generate_client_identifier();
        let client_id2 = ibc_store.generate_client_identifier();

        assert!(client_id1 == 1, 0);
        assert!(client_id2 == 2, 0);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_generate_connection_identifier() {
        let mut ctx = tx_context::dummy();

        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();
        
        let connection_id1 = ibc_store.generate_connection_identifier();
        let connection_id2 = ibc_store.generate_connection_identifier();

        assert!(connection_id1 == 1, 0);
        assert!(connection_id2 == 2, 0);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_commit_connection() {
        let mut ctx = tx_context::dummy();

        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();

        let connection_id = ibc_store.generate_connection_identifier();
        let connection = connection_end::new(
            CONN_STATE_INIT,
            1, // client_id
            2, // counterparty_client_id
            0, // counterparty_connection_id
        );

        // First commit
        ibc_store.commit_connection(connection_id, connection);

        // Verify the commitment exists
        let key = commitment::connection_commitment_key(connection_id);
        assert!(ibc_store.commitments.contains(key), E_CONNECTION_DOES_NOT_EXIST);

        // Update connection state
        let updated_connection = connection_end::new(
            CONN_STATE_OPEN,
            1,
            2,
            0,
        );
        ibc_store.commit_connection(connection_id, updated_connection);

        // Verify that the commitment is updated
        let encoded_connection = encode_connection(updated_connection);
        assert!(ibc_store.commitments.borrow(key) == encoded_connection, 0);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_connection_open_init() {
        let mut ctx = tx_context::dummy();
        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();

        create_client(&mut ibc_store, utf8(b"cometbls"), b"client_state", b"proof", &mut ctx);

        // Set up necessary inputs
        let client_id = 1;
        let counterparty_client_id = 2;

        // Call connection_open_init
        connection_open_init(&mut ibc_store, client_id, counterparty_client_id);

        // Verify connection state
        let connection_id = 1; // First generated connection ID should be 1
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(connection_end::state(connection) == CONN_STATE_INIT, E_INVALID_CONNECTION_STATE);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_connection_open_try() {
        let mut ctx = tx_context::dummy();
        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        
        let mut ibc_store = test_case.take_shared<IBCStore>();

        create_client(&mut ibc_store, utf8(b"cometbls"), b"client_state", b"proof", &mut ctx);

        // Initialize connection first
        let client_id = 1;
        let counterparty_client_id = 2;
        connection_open_init(&mut ibc_store, client_id, counterparty_client_id);

        // Prepare inputs for connection_open_try
        let connection_id = 1;
        let proof_init = b"proof";
        let proof_height = 1;

        // Call connection_open_try
        connection_open_try(&mut ibc_store, client_id, counterparty_client_id, connection_id, proof_init, proof_height);

        // Verify state transition to TRYOPEN
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(connection_end::state(connection) == CONN_STATE_INIT, E_INVALID_CONNECTION_STATE);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_connection_open_ack() {
        let mut ctx = tx_context::dummy();
        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();

        create_client(&mut ibc_store, utf8(b"cometbls"), b"client_state", b"proof", &mut ctx);
        // Initialize and try-open connection first
        let client_id = 1;
        let proof_height = 1;
        let counterparty_client_id = 2;
        connection_open_init(&mut ibc_store, client_id, counterparty_client_id);
        connection_open_try(&mut ibc_store, client_id, counterparty_client_id, client_id, b"proof", proof_height);

        // Prepare inputs for connection_open_ack
        let connection_id = 1;
        let counterparty_connection_id = 1;
        let proof_try = b"proof";

        // Call connection_open_ack
        connection_open_ack(&mut ibc_store, connection_id, counterparty_connection_id, proof_try, proof_height);

        // Verify state transition to OPEN
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(connection_end::state(connection) == CONN_STATE_TRYOPEN, E_INVALID_CONNECTION_STATE);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_connection_open_confirm() {
        let mut ctx = tx_context::dummy();
        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();

        create_client(&mut ibc_store, utf8(b"cometbls"), b"client_state", b"proof", &mut ctx);
        // Initialize, try-open, and ack the connection first
        let client_id = 1;
        let proof_height = 1;
        let counterparty_client_id = 2;
        connection_open_init(&mut ibc_store, client_id, counterparty_client_id);
        connection_open_try(&mut ibc_store, client_id, counterparty_client_id, client_id, b"proof", proof_height);
        connection_open_ack(&mut ibc_store, client_id, counterparty_client_id, b"proof", proof_height);

        // Prepare inputs for connection_open_confirm
        let connection_id = 1;
        let proof_ack = b"proof";

        // Call connection_open_confirm
        connection_open_confirm(&mut ibc_store, connection_id, proof_ack, proof_height);

        // Verify final state is OPEN
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(connection_end::state(connection) == CONN_STATE_OPEN, E_INVALID_CONNECTION_STATE);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test_only]
    fun mock_valid_connection(ibc_store: &mut IBCStore, ctx: &mut TxContext): u32 {
        // Set up initial details for the connection
        let client_id = 1;
        let counterparty_client_id = 2;
        
        create_client(ibc_store, utf8(b"cometbls"), b"client_state", b"proof", ctx);
        // Initialize the connection
        connection_open_init(ibc_store, client_id, counterparty_client_id);
        
        // Move to the TRYOPEN state
        let connection_id = 1;
        connection_open_try(ibc_store, client_id, counterparty_client_id, connection_id, b"proof", 1);
        
        // Move to the ACK state
        let counterparty_connection_id = 1;
        connection_open_ack(ibc_store, connection_id, counterparty_connection_id, b"proof", 1);
        
        // Move to the final OPEN state
        connection_open_confirm(ibc_store, connection_id, b"proof", 1);

        connection_id // Return the connection ID for reuse
    }


    #[test]
    fun test_channel_open_init() {
        let mut ctx = tx_context::dummy();
        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();

        // Use the mock function to create a valid connection
        let connection_id = mock_valid_connection(&mut ibc_store, &mut ctx);

        // Now proceed with the channel setup using the valid connection
        let port_id = utf8(b"test_port");
        let counterparty_port_id = b"counterparty_test_port";
        let version = utf8(b"test_version");

        // Call channel_open_init
        channel_open_init(&mut ibc_store, port_id, counterparty_port_id, connection_id, version);

        // Verify the channel state
        let channel_id = 1;
        let channel = ibc_store.channels.borrow(channel_id);
        assert!(channel::state(channel) == CHAN_STATE_INIT, E_INVALID_CHANNEL_STATE);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_channel_open_try() {
        let mut ctx = tx_context::dummy();
        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();

        // Use the mock function to create a valid connection
        let connection_id = mock_valid_connection(&mut ibc_store, &mut ctx);

        // Set up necessary inputs for channel_open_try
        let port_id = utf8(b"test_port");
        let channel_state = CHAN_STATE_TRYOPEN; // Assuming we want the state to be TRYOPEN
        let counterparty_port_id = b"counterparty_test_port";
        let version = utf8(b"test_version");
        let counterparty_version = utf8(b"counterparty_version");
        let counterparty_channel_id = 1;
        let proof_init = b"proof";
        let proof_height = 1;

        // Call channel_open_try
        channel_open_try(
            &mut ibc_store,
            port_id,
            connection_id,
            counterparty_channel_id,
            counterparty_port_id,
            version,
            counterparty_version,
            proof_init,
            proof_height
        );

        // Retrieve the generated channel ID for verification (assuming it starts from 0)
        let channel_id = 1;
        let channel = ibc_store.channels.borrow(channel_id);

        // Verify that the channel state is set to TRYOPEN
        std::debug::print(&utf8(b"channel is:"));
        std::debug::print(&channel::state(channel));
        assert!(channel::state(channel) == CHAN_STATE_TRYOPEN, E_INVALID_CHANNEL_STATE);

        assert!(channel::connection_id(channel) == connection_id, E_CONNECTION_DOES_NOT_EXIST);

        // Verify the version is set as expected
        assert!(channel::version(channel) == version, E_UNSUPPORTED_VERSION);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }


    #[test]
    fun test_channel_open_ack() {
        let mut ctx = tx_context::dummy();
        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();

        // Use the mock function to create a valid connection
        let connection_id = mock_valid_connection(&mut ibc_store, &mut ctx);

        // Set up necessary inputs for channel_open_try
        let port_id = utf8(b"test_port");
        let channel_state = CHAN_STATE_TRYOPEN; // Assuming we want the state to be TRYOPEN
        let counterparty_port_id = b"counterparty_test_port";
        let version = utf8(b"test_version");
        let counterparty_version = utf8(b"counterparty_version");
        let counterparty_channel_id = 1;
        let proof_init = b"proof";
        let proof_height = 1;

        // Call channel_open_try
        channel_open_try(
            &mut ibc_store,
            port_id,
            connection_id,
            counterparty_channel_id,
            counterparty_port_id,
            version,
            counterparty_version,
            proof_init,
            proof_height
        );

        // Prepare inputs for channel_open_ack
        let channel_id = 1; // Assuming the generated ID is 0 for the first channel
        let proof_try = b"proof";
        let proof_height_ack = 1;

        // Call channel_open_ack
        channel_open_ack(
            &mut ibc_store,
            port_id,
            channel_id,
            counterparty_version,
            counterparty_channel_id,
            proof_try,
            proof_height_ack
        );

        // Verify state transition to OPEN
        let channel = ibc_store.channels.borrow(channel_id);
        assert!(channel::state(channel) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }


    #[test]
    fun test_channel_open_confirm() {
        let mut ctx = tx_context::dummy();
        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();

        // Use the mock function to create a valid connection
        let connection_id = mock_valid_connection(&mut ibc_store, &mut ctx);

        // Set up necessary inputs for channel_open_try
        let port_id = utf8(b"test_port");
        let channel_state = CHAN_STATE_TRYOPEN; // Assuming we want the state to be TRYOPEN
        let counterparty_port_id = b"counterparty_test_port";
        let version = utf8(b"test_version");
        let counterparty_version = utf8(b"counterparty_version");
        let counterparty_channel_id = 1;
        let proof_init = b"proof";
        let proof_height = 1;

        // Call channel_open_try
        channel_open_try(
            &mut ibc_store,
            port_id,
            connection_id,
            counterparty_channel_id,
            counterparty_port_id,
            version,
            counterparty_version,
            proof_init,
            proof_height
        );

        // Call channel_open_ack to move to the open state
        let channel_id = 1;
        // let proof_try = b"proof";
        // let proof_height_ack = 1;
        // channel_open_ack(
        //     &mut ibc_store,
        //     port_id,
        //     channel_id,
        //     version,
        //     counterparty_channel_id,
        //     proof_try,
        //     proof_height_ack
        // );

        // Prepare inputs for channel_open_confirm
        let proof_ack = b"proof";
        let proof_height_confirm = 1;

        // Call channel_open_confirm
        channel_open_confirm(
            &mut ibc_store,
            port_id,
            channel_id,
            proof_ack,
            proof_height_confirm
        );

        // Verify final state is OPEN
        let channel = ibc_store.channels.borrow(channel_id);
        assert!(channel::state(channel) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }


    #[test]
    fun test_send_packet() {
        let mut ctx = tx_context::dummy();

        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();

        // Set up a channel in the OPEN state
        let channel_id = ibc_store.generate_channel_identifier();
        let mut channel = channel::default();
        channel::set_state(&mut channel, CHAN_STATE_OPEN);
        channel::set_counterparty_channel_id(&mut channel, 1); // Assume the counterparty channel ID is 1
        ibc_store.channels.add(channel_id, channel);

        // Set up the next sequence number for the send packet
        let next_sequence_key = commitment::next_sequence_send_commitment_key(channel_id);
        add_or_update_table<vector<u8>, vector<u8>>(
            &mut ibc_store.commitments,
            next_sequence_key,
            bcs::to_bytes(&1u64)
        );

        // Define packet data
        let timeout_height = 100;
        let timeout_timestamp = 1_000_000_000;
        let data = b"Hello, IBC!";

        // Call send_packet
        send_packet(
            &mut ibc_store,
            // @0x0, // assuming @0x0 as source port
            channel_id,
            timeout_height,
            timeout_timestamp,
            data
        );
        // Verify packet commitment
        let commitment_key = commitment::batch_packets_commitment_key(
            channel_id,
            commitment::commit_packet(
                &packet::new(
                    1,
                    channel_id,
                    data,
                    timeout_height,
                    timeout_timestamp,
                )
            )
        );

        let commitment = ibc_store.commitments.borrow(commitment_key);
        assert!(commitment == COMMITMENT_MAGIC, 1);


        // Clean up
        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_process_receive() {
        let mut ctx = tx_context::dummy();
        let mut test_case = test_scenario::begin(@0x0);
        init(test_case.ctx());
        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();

        // Set up a valid connection and channel in the OPEN state
        let connection_id = mock_valid_connection(&mut ibc_store, &mut ctx);

        let channel_id = ibc_store.generate_channel_identifier();
        let mut channel = channel::default();
        channel::set_state(&mut channel, CHAN_STATE_OPEN);
        channel::set_connection_id(&mut channel, connection_id);
        channel::set_counterparty_channel_id(&mut channel, 1); // Set counterparty channel ID
        ibc_store.channels.add(channel_id, channel);

        // Prepare packet data
        let sequence = 1;
        let packet = packet::new(
            sequence,
            channel_id,
            b"Test data",
            100, // Timeout height
            1_000_000_000 // Timeout timestamp
        );
        let packets = vector::singleton(packet);

        // Call process_receive
        let proof = b"valid_proof"; // Mock proof
        let proof_height = 100;
        let acknowledgement = b"";

        let commitment_key =
                commitment::batch_receipts_commitment_key(
                    packet::destination_channel_id(&packet),
                    commitment::commit_packet(&packet)
                );
                
        let mut clock = clock::create_for_testing(&mut ctx);
        clock.set_for_testing(99);

        process_receive(
            &mut ibc_store,
            &clock,
            packets,
            proof_height,
            proof,
            false, // No intent
            acknowledgement
        );

        assert!(ibc_store.commitments.contains(commitment_key), E_PACKET_NOT_RECEIVED);
        let commitment = ibc_store.commitments.borrow(commitment_key);
        assert!(commitment == COMMITMENT_MAGIC, 0);


        clock::destroy_for_testing(clock);
        test_scenario::return_shared(ibc_store);
        test_case.end();
    }
}
