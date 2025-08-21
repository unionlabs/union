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
    use sui::hash::keccak256;
    use sui::clock;
    use sui::transfer;
    use std::string;
    use sui::clock::Clock;
    #[test_only]
    use sui::test_scenario;

    use ibc::packet::{Self, Packet};
    use ibc::connection_end::{Self, ConnectionEnd};
    use ibc::channel::{Self, Channel}; 
    use ibc::light_client::{Self, LightClientManager};
    use ibc::commitment;
    use ibc::create_lens_client_event;

    use std::debug;
    use sui::bcs;
    use sui::event;

    const IBC_APP_SEED: vector<u8> = b"union-ibc-app-v1";
    const COMMITMENT_MAGIC: vector<u8>     = x"0100000000000000000000000000000000000000000000000000000000000000";
    const COMMITMENT_MAGIC_ACK: vector<u8> = x"0200000000000000000000000000000000000000000000000000000000000000";

    const CLIENT_TYPE_COMETBLS: vector<u8> = b"cometbls";

    const CHAN_STATE_UNINITIALIZED: u8 = 0;
    const CHAN_STATE_INIT: u8 = 1;
    const CHAN_STATE_TRYOPEN: u8 = 2;
    const CHAN_STATE_OPEN: u8 = 3;
    const CHAN_STATE_CLOSED: u8 = 4;

    const CHAN_ORDERING_NONE: u8 = 0;
    const CHAN_ORDERING_UNORDERED: u8 = 1;
    const CHAN_ORDERING_ORDERED: u8 = 2;

    const CONN_STATE_UNSPECIFIED: u8 = 0;
    const CONN_STATE_INIT: u8 = 1;
    const CONN_STATE_TRYOPEN: u8 = 2;
    const CONN_STATE_OPEN: u8 = 3;

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
    const E_NOT_ENOUGH_PACKETS: u64 = 1040;
    const E_PACKET_NOT_RECEIVED: u64 = 1041;
    const E_ACK_ALREADY_EXIST: u64 = 1042;
    const E_TIMEOUT_MUST_BE_SET: u64 = 1044;
    const E_ACK_LEN_MISMATCH: u64 = 1046;
    const E_CHANNEL_NOT_FOUND: u64 = 1047;
    const E_CONNECTION_NOT_FOUND: u64 = 1048;
    const E_TIMEOUT_HEIGHT_NOT_SUPPORTED: u64 = 1049;
    const E_PACKET_ALREADY_SENT: u64 = 1050;
    const E_BATCH_SAME_CHANNEL_ONLY: u64 = 1051;
    const E_PACKET_ALREADY_ACKNOWLEDGED: u64 = 1061;
    const E_MAKER_MSG_LEN_MISMATCH: u64 = 1062;

    // This event is only emitted during the `init` phase
    // since the voyager event source module requires at least
    // a single event to be emitted to be able to process the
    // events
    public struct Initiated has copy, drop, store {}

    public struct CreateClient has copy, drop, store {
        client_id: u32,
        client_type: String,
        counterparty_chain_id: String,
    }

    public struct CreateLensClient has copy, drop, store {
        client_id: u32,
        l2_chain_id: String,
        l1_client_id: u32,
        l2_client_id: u32
    }

    public struct UpdateClient has copy, drop, store {
        client_id: u32,
        height: u64
    }

    public struct ConnectionOpenInit has copy, drop, store {
        connection_id: u32,
        client_id: u32,
        counterparty_client_id: u32
    }

    public struct ConnectionOpenTry has copy, drop, store {
        connection_id: u32,
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    }

    public struct ConnectionOpenAck has copy, drop, store {
        connection_id: u32,
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    }

    public struct ConnectionOpenConfirm has copy, drop, store {
        connection_id: u32,
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    }

    public struct ChannelOpenInit has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_port_id: vector<u8>,
        connection_id: u32,
        version: String
    }

    public struct ChannelOpenTry has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_port_id: vector<u8>,
        counterparty_channel_id: u32,
        connection_id: u32,
        counterparty_version: String
    }

    public struct ChannelOpenAck has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_port_id: vector<u8>,
        counterparty_channel_id: u32,
        connection_id: u32
    }

    public struct ChannelOpenConfirm has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_port_id: vector<u8>,
        counterparty_channel_id: u32,
        connection_id: u32
    }

    public struct PacketSend has drop, copy, store {
        channel_id: u32,
        packet_hash: vector<u8>,
        packet: Packet
    }

    public struct PacketRecv has drop, store, copy {
        channel_id: u32,
        packet_hash: vector<u8>,
        maker: address,
        maker_msg: vector<u8>
    }

    public struct IntentPacketRecv has drop, store, copy {
        channel_id: u32,
        packet_hash: vector<u8>,
        maker: address,
        maker_msg: vector<u8>
    }

    public struct TimeoutPacket has drop, store, copy {
        packet: Packet
    }

    public struct WriteAck has drop, store, copy {
        channel_id: u32,
        packet_hash: vector<u8>,
        acknowledgement: vector<u8>
    }

    public struct PacketAck has drop, store, copy {
        channel_id: u32,
        packet_hash: vector<u8>,
        acknowledgement: vector<u8>,
        maker: address,
    }

    public struct Misbehaviour has drop, store, copy {
        client_id: u32
    }

    // Resource to hold the global state
    public struct IBCStore has key {
        id: UID,
        client_mgr: LightClientManager,
        commitments: Table<vector<u8>, vector<u8>>,
        connections: Table<u32, ConnectionEnd>,
        channels: Table<u32, Channel>,
        channel_to_port: Table<u32, String>,
        next_client_sequence: u32,
        next_channel_sequence: u32,
        next_connection_sequence: u32,
        packet_hash_to_digest: Table<vector<u8>, vector<u8>>
    }

    fun init(ctx: &mut TxContext) {
        event::emit(Initiated {});
        transfer::share_object(IBCStore {
            id: object::new(ctx),
            commitments: table::new(ctx),
            connections: table::new(ctx),
            channels: table::new(ctx),
            client_mgr: light_client::new(ctx),
            channel_to_port: table::new(ctx),
            next_client_sequence: 1,
            next_channel_sequence: 1,
            next_connection_sequence: 1,
            packet_hash_to_digest: table::new(ctx)
        });
    }

    /// Create a client with an initial client and consensus state
    ///
    /// `client_type`: type of the client (only "cometbls" is supported for now)
    /// `client_state_bytes`: the initial client state bytes with the client-defined encoding
    /// `consensus_state_bytes`: the initial consensus state bytes with the client-defined encoding
    public fun create_client(
        ibc_store: &mut IBCStore,
        client_type: String, 
        client_state_bytes: vector<u8>, 
        consensus_state_bytes: vector<u8>,
        ctx: &mut TxContext,
    ) {
        let client_id = ibc_store.generate_client_identifier();
        
        let (client_state_bytes, consensus_state_bytes, counterparty_chain_id, mut lens_client_event) = ibc_store.client_mgr.create_client(
            client_type,
            client_id,
            client_state_bytes,
            consensus_state_bytes,
            ctx,
        );

        if (lens_client_event.is_some()) {
            let lens_client_event = lens_client_event.extract();
            event::emit(
                CreateLensClient {
                    client_id: lens_client_event.client_id(),
                    l2_chain_id: lens_client_event.l2_chain_id(),
                    l1_client_id: lens_client_event.l1_client_id(),
                    l2_client_id: lens_client_event.l2_client_id()
                }
            );
        };

        assert!(ibc_store.client_mgr.status(client_id) == 0, E_CLIENT_NOT_ACTIVE);

        ibc_store.commitments.add(
            commitment::client_state_commitment_key(client_id),
            client_state_bytes
        );

        ibc_store.commitments.add(
            commitment::consensus_state_commitment_key(client_id, ibc_store.client_mgr.latest_height(client_id)),
            consensus_state_bytes
        );

        event::emit(
            CreateClient {
                client_id,
                client_type,
                counterparty_chain_id
            },
        )
    }

    /// Update a client with the `client_message`
    ///
    /// `client_id`: the id of the client to be updated
    /// `client_message`: the client-defined update message
    public fun update_client(
        ibc_store: &mut IBCStore,
        clock: &clock::Clock,
        client_id: u32,
        client_message: vector<u8>,
        relayer: address
    ) {
        assert!(ibc_store.client_mgr.status(client_id) == 0, E_CLIENT_NOT_ACTIVE);

        // Update the client and consensus states using the client message
        let (client_state, consensus_state, height) =
            ibc_store.client_mgr.update_client(client_id, clock, client_message, relayer);

        // Update the client state commitment
        *ibc_store.commitments.borrow_mut(commitment::client_state_commitment_key(client_id)) = client_state;

        // Update the consensus state commitment
        add_or_update_table<vector<u8>, vector<u8>>(&mut ibc_store.commitments,
            commitment::consensus_state_commitment_key(client_id, height),
            keccak256(&consensus_state)
        );

        event::emit(
            UpdateClient {
                client_id,
                height
            }
        );
    }

    /// Submit a misbehaviour to the client to freeze it in an event of misbehaviour
    ///
    /// `client_id`: the id of the client
    /// `misbehaviour`: client-defined misbehaviour
    public fun misbehaviour(
        ibc_store: &mut IBCStore,
        client_id: u32,
        misbehaviour: vector<u8>,
        relayer: address
    ) {
        assert!(ibc_store.client_mgr.status(client_id) == 0, E_CLIENT_NOT_ACTIVE);

        ibc_store.client_mgr.misbehaviour(client_id, misbehaviour, relayer);

        event::emit(
            Misbehaviour {
                client_id,
            }
        );
    }    

    /// Initiate the connection handshake using client at `client_id`. The next call is `connection_open_try`
    /// to advance the connection handshake.
    ///
    /// `client_id`: the id of the client which will be used for verification in for this connection
    /// `counterparty_client_id`: the id of the client on the counterparty chain
    public fun connection_open_init(
        ibc_store: &mut IBCStore,
        client_id: u32,
        counterparty_client_id: u32
    ) {
        assert!(ibc_store.client_mgr.status(client_id) == 0, E_CLIENT_NOT_ACTIVE);

        let connection_id = ibc_store.generate_connection_identifier();

        let connection =
            connection_end::new(
                CONN_STATE_INIT,
                client_id,
                counterparty_client_id,
                0
            );

        ibc_store.commit_connection(connection_id, connection);

        ibc_store.connections.add(connection_id, connection);

        event::emit(
            ConnectionOpenInit {
                connection_id,
                client_id,
                counterparty_client_id,
            }
        )
    }

    /// Run the second step of the connection handshake. The next call is `connection_open_ack` on the counterparty to
    /// progress the handshake.
    ///
    /// `counterparty_client_id`: the id of the client running on the counterparty chain
    /// `counterparty_connection_id`: the id of the connection that is generated during the init phase on the counterparty
    /// `client_id`: the id of the client running on this chain
    /// `proof_init`: the proof of the `ConnectionEnd` commitment on the counterparty
    /// `proof_height`: the height when this proof is generated
    public fun connection_open_try(
        ibc_store: &mut IBCStore,
        counterparty_client_id: u32,
        counterparty_connection_id: u32,
        client_id: u32,
        proof_init: vector<u8>,
        proof_height: u64
    ) {
        assert!(ibc_store.client_mgr.status(client_id) == 0, E_CLIENT_NOT_ACTIVE);

        let connection_id = ibc_store.generate_connection_identifier();

        let connection = connection_end::new(
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

        // Verify the connection state using the provided proof and expected state
        let res =
            verify_connection_state(
                &ibc_store.client_mgr,
                connection.client_id(),
                proof_height,
                proof_init,
                counterparty_connection_id,
                expected_connection
        );

        assert!(res == 0, res);

        ibc_store.commit_connection(connection_id, connection);

        ibc_store.connections.add(connection_id, connection);

        event::emit(
            ConnectionOpenTry {
                connection_id,
                client_id,
                counterparty_client_id,
                counterparty_connection_id,
            }
        );

    }

    /// Run the third step of the connection handshake. The connection will be open after this call on this end.
    /// `connection_open_confirm` is still needed to be run on the counterparty.
    ///
    /// `connection_id`: the id of the connection that is generated during the init phase
    /// `counterparty_connection_id`: the id of the connection that is generated during the try phase on the counterparty
    /// `proof_try`: the proof of the `ConnectionEnd` commitment on the counterparty
    /// `proof_height`: the height when this proof is generated
    public fun connection_open_ack(
        ibc_store: &mut IBCStore,
        connection_id: u32,
        counterparty_connection_id: u32,
        proof_try: vector<u8>,
        proof_height: u64
    ) {
        let connection = ibc_store.connections.borrow_mut(connection_id);

        // assert that this connection is at the `INIT` phase
        assert!(
            connection.state() == CONN_STATE_INIT,
            E_INVALID_CONNECTION_STATE
        );

        // Create the expected connection state to verify against the proof
        let expected_connection = connection_end::new(
            CONN_STATE_TRYOPEN,
            connection.counterparty_client_id(),
            connection.client_id(),
            connection_id
        );

        // Verify the connection state using the provided proof and expected state
        let res = verify_connection_state(
            &ibc_store.client_mgr,
            connection.client_id(),
            proof_height,
            proof_try,
            counterparty_connection_id,
            expected_connection
        );
        assert!(res == 0, res);

        // Update the connection state to TRYOPEN and set the counterparty connection ID
        connection.set_state(CONN_STATE_OPEN);
        connection.set_counterparty_connection_id(counterparty_connection_id);

        event::emit(
            ConnectionOpenAck {
                connection_id,
                client_id: connection.client_id(),
                counterparty_client_id: connection.counterparty_client_id(),
                counterparty_connection_id: connection.counterparty_connection_id()
            }
        );

        // Commit the updated connection to storage
        ibc_store.commit_connection(connection_id, *connection);
    }

    /// Run the final step of the connection handshake. The connection will be fully open on both ends after this.
    ///
    /// `connection_id`: the id of the connection that is generated during the try phase
    /// `proof_ack`: the proof of the `ConnectionEnd` commitment on the counterparty
    /// `proof_height`: the height when this proof is generated
    public fun connection_open_confirm(
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
            connection.counterparty_client_id(),
            connection.client_id(),
            connection_id
        );

        let counterparty_connection_id = connection.counterparty_connection_id();

        // Verify the connection state using the provided proof and expected state
        let res = verify_connection_state(
            &ibc_store.client_mgr,
            connection.client_id(),
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
                client_id: connection.client_id(),
                counterparty_client_id: connection.counterparty_client_id(),
                counterparty_connection_id
            }
        );

        // Commit the final state of the connection to storage
        ibc_store.commit_connection(connection_id, *connection);
    }

    /// Initiate a channel opening between two apps on the previously connected chains.
    ///
    /// `port_id`: The unique identifier of the app who will own this channel. The port must have the same
    /// address as the `witness` struct.
    /// `counterparty_port_id`: The port id of the app running on the counterparty chain.
    /// `connection_id`: The id of the connection where this channel will be based on.
    /// `version`: The app-defined version.
    /// `witness`: A struct where only the app WILL be able to get an instance of, for authentication.
    /// The name MUST be `IbcAppWitness`.
    public fun channel_open_init<T: drop>(
        ibc_store: &mut IBCStore,
        port_id: String,
        counterparty_port_id: vector<u8>,
        connection_id: u32,
        version: String,
        witness: T,
    ) {
        // Make sure that the `port_id` confirms the witness.
        validate_port(port_id, witness);

        // Ensure the connection exists and is in the OPEN state
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );

        // Generate a new channel ID
        let channel_id = ibc_store.generate_channel_identifier();

        // Create a new channel and set its properties
        let channel = channel::new(
            CHAN_STATE_INIT,
            connection_id,
            0,
            counterparty_port_id,
            version
        );

        ibc_store.channel_to_port.add(channel_id, port_id);

        ibc_store.commit_channel(channel_id, channel);

        ibc_store.channels.add(channel_id, channel);

        event::emit(
            ChannelOpenInit {
                port_id,
                channel_id,
                counterparty_port_id,
                connection_id,
                version
            }
        );

    }

    /// Run the second step of a channel handshake to open a channel between two apps on the previously connected chains.
    ///
    /// `port_id`: The unique identifier of the app who will own this channel. The port must have the same
    /// address as the `witness` struct.
    /// `connection_id`: The id of the connection where this channel will be based on.
    /// `counterparty_channel_id`: The id of the channel initiated on the counterparty chain.
    /// `counterparty_port_id`: The port id of the app running on the counterparty chain.
    /// `version`: The app-defined version.
    /// `counterparty_version`: The app-defined version that is used in the counterparty chain.
    /// `proof_init`: The proof of the channel end on the counterparty chain.
    /// `proof_height`: The height at where this proof is verifiable.
    /// `witness`: A struct where only the app WILL be able to get an instance of, for authentication.
    /// The name MUST be `IbcAppWitness`.
    public fun channel_open_try<T: drop>(
        ibc_store: &mut IBCStore,
        port_id: String,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String,
        counterparty_version: String,
        proof_init: vector<u8>,
        proof_height: u64,
        witness: T,
    ) {
        validate_port(port_id, witness);

        // Ensure the connection exists and is in the OPEN state
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection.state() == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );

        // Construct the expected channel state to verify against the proof
        let expected_channel = channel::new(
            CHAN_STATE_INIT,
            connection.counterparty_connection_id(),
            0,
            *port_id.bytes(),
            counterparty_version
        );

        // let light_client = ibc_store.clients.borrow(connection.client_id());
        // Verify the channel state using the provided proof and expected state
        let res = verify_channel_state(
            &ibc_store.client_mgr,
            connection.client_id(),
            proof_height,
            proof_init,
            counterparty_channel_id,
            expected_channel
        );
        assert!(res == 0, res);

        // Generate a new channel ID
        let channel_id = ibc_store.generate_channel_identifier();

        // Create a new channel and set its properties
        let channel = channel::new(
                CHAN_STATE_TRYOPEN,
                connection_id,
                counterparty_channel_id,
                counterparty_port_id,
                version
        );

        // Commit the created channel to the storage
        ibc_store.channel_to_port.add(channel_id, port_id);

        ibc_store.commit_channel(channel_id, channel);

        ibc_store.channels.add(channel_id, channel);

        event::emit(
            ChannelOpenTry {
                port_id,
                channel_id,
                counterparty_port_id,
                counterparty_channel_id,
                connection_id,
                counterparty_version
            }
        );
    }

    /// Run the third step of a channel handshake to open a channel between two apps on the previously connected chains.
    /// This runs after the `channel_open_init`, and `channel_open_confirm` should be run on the counterparty for the channel
    /// to be fully open.
    ///
    /// `port_id`: The unique identifier of the app who will own this channel. The port must have the same
    /// address as the `witness` struct.
    /// `channel_id`: The id of the channel that is created on the `try` phase.
    /// `counterparty_version`: The app-defined version that is used in the counterparty chain.
    /// `proof_try`: The proof of the channel end on the counterparty chain.
    /// `proof_height`: The height at where this proof is verifiable.
    /// `witness`: A struct where only the app WILL be able to get an instance of, for authentication.
    /// The name MUST be `IbcAppWitness`.
    public fun channel_open_ack<T: drop>(
        ibc_store: &mut IBCStore,
        port_id: String,
        channel_id: u32,
        counterparty_version: String,
        counterparty_channel_id: u32,
        proof_try: vector<u8>,
        proof_height: u64,
        witness: T,
    ) {
        let port_id = *ibc_store.channel_to_port.borrow(channel_id);
        validate_port(port_id, witness);

        // Ensure the channel is owned by this port
        assert!(
            *ibc_store.channel_to_port.borrow(channel_id) == port_id,
            E_UNAUTHORIZED
        );

        // Ensure the channel exists and is in the INIT state
        let channel = ibc_store.channels.borrow_mut(channel_id);
        assert!(
            channel.state() == CHAN_STATE_INIT,
            E_INVALID_CHANNEL_STATE
        );

        // Ensure the associated connection is in the OPEN state
        let connection_id = channel.connection_id();
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection.state() == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );

        // Construct the expected channel state to verify against the proof
        let expected_channel =
            channel::new(
                CHAN_STATE_TRYOPEN,
                connection.counterparty_connection_id(),
                channel_id,
                *port_id.bytes(),
                counterparty_version
            );

        // Verify the channel state using the provided proof and expected state
        let verification_result = verify_channel_state(
            &ibc_store.client_mgr,
            connection.client_id(),
            proof_height,
            proof_try,
            counterparty_channel_id,
            expected_channel
        );
        assert!(verification_result == 0, verification_result);

        // Update the channel state to OPEN and set the counterparty channel ID
        channel.set_state(CHAN_STATE_OPEN);
        channel.set_counterparty_channel_id(counterparty_channel_id);
        channel.set_version(counterparty_version);

        // Emit an event for the channel open acknowledgment
        event::emit(
            ChannelOpenAck {
                port_id,
                channel_id,
                counterparty_port_id: *channel.counterparty_port_id(),
                counterparty_channel_id,
                connection_id
            }
        );

        // Commit the updated channel to storage
        ibc_store.commit_channel(channel_id, *channel);
    }

    /// Run the final step of a channel handshake to open a channel between two apps on the previously connected chains.
    /// The channel will be open in both ends after this call.
    ///
    /// `port_id`: The unique identifier of the app who will own this channel. The port must have the same
    /// address as the `witness` struct.
    /// `channel_id`: The id of the channel that is created on the `try` phase.
    /// `proof_ack`: The proof of the channel end on the counterparty chain.
    /// `proof_height`: The height at where this proof is verifiable.
    /// `witness`: A struct where only the app WILL be able to get an instance of, for authentication.
    /// The name MUST be `IbcAppWitness`.
    public fun channel_open_confirm<T: drop>(
        ibc_store: &mut IBCStore,
        channel_id: u32,
        proof_ack: vector<u8>,
        proof_height: u64,
        witness: T
    ) {
        let port_id = *ibc_store.channel_to_port.borrow(channel_id);
        validate_port(port_id, witness);

        // Ensure the channel exists and is in the TRYOPEN state
        let channel = ibc_store.channels.borrow_mut(channel_id);
        assert!(
            channel.state() == CHAN_STATE_TRYOPEN,
            E_INVALID_CHANNEL_STATE
        );

        // Ensure the associated connection is in the OPEN state
        let connection_id = channel::connection_id(channel);
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection.state() == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );

        let connection = ibc_store.connections.borrow(connection_id);

        // Construct the expected channel state in the OPEN state to verify against the proof
        let expected_channel =
            channel::new(
                CHAN_STATE_OPEN,
                connection.counterparty_connection_id(),
                channel_id,
                *port_id.bytes(),
                *channel.version()
            );

        // Verify the channel state using the provided proof and expected state
        let verification_result = verify_channel_state(
            &ibc_store.client_mgr,
            connection.client_id(),
            proof_height,
            proof_ack,
            channel.counterparty_channel_id(),
            expected_channel
        );
        assert!(verification_result == 0, verification_result);

        channel.set_state(CHAN_STATE_OPEN);

        event::emit(
            ChannelOpenConfirm {
                port_id,
                channel_id,
                counterparty_port_id: *channel.counterparty_port_id(),
                counterparty_channel_id: channel.counterparty_channel_id(),
                connection_id: channel.connection_id()
            }
        );

        // Commit the final state of the channel to storage
        ibc_store.commit_channel(channel_id, *channel);
    }


    /// Function to send a packet through an open channel
    public fun send_packet<T: drop>(
        ibc_store: &mut IBCStore,
        clock: &Clock,
        source_channel: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        data: vector<u8>,
        witness: T,
        ctx: &TxContext
    ): packet::Packet {
        // Check if the channel exists in the store
        if(!ibc_store.channels.contains(source_channel)) {
            abort E_CHANNEL_NOT_FOUND
        };

        // Validate timeout values
        assert!(
            timeout_height == 0, E_TIMEOUT_HEIGHT_NOT_SUPPORTED
        );

        assert!(
            timeout_timestamp > (clock.timestamp_ms() * 1_000_000),
            E_TIMESTAMP_TIMEOUT
        );

        let port_id = *ibc_store.channel_to_port.borrow(source_channel);
        validate_port(port_id, witness);

        let channel = *ibc_store.channels.borrow(source_channel);
        assert!(channel.state() == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);
        
        // Prepare packet for commitment
        let packet =
            packet::new(
                source_channel,
                channel.counterparty_channel_id(),
                data,
                timeout_height,
                timeout_timestamp
            );
        let packet_hash = commitment::commit_packet(&packet);
        let commitment_key =
            commitment::batch_packets_commitment_key(
                packet_hash
            );

        assert!(!ibc_store.commitments.contains(commitment_key), E_PACKET_ALREADY_SENT);

        ibc_store.commitments.add(commitment_key, COMMITMENT_MAGIC);

        // This is very important for the relayers to be able to get the exact transaction from the `packet_hash`.
        // They will later use this to get the full packet.
        ibc_store.packet_hash_to_digest.add(packet_hash, *ctx.digest());

        event::emit(
            PacketSend {
                channel_id: source_channel,
                packet_hash,
                packet
            }
        );

        packet
    }

    /// Function to send a packet through an open channel
    public fun recv_packet<T: drop>(
        ibc_store: &mut IBCStore,
        clock: &clock::Clock,
        packets: vector<Packet>,
        maker: address,
        maker_msgs: vector<vector<u8>>,
        proof: vector<u8>,
        proof_height: u64,
        acknowledgements: vector<vector<u8>>,
        witness: T,
    ) {
        let port_id = *ibc_store.channel_to_port.borrow(packets[0].destination_channel_id());
        validate_port(port_id, witness);

        process_receive(
            ibc_store,
            clock,
            packets,
            maker,
            maker_msgs,
            proof_height,
            proof,
            false,
            acknowledgements
        );
    }

    /// Function to send a packet through an open channel
    public fun recv_intent_packet<T: drop>(
        ibc_store: &mut IBCStore,
        clock: &clock::Clock,
        packets: vector<Packet>,
        maker: address,
        maker_msgs: vector<vector<u8>>,
        acknowledgements: vector<vector<u8>>,
        witness: T,
    ) {
        let port_id = *ibc_store.channel_to_port.borrow(packets[0].destination_channel_id());
        validate_port(port_id, witness);

        process_receive(
            ibc_store,
            clock,
            packets,
            maker,
            maker_msgs,
            0,
            vector::empty(),
            true,
            acknowledgements
        );
    }

    // Function to generate a client identifier
    fun generate_client_identifier(ibc_store: &mut IBCStore): u32 {
        let seq = ibc_store.next_client_sequence;
        ibc_store.next_client_sequence = ibc_store.next_client_sequence + 1;
        seq
    }

    fun generate_connection_identifier(ibc_store: &mut IBCStore): u32 {
        let seq = ibc_store.next_connection_sequence;
        ibc_store.next_connection_sequence = ibc_store.next_connection_sequence + 1;
        seq
    }

    fun generate_channel_identifier(ibc_store: &mut IBCStore): u32 {
        let seq = ibc_store.next_channel_sequence;
        ibc_store.next_channel_sequence = ibc_store.next_channel_sequence + 1;
        seq
    }

    // TODO(aeryz): update should only be allowed for:
    // - client state,
    // - connection end
    // - channel end commitments, the rest should abort in `update`
    fun add_or_update_table<T: drop + store + copy, P: drop + store>(table: &mut Table<T, P>, key: T, value: P) {
        if (table.contains(key)) {
            let val = table.borrow_mut(key);
            *val = value;
        } else {
            table.add(key, value);
        }
    }

    fun commit_connection(ibc_store: &mut IBCStore, connection_id: u32, connection: ConnectionEnd) {
        add_or_update_table(
            &mut ibc_store.commitments,
            commitment::connection_commitment_key(connection_id),
            keccak256(&connection.encode())
        );
    }

    fun commit_channel(ibc_store: &mut IBCStore, channel_id: u32, channel: Channel) {
       add_or_update_table(
            &mut ibc_store.commitments,
            commitment::channel_commitment_key(channel_id),
            keccak256(&channel.encode())
        );
    }

    fun verify_connection_state(
        client_mgr: &LightClientManager,
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        connection_id: u32,
        counterparty_connection: ConnectionEnd
    ): u64 {
        client_mgr.verify_membership(
            client_id,
            height,
            proof,
            commitment::connection_commitment_key(connection_id),
            keccak256(&connection_end::encode(&counterparty_connection))
        )
    }

    fun verify_channel_state(
        client_mgr: &LightClientManager,
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        channel_id: u32,
        channel: Channel
    ): u64 {
        client_mgr.verify_membership(
            client_id,
            height,
            proof,
            commitment::channel_commitment_key(channel_id),
            keccak256(&channel::encode(&channel))
        )
    }

    // module_address::module_name::store_1::store_2::..::store_n
    fun deconstruct_port_id(mut port_id: String): std::ascii::String {
        if (port_id.substring(0, 2) == utf8(b"0x")) {
            port_id = port_id.substring(2, port_id.length());
        };

        let mut parts = vector::empty();
        while (true) {
            let split = utf8(b"::");
            let first = port_id.index_of(&split);            
            // invalid port
            assert!(first != 0, 1);
            if (first == port_id.length()) {
                // last one
                parts.push_back(port_id);
                break
            };
            let lhs = port_id.substring(0, first);
            let rhs = port_id.substring(first + 2, port_id.length());
            parts.push_back(lhs);
            port_id = rhs;
        };

        assert!(parts.length() >= 3, 1);

        let mut a = sui::address::from_ascii_bytes(parts[0].bytes()).to_ascii_string();
        a.append(std::ascii::string(b"::"));
        a.append(parts[1].to_ascii());

        a
    }

    fun validate_port<T: drop>(
        port_id: String,
        _: T,
    ) {       
        let caller_t = std::type_name::get<T>();

        let mut addr_module = deconstruct_port_id(port_id);
        addr_module.append(std::ascii::string(b"::IbcAppWitness"));
        
        // ensure the port info matches the caller
        assert!(addr_module == std::type_name::get<T>().into_string(), 1)
    }

    public fun get_counterparty_connection(
        ibc_store: &mut IBCStore,
        connection_id: u32
    ): u32 {
        let connection = ibc_store.connections.borrow(connection_id);
        connection_end::counterparty_connection_id(connection)
    }

    fun set_packet_receive(ibc_store: &mut IBCStore, commitment_key: vector<u8>): bool {
        if (ibc_store.commitments.contains(commitment_key)) {
            true
        } else {
            ibc_store.commitments.add(commitment_key, COMMITMENT_MAGIC);
            false
        }
    }

    fun set_packet_acknowledged(ibc_store: &mut IBCStore, commitment_key: vector<u8>) {
        assert!(ibc_store.commitments.contains(commitment_key), E_PACKET_COMMITMENT_NOT_FOUND);

        let commitment = ibc_store.commitments.borrow_mut(commitment_key);
        assert!(commitment != COMMITMENT_MAGIC_ACK, E_PACKET_ALREADY_ACKNOWLEDGED);
        assert!(commitment == COMMITMENT_MAGIC, E_PACKET_COMMITMENT_NOT_FOUND);

        *commitment = COMMITMENT_MAGIC_ACK;
    }

    fun process_receive(
        ibc_store: &mut IBCStore,
        clock: &clock::Clock,
        packets: vector<Packet>,
        maker: address,
        maker_msgs: vector<vector<u8>>,
        proof_height: u64,
        proof: vector<u8>,
        intent: bool,
        acknowledgements: vector<vector<u8>>
    ) {
        let l = vector::length(&packets);
        assert!(l != 0, E_NOT_ENOUGH_PACKETS);

        assert!(l == acknowledgements.length(), E_ACK_LEN_MISMATCH);
        assert!(l == maker_msgs.length(), E_MAKER_MSG_LEN_MISMATCH);

        let destination_channel = packet::destination_channel_id(&packets[0]);

        if(!ibc_store.channels.contains(destination_channel)) {
            abort E_CHANNEL_NOT_FOUND
        };
        let channel = ibc_store.channels.borrow(destination_channel);
        assert!(channel.state() == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        let connection = ibc_store.connections.borrow(channel.connection_id()); 
        assert!(connection.state() == CONN_STATE_OPEN, E_INVALID_CONNECTION_STATE);

        let client_id = connection.client_id();

        if(!ibc_store.client_mgr.exists(client_id)) {
            abort E_CLIENT_NOT_FOUND
        };

        if (!intent) {
            let commitment_key = commitment::batch_packets_commitment_key(
                commitment::commit_packets(&packets)
            );

            let err =
                ibc_store.client_mgr.verify_membership(
                    client_id,
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

            assert!(packet.destination_channel_id() == destination_channel, E_BATCH_SAME_CHANNEL_ONLY);

            if (packet.timeout_height() != 0) {
                abort E_TIMEOUT_HEIGHT_NOT_SUPPORTED
            };


            let current_timestamp = clock::timestamp_ms(clock) * 1_000_000; 
            assert!(
                current_timestamp < packet.timeout_timestamp(),
                E_TIMESTAMP_TIMEOUT
            );

            let packet_hash = commitment::commit_packet(&packet);
            let commitment_key =
                commitment::batch_receipts_commitment_key(
                    packet_hash
                );

            if(!set_packet_receive(ibc_store, commitment_key)) {
                let maker_msg = maker_msgs[i];
                if (intent) {
                    event::emit(IntentPacketRecv {
                        channel_id: destination_channel,
                        packet_hash,
                        maker,
                        maker_msg
                    });
                } else {
                    event::emit(
                        PacketRecv {
                            channel_id: destination_channel,
                            packet_hash,
                            maker,
                            maker_msg
                        }
                    )
                };

                let acknowledgement = acknowledgements[i];
                if (!acknowledgement.is_empty()) {
                    inner_write_acknowledgement(ibc_store, commitment_key, acknowledgement);
                    event::emit(
                        WriteAck {
                            channel_id: destination_channel,
                            packet_hash,
                            acknowledgement
                        }
                    );
                };
            };
            i = i + 1;
        }
    }

    public fun write_acknowledgement<T: drop>(
        ibc_store: &mut IBCStore,
        packet: packet::Packet,
        acknowledgement: vector<u8>,
        witness: T,
    ) {
        assert!(!acknowledgement.is_empty(), E_ACKNOWLEDGEMENT_IS_EMPTY);

        if(!ibc_store.channels.contains(packet.destination_channel_id())) {
            abort E_CHANNEL_NOT_FOUND
        };

        let channel = *ibc_store.channels.borrow(packet.destination_channel_id());
        assert!(channel.state() == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        let port_id = *ibc_store.channel_to_port.borrow(packet.destination_channel_id());
        validate_port(port_id, witness);

        let packet_hash = commitment::commit_packet(&packet);

        let commitment_key =
            commitment::batch_receipts_commitment_key(
                packet_hash
            );

        inner_write_acknowledgement(ibc_store, commitment_key, acknowledgement);

        event::emit(
            WriteAck{
                channel_id: packet.destination_channel_id(),
                packet_hash,
                acknowledgement,
            }
        )
    }

    public fun acknowledge_packet<T: drop>(
        ibc_store: &mut IBCStore,
        packets: vector<packet::Packet>,
        acknowledgements: vector<vector<u8>>,
        proof: vector<u8>,
        proof_height: u64,
        relayer: address,
        witness: T
    )  {
        let l = vector::length(&packets);
        assert!(l > 0, E_NOT_ENOUGH_PACKETS);

        let source_channel = packets[0].source_channel_id();

        let port_id = *ibc_store.channel_to_port.borrow(source_channel);
        validate_port(port_id, witness);

        if(!ibc_store.channels.contains(source_channel)) {
            abort E_CHANNEL_NOT_FOUND
        };
        let channel = ibc_store.channels.borrow(source_channel);
        assert!(channel.state() == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        let destination_channel = packets[0].destination_channel_id();

        let connection = ibc_store.connections.borrow(channel.connection_id());
        assert!(connection.state() == CONN_STATE_OPEN, E_INVALID_CONNECTION_STATE);

        let client_id = connection.client_id();

        if (!ibc_store.client_mgr.exists(client_id)) {
            abort E_CLIENT_NOT_FOUND
        };

        let commitment_key = commitment::batch_receipts_commitment_key(
            commitment::commit_packets(&packets)
        );

        let err =
            ibc_store.client_mgr.verify_membership(
                client_id,
                proof_height,
                proof,
                commitment_key,
                commitment::commit_acks(acknowledgements)
            );

        assert!(err == 0, err);

        let mut i = 0;
        while (i < l) {
            let packet = packets[i];

            assert!(packet.source_channel_id() == source_channel, E_BATCH_SAME_CHANNEL_ONLY);

            let packet_hash = commitment::commit_packet(&packet);
            let commitment_key =
                commitment::batch_packets_commitment_key(
                    packet_hash
                );
            set_packet_acknowledged(ibc_store, commitment_key);
            
            event::emit(
                PacketAck {
                    channel_id: source_channel,
                    packet_hash,
                    acknowledgement: acknowledgements[i],
                    maker: relayer
                }
            );

            i = i + 1;
        }
    }

    public fun get_client_state(ibc_store: &IBCStore, client_id: u32): vector<u8> {
        if (!ibc_store.client_mgr.exists(client_id)) {
            abort E_CLIENT_NOT_FOUND
        };

        ibc_store.client_mgr.get_client_state(client_id)
    }

    public fun get_port_id(ibc_store: &IBCStore, channel_id: u32): String {
        if (!ibc_store.channel_to_port.contains(channel_id)) {
            abort E_CHANNEL_NOT_FOUND
        };
        *ibc_store.channel_to_port.borrow(channel_id)
    }

    public fun get_consensus_state(ibc_store: &IBCStore, client_id: u32, height: u64): vector<u8> {
        if (!ibc_store.client_mgr.exists(client_id)) {
            abort E_CLIENT_NOT_FOUND
        };

        ibc_store.client_mgr.get_consensus_state(client_id, height)
    }

    public fun get_connection(ibc_store: &IBCStore, connection_id: u32): ConnectionEnd {
        if (!ibc_store.connections.contains(connection_id)) {
            abort E_CONNECTION_NOT_FOUND
        };
        *ibc_store.connections.borrow(connection_id)
    }

    public fun get_channel(ibc_store: &IBCStore, channel_id: u32): Channel {
        if (!ibc_store.channels.contains(channel_id)) {
            abort E_CHANNEL_NOT_FOUND
        };
        *ibc_store.channels.borrow(channel_id)
    }

    fun inner_write_acknowledgement(
        ibc_store: &mut IBCStore,
        commitment_key: vector<u8>, acknowledgement: vector<u8>
    ) {
        if (!ibc_store.commitments.contains(commitment_key)) {
            abort E_PACKET_NOT_RECEIVED
        };
        let commitment = ibc_store.commitments.borrow_mut(commitment_key);
        assert!(
            *commitment == COMMITMENT_MAGIC,
            E_ACK_ALREADY_EXIST
        );

        *commitment = commitment::commit_ack(acknowledgement);
    }

    public fun timeout_packet(
        ibc_store: &mut IBCStore,
        packet: Packet,
        proof: vector<u8>,
        proof_height: u64,
    ) {
        let source_channel = packet::source_channel_id(&packet);
        let destination_channel = packet::destination_channel_id(&packet);

        if(!ibc_store.channels.contains(source_channel)) {
            abort E_CHANNEL_NOT_FOUND
        };
        let channel = ibc_store.channels.borrow(source_channel);
        assert!(channel::state(channel) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        let connection_id = channel::connection_id(channel);

        if(!ibc_store.connections.contains(connection_id)) {
            abort E_CONNECTION_NOT_FOUND
        };
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );
        let client_id = connection_end::client_id(connection);

        if(!ibc_store.client_mgr.exists(client_id)) {
            abort E_CLIENT_NOT_FOUND
        };
        let proof_timestamp =
            ibc_store.client_mgr.get_timestamp_at_height(client_id, proof_height);
        assert!(proof_timestamp != 0, E_LATEST_TIMESTAMP_NOT_FOUND);


        let commitment_key =
                commitment::batch_receipts_commitment_key(
                    commitment::commit_packet(&packet)
                );
        let err = ibc_store.client_mgr.verify_non_membership(client_id, proof_height, proof, commitment_key);
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
                commitment::commit_packet(&packet)
            );
        
        ibc_store.commitments.remove(commitment_key);

        event::emit(TimeoutPacket { packet });
    }

    // #[test]
    // fun test_generate_channel_identifier() {
    //     let mut ctx = tx_context::dummy();

    //     let mut test_case = test_scenario::begin(@0x0);
    //     init(test_case.ctx());
    //     test_case.next_tx(@0x0);
    //     let mut ibc_store = test_case.take_shared<IBCStore>();
        
    //     let channel_id1 = ibc_store.generate_channel_identifier();
    //     let channel_id2 = ibc_store.generate_channel_identifier();

    //     assert!(channel_id1 == 1, 0);
    //     assert!(channel_id2 == 2, 0);
    
    //     test_scenario::return_shared(ibc_store);
    //     test_case.end();
    // }

    // #[test]
    // fun test_generate_client_identifier() {
    //     let mut ctx = tx_context::dummy();

    //     let mut test_case = test_scenario::begin(@0x0);
    //     init(test_case.ctx());
    //     test_case.next_tx(@0x0);
    //     let mut ibc_store = test_case.take_shared<IBCStore>();
        
    //     let client_id1 = ibc_store.generate_client_identifier();
    //     let client_id2 = ibc_store.generate_client_identifier();

    //     assert!(client_id1 == 1, 0);
    //     assert!(client_id2 == 2, 0);

    //     test_scenario::return_shared(ibc_store);
    //     test_case.end();
    // }

    // #[test]
    // fun test_generate_connection_identifier() {
    //     let mut ctx = tx_context::dummy();

    //     let mut test_case = test_scenario::begin(@0x0);
    //     init(test_case.ctx());
    //     test_case.next_tx(@0x0);
    //     let mut ibc_store = test_case.take_shared<IBCStore>();
        
    //     let connection_id1 = ibc_store.generate_connection_identifier();
    //     let connection_id2 = ibc_store.generate_connection_identifier();

    //     assert!(connection_id1 == 1, 0);
    //     assert!(connection_id2 == 2, 0);

    //     test_scenario::return_shared(ibc_store);
    //     test_case.end();
    // }

    // #[test]
    // fun test_commit_connection() {
    //     let mut ctx = tx_context::dummy();

    //     let mut test_case = test_scenario::begin(@0x0);
    //     init(test_case.ctx());
    //     test_case.next_tx(@0x0);
    //     let mut ibc_store = test_case.take_shared<IBCStore>();

    //     let connection_id = ibc_store.generate_connection_identifier();
    //     let connection = connection_end::new(
    //         CONN_STATE_INIT,
    //         1, // client_id
    //         2, // counterparty_client_id
    //         0, // counterparty_connection_id
    //     );

    //     // First commit
    //     ibc_store.commit_connection(connection_id, connection);

    //     // Verify the commitment exists
    //     let key = commitment::connection_commitment_key(connection_id);
    //     assert!(ibc_store.commitments.contains(key), E_CONNECTION_DOES_NOT_EXIST);

    //     // Update connection state
    //     let updated_connection = connection_end::new(
    //         CONN_STATE_OPEN,
    //         1,
    //         2,
    //         0,
    //     );
    //     ibc_store.commit_connection(connection_id, updated_connection);

    //     // Verify that the commitment is updated
    //     let encoded_connection = encode_connection(updated_connection);
    //     assert!(ibc_store.commitments.borrow(key) == encoded_connection, 0);

    //     test_scenario::return_shared(ibc_store);
    //     test_case.end();
    // }

    // #[test]
    // fun test_connection_open_init() {
    //     let mut ctx = tx_context::dummy();
    //     let mut test_case = test_scenario::begin(@0x0);
    //     init(test_case.ctx());
    //     test_case.next_tx(@0x0);
    //     let mut ibc_store = test_case.take_shared<IBCStore>();

    //     create_client(&mut ibc_store, utf8(b"cometbls"), b"client_state", b"proof", &mut ctx);

    //     // Set up necessary inputs
    //     let client_id = 1;
    //     let counterparty_client_id = 2;

    //     // Call connection_open_init
    //     connection_open_init(&mut ibc_store, client_id, counterparty_client_id);

    //     // Verify connection state
    //     let connection_id = 1; // First generated connection ID should be 1
    //     let connection = ibc_store.connections.borrow(connection_id);
    //     assert!(connection_end::state(connection) == CONN_STATE_INIT, E_INVALID_CONNECTION_STATE);

    //     test_scenario::return_shared(ibc_store);
    //     test_case.end();
    // }

    // #[test]
    // fun test_connection_open_try() {
    //     let mut ctx = tx_context::dummy();
    //     let mut test_case = test_scenario::begin(@0x0);
    //     init(test_case.ctx());
    //     test_case.next_tx(@0x0);
        
    //     let mut ibc_store = test_case.take_shared<IBCStore>();

    //     create_client(&mut ibc_store, utf8(b"cometbls"), b"client_state", b"proof", &mut ctx);

    //     // Initialize connection first
    //     let client_id = 1;
    //     let counterparty_client_id = 2;
    //     connection_open_init(&mut ibc_store, client_id, counterparty_client_id);

    //     // Prepare inputs for connection_open_try
    //     let connection_id = 1;
    //     let proof_init = b"proof";
    //     let proof_height = 1;

    //     // Call connection_open_try
    //     connection_open_try(&mut ibc_store, client_id, counterparty_client_id, connection_id, proof_init, proof_height);

    //     // Verify state transition to TRYOPEN
    //     let connection = ibc_store.connections.borrow(connection_id);
    //     assert!(connection_end::state(connection) == CONN_STATE_INIT, E_INVALID_CONNECTION_STATE);

    //     test_scenario::return_shared(ibc_store);
    //     test_case.end();
    // }

    // #[test]
    // fun test_connection_open_ack() {
    //     let mut ctx = tx_context::dummy();
    //     let mut test_case = test_scenario::begin(@0x0);
    //     init(test_case.ctx());
    //     test_case.next_tx(@0x0);
    //     let mut ibc_store = test_case.take_shared<IBCStore>();

    //     create_client(&mut ibc_store, utf8(b"cometbls"), b"client_state", b"proof", &mut ctx);
    //     // Initialize and try-open connection first
    //     let client_id = 1;
    //     let proof_height = 1;
    //     let counterparty_client_id = 2;
    //     connection_open_init(&mut ibc_store, client_id, counterparty_client_id);
    //     connection_open_try(&mut ibc_store, client_id, counterparty_client_id, client_id, b"proof", proof_height);

    //     // Prepare inputs for connection_open_ack
    //     let connection_id = 1;
    //     let counterparty_connection_id = 1;
    //     let proof_try = b"proof";

    //     // Call connection_open_ack
    //     connection_open_ack(&mut ibc_store, connection_id, counterparty_connection_id, proof_try, proof_height);

    //     // Verify state transition to OPEN
    //     let connection = ibc_store.connections.borrow(connection_id);
    //     assert!(connection_end::state(connection) == CONN_STATE_TRYOPEN, E_INVALID_CONNECTION_STATE);

    //     test_scenario::return_shared(ibc_store);
    //     test_case.end();
    // }

    // #[test]
    // fun test_connection_open_confirm() {
    //     let mut ctx = tx_context::dummy();
    //     let mut test_case = test_scenario::begin(@0x0);
    //     init(test_case.ctx());
    //     test_case.next_tx(@0x0);
    //     let mut ibc_store = test_case.take_shared<IBCStore>();

    //     create_client(&mut ibc_store, utf8(b"cometbls"), b"client_state", b"proof", &mut ctx);
    //     // Initialize, try-open, and ack the connection first
    //     let client_id = 1;
    //     let proof_height = 1;
    //     let counterparty_client_id = 2;
    //     connection_open_init(&mut ibc_store, client_id, counterparty_client_id);
    //     connection_open_try(&mut ibc_store, client_id, counterparty_client_id, client_id, b"proof", proof_height);
    //     connection_open_ack(&mut ibc_store, client_id, counterparty_client_id, b"proof", proof_height);

    //     // Prepare inputs for connection_open_confirm
    //     let connection_id = 1;
    //     let proof_ack = b"proof";

    //     // Call connection_open_confirm
    //     connection_open_confirm(&mut ibc_store, connection_id, proof_ack, proof_height);

    //     // Verify final state is OPEN
    //     let connection = ibc_store.connections.borrow(connection_id);
    //     assert!(connection_end::state(connection) == CONN_STATE_OPEN, E_INVALID_CONNECTION_STATE);

    //     test_scenario::return_shared(ibc_store);
    //     test_case.end();
    // }

    // #[test_only]
    // fun mock_valid_connection(ibc_store: &mut IBCStore, ctx: &mut TxContext): u32 {
    //     // Set up initial details for the connection
    //     let client_id = 1;
    //     let counterparty_client_id = 2;
        
    //     create_client(ibc_store, utf8(b"cometbls"), b"client_state", b"proof", ctx);
    //     // Initialize the connection
    //     connection_open_init(ibc_store, client_id, counterparty_client_id);
        
    //     // Move to the TRYOPEN state
    //     let connection_id = 1;
    //     connection_open_try(ibc_store, client_id, counterparty_client_id, connection_id, b"proof", 1);
        
    //     // Move to the ACK state
    //     let counterparty_connection_id = 1;
    //     connection_open_ack(ibc_store, connection_id, counterparty_connection_id, b"proof", 1);
        
    //     // Move to the final OPEN state
    //     connection_open_confirm(ibc_store, connection_id, b"proof", 1);

    //     connection_id // Return the connection ID for reuse
    // }


    // #[test]
    // fun test_channel_open_init() {
    //     let mut ctx = tx_context::dummy();
    //     let mut test_case = test_scenario::begin(@0x0);
    //     init(test_case.ctx());
    //     test_case.next_tx(@0x0);
    //     let mut ibc_store = test_case.take_shared<IBCStore>();

    //     // Use the mock function to create a valid connection
    //     let connection_id = mock_valid_connection(&mut ibc_store, &mut ctx);

    //     // Now proceed with the channel setup using the valid connection
    //     let port_id = utf8(b"test_port");
    //     let counterparty_port_id = b"counterparty_test_port";
    //     let version = utf8(b"test_version");

    //     // Call channel_open_init
    //     channel_open_init(&mut ibc_store, port_id, counterparty_port_id, connection_id, version);

    //     // Verify the channel state
    //     let channel_id = 1;
    //     let channel = ibc_store.channels.borrow(channel_id);
    //     assert!(channel::state(channel) == CHAN_STATE_INIT, E_INVALID_CHANNEL_STATE);

    //     test_scenario::return_shared(ibc_store);
    //     test_case.end();
    // }

    // #[test]
    // fun test_channel_open_try() {
    //     let mut ctx = tx_context::dummy();
    //     let mut test_case = test_scenario::begin(@0x0);
    //     init(test_case.ctx());
    //     test_case.next_tx(@0x0);
    //     let mut ibc_store = test_case.take_shared<IBCStore>();

    //     // Use the mock function to create a valid connection
    //     let connection_id = mock_valid_connection(&mut ibc_store, &mut ctx);

    //     // Set up necessary inputs for channel_open_try
    //     let port_id = utf8(b"test_port");
    //     let channel_state = CHAN_STATE_TRYOPEN; // Assuming we want the state to be TRYOPEN
    //     let counterparty_port_id = b"counterparty_test_port";
    //     let version = utf8(b"test_version");
    //     let counterparty_version = utf8(b"counterparty_version");
    //     let counterparty_channel_id = 1;
    //     let proof_init = b"proof";
    //     let proof_height = 1;

    //     // Call channel_open_try
    //     channel_open_try(
    //         &mut ibc_store,
    //         port_id,
    //         connection_id,
    //         counterparty_channel_id,
    //         counterparty_port_id,
    //         version,
    //         counterparty_version,
    //         proof_init,
    //         proof_height
    //     );

    //     // Retrieve the generated channel ID for verification (assuming it starts from 0)
    //     let channel_id = 1;
    //     let channel = ibc_store.channels.borrow(channel_id);

    //     // Verify that the channel state is set to TRYOPEN
    //     std::debug::print(&utf8(b"channel is:"));
    //     std::debug::print(&channel::state(channel));
    //     assert!(channel::state(channel) == CHAN_STATE_TRYOPEN, E_INVALID_CHANNEL_STATE);

    //     assert!(channel::connection_id(channel) == connection_id, E_CONNECTION_DOES_NOT_EXIST);

    //     // Verify the version is set as expected
    //     assert!(channel::version(channel) == version, E_UNSUPPORTED_VERSION);

    //     test_scenario::return_shared(ibc_store);
    //     test_case.end();
    // }


    // #[test]
    // fun test_channel_open_ack() {
    //     let mut ctx = tx_context::dummy();
    //     let mut test_case = test_scenario::begin(@0x0);
    //     init(test_case.ctx());
    //     test_case.next_tx(@0x0);
    //     let mut ibc_store = test_case.take_shared<IBCStore>();

    //     // Use the mock function to create a valid connection
    //     let connection_id = mock_valid_connection(&mut ibc_store, &mut ctx);

    //     // Set up necessary inputs for channel_open_try
    //     let port_id = utf8(b"test_port");
    //     let channel_state = CHAN_STATE_TRYOPEN; // Assuming we want the state to be TRYOPEN
    //     let counterparty_port_id = b"counterparty_test_port";
    //     let version = utf8(b"test_version");
    //     let counterparty_version = utf8(b"counterparty_version");
    //     let counterparty_channel_id = 1;
    //     let proof_init = b"proof";
    //     let proof_height = 1;

    //     // Call channel_open_try
    //     channel_open_try(
    //         &mut ibc_store,
    //         port_id,
    //         connection_id,
    //         counterparty_channel_id,
    //         counterparty_port_id,
    //         version,
    //         counterparty_version,
    //         proof_init,
    //         proof_height
    //     );

    //     // Prepare inputs for channel_open_ack
    //     let channel_id = 1; // Assuming the generated ID is 0 for the first channel
    //     let proof_try = b"proof";
    //     let proof_height_ack = 1;

    //     // Call channel_open_ack
    //     channel_open_ack(
    //         &mut ibc_store,
    //         port_id,
    //         channel_id,
    //         counterparty_version,
    //         counterparty_channel_id,
    //         proof_try,
    //         proof_height_ack
    //     );

    //     // Verify state transition to OPEN
    //     let channel = ibc_store.channels.borrow(channel_id);
    //     assert!(channel::state(channel) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

    //     test_scenario::return_shared(ibc_store);
    //     test_case.end();
    // }


    // #[test]
    // fun test_channel_open_confirm() {
    //     let mut ctx = tx_context::dummy();
    //     let mut test_case = test_scenario::begin(@0x0);
    //     init(test_case.ctx());
    //     test_case.next_tx(@0x0);
    //     let mut ibc_store = test_case.take_shared<IBCStore>();

    //     // Use the mock function to create a valid connection
    //     let connection_id = mock_valid_connection(&mut ibc_store, &mut ctx);

    //     // Set up necessary inputs for channel_open_try
    //     let port_id = utf8(b"test_port");
    //     let channel_state = CHAN_STATE_TRYOPEN; // Assuming we want the state to be TRYOPEN
    //     let counterparty_port_id = b"counterparty_test_port";
    //     let version = utf8(b"test_version");
    //     let counterparty_version = utf8(b"counterparty_version");
    //     let counterparty_channel_id = 1;
    //     let proof_init = b"proof";
    //     let proof_height = 1;

    //     // Call channel_open_try
    //     channel_open_try(
    //         &mut ibc_store,
    //         port_id,
    //         connection_id,
    //         counterparty_channel_id,
    //         counterparty_port_id,
    //         version,
    //         counterparty_version,
    //         proof_init,
    //         proof_height
    //     );

    //     // Call channel_open_ack to move to the open state
    //     let channel_id = 1;
    //     // let proof_try = b"proof";
    //     // let proof_height_ack = 1;
    //     // channel_open_ack(
    //     //     &mut ibc_store,
    //     //     port_id,
    //     //     channel_id,
    //     //     version,
    //     //     counterparty_channel_id,
    //     //     proof_try,
    //     //     proof_height_ack
    //     // );

    //     // Prepare inputs for channel_open_confirm
    //     let proof_ack = b"proof";
    //     let proof_height_confirm = 1;

    //     // Call channel_open_confirm
    //     channel_open_confirm(
    //         &mut ibc_store,
    //         port_id,
    //         channel_id,
    //         proof_ack,
    //         proof_height_confirm
    //     );

    //     // Verify final state is OPEN
    //     let channel = ibc_store.channels.borrow(channel_id);
    //     assert!(channel::state(channel) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

    //     test_scenario::return_shared(ibc_store);
    //     test_case.end();
    // }


    // #[test]
    // fun test_send_packet() {
    //     let mut ctx = tx_context::dummy();

    //     let mut test_case = test_scenario::begin(@0x0);
    //     init(test_case.ctx());
    //     test_case.next_tx(@0x0);
    //     let mut ibc_store = test_case.take_shared<IBCStore>();

    //     // Set up a channel in the OPEN state
    //     let channel_id = ibc_store.generate_channel_identifier();
    //     let mut channel = channel::default();
    //     channel::set_state(&mut channel, CHAN_STATE_OPEN);
    //     channel::set_counterparty_channel_id(&mut channel, 1); // Assume the counterparty channel ID is 1
    //     ibc_store.channels.add(channel_id, channel);

    //     // Set up the next sequence number for the send packet
    //     let next_sequence_key = commitment::next_sequence_send_commitment_key(channel_id);
    //     add_or_update_table<vector<u8>, vector<u8>>(
    //         &mut ibc_store.commitments,
    //         next_sequence_key,
    //         bcs::to_bytes(&1u64)
    //     );

    //     // Define packet data
    //     let timeout_height = 100;
    //     let timeout_timestamp = 1_000_000_000;
    //     let data = b"Hello, IBC!";

    //     // Call send_packet
    //     send_packet(
    //         &mut ibc_store,
    //         // @0x0, // assuming @0x0 as source port
    //         channel_id,
    //         timeout_height,
    //         timeout_timestamp,
    //         data
    //     );
    //     // Verify packet commitment
    //     let commitment_key = commitment::batch_packets_commitment_key(
    //         channel_id,
    //         commitment::commit_packet(
    //             &packet::new(
    //                 1,
    //                 channel_id,
    //                 data,
    //                 timeout_height,
    //                 timeout_timestamp,
    //             )
    //         )
    //     );

    //     let commitment = ibc_store.commitments.borrow(commitment_key);
    //     assert!(commitment == COMMITMENT_MAGIC, 1);


    //     // Clean up
    //     test_scenario::return_shared(ibc_store);
    //     test_case.end();
    // }

    // // #[test]
    // // fun test_process_receive() {
    // //     let mut ctx = tx_context::dummy();
    // //     let mut test_case = test_scenario::begin(@0x0);
    // //     init(test_case.ctx());
    // //     test_case.next_tx(@0x0);
    // //     let mut ibc_store = test_case.take_shared<IBCStore>();

    // //     // Set up a valid connection and channel in the OPEN state
    // //     let connection_id = mock_valid_connection(&mut ibc_store, &mut ctx);

    // //     let channel_id = ibc_store.generate_channel_identifier();
    // //     let mut channel = channel::default();
    // //     channel::set_state(&mut channel, CHAN_STATE_OPEN);
    // //     channel::set_connection_id(&mut channel, connection_id);
    // //     channel::set_counterparty_channel_id(&mut channel, 1); // Set counterparty channel ID
    // //     ibc_store.channels.add(channel_id, channel);

    // //     // Prepare packet data
    // //     let sequence = 1;
    // //     let packet = packet::new(
    // //         sequence,
    // //         channel_id,
    // //         b"Test data",
    // //         100, // Timeout height
    // //         1_000_000_000 // Timeout timestamp
    // //     );
    // //     let packets = vector::singleton(packet);

    // //     // Call process_receive
    // //     let proof = b"valid_proof"; // Mock proof
    // //     let proof_height = 100;
    // //     let acknowledgement = b"";

    // //     let commitment_key =
    // //             commitment::batch_receipts_commitment_key(
    // //                 packet::destination_channel_id(&packet),
    // //                 commitment::commit_packet(&packet)
    // //             );
                
    // //     let mut clock = clock::create_for_testing(&mut ctx);
    // //     clock.set_for_testing(99);

    // //     process_receive(
    // //         &mut ibc_store,
    // //         &clock,
    // //         packets,
    // //         proof_height,
    // //         proof,
    // //         false, // No intent
    // //         acknowledgement
    // //     );

    // //     assert!(ibc_store.commitments.contains(commitment_key), E_PACKET_NOT_RECEIVED);
    // //     let commitment = ibc_store.commitments.borrow(commitment_key);
    // //     assert!(commitment == COMMITMENT_MAGIC, 0);


    // //     clock::destroy_for_testing(clock);
    // //     test_scenario::return_shared(ibc_store);
    // //     test_case.end();
    // // }
    public struct IbcAppWitness has drop {}
    #[test]
    fun validate_port_bro() {
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000022222::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");

        validate_port(
            port,
            IbcAppWitness {}
        );
    }

    #[test]
    fun test_proof_bro() {
        let proof = vector[66,3,157,163,60,194,185,78,153,137,118,116,88,181,5,164,168,56,94,112,42,200,1,32,109,2,153,49,174,114,139,182,11,247,0,230,95,246,128,140,252,192,16,252,63,105,55,31,168,195,28,213,244,195,214,49,113,37,39,71,124,87,139,52,8,77,241,32,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,0,2,226,34,8,38,2,4,226,34,32,6,244,144,16,166,62,131,72,63,110,140,80,68,56,17,205,5,81,152,192,156,19,38,64,129,239,188,82,220,80,10,172,32,0,38,4,6,226,34,32,26,34,239,233,254,192,95,202,132,192,62,204,91,211,180,148,89,83,213,224,39,36,1,16,9,99,214,209,129,142,123,201,32,0,5,6,10,226,34,32,33,32,246,158,154,124,28,103,4,32,35,158,173,157,123,111,139,167,158,232,170,134,126,91,217,78,206,212,180,160,165,6,233,246,5,8,16,226,34,32,33,32,165,28,108,126,162,136,226,141,54,82,253,35,183,63,253,107,66,25,135,8,233,114,49,195,108,180,148,89,236,223,220,158,5,10,28,226,34,32,33,32,232,158,110,200,166,214,0,77,69,210,84,233,29,217,85,205,3,125,200,86,247,61,96,62,99,195,59,114,102,250,11,150,5,12,46,226,34,32,33,32,69,127,104,73,59,175,120,242,44,215,154,107,16,118,172,161,73,26,58,205,103,39,58,112,27,170,101,147,13,186,194,148,38,14,100,226,34,32,99,64,196,88,239,62,120,58,187,98,35,91,198,226,113,62,117,175,113,122,143,18,82,250,3,216,207,84,28,35,37,130,32,0,6,16,154,1,226,34,32,33,32,94,9,159,210,244,168,3,254,192,41,45,181,221,82,164,143,172,53,0,28,201,94,117,231,43,60,203,197,10,148,138,249,4,119,97,115,109,32,57,3,118,217,61,244,79,2,241,168,95,102,130,112,144,213,213,64,116,170,6,46,48,165,77,47,72,22,239,112,128,218,1,0,3,33,1,2,163,112,214,244,208,16,106,162,251,190,254,83,88,186,2,81,154,49,248,130,202,227,178,59,147,127,0,47,194,183,150,0,33,1,203,227,131,25,2,252,250,94,200,191,229,120,254,131,31,121,86,45,230,213,180,224,159,117,29,73,134,123,108,85,158,200,0,33,1,255,201,233,159,104,240,196,120,96,190,81,147,182,148,20,24,89,245,220,63,245,230,130,179,84,241,234,117,202,19,91,148,0];

        let proof = ibc::ics23::decode_membership_proof(proof); 
    
        std::debug::print(&proof);
    }
}
