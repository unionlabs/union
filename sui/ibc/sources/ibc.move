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
module ibc::ibc {
    use std::string::{String, utf8};
    use sui::table::{Self, Table};
    use sui::hash::keccak256;
    use sui::clock;
    use sui::clock::Clock;
    use ibc::packet::{Self, Packet};
    use ibc::connection_end::{Self, ConnectionEnd};
    use ibc::channel::{Self, Channel}; 
    use ibc::light_client::{Self, LightClientManager};
    use ibc::commitment;
    use ibc::events;
    use ibc::state;

    const VERSION: u32 = 1;

    const COMMITMENT_MAGIC: vector<u8>     = x"0100000000000000000000000000000000000000000000000000000000000000";
    const COMMITMENT_MAGIC_ACK: vector<u8> = x"0200000000000000000000000000000000000000000000000000000000000000";

    const CHAN_STATE_INIT: u8 = 1;
    const CHAN_STATE_TRYOPEN: u8 = 2;
    const CHAN_STATE_OPEN: u8 = 3;

    const CONN_STATE_INIT: u8 = 1;
    const CONN_STATE_TRYOPEN: u8 = 2;
    const CONN_STATE_OPEN: u8 = 3;

    const E_VERSION_MISMATCH: u64 = 1001;
    const E_CLIENT_NOT_FOUND: u64 = 1002;
    const E_INVALID_CONNECTION_STATE: u64 = 1008;
    const E_INVALID_CHANNEL_STATE: u64 = 1016;
    const E_LATEST_TIMESTAMP_NOT_FOUND: u64 = 1019;
    const E_UNAUTHORIZED: u64 = 1020;
    const E_TIMESTAMP_TIMEOUT: u64 = 1023;
    const E_PACKET_ALREADY_RECEIVED: u64 = 1025;
    const E_ACKNOWLEDGEMENT_IS_EMPTY: u64 = 1028;
    const E_PACKET_COMMITMENT_NOT_FOUND: u64 = 1032;
    const E_TIMESTAMP_TIMEOUT_NOT_REACHED: u64 = 1034;
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
    const E_ALREADY_RECEIVED: u64 = 1063;
    const E_PACKET_HAVE_NOT_TIMED_OUT: u64 = 1064;
    const E_COMMITMENT_NOT_FOUND: u64 = 1065;

    public struct IBCStore has key {
        id: UID,
        version: u32,
        client_mgr: LightClientManager,
        connections: Table<u32, ConnectionEnd>,
        channels: Table<u32, Channel>,
        channel_to_port: Table<u32, String>,
        next_client_sequence: u32,
        next_channel_sequence: u32,
        next_connection_sequence: u32,
    }

    fun init(ctx: &mut TxContext) {
        events::emit_initiated();
        transfer::share_object(IBCStore {
            id: object::new(ctx),
            version: VERSION,
            connections: table::new(ctx),
            channels: table::new(ctx),
            client_mgr: light_client::new(ctx, false),
            channel_to_port: table::new(ctx),
            next_client_sequence: 1,
            next_channel_sequence: 1,
            next_connection_sequence: 1
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
        ibc_store.assert_version();

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
            events::emit_create_lens_client(
                    lens_client_event.client_id(),
                    lens_client_event.l2_chain_id(),
                    lens_client_event.l1_client_id(),
                    lens_client_event.l2_client_id()
            );
        };

        assert!(ibc_store.client_mgr.status(client_id) == 0, E_CLIENT_NOT_ACTIVE);

        state::commit(
            &mut ibc_store.id,
            commitment::client_state_commitment_key(client_id),
            client_state_bytes
        );

        let latest_height = ibc_store.client_mgr.latest_height(client_id);
        state::commit(
            &mut ibc_store.id,
            commitment::consensus_state_commitment_key(client_id, latest_height),
            consensus_state_bytes
        );

        events::emit_create_client(
            client_id,
            client_type,
            counterparty_chain_id
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
        ibc_store.assert_version();

        assert!(ibc_store.client_mgr.status(client_id) == 0, E_CLIENT_NOT_ACTIVE);

        // Update the client and consensus states using the client message
        let (client_state, consensus_state, height) =
            ibc_store.client_mgr.update_client(client_id, clock, client_message, relayer);

        // Update the client state commitment
        *state::borrow_commitment_mut(&mut ibc_store.id, commitment::client_state_commitment_key(client_id)) = client_state;

        // Update the consensus state commitment
        ibc_store.add_or_update_commitment(
            commitment::consensus_state_commitment_key(client_id, height),
            keccak256(&consensus_state)
        );

        events::emit_update_client(
            client_id,
            height
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
        ibc_store.assert_version();

        assert!(ibc_store.client_mgr.status(client_id) == 0, E_CLIENT_NOT_ACTIVE);

        ibc_store.client_mgr.misbehaviour(client_id, misbehaviour, relayer);

        events::emit_misbehaviour(
            client_id,
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
        ibc_store.assert_version();

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

        ibc::events::emit_connection_open_init(
                connection_id,
                client_id,
                counterparty_client_id,
            
        );
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
        ibc_store.assert_version();

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

        events::emit_connection_open_try(
            connection_id,
            client_id,
            counterparty_client_id,
            counterparty_connection_id,
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
        ibc_store.assert_version();

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

        events::emit_connection_open_ack(
            connection_id,
            connection.client_id(),
            connection.counterparty_client_id(),
            connection.counterparty_connection_id()
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
        ibc_store.assert_version();

        let connection = ibc_store.connections.borrow_mut(connection_id);
        assert!(
            connection.state() == CONN_STATE_TRYOPEN,
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
        events::emit_connection_open_confirm(
            connection_id,
            connection.client_id(),
            connection.counterparty_client_id(),
            counterparty_connection_id
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
        ibc_store.assert_version();

        // Make sure that the `port_id` confirms the witness.
        validate_port(port_id, witness);

        // Ensure the connection exists and is in the OPEN state
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection.state() == CONN_STATE_OPEN,
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

        events::emit_channel_open_init(
            port_id,
            channel_id,
            counterparty_port_id,
            connection_id,
            version
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
        ibc_store.assert_version();

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
            *port_id.as_bytes(),
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

        events::emit_channel_open_try(
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
            counterparty_version
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
        channel_id: u32,
        counterparty_version: String,
        counterparty_channel_id: u32,
        proof_try: vector<u8>,
        proof_height: u64,
        witness: T,
    ) {
        ibc_store.assert_version();

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
                *port_id.as_bytes(),
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
        events::emit_channel_open_ack(
            port_id,
            channel_id,
            *channel.counterparty_port_id(),
            counterparty_channel_id,
            connection_id
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
        ibc_store.assert_version();

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
                *port_id.as_bytes(),
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

        events::emit_channel_open_confirm(
            port_id,
            channel_id,
            *channel.counterparty_port_id(),
            channel.counterparty_channel_id(),
            channel.connection_id()
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
        ibc_store.assert_version();

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

        assert!(!state::has_commitment(&ibc_store.id, commitment_key), E_PACKET_ALREADY_SENT);

        state::commit(&mut ibc_store.id, commitment_key, COMMITMENT_MAGIC);

        // This is very important for the relayers to be able to get the exact transaction from the `packet_hash`.
        // They will later use this to get the full packet.
        state::add_commitment_to_digest(&mut ibc_store.id, packet_hash, *ctx.digest());

        events::emit_packet_send(
            source_channel,
            packet_hash,
            packet
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
        ibc_store.assert_version();

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
        ibc_store.assert_version();

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

    fun add_or_update_commitment(ibc_store: &mut IBCStore, key: vector<u8>, value: vector<u8>) {
        if (state::has_commitment(&ibc_store.id, key)) {
            *state::borrow_commitment_mut(&mut ibc_store.id, key) = value;
        } else {
            state::commit(&mut ibc_store.id, key, value);
        };
    }

    fun commit_connection(ibc_store: &mut IBCStore, connection_id: u32, connection: ConnectionEnd) {
        state::commit(  
            &mut ibc_store.id,
            commitment::connection_commitment_key(connection_id),
            keccak256(&connection.encode())
        );
    }

    fun commit_channel(ibc_store: &mut IBCStore, channel_id: u32, channel: Channel) {
        state::commit(  
            &mut ibc_store.id,
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
            keccak256(&counterparty_connection.encode())
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

        let mut a = sui::address::from_ascii_bytes(parts[0].as_bytes()).to_ascii_string();
        a.append(std::ascii::string(b"::"));
        a.append(parts[1].to_ascii());

        a
    }

    fun validate_port<T: drop>(
        port_id: String,
        _: T,
    ) {       
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
        connection.counterparty_connection_id()
    }

    fun set_packet_receive(ibc_store: &mut IBCStore, commitment_key: vector<u8>): bool {
        if (state::has_commitment(&ibc_store.id, commitment_key)) {
            true
        } else {
            state::commit(&mut ibc_store.id, commitment_key, COMMITMENT_MAGIC);
            false
        }
    }

    fun set_packet_acknowledged(ibc_store: &mut IBCStore, commitment_key: vector<u8>) {
        assert!(state::has_commitment(&ibc_store.id, commitment_key), E_PACKET_COMMITMENT_NOT_FOUND);

        let commitment = state::borrow_commitment_mut(&mut ibc_store.id, commitment_key);
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


            if(set_packet_receive(ibc_store, commitment_key)) {
                // Normally this is not an error and results in noop in the traditional impls where
                // the app is called by the ibc. But since it is the other way here, we have to abort
                // to prevent double processing in the app side. 
                abort E_ALREADY_RECEIVED;
            } else {
                let maker_msg = maker_msgs[i];
                if (intent) {
                    events::emit_intent_packet_recv(
                        destination_channel,
                        packet_hash,
                        maker,
                        maker_msg
                    );
                } else {
                    events::emit_packet_recv(
                        destination_channel,
                        packet_hash,
                        maker,
                        maker_msg
                    );
                };

                let acknowledgement = acknowledgements[i];
                if (!acknowledgement.is_empty()) {
                    inner_write_acknowledgement(ibc_store, commitment_key, acknowledgement);
                    events::emit_write_ack(
                        destination_channel,
                        packet_hash,
                        acknowledgement
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
        ibc_store.assert_version();

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

        events::emit_write_ack(
            packet.destination_channel_id(),
            packet_hash,
            acknowledgement,
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
        ibc_store.assert_version();

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
            ibc_store.set_packet_acknowledged(commitment_key);
            
            events::emit_packet_ack(
                source_channel,
                packet_hash,
                acknowledgements[i],
                relayer
            );

            i = i + 1;
        }
    }

    public fun commit_packet_timeout(
        ibc_store: &mut IBCStore,
        clock: &clock::Clock,
        packet: Packet,
        proof: vector<u8>,
        proof_height: u64,
        ctx: &mut TxContext,
    ) {
        ibc_store.assert_version();

        let current_timestamp = clock::timestamp_ms(clock) * 1_000_000; 
        assert!(
            current_timestamp >= packet.timeout_timestamp(),
            E_PACKET_HAVE_NOT_TIMED_OUT
        );

        let packet_hash = commitment::commit_packet(&packet);

        assert!(
            !state::has_commitment(&ibc_store.id, commitment::batch_receipts_commitment_key(packet_hash)),
            E_PACKET_ALREADY_RECEIVED
        );

        let channel = ibc_store.channels.borrow(packet.destination_channel_id());
        assert!(channel.state() == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        let connection_id = channel.connection_id();

        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection.state() == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );
        let client_id = connection.client_id();

        // make sure that the packet is sent
        let err =
            ibc_store.client_mgr.verify_membership(
                client_id,
                proof_height,
                proof,
                commitment::batch_packets_commitment_key(
                    packet_hash
                ),
                COMMITMENT_MAGIC
            );

        if (err != 0) {
            abort err
        };

        let commitment_key = commitment::packet_timeout_commitment_key(packet_hash);

        state::add_commitment_to_digest(
            &mut ibc_store.id,
            commitment_key,
            *ctx.digest()
        );

        state::commit(
            &mut ibc_store.id,
            commitment_key,
            COMMITMENT_MAGIC
        );
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

    public fun get_commitment(ibc_store: &IBCStore, commitment_key: vector<u8>): vector<u8> {
        if (!state::has_commitment(&ibc_store.id, commitment_key)) {
            abort E_COMMITMENT_NOT_FOUND
        };
        *state::borrow_commitment(&ibc_store.id, commitment_key)
    }

    fun inner_write_acknowledgement(
        ibc_store: &mut IBCStore,
        commitment_key: vector<u8>, acknowledgement: vector<u8>
    ) {
        if (!state::has_commitment(&ibc_store.id, commitment_key)) {
            abort E_PACKET_NOT_RECEIVED
        };
        let commitment = state::borrow_commitment_mut(&mut ibc_store.id, commitment_key);
        assert!(
            *commitment == COMMITMENT_MAGIC,
            E_ACK_ALREADY_EXIST
        );

        *commitment = commitment::commit_ack(acknowledgement);
    }

    public fun timeout_packet<T: drop>(
        ibc_store: &mut IBCStore,
        packet: Packet,
        proof: vector<u8>,
        proof_height: u64,
        witness: T,
    ) {
        ibc_store.assert_version();

        let source_channel = packet.source_channel_id();

        let port_id = *ibc_store.channel_to_port.borrow(source_channel);
        validate_port(port_id, witness);

        if(!ibc_store.channels.contains(source_channel)) {
            abort E_CHANNEL_NOT_FOUND
        };
        let channel = ibc_store.channels.borrow(source_channel);
        assert!(channel.state() == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        let connection_id = channel.connection_id();

        if(!ibc_store.connections.contains(connection_id)) {
            abort E_CONNECTION_NOT_FOUND
        };
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(
            connection.state() == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );
        let client_id = connection.client_id();

        if(!ibc_store.client_mgr.exists(client_id)) {
            abort E_CLIENT_NOT_FOUND
        };
        let proof_timestamp =
            ibc_store.client_mgr.get_timestamp_at_height(client_id, proof_height);
        assert!(proof_timestamp != 0, E_LATEST_TIMESTAMP_NOT_FOUND);

        let packet_hash = commitment::commit_packet(&packet);
        let commitment_key = commitment::batch_receipts_commitment_key(packet_hash);

        let err = ibc_store.client_mgr.verify_non_membership(
            client_id, proof_height, proof, commitment_key);

        assert!(err == 0, err);

        if (packet.timeout_timestamp() == 0) {
            abort E_TIMEOUT_MUST_BE_SET
        } else {
            assert!(
                packet.timeout_timestamp() < proof_timestamp,
                E_TIMESTAMP_TIMEOUT_NOT_REACHED
            );
        };

        ibc_store.set_packet_acknowledged(
            commitment::batch_packets_commitment_key(
                packet_hash
            )
        );

        events::emit_timeout_packet(packet_hash);
    }

    fun assert_version(ibc_store: &IBCStore) {
        assert!(ibc_store.version == VERSION, E_VERSION_MISMATCH);
    }

    #[test_only]
    const TEST_LATEST_HEIGHT: u64 = 10_000;

    #[test_only]
    use sui::test_scenario;

    #[test_only]
    use std::string;

    #[test_only]
    public fun init_for_tests(ctx: &mut TxContext) {
        transfer::share_object(IBCStore {
            id: object::new(ctx),
            version: VERSION,
            connections: table::new(ctx),
            channels: table::new(ctx),
            client_mgr: light_client::new(ctx, true),
            channel_to_port: table::new(ctx),
            next_client_sequence: 1,
            next_channel_sequence: 1,
            next_connection_sequence: 1,
        });
    }

    #[test_only]
    fun open_channel_for_tests(t: &mut test_scenario::Scenario) {
        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs1", b"cons1", t.ctx()); // id = 1
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs2", b"cons2", t.ctx()); // id = 2

        t.next_tx(@0x0);
        ibc_store.connection_open_init(2, 1);

        t.next_tx(@0x0);
        ibc_store.connection_open_ack(1, 9, b"p", 1);

        t.next_tx(@0x0);
        let port = string::utf8(
            b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea"
        );
        ibc_store.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});

        t.next_tx(@0x0);
        ibc_store.channel_open_ack(
            1,
            string::utf8(b"v1-cp"),
            1,
            b"p",
            1,
            IbcAppWitness {}
        );
        test_scenario::return_shared(ibc_store);
    }



    #[test]
    fun test_create_client() {
        let mut test_case = test_scenario::begin(@0x0);
        init_for_tests(test_case.ctx());

        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();

        let client_state = b"client-state";
        let consensus_state = b"consensus-state";

        ibc_store.create_client(
            utf8(b"doesntmatter"),
            client_state,
            consensus_state,
            test_case.ctx()
        );

        assert!(
            state::borrow_commitment(
                &ibc_store.id,
                commitment::client_state_commitment_key(1),
            ) == &client_state,
            1
        );

        assert!(
            state::borrow_commitment(
                &ibc_store.id,
                commitment::consensus_state_commitment_key(1, TEST_LATEST_HEIGHT),
            ) == &consensus_state,
            1
        );

        assert!(
            ibc_store.next_client_sequence == 2,
            1
        );

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_connection_open_init_success() {
        let mut test_case = test_scenario::begin(@0x0);
        init_for_tests(test_case.ctx());

        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();

        let client_state = b"client-state";
        let consensus_state = b"consensus-state";

        ibc_store.create_client(
            string::utf8(b"cometbls"),
            client_state,
            consensus_state,
            test_case.ctx()
        );

        test_case.next_tx(@0x0);

        let client_id = 1;
        let counterparty_client_id = 2;

        ibc_store.connection_open_init(client_id, counterparty_client_id);

        let connection_id = 1;
        let connection = ibc_store.connections.borrow(connection_id);
        assert!(connection.state() == CONN_STATE_INIT, E_INVALID_CONNECTION_STATE);
        assert!(connection.client_id() == client_id, E_CONNECTION_NOT_FOUND);
        assert!(connection.counterparty_client_id() == counterparty_client_id, E_CONNECTION_NOT_FOUND);

        let key = commitment::connection_commitment_key(connection_id);
        assert!(state::has_commitment(&ibc_store.id, key), 1);

        assert!(ibc_store.next_connection_sequence == 2, 1);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_connection_open_init_multiple_connections() {
        let mut test_case = test_scenario::begin(@0x0);
        init_for_tests(test_case.ctx());

        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();
        ibc_store.create_client(
            string::utf8(b"cometbls"),
            b"cs",
            b"cons",
            test_case.ctx()
        );

        test_case.next_tx(@0x0);
        ibc_store.connection_open_init(1, /*counterparty_client_id=*/42);

        test_case.next_tx(@0x0);
        ibc_store.connection_open_init(1, /*counterparty_client_id=*/43);


        let c1 = ibc_store.connections.borrow(1);
        let c2 = ibc_store.connections.borrow(2);
        assert!(c1.state() == CONN_STATE_INIT, E_INVALID_CONNECTION_STATE);
        assert!(c2.state() == CONN_STATE_INIT, E_INVALID_CONNECTION_STATE);
        assert!(c1.counterparty_client_id() == 42, 1);
        assert!(c2.counterparty_client_id() == 43, 1);

        let k1 = commitment::connection_commitment_key(1);
        let k2 = commitment::connection_commitment_key(2);
        assert!(state::has_commitment(&ibc_store.id, k1), 1);
        assert!(state::has_commitment(&ibc_store.id, k2), 1);

        assert!(ibc_store.next_connection_sequence == 3, 1);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }
    #[test]
    fun test_connection_open_try_success() {
        let mut test_case = test_scenario::begin(@0x0);
        init_for_tests(test_case.ctx());

        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();
        ibc_store.create_client(
            string::utf8(b"cometbls"),
            b"cs",
            b"cons",
            test_case.ctx()
        );

        test_case.next_tx(@0x0);
        ibc_store.connection_open_try(2, 11, 1, b"p", 1);

        let connection_id = 1;
        let c = ibc_store.connections.borrow(connection_id);
        assert!(c.state() == CONN_STATE_TRYOPEN, E_INVALID_CONNECTION_STATE);
        assert!(c.client_id() == 1, 1);
        assert!(c.counterparty_client_id() == 2, 1);
        assert!(c.counterparty_connection_id() == 11, 1);

        let key = commitment::connection_commitment_key(connection_id);
        assert!(state::has_commitment(&ibc_store.id, key), 1);
        assert!(ibc_store.next_connection_sequence == 2, 1);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_connection_open_ack_success() {
        let mut test_case = test_scenario::begin(@0x0);
        init_for_tests(test_case.ctx());

        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();
        ibc_store.create_client(
            string::utf8(b"cometbls"),
            b"cs",
            b"cons",
            test_case.ctx()
        );

        test_case.next_tx(@0x0);
        ibc_store.connection_open_init(1, 2);

        test_case.next_tx(@0x0);
        ibc_store.connection_open_ack(1, 9, b"p", 1);

        let c = ibc_store.connections.borrow(1);
        assert!(c.state() == CONN_STATE_OPEN, E_INVALID_CONNECTION_STATE);
        assert!(c.counterparty_connection_id() == 9, 1);

        let key = commitment::connection_commitment_key(1);
        assert!(state::has_commitment(&ibc_store.id, key), 1);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_connection_open_confirm_success() {
        let mut test_case = test_scenario::begin(@0x0);
        init_for_tests(test_case.ctx());

        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();
        ibc_store.create_client(
            string::utf8(b"cometbls"),
            b"cs",
            b"cons",
            test_case.ctx()
        );

        test_case.next_tx(@0x0);
        ibc_store.connection_open_try(2, 13, 1, b"p", 1);

        test_case.next_tx(@0x0);
        ibc_store.connection_open_confirm(1, b"p", 1);

        let c = ibc_store.connections.borrow(1);
        assert!(c.state() == CONN_STATE_OPEN, E_INVALID_CONNECTION_STATE);

        let key = commitment::connection_commitment_key(1);
        assert!(state::has_commitment(&ibc_store.id, key), 1);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    
    #[test]
    fun test_channel_open_init_success() {
        let mut test_case = test_scenario::begin(@0x0);
        init_for_tests(test_case.ctx());

        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs", b"cons", test_case.ctx());

        test_case.next_tx(@0x0);
        ibc_store.connection_open_init(1, 2);

        test_case.next_tx(@0x0);
        ibc_store.connection_open_ack(1, 9, b"p", 1);

        test_case.next_tx(@0x0);
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");
        ibc_store.channel_open_init(
            port,
            b"cp-port",
            1,
            string::utf8(b"v1"),
            IbcAppWitness {}
        );

        let ch_id = 1;
        let ch = ibc_store.channels.borrow(ch_id);
        assert!(channel::state(ch) == CHAN_STATE_INIT, E_INVALID_CHANNEL_STATE);
        assert!(*ibc_store.channel_to_port.borrow(ch_id) == string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea"), 1);
        let key = commitment::channel_commitment_key(ch_id);
        assert!(state::has_commitment(&ibc_store.id, key), 1);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_channel_open_try_success() {
        let mut test_case = test_scenario::begin(@0x0);
        init_for_tests(test_case.ctx());

        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs", b"cons", test_case.ctx());

        test_case.next_tx(@0x0);
        ibc_store.connection_open_try(2, 15, 1, b"p", 1);

        test_case.next_tx(@0x0);
        ibc_store.connection_open_confirm(1, b"p", 1);
        
        test_case.next_tx(@0x0);
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");
        ibc_store.channel_open_try(
            port,
            1,
            7,
            b"cp-port",
            string::utf8(b"v1"),
            string::utf8(b"v1-cp"),
            b"p",
            1,
            IbcAppWitness {}
        );

        let ch_id = 1;
        let ch = ibc_store.channels.borrow(ch_id);
        assert!(channel::state(ch) == CHAN_STATE_TRYOPEN, E_INVALID_CHANNEL_STATE);
        assert!(channel::connection_id(ch) == 1, 1);
        assert!(channel::counterparty_channel_id(ch) == 7, 1);
        let key = commitment::channel_commitment_key(ch_id);
        assert!(state::has_commitment(&ibc_store.id, key), 1);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_channel_open_ack_success() {
        let mut test_case = test_scenario::begin(@0x0);
        init_for_tests(test_case.ctx());

        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs", b"cons", test_case.ctx());

        test_case.next_tx(@0x0);
        ibc_store.connection_open_init(1, 2);

        test_case.next_tx(@0x0);
        ibc_store.connection_open_ack(1, 9, b"p", 1);

        test_case.next_tx(@0x0);
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");
        ibc_store.channel_open_init(
            port,
            b"cp-port",
            1,
            string::utf8(b"v1"),
            IbcAppWitness {}
        );

        test_case.next_tx(@0x0);
        ibc_store.channel_open_ack(
            1,
            string::utf8(b"v1-cp"),
            22,
            b"p",
            1,
            IbcAppWitness {}
        );

        let ch = ibc_store.channels.borrow(1);
        assert!(channel::state(ch) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);
        assert!(channel::counterparty_channel_id(ch) == 22, 1);
        let key = commitment::channel_commitment_key(1);
        assert!(state::has_commitment(&ibc_store.id, key), 1);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }

    #[test]
    fun test_channel_open_confirm_success() {
        let mut test_case = test_scenario::begin(@0x0);
        init_for_tests(test_case.ctx());

        test_case.next_tx(@0x0);
        let mut ibc_store = test_case.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs", b"cons", test_case.ctx());

        test_case.next_tx(@0x0);
        ibc_store.connection_open_try(2, 17, 1, b"p", 1);

        test_case.next_tx(@0x0);
        ibc_store.connection_open_confirm(1, b"p", 1);

        test_case.next_tx(@0x0);
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");
        ibc_store.channel_open_try(
            port,
            1,
            5,
            b"cp-port",
            string::utf8(b"v1"),
            string::utf8(b"v1-cp"),
            b"p",
            1,
            IbcAppWitness {}
        );

        test_case.next_tx(@0x0);
        ibc_store.channel_open_confirm(1, b"p", 1, IbcAppWitness {});

        let ch = ibc_store.channels.borrow(1);
        assert!(channel::state(ch) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);
        let key = commitment::channel_commitment_key(1);
        assert!(state::has_commitment(&ibc_store.id, key), 1);

        test_scenario::return_shared(ibc_store);
        test_case.end();
    }   

    #[test]
    fun test_packet_send_success_only() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, /*millis=*/ 1_000);
        clock::share_for_testing(clk0);

        open_channel_for_tests(&mut t);

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        let clk = t.take_shared<Clock>();
        let now_ns = clock::timestamp_ms(&clk) * 1_000_000;
        let timeout_ts = now_ns + 1_000_000_000;

        let pkt = ibc_store.send_packet(
            &clk,
            1,
            0,
            timeout_ts,
            b"hello",
            IbcAppWitness {},
            t.ctx()
        );
        test_scenario::return_shared(clk);

        let pkt_hash = commitment::commit_packet(&pkt);
        let packets_key = commitment::batch_packets_commitment_key(pkt_hash);
        let packets_val = ibc_store.get_commitment(packets_key);
        assert!(packets_val == COMMITMENT_MAGIC, 1);

        test_scenario::return_shared(ibc_store);
        t.end();
    }
    
    #[test]
    fun test_packet_flow_recv_and_ack_success() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        open_channel_for_tests(&mut t);

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();

        t.next_tx(@0x0);
        let clk = t.take_shared<Clock>();
        let now_ns = clock::timestamp_ms(&clk) * 1_000_000;
        let timeout_ts = now_ns + 1_000_000_000;

        let pkt = ibc_store.send_packet(
            &clk,
            1,
            0,
            timeout_ts,
            b"hello",
            IbcAppWitness {},
            t.ctx()
        );
        test_scenario::return_shared(clk);

        let pkt_hash = commitment::commit_packet(&pkt);
        let packets_key = commitment::batch_packets_commitment_key(pkt_hash);
        assert!(ibc_store.get_commitment(packets_key) == COMMITMENT_MAGIC, 1);

        t.next_tx(@0x0);
        let clk2 = t.take_shared<Clock>();
        ibc_store.recv_packet(
            &clk2,
            vector[pkt],
            @0x111,
            vector[b"maker-msg"],
            b"p",
            1,
            vector[b"ack-ok"],
            IbcAppWitness {}
        );
        test_scenario::return_shared(clk2);

        let receipts_key = commitment::batch_receipts_commitment_key(pkt_hash);
        let receipts_val = ibc_store.get_commitment(receipts_key);
        assert!(receipts_val == commitment::commit_ack(b"ack-ok"), 1);

        t.next_tx(@0x0);
        ibc_store.acknowledge_packet(
            vector[pkt],
            vector[b"ack-ok"],
            b"p",
            1,
            @0x222,
            IbcAppWitness {}
        );

        let packets_val_after = ibc_store.get_commitment(packets_key);
        assert!(packets_val_after == COMMITMENT_MAGIC_ACK, 1);

        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_INVALID_CONNECTION_STATE)]
    fun test_connection_open_ack_wrong_state_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs", b"cons", t.ctx());

        t.next_tx(@0x0);
        ibc_store.connection_open_try(2, 11, 1, b"p", 1);

        t.next_tx(@0x0);
        ibc_store.connection_open_ack(1, 99, b"p", 1);

        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_INVALID_CONNECTION_STATE)]
    fun test_connection_open_confirm_wrong_state_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs", b"cons", t.ctx());

        t.next_tx(@0x0);
        ibc_store.connection_open_init(1, 2);

        t.next_tx(@0x0);
        ibc_store.connection_open_confirm(1, b"p", 1);

        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_INVALID_CONNECTION_STATE)]
    fun test_channel_open_init_connection_not_open_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs", b"cons", t.ctx());

        t.next_tx(@0x0);
        ibc_store.connection_open_init(1, 2);

        t.next_tx(@0x0);
        let port = string::utf8(
            b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea"
        );
        ibc_store.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});

        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_INVALID_CONNECTION_STATE)]
    fun test_channel_open_try_connection_not_open_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs", b"cons", t.ctx());

        t.next_tx(@0x0);
        ibc_store.connection_open_init(1, 2);

        t.next_tx(@0x0);
        let port = string::utf8(
            b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea"
        );
        ibc_store.channel_open_try(
            port,
            1,
            7,
            b"cp-port",
            string::utf8(b"v1"),
            string::utf8(b"v1-cp"),
            b"p",
            1,
            IbcAppWitness {}
        );

        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_INVALID_CHANNEL_STATE)]
    fun test_channel_open_ack_wrong_state_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs", b"cons", t.ctx());

        t.next_tx(@0x0);
        ibc_store.connection_open_try(2, 15, 1, b"p", 1);

        t.next_tx(@0x0);
        ibc_store.connection_open_confirm(1, b"p", 1);

        t.next_tx(@0x0);
        let port = string::utf8(
            b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea"
        );
        ibc_store.channel_open_try(
            port,
            1,
            7,
            b"cp-port",
            string::utf8(b"v1"),
            string::utf8(b"v1-cp"),
            b"p",
            1,
            IbcAppWitness {}
        );

        t.next_tx(@0x0);
        ibc_store.channel_open_ack(
            1,
            string::utf8(b"v1-cp"),
            22,
            b"p",
            1,
            IbcAppWitness {}
        );

        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_INVALID_CHANNEL_STATE)]
    fun test_channel_open_confirm_wrong_state_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs", b"cons", t.ctx());

        t.next_tx(@0x0);
        ibc_store.connection_open_init(1, 2);

        t.next_tx(@0x0);
        ibc_store.connection_open_ack(1, 9, b"p", 1);

        t.next_tx(@0x0);
        let port = string::utf8(
            b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea"
        );
        ibc_store.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});

        t.next_tx(@0x0);
        ibc_store.channel_open_confirm(1, b"p", 1, IbcAppWitness {});

        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_TIMEOUT_HEIGHT_NOT_SUPPORTED)]
    fun test_send_packet_timeout_height_unsupported_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs1", b"cons1", t.ctx());
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs2", b"cons2", t.ctx());

        t.next_tx(@0x0);
        ibc_store.connection_open_init(2, 1);

        t.next_tx(@0x0);
        ibc_store.connection_open_ack(1, 9, b"p", 1);

        t.next_tx(@0x0);
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");
        ibc_store.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});

        t.next_tx(@0x0);
        ibc_store.channel_open_ack(1, string::utf8(b"v1-cp"), 1, b"p", 1, IbcAppWitness {});

        t.next_tx(@0x0);
        let clk = t.take_shared<Clock>();
        let now_ns = clock::timestamp_ms(&clk) * 1_000_000;

        ibc_store.send_packet(&clk, 1, 1, now_ns + 1_000_000_000, b"data", IbcAppWitness {}, t.ctx());
        test_scenario::return_shared(clk);
        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_TIMESTAMP_TIMEOUT)]
    fun test_send_packet_timestamp_in_past_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs1", b"cons1", t.ctx());
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs2", b"cons2", t.ctx());

        t.next_tx(@0x0);
        ibc_store.connection_open_init(2, 1);

        t.next_tx(@0x0);
        ibc_store.connection_open_ack(1, 9, b"p", 1);

        t.next_tx(@0x0);
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");
        ibc_store.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});

        t.next_tx(@0x0);
        ibc_store.channel_open_ack(1, string::utf8(b"v1-cp"), 1, b"p", 1, IbcAppWitness {});

        t.next_tx(@0x0);
        let clk = t.take_shared<Clock>();
        let now_ns = clock::timestamp_ms(&clk) * 1_000_000;

        ibc_store.send_packet(&clk, 1, 0, now_ns - 1, b"data", IbcAppWitness {}, t.ctx());
        test_scenario::return_shared(clk);
        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_TIMESTAMP_TIMEOUT)]
    fun test_recv_packet_timestamp_timeout_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs1", b"cons1", t.ctx());
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs2", b"cons2", t.ctx());

        t.next_tx(@0x0);
        ibc_store.connection_open_init(2, 1);

        t.next_tx(@0x0);
        ibc_store.connection_open_ack(1, 9, b"p", 1);

        t.next_tx(@0x0);
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");
        ibc_store.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});

        t.next_tx(@0x0);
        ibc_store.channel_open_ack(1, string::utf8(b"v1-cp"), 1, b"p", 1, IbcAppWitness {});

        t.next_tx(@0x0);
        let clk = t.take_shared<Clock>();
        let now_ns = clock::timestamp_ms(&clk) * 1_000_000;
        let pkt = packet::new(1, 1, b"x", 0, now_ns - 1);
        ibc_store.recv_packet(&clk, vector[pkt], @0x111, vector[b"m"], b"p", 1, vector[b"ack"], IbcAppWitness {});
        test_scenario::return_shared(clk);
        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_ALREADY_RECEIVED)]
    fun test_recv_packet_already_received_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs1", b"cons1", t.ctx());
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs2", b"cons2", t.ctx());

        t.next_tx(@0x0);
        ibc_store.connection_open_init(2, 1);

        t.next_tx(@0x0);
        ibc_store.connection_open_ack(1, 9, b"p", 1);

        t.next_tx(@0x0);
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");
        ibc_store.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});

        t.next_tx(@0x0);
        ibc_store.channel_open_ack(1, string::utf8(b"v1-cp"), 1, b"p", 1, IbcAppWitness {});

        t.next_tx(@0x0);
        let clk = t.take_shared<Clock>();
        let now_ns = clock::timestamp_ms(&clk) * 1_000_000;
        let pkt = packet::new(1, 1, b"x", 0, now_ns + 1_000_000_000);
        ibc_store.recv_packet(&clk, vector[pkt], @0x111, vector[b"m"], b"p", 1, vector[b"a"], IbcAppWitness {});
        ibc_store.recv_packet(&clk, vector[pkt], @0x111, vector[b"m"], b"p", 1, vector[b"a"], IbcAppWitness {});
        test_scenario::return_shared(clk);
        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_ACK_LEN_MISMATCH)]
    fun test_recv_packet_ack_len_mismatch_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs1", b"cons1", t.ctx());
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs2", b"cons2", t.ctx());

        t.next_tx(@0x0);
        ibc_store.connection_open_init(2, 1);

        t.next_tx(@0x0);
        ibc_store.connection_open_ack(1, 9, b"p", 1);

        t.next_tx(@0x0);
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");
        ibc_store.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});

        t.next_tx(@0x0);
        ibc_store.channel_open_ack(1, string::utf8(b"v1-cp"), 1, b"p", 1, IbcAppWitness {});

        t.next_tx(@0x0);
        let clk = t.take_shared<Clock>();
        let now_ns = clock::timestamp_ms(&clk) * 1_000_000;
        let pkt = packet::new(1, 1, b"x", 0, now_ns + 1_000_000_000);
        ibc_store.recv_packet(&clk, vector[pkt], @0x111, vector[b"m"], b"p", 1, vector[], IbcAppWitness {});
        test_scenario::return_shared(clk);
        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_MAKER_MSG_LEN_MISMATCH)]
    fun test_recv_packet_maker_msg_len_mismatch_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs1", b"cons1", t.ctx());
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs2", b"cons2", t.ctx());

        t.next_tx(@0x0);
        ibc_store.connection_open_init(2, 1);

        t.next_tx(@0x0);
        ibc_store.connection_open_ack(1, 9, b"p", 1);

        t.next_tx(@0x0);
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");
        ibc_store.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});

        t.next_tx(@0x0);
        ibc_store.channel_open_ack(1, string::utf8(b"v1-cp"), 1, b"p", 1, IbcAppWitness {});

        t.next_tx(@0x0);
        let clk = t.take_shared<Clock>();
        let now_ns = clock::timestamp_ms(&clk) * 1_000_000;
        let pkt = packet::new(1, 1, b"x", 0, now_ns + 1_000_000_000);
        ibc_store.recv_packet(&clk, vector[pkt], @0x111, vector[], b"p", 1, vector[b"a"], IbcAppWitness {});
        test_scenario::return_shared(clk);
        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_PACKET_COMMITMENT_NOT_FOUND)]
    fun test_acknowledge_packet_no_commitment_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs1", b"cons1", t.ctx());
        ibc_store.create_client(string::utf8(b"cometbls"), b"cs2", b"cons2", t.ctx());

        t.next_tx(@0x0);
        ibc_store.connection_open_init(2, 1);

        t.next_tx(@0x0);
        ibc_store.connection_open_ack(1, 9, b"p", 1);

        t.next_tx(@0x0);
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");
        ibc_store.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});

        t.next_tx(@0x0);
        ibc_store.channel_open_ack(1, string::utf8(b"v1-cp"), 1, b"p", 1, IbcAppWitness {});

        t.next_tx(@0x0);
        let pkt = packet::new(1, 1, b"x", 0, 999999999999);
        ibc_store.acknowledge_packet(vector[pkt], vector[b"ack"], b"p", 1, @0x222, IbcAppWitness {});
        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    fun test_update_client_and_misbehaviour() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut ibc = t.take_shared<IBCStore>();
        ibc.create_client(string::utf8(b"cometbls"), b"cs0", b"cons0", t.ctx());

        t.next_tx(@0x0);
        let mut clk = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk, /*millis=*/ 1_000);
        clock::share_for_testing(clk);

        t.next_tx(@0x0);
        // Succeeds and updates commitments
        let clk2 = t.take_shared<Clock>();
        ibc.update_client(&clk2, /*client_id=*/1, b"msg", @0xAAA);
        
        test_scenario::return_shared(clk2);
        assert!(state::has_commitment(&ibc.id, commitment::client_state_commitment_key(1)), 1);

        t.next_tx(@0x0);
        // Freeze the client
        ibc.misbehaviour(1, b"evil", @0xAAA);

        t.next_tx(@0x0);
        // Any op that checks status==0 should now fail
        // e.g. connection_open_init(1, ..)
        ibc.connection_open_init(1, 2);

        test_scenario::return_shared(ibc);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_PACKET_HAVE_NOT_TIMED_OUT)]
    fun test_commit_packet_timeout_early_fail() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, /*millis=*/ 1_000);
        clock::share_for_testing(clk0);

        t.next_tx(@0x0);
        let mut ibc = t.take_shared<IBCStore>();
        ibc.create_client(string::utf8(b"cometbls"), b"cs1", b"cons1", t.ctx()); 
        ibc.create_client(string::utf8(b"cometbls"), b"cs2", b"cons2", t.ctx());
        t.next_tx(@0x0);
        ibc.connection_open_init(2, 1);
        t.next_tx(@0x0);
        ibc.connection_open_ack(1, 9, b"p", 1);

        t.next_tx(@0x0);
        let port = string::utf8(
            b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea"
        );
        ibc.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});
        t.next_tx(@0x0);
        ibc.channel_open_ack(
            1,
            string::utf8(b"v1-cp"),
            1,
            b"p",
            1,
            IbcAppWitness {}
        );

        t.next_tx(@0x0);
        let clk = t.take_shared<Clock>();
            let now_ns = clock::timestamp_ms(&clk) * 1_000_000;
            let timeout_ns = now_ns + 2_000_000; 
        let pkt = ibc.send_packet(&clk, 1, 0, timeout_ns, b"d", IbcAppWitness {}, t.ctx());
        test_scenario::return_shared(clk);

        t.next_tx(@0x0);
        let clk_early = t.take_shared<Clock>(); 
        ibc.commit_packet_timeout(&clk_early, pkt, b"p",  1, t.ctx());

        test_scenario::return_shared(clk_early);
        test_scenario::return_shared(ibc);
        t.end();
    }

    #[test]
    fun test_commit_packet_timeout_success() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        t.next_tx(@0x0);
        let mut ibc = t.take_shared<IBCStore>();
        ibc.create_client(string::utf8(b"cometbls"), b"cs1", b"cons1", t.ctx());
        ibc.create_client(string::utf8(b"cometbls"), b"cs2", b"cons2", t.ctx());
        t.next_tx(@0x0);
        ibc.connection_open_init(2, 1);
        t.next_tx(@0x0);
        ibc.connection_open_ack(1, 9, b"p", 1);

        t.next_tx(@0x0);
        let port = string::utf8(
            b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea"
        );
        ibc.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});
        t.next_tx(@0x0);
        ibc.channel_open_ack(
            1,
            string::utf8(b"v1-cp"),
            1,
            b"p",
            1,
            IbcAppWitness {}
        );

        t.next_tx(@0x0);
        let clk = t.take_shared<Clock>();
        let now_ns = clock::timestamp_ms(&clk) * 1_000_000;
        let timeout_ns = now_ns + 2_000_000;
        let pkt = ibc.send_packet(&clk, 1, 0, timeout_ns, b"d", IbcAppWitness {}, t.ctx());
        test_scenario::return_shared(clk);

        t.next_tx(@0x0);

            
        let mut clk_late =  t.take_shared<Clock>();
        clock::increment_for_testing(&mut clk_late, 10_000);

        ibc.commit_packet_timeout(&clk_late, pkt, b"p", 1, t.ctx());

        let pkt_hash = commitment::commit_packet(&pkt);
        let timeout_key = commitment::packet_timeout_commitment_key(pkt_hash);
        let v = ibc.get_commitment(timeout_key);
        assert!(v == COMMITMENT_MAGIC, 1);

        test_scenario::return_shared(ibc);
        test_scenario::return_shared(clk_late);
        t.end();
    }


    #[test]
    fun test_recv_intent_packet_and_ack_success() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        
        open_channel_for_tests(&mut t);

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();

        t.next_tx(@0x0);
        let clk = t.take_shared<Clock>();
        let now_ns = clock::timestamp_ms(&clk) * 1_000_000;
        let timeout_ts = now_ns + 1_000_000_000;

        let pkt = ibc_store.send_packet(
            &clk,
            1,
            0,
            timeout_ts,
            b"hi-intent",
            IbcAppWitness {},
            t.ctx()
        );
        test_scenario::return_shared(clk);

        let pkt_hash = commitment::commit_packet(&pkt);

        t.next_tx(@0x0);
        let clk2 = t.take_shared<Clock>();
        ibc_store.recv_intent_packet(
            &clk2,
            vector[pkt],
            @0x111,
            vector[b"maker-i"],
            vector[b"ack-i"],
            IbcAppWitness {}
        );
        test_scenario::return_shared(clk2);

        let receipts_key = commitment::batch_receipts_commitment_key(pkt_hash);
        let receipts_val = ibc_store.get_commitment(receipts_key);
        assert!(receipts_val == commitment::commit_ack(b"ack-i"), 1);

        test_scenario::return_shared(ibc_store);
        t.end();
    }

    #[test]
    fun test_write_ack_after_recv_without_ack_success() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        
        open_channel_for_tests(&mut t);

        t.next_tx(@0x0);
        let mut ibc_store = t.take_shared<IBCStore>();

        t.next_tx(@0x0);
        let clk = t.take_shared<Clock>();
        let now_ns = clock::timestamp_ms(&clk) * 1_000_000;
        let timeout_ts = now_ns + 1_000_000_000;

        let pkt = ibc_store.send_packet(
            &clk,
            1,
            0,
            timeout_ts,
            b"later-ack",
            IbcAppWitness {},
            t.ctx()
        );
        test_scenario::return_shared(clk);

        let pkt_hash = commitment::commit_packet(&pkt);

        t.next_tx(@0x0);
        let clk2 = t.take_shared<Clock>();
        ibc_store.recv_packet(
            &clk2,
            vector[pkt],
            @0xAB,
            vector[b"maker-l"],
            b"p",
            1,
            vector[b""],
            IbcAppWitness {}
        );
        test_scenario::return_shared(clk2);

        let receipts_key = commitment::batch_receipts_commitment_key(pkt_hash);
        let receipts_val0 = ibc_store.get_commitment(receipts_key);
        assert!(receipts_val0 == COMMITMENT_MAGIC, 1);

        t.next_tx(@0x0);
        ibc_store.write_acknowledgement(
            pkt,
            b"ack-later",
            IbcAppWitness {}
        );

        let receipts_val = ibc_store.get_commitment(receipts_key);
        assert!(receipts_val == commitment::commit_ack(b"ack-later"), 1);

        test_scenario::return_shared(ibc_store);
        t.end();
    }


    #[test]
    fun test_timeout_packet_success_and_sets_acknowledged() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        t.next_tx(@0x0);
        let mut ibc = t.take_shared<IBCStore>();
        ibc.create_client(string::utf8(b"cometbls"), b"cs1", b"cons1", t.ctx());
        ibc.create_client(string::utf8(b"cometbls"), b"cs2", b"cons2", t.ctx());
        t.next_tx(@0x0);
        ibc.connection_open_init(2, 1);
        t.next_tx(@0x0);
        ibc.connection_open_ack(1, 9, b"p", 1);
        t.next_tx(@0x0);
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");
        ibc.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});
        t.next_tx(@0x0);
        ibc.channel_open_ack(1, string::utf8(b"v1-cp"), 1, b"p", 1, IbcAppWitness {});

        t.next_tx(@0x0);
        let pkt = packet::new(1, 1, b"z", 0, 1);
        let h = commitment::commit_packet(&pkt);
        let pk = commitment::batch_packets_commitment_key(h);
        state::commit(&mut ibc.id, pk, COMMITMENT_MAGIC);

        t.next_tx(@0x0);
        ibc.timeout_packet(pkt, b"p", 1, IbcAppWitness {});
        assert!(state::borrow_commitment(&ibc.id, pk) == &COMMITMENT_MAGIC_ACK, 1);

        test_scenario::return_shared(ibc);
        t.end();
    }

    #[test]
    #[expected_failure(abort_code = E_ACKNOWLEDGEMENT_IS_EMPTY)]
    fun test_write_acknowledgement_empty_fails() {
        let mut t = test_scenario::begin(@0x0);
        init_for_tests(t.ctx());

        t.next_tx(@0x0);
        let mut clk0 = clock::create_for_testing(t.ctx());
        clock::increment_for_testing(&mut clk0, 1_000);
        clock::share_for_testing(clk0);

        t.next_tx(@0x0);
        let mut ibc = t.take_shared<IBCStore>();
        ibc.create_client(string::utf8(b"cometbls"), b"cs1", b"cons1", t.ctx());
        ibc.create_client(string::utf8(b"cometbls"), b"cs2", b"cons2", t.ctx());
        t.next_tx(@0x0);
        ibc.connection_open_init(2, 1);
        t.next_tx(@0x0);
        ibc.connection_open_ack(1, 9, b"p", 1);
        t.next_tx(@0x0);
        let port = string::utf8(b"0x0000000000000000000000000000000000000000000000000000000000001111::ibc::0xbe0f436bb8f8b30e0cad1c1bf27ede5bb158d47375c3a4ce108f435bd1cc9bea");
        ibc.channel_open_init(port, b"cp-port", 1, string::utf8(b"v1"), IbcAppWitness {});
        t.next_tx(@0x0);
        ibc.channel_open_ack(1, string::utf8(b"v1-cp"), 1, b"p", 1, IbcAppWitness {});
        t.next_tx(@0x0);
        let clk = t.take_shared<Clock>();
        let now_ns = clock::timestamp_ms(&clk) * 1_000_000;
        let pkt = ibc.send_packet(&clk, 1, 0, now_ns + 1_000_000_000, b"d", IbcAppWitness {}, t.ctx());
        test_scenario::return_shared(clk);

        ibc.write_acknowledgement(pkt, b"", IbcAppWitness {});

        test_scenario::return_shared(ibc);
        t.end();
    }

    #[test_only]
    public struct IbcAppWitness has drop {}
}
