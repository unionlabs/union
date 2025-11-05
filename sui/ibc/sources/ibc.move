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
    use std::string::String;
    use sui::table::{Self, Table};
    use sui::clock::{Self, Clock};
    use ibc::ibc_connection;
    use ibc::ibc_channel;
    use ibc::ibc_client;
    use ibc::ibc_packet;
    use ibc::packet::{Self, Packet};
    use ibc::connection_end::ConnectionEnd;
    use ibc::channel::Channel; 
    use ibc::light_client::{Self, LightClientManager};
    use ibc::events;
    use ibc::state;

    const VERSION: u32 = 1;

    const E_VERSION_MISMATCH: u64 = 1001;
    const E_CLIENT_NOT_FOUND: u64 = 1002;
    const E_UNAUTHORIZED: u64 = 1020;
    const E_CHANNEL_NOT_FOUND: u64 = 1047;
    const E_CONNECTION_NOT_FOUND: u64 = 1048;
    const E_COMMITMENT_NOT_FOUND: u64 = 1065;

    public struct Port<T: drop> has key, store {
        id: UID,
        _module_address: address,
        data: T,
    }

    public struct IBCStore has key {
        id: UID,
        version: u32,
        client_mgr: LightClientManager,
        connections: Table<u32, ConnectionEnd>,
        channels: Table<u32, Channel>,
    }

    fun init(ctx: &mut TxContext) {
        events::emit_initiated();
        transfer::share_object(IBCStore {
            id: object::new(ctx),
            version: VERSION,
            connections: table::new(ctx),
            channels: table::new(ctx),
            client_mgr: light_client::new(ctx, false),
        });
    }

    public fun create_port<T: drop>(
        module_address: address,
        data: T,
        ctx: &mut TxContext
    ): Port<T> {
        Port {
            id: object::new(ctx),
            _module_address: module_address,
            data,
        }
    }

    public fun port_data<T: drop>(port: &Port<T>): &T {
        &port.data
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

        ibc_client::create_client(
            &mut ibc_store.id,
            &mut ibc_store.client_mgr,
            client_type, 
            client_state_bytes, 
            consensus_state_bytes,
            ctx,
        );
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
        relayer: address,
        _: &TxContext,
    ) {
        ibc_store.assert_version();

        ibc_client::update_client(
            &mut ibc_store.id,
            &mut ibc_store.client_mgr,
            clock,
            client_id,
            client_message,
            relayer
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
        relayer: address,
        _: &TxContext,
    ) {
        ibc_store.assert_version();

        ibc_client::misbehaviour(&mut ibc_store.client_mgr, client_id, misbehaviour, relayer);
    }    

    /// Initiate the connection handshake using client at `client_id`. The next call is `connection_open_try`
    /// to advance the connection handshake.
    ///
    /// `client_id`: the id of the client which will be used for verification in for this connection
    /// `counterparty_client_id`: the id of the client on the counterparty chain
    public fun connection_open_init(
        ibc_store: &mut IBCStore,
        client_id: u32,
        counterparty_client_id: u32,
        _: &TxContext
    ) {
        ibc_store.assert_version();

        ibc_connection::connection_open_init(
            &mut ibc_store.id,
            &ibc_store.client_mgr,
            &mut ibc_store.connections,
            client_id,
            counterparty_client_id
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
        proof_height: u64,
        _: &TxContext
    ) {
        ibc_store.assert_version();

        ibc_connection::connection_open_try(
            &mut ibc_store.id,
            &ibc_store.client_mgr,
            &mut ibc_store.connections,
            counterparty_client_id,
            counterparty_connection_id,
            client_id,
            proof_init,
            proof_height,
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
        proof_height: u64,
        _: &TxContext
    ) {
        ibc_store.assert_version();

        ibc_connection::connection_open_ack(
            &mut ibc_store.id,
            &ibc_store.client_mgr,
            &mut ibc_store.connections,
            connection_id,
            counterparty_connection_id,
            proof_try,
            proof_height,
        );
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
        proof_height: u64,
        _: &TxContext
    ) {
        ibc_store.assert_version();

        ibc_connection::connection_open_confirm(
            &mut ibc_store.id,
            &ibc_store.client_mgr,
            &mut ibc_store.connections,
            connection_id,
            proof_ack,
            proof_height,
        );

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
        counterparty_port_id: vector<u8>,
        connection_id: u32,
        version: String,
        port: &Port<T>,
        _: &TxContext
    ) {
        ibc_store.assert_version();

        ibc_channel::channel_open_init(
            &mut ibc_store.id,
            &ibc_store.connections,
            &mut ibc_store.channels,
            object::uid_to_address(&port.id),
            counterparty_port_id,
            connection_id,
            version,
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
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String,
        counterparty_version: String,
        proof_init: vector<u8>,
        proof_height: u64,
        port: &Port<T>,
        _: &TxContext
    ) {
        ibc_store.assert_version();

        ibc_channel::channel_open_try(
            &mut ibc_store.id,
            &ibc_store.client_mgr,
            &ibc_store.connections,
            &mut ibc_store.channels,
            object::uid_to_address(&port.id),
            connection_id,
            counterparty_channel_id,
            counterparty_port_id,
            version,
            counterparty_version,
            proof_init,
            proof_height,  
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
        port: &Port<T>,
        _: &TxContext
    ) {
        ibc_store.assert_version();

        let port_id = state::borrow_channel_to_port(&ibc_store.id, channel_id);
        validate_port(port_id, port);

        ibc_channel::channel_open_ack(
            &mut ibc_store.id,
            &ibc_store.client_mgr,
            &ibc_store.connections,
            &mut ibc_store.channels,
            port_id,
            channel_id,
            counterparty_version,
            counterparty_channel_id,
            proof_try,
            proof_height,
        );
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
        port: &Port<T>,
        _: &TxContext
    ) {
        ibc_store.assert_version();

        let port_id = state::borrow_channel_to_port(&ibc_store.id, channel_id);
        validate_port(port_id, port);

        ibc_channel::channel_open_confirm(
            &mut ibc_store.id,
            &ibc_store.client_mgr,
            &ibc_store.connections,
            &mut ibc_store.channels,
            port_id,
            channel_id,
            proof_ack,
            proof_height
        );
    }


    /// Function to send a packet through an open channel
    public fun send_packet<T: drop>(
        ibc_store: &mut IBCStore,
        clock: &Clock,
        source_channel: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        data: vector<u8>,
        port: &Port<T>,
        ctx: &TxContext
    ): packet::Packet {
        ibc_store.assert_version();

        let port_id = state::borrow_channel_to_port(&ibc_store.id, source_channel);
        validate_port(port_id, port);

        ibc_packet::send_packet(
            &mut ibc_store.id,
            &ibc_store.channels,
            clock,
            source_channel,
            timeout_height,
            timeout_timestamp,
            data,
            ctx,
        )
    }

    /// Receive an IBC packet. This function must be called from the apps.  
    ///
    /// IMPORTANT: The function aborts if the sanity checks don't pass. This will result
    /// in a timeout since technically the tx is never completed.
    ///
    /// `packets`: Packet's to be received. This needs to match the exact batch order.
    /// `maker`: Relayer of this transaction.
    /// `maker_msgs`: The relayer message per packet.
    /// `proof`: The client-defined proof of the packet(s) commitment.
    /// `proof_height`: The height of the proof generation on the counterparty chain.
    /// `acknowledgements`: The acknowledgements per packet generated by the app. Use empty vectors(per packet) for
    /// deferring an acknowledgement. The empty data for a packet won't result in an acknowledge event and it won't
    /// be committed.
    /// `witness`: A struct where only the app WILL be able to get an instance of, for authentication.
    /// The name MUST be `IbcAppWitness`.
    public fun recv_packet<T: drop>(
        ibc_store: &mut IBCStore,
        clock: &Clock,
        packets: vector<Packet>,
        maker: address,
        maker_msgs: vector<vector<u8>>,
        proof: vector<u8>,
        proof_height: u64,
        acknowledgements: vector<vector<u8>>,
        port: &Port<T>,
        _: &TxContext
    ) {
        ibc_store.assert_version();

        let port_id = state::borrow_channel_to_port(&ibc_store.id, packets[0].destination_channel_id());
        validate_port(port_id, port);

        ibc_packet::recv_packet(
            &mut ibc_store.id,
            &ibc_store.client_mgr,
            &ibc_store.connections,
            &ibc_store.channels,
            clock,
            packets,
            maker,
            maker_msgs,
            proof,
            proof_height,
            acknowledgements
        );
    }

    /// Receive an IBC packet, except with intent functionality this time.
    /// See `recv_packet` for more.
    public fun recv_intent_packet<T: drop>(
        ibc_store: &mut IBCStore,
        clock: &clock::Clock,
        packets: vector<Packet>,
        maker: address,
        maker_msgs: vector<vector<u8>>,
        acknowledgements: vector<vector<u8>>,
        port: &Port<T>,
        _: &TxContext
    ) {
        ibc_store.assert_version();

        let port_id = state::borrow_channel_to_port(&ibc_store.id, packets[0].destination_channel_id());
        validate_port(port_id, port);

        ibc_packet::recv_intent_packet(
            &mut ibc_store.id,
            &ibc_store.client_mgr,
            &ibc_store.connections,
            &ibc_store.channels,
            clock,
            packets,
            maker,
            maker_msgs,
            acknowledgements
        );
    }

    /// Write the acknowledgement of a `packet`. The apps don't need to call this to write an ack for a packet
    /// if they do not want a deferred ack behaviour where the ack is deferred until a certain thing happens.
    /// For example, the app might forward a packet by initiating a `send` on `recv`. And then it can choose to
    /// write the ack when the forwarded packet is acknowledged.
    public fun write_acknowledgement<T: drop>(
        ibc_store: &mut IBCStore,
        packet: packet::Packet,
        acknowledgement: vector<u8>,
        port: &Port<T>,
    ) {
        ibc_store.assert_version();

        let port_id = state::borrow_channel_to_port(&ibc_store.id, packet.destination_channel_id());
        validate_port(port_id, port);

        ibc_packet::write_acknowledgement(
            &mut ibc_store.id,
            &ibc_store.channels,
            packet,
            acknowledgement,
        );
    }

    /// Acknowledge a packet that is received on the counuterparty chain. This must be called by the apps.
    ///
    /// `packets`: Packet's to be received. This needs to match the exact batch order.
    /// `acknowledgements`: The acknowledgements per packet generated by the app in the counterparty chain.
    /// `proof`: The client-defined proof of the ack(s) commitment.
    /// `proof_height`: The height of the proof generation on the counterparty chain.
    /// `relayer`: Relayer-defined address that might be used by the app or the light client.
    /// be committed.
    /// `witness`: A struct where only the app WILL be able to get an instance of, for authentication.
    /// The name MUST be `IbcAppWitness`.
    public fun acknowledge_packet<T: drop>(
        ibc_store: &mut IBCStore,
        packets: vector<packet::Packet>,
        acknowledgements: vector<vector<u8>>,
        proof: vector<u8>,
        proof_height: u64,
        relayer: address,
        port: &Port<T>,
        _: &TxContext
    )  {
        ibc_store.assert_version();

        let port_id = state::borrow_channel_to_port(&ibc_store.id, packets[0].source_channel_id());
        validate_port(port_id, port);

        ibc_packet::acknowledge_packet(
            &mut ibc_store.id,
            &ibc_store.client_mgr,
            &ibc_store.connections,
            &ibc_store.channels,
            packets,
            acknowledgements,
            proof,
            proof_height,
            relayer,
        );
    }

    /// Timeout a packet that is not received at the counterparty chain before a timeout point. This will be
    /// called by the IBC apps.
    ///
    /// `packet`: Packet to be timed-out.
    /// `proof`: The client-defined proof of the non-existence of the packet at the counterparty chain.
    /// `proof_height`: The height of the proof generation on the counterparty chain.
    /// `witness`: A struct where only the app WILL be able to get an instance of, for authentication.
    /// The name MUST be `IbcAppWitness`.
    public fun timeout_packet<T: drop>(
        ibc_store: &mut IBCStore,
        packet: Packet,
        proof: vector<u8>,
        proof_height: u64,
        port: &Port<T>,
        _: &TxContext
    ) {
        ibc_store.assert_version();

        let port_id = state::borrow_channel_to_port(&ibc_store.id, packet.source_channel_id());
        validate_port(port_id, port);

        ibc_packet::timeout_packet(
            &mut ibc_store.id,
            &ibc_store.client_mgr,
            &ibc_store.connections,
            &ibc_store.channels,
            packet,
            proof,
            proof_height,
        );
    }

    /// Commit a packet timeout commitment when a packet times out. This is meant to be called by offchain tools
    /// or the users.
    ///
    /// `packet`: An unreceived and timed-out packet.
    /// `proof`: The client-defined proof of the existence of a packet at the counterparty chain.
    /// `proof_height`: The height of the proof generation on the counterparty chain.
    public fun commit_packet_timeout(
        ibc_store: &mut IBCStore,
        clock: &clock::Clock,
        packet: Packet,
        proof: vector<u8>,
        proof_height: u64,
        ctx: &mut TxContext,
    ) {
        ibc_store.assert_version();

        ibc_packet::commit_packet_timeout(
            &mut ibc_store.id,
            &ibc_store.client_mgr,
            &ibc_store.connections,
            &ibc_store.channels,
            clock,
            packet,
            proof,
            proof_height,
            ctx,
        );
    }

    fun validate_port<T: drop>(
        port_id: address,
        port: &Port<T>,
    ) {
        assert!(port_id == object::uid_to_address(&port.id), E_UNAUTHORIZED);
    }

    public fun get_counterparty_connection(
        ibc_store: &mut IBCStore,
        connection_id: u32
    ): u32 {
        let connection = ibc_store.connections.borrow(connection_id);
        connection.counterparty_connection_id()
    }

    public fun get_client_state(ibc_store: &IBCStore, client_id: u32): vector<u8> {
        if (!ibc_store.client_mgr.exists(client_id)) {
            abort E_CLIENT_NOT_FOUND
        };

        ibc_store.client_mgr.get_client_state(client_id)
    }

    public fun get_port_id(ibc_store: &IBCStore, channel_id: u32): address {
        if (!state::has_channel_to_port(&ibc_store.id, channel_id)) {
            abort E_CHANNEL_NOT_FOUND
        };
        state::borrow_channel_to_port(&ibc_store.id, channel_id)
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
