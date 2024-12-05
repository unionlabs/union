module ibc::ibc {
    use std::signer;
    use std::vector;
    use aptos_std::smart_table::{Self, SmartTable};
    use aptos_std::table::{Self, Table};
    use std::block;
    use std::from_bcs;
    use std::event;
    use std::bcs;
    use std::object;
    use std::string::{Self, String, utf8};
    use std::hash;
    use std::timestamp;
    use std::option::{Self, Option};
    use aptos_std::any;
    use aptos_std::copyable_any;
    use std::string_utils;
    use ibc::commitment;
    use ibc::light_client;
    use ibc::connection_end::{Self, ConnectionEnd};
    use ibc::channel::{Self, Channel};
    use ibc::packet::{Self, Packet};

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
    struct ClientCreatedEvent has copy, drop, store {
        client_id: u32,
        client_type: String,
        consensus_height: u64
    }

    #[event]
    struct ClientUpdated has copy, drop, store {
        client_id: u32,
        client_type: String,
        height: u64
    }

    #[event]
    struct ConnectionOpenInit has copy, drop, store {
        connection_id: u32,
        client_id: u32,
        counterparty_client_id: u32
    }

    #[event]
    struct ChannelOpenInit has copy, drop, store {
        port_id: String,
        channel_id: u32,
        connection_id: u32,
        version: String,
    }

    #[event]
    struct ChannelOpenTry has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_channel_id: u32,
        connection_id: u32,
        version: String,
    }

    #[event]
    struct ChannelOpenAck has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_channel_id: u32,
        connection_id: u32
    }

    #[event]
    struct ChannelOpenConfirm has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_channel_id: u32,
        connection_id: u32
    }

    #[event]
    struct ConnectionOpenTry has copy, drop, store {
        connection_id: u32,
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    }

    #[event]
    struct ConnectionOpenAck has copy, drop, store {
        connection_id: u32,
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    }

    #[event]
    struct ConnectionOpenConfirm has copy, drop, store {
        connection_id: u32,
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    }

    #[event]
    struct SendPacket has drop, store {
        sequence: u64,
        source_channel: u32,
        destination_channel: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64
    }

    #[event]
    struct RecvPacket has drop, store {
        packet: Packet
    }

    #[event]
    struct RecvIntentPacket has drop, store {
        packet: Packet
    }

    #[event]
    struct TimeoutPacket has drop, store {
        packet: Packet
    }

    #[event]
    struct WriteAcknowledgement has drop, store {
        packet: Packet,
        acknowledgement: vector<u8>
    }

    #[event]
    struct AcknowledgePacket has drop, store {
        packet: Packet,
        acknowledgement: vector<u8>
    }

    #[event]
    struct SubmitMisbehaviour has drop, store {
        client_id: u32,
        client_type: String
    }

    struct Port<T: key + store + drop> has key, copy, drop, store {
        port_id: address
    }

    use aptos_framework::function_info;
    use aptos_framework::function_info::FunctionInfo;
    use ibc::dispatcher;
    use ibc::engine;

    struct RecvPacketParams has copy, drop, store {
        packet: Packet
    }

    struct RecvIntentPacketParams has copy, drop, store {
        packet: Packet
    }

    struct AcknowledgePacketParams has copy, drop, store {
        packet: Packet,
        acknowledgement: vector<u8>
    }

    struct TimeoutPacketParams has copy, drop, store {
        packet: Packet
    }

    struct ChannelOpenInitParams has copy, drop, store {
        ordering: u8,
        connection_id: u32,
        channel_id: u32,
        version: String
    }

    struct ChannelOpenTryParams has copy, drop, store {
        ordering: u8,
        connection_id: u32,
        channel_id: u32,
        counterparty_channel_id: u32,
        version: String,
        counterparty_version: String,
    }

    struct ChannelOpenAckParams has copy, drop, store {
        channel_id: u32,
        counterparty_channel_id: u32,
        counterparty_version: String,
    }

    struct ChannelOpenConfirmParams has copy, drop, store {
        channel_id: u32
    }

    struct ChannelCloseInitParams has copy, drop, store {
        channel_id: u32
    }

    struct ChannelCloseConfirmParams has copy, drop, store {
        channel_id: u32
    }

    // Getter for RecvPacketParams
    public fun get_packet_from_recv_param(param: &RecvPacketParams): &Packet {
        &param.packet
    }

    // Getter for RecvPacketParams
    public fun get_packet_from_recv_intent_param(
        param: &RecvIntentPacketParams
    ): &Packet {
        &param.packet
    }

    // Getters for AcknowledgePacketParams
    public fun get_packet_from_ack_param(param: &AcknowledgePacketParams): &Packet {
        &param.packet
    }

    public fun get_acknowledgement_from_ack_param(
        param: &AcknowledgePacketParams
    ): &vector<u8> {
        &param.acknowledgement
    }

    // Getter for TimeoutPacketParams
    public fun get_packet_from_timeout_param(param: &TimeoutPacketParams): &Packet {
        &param.packet
    }

    // Getters for ChannelOpenInitParams
    public fun get_ordering_from_channel_open_init_param(
        param: &ChannelOpenInitParams
    ): u8 {
        param.ordering
    }

    public fun get_connection_id_from_channel_open_init_param(
        param: &ChannelOpenInitParams
    ): u32 {
        param.connection_id
    }

    public fun get_channel_id_from_channel_open_init_param(
        param: &ChannelOpenInitParams
    ): u32 {
        param.channel_id
    }

    public fun get_version_from_channel_open_init_param(
        param: &ChannelOpenInitParams
    ): &String {
        &param.version
    }

    // Getters for ChannelOpenTryParams
    public fun get_ordering_from_channel_open_try_param(
        param: &ChannelOpenTryParams
    ): u8 {
        param.ordering
    }

    public fun get_connection_id_from_channel_open_try_param(
        param: &ChannelOpenTryParams
    ): u32 {
        param.connection_id
    }

    public fun get_channel_id_from_channel_open_try_param(
        param: &ChannelOpenTryParams
    ): u32 {
        param.channel_id
    }

    public fun get_counterparty_channel_id_from_channel_open_try_param(
        param: &ChannelOpenTryParams
    ): u32 {
        param.counterparty_channel_id
    }

    public fun get_version_from_channel_open_try_param(
        param: &ChannelOpenTryParams
    ): &String {
        &param.version
    }

    public fun get_counterparty_version_from_channel_open_try_param(
        param: &ChannelOpenTryParams
    ): &String {
        &param.counterparty_version
    }

    // Getters for ChannelOpenAckParams
    public fun get_channel_id_from_channel_open_ack_param(
        param: &ChannelOpenAckParams
    ): u32 {
        param.channel_id
    }

    public fun get_counterparty_channel_id_from_channel_open_ack_param(
        param: &ChannelOpenAckParams
    ): u32 {
        param.counterparty_channel_id
    }

    public fun get_counterparty_version_from_channel_open_ack_param(
        param: &ChannelOpenAckParams
    ): &String {
        &param.counterparty_version
    }

    // Getter for ChannelOpenConfirmParams
    public fun get_channel_id_from_channel_open_confirm_param(
        param: &ChannelOpenConfirmParams
    ): u32 {
        param.channel_id
    }

    // Getter for ChannelCloseInitParams
    public fun get_channel_id_from_channel_close_init_param(
        param: &ChannelCloseInitParams
    ): u32 {
        param.channel_id
    }

    // Getter for ChannelCloseConfirmParams
    public fun get_channel_id_from_channel_close_confirm_param(
        param: &ChannelCloseConfirmParams
    ): u32 {
        param.channel_id
    }

    public fun register_application<T: key + store + drop>(
        ibc_app: &signer, cb: FunctionInfo, type: T
    ) acquires SignerRef {
        dispatcher::register<T>(cb, type, bcs::to_bytes(&signer::address_of(ibc_app)));
        move_to(
            &get_ibc_signer(),
            Port<T> { port_id: signer::address_of(ibc_app) }
        );
    }

    // Resource to hold the global state
    struct IBCStore has key {
        client_impls: SmartTable<String, address>,
        client_registry: SmartTable<String, address>,
        commitments: Table<vector<u8>, vector<u8>>,
        connections: SmartTable<u32, ConnectionEnd>,
        channels: SmartTable<u32, Channel>
    }

    struct SignerRef has key {
        self_ref: object::ExtendRef
    }

    // Initializes the IBCStore resource in the signer's account
    fun init_module(account: &signer) {
        assert!(
            signer::address_of(account) == @ibc, E_NOT_ENOUGH_PERMISSIONS_TO_INITIALIZE
        );
        let vault_constructor_ref = &object::create_named_object(account, VAULT_SEED);
        let vault_signer = &object::generate_signer(vault_constructor_ref);

        let store = IBCStore {
            client_registry: smart_table::new(),
            commitments: table::new(),
            client_impls: smart_table::new(),
            connections: smart_table::new(),
            channels: smart_table::new()
        };

        move_to(vault_signer, store);

        move_to(
            vault_signer,
            SignerRef { self_ref: object::generate_extend_ref(vault_constructor_ref) }
        );
    }

    /// Create a client with an initial client and consensus state
    public entry fun create_client(
        client_type: String, client_state: vector<u8>, consensus_state: vector<u8>
    ) acquires IBCStore, SignerRef {
        // NOTE(aeryz): At this point, we don't need to have a routing mechanism because it will introduce
        // additional gas cost. We should only enforce the use of `cometbls` for the `client_type`
        assert!(string::bytes(&client_type) == &b"cometbls", E_UNKNOWN_CLIENT_TYPE);

        let client_id = generate_client_identifier();
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let (client_state, consensus_state) =
            light_client::create_client(
                &get_ibc_signer(),
                client_id,
                // from_bcs::to_bytes(client_state),
                // from_bcs::to_bytes(consensus_state),
                client_state,
                consensus_state
            );

        // TODO(aeryz): fetch these status from proper exported consts
        assert!(light_client::status(client_id) == 0, E_CLIENT_NOT_ACTIVE);

        // Update commitments
        table::upsert(
            &mut store.commitments,
            commitment::client_state_commitment_key(client_id),
            client_state
        );

        let latest_height = light_client::latest_height(client_id);

        table::upsert(
            &mut store.commitments,
            commitment::consensus_state_commitment_key(client_id, latest_height),
            consensus_state
        );

        event::emit(
            ClientCreatedEvent { client_id, client_type, consensus_height: latest_height }
        );
    }

    public entry fun connection_open_init(
        client_type: String,
        client_id: u32,
        counterparty_client_type: String,
        counterparty_client_id: u32
    ) acquires IBCStore {
        let connection_id = generate_connection_identifier();

        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let connection =
            connection_end::new(
                CONN_STATE_INIT,
                client_id,
                counterparty_client_id,
                0,
            );

        smart_table::upsert(&mut store.connections, connection_id, connection);

        commit_connection(connection_id, connection);

        event::emit(
            ConnectionOpenInit {
                connection_id: connection_id,
                client_id: client_id,
                counterparty_client_id: counterparty_client_id
            }
        )
    }

    public entry fun connection_open_try(
        counterparty_client_type: String,
        counterparty_client_id: u32,
        counterparty_connection_id: u32,
        client_type: String,
        client_id: u32,
        proof_init: vector<u8>,
        proof_height: u64
    ) acquires IBCStore {
        let connection_id = generate_connection_identifier();

        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let connection =
            smart_table::borrow_mut_with_default(
                &mut store.connections,
                connection_id,
                connection_end::new(
                    CONN_STATE_TRYOPEN,
                    client_id,
                    counterparty_client_id,
                    counterparty_connection_id,
                )
            );

        // Create the expected connection
        let expected_connection =
            connection_end::new(
                CONN_STATE_INIT,
                counterparty_client_id,
                client_id,
                0, // counterparty_connection_id
            );

        // Verify the connection state
        let err =
            verify_connection_state(
                connection,
                proof_height,
                proof_init,
                counterparty_connection_id,
                expected_connection
            );
        assert!(err == 0, err);

        event::emit(
            ConnectionOpenTry {
                connection_id,
                client_id: client_id,
                counterparty_client_id: counterparty_client_id,
                counterparty_connection_id: counterparty_connection_id
            }
        );

        commit_connection(connection_id, *connection);
    }

    public entry fun connection_open_ack(
        connection_id: u32,
        counterparty_connection_id: u32,
        proof_try: vector<u8>,
        proof_height: u64
    ) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        assert!(
            smart_table::contains(&store.connections, connection_id),
            E_CONNECTION_DOES_NOT_EXIST
        );

        let connection = smart_table::borrow_mut(&mut store.connections, connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_INIT,
            E_INVALID_CONNECTION_STATE
        );

        // Create the expected connection
        let expected_connection =
            connection_end::new(
                CONN_STATE_TRYOPEN,
                connection_end::counterparty_client_id(connection),
                connection_end::client_id(connection),
                connection_id,
            );

        // Verify the connection state
        let err =
            verify_connection_state(
                connection,
                proof_height,
                proof_try,
                counterparty_connection_id,
                expected_connection
            );
        assert!(err == 0, err);

        connection_end::set_state(connection, CONN_STATE_TRYOPEN);
        connection_end::set_counterparty_connection_id(
            connection, counterparty_connection_id
        );

        event::emit(
            ConnectionOpenAck {
                connection_id,
                client_id: connection_end::client_id(connection),
                counterparty_client_id: connection_end::counterparty_client_id(
                    connection
                ),
                counterparty_connection_id: connection_end::counterparty_connection_id(
                    connection
                ),
            }
        );

        commit_connection(connection_id, *connection);
    }

    public entry fun connection_open_confirm(
        connection_id: u32, proof_ack: vector<u8>, proof_height: u64
    ) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        assert!(
            smart_table::contains(&store.connections, connection_id),
            E_CONNECTION_DOES_NOT_EXIST
        );

        let connection = smart_table::borrow_mut(&mut store.connections, connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_TRYOPEN,
            E_INVALID_CONNECTION_STATE
        );

        // Create the expected connection
        let expected_connection =
            connection_end::new(
                CONN_STATE_OPEN,
                connection_end::counterparty_client_id(connection),
                connection_end::client_id(connection),
                connection_id,
            );
        let counterparty_connection_id =
            connection_end::counterparty_connection_id(connection);

        // Verify the connection state
        let err =
            verify_connection_state(
                connection,
                proof_height,
                proof_ack,
                counterparty_connection_id,
                expected_connection
            );
        assert!(err == 0, err);

        connection_end::set_state(connection, CONN_STATE_OPEN);

        event::emit(
            ConnectionOpenAck {
                connection_id: connection_id,
                client_id: connection_end::client_id(connection),
                counterparty_client_id: connection_end::counterparty_client_id(connection),
                counterparty_connection_id: connection_end::counterparty_connection_id(
                    connection
                )
            }
        );

        commit_connection(connection_id, *connection);
    }

    public entry fun update_client(
        client_id: u32, client_message: vector<u8>
    ) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        assert!(
            table::contains(
                &store.commitments,
                commitment::client_state_commitment_key(client_id)
            ),
            E_CLIENT_NOT_FOUND
        );

        if (light_client::check_for_misbehaviour(client_id, client_message)) {
            event::emit(
                SubmitMisbehaviour {
                    client_id,
                    client_type: string::utf8(CLIENT_TYPE_COMETBLS)
                }
            );
            return
        };

        let (client_state, consensus_states, heights) =
            light_client::update_client(client_id, client_message);

        let heights_len = vector::length(&heights);

        assert!(
            !vector::is_empty(&consensus_states)
                && !vector::is_empty(&heights)
                && heights_len == vector::length(&consensus_states),
            E_INVALID_UPDATE
        );

        table::upsert(
            &mut store.commitments,
            commitment::client_state_commitment_key(client_id),
            client_state
        );

        let i = 0;
        while (i < heights_len) {
            let height = *vector::borrow(&heights, i);

            table::upsert(
                &mut store.commitments,
                commitment::consensus_state_commitment_key(client_id, height),
                hash::sha2_256(*vector::borrow(&consensus_states, i))
            );

            event::emit(
                ClientUpdated {
                    client_id,
                    // NOTE: This is currently enforced, if/when we refactor to be more general across clients then this will need to be modified accordingly
                    client_type: string::utf8(CLIENT_TYPE_COMETBLS),
                    height
                }
            );

            i = i + 1;
        };
    }

    public entry fun submit_misbehaviour(
        client_id: u32, misbehaviour: vector<u8>
    ) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        assert!(
            table::contains(
                &store.commitments,
                commitment::client_state_commitment_key(client_id)
            ),
            E_CLIENT_NOT_FOUND
        );

        light_client::report_misbehaviour(client_id, misbehaviour);

        event::emit(
            SubmitMisbehaviour {
                client_id,
                client_type: string::utf8(CLIENT_TYPE_COMETBLS)
            }
        );
    }

    public entry fun channel_open_init<T: key + store + drop>(
        port_id: address,
        connection_id: u32,
        ordering: u8,
        version: String,
    ) acquires IBCStore, Port {
        let port = borrow_global<Port<T>>(get_vault_addr());
        assert!(port.port_id == port_id, E_UNAUTHORIZED);

        let port_id = address_to_string(port_id);

        ensure_connection_state(connection_id);

        let channel_id = generate_channel_identifier();

        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let channel = channel::default();
        channel::set_state(&mut channel, CHAN_STATE_INIT);
        channel::set_ordering(&mut channel, ordering);
        channel::set_connection_id(&mut channel, connection_id);
        channel::set_version(&mut channel, version);

        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_send_commitment_key(channel_id),
            bcs::to_bytes(&1)
        );

        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_recv_commitment_key(channel_id),
            bcs::to_bytes(&1)
        );

        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_ack_commitment_key(channel_id),
            bcs::to_bytes(&1)
        );

        commit_channel(channel_id, channel);

        let param =
            copyable_any::pack<ChannelOpenInitParams>(
                ChannelOpenInitParams {
                    ordering: ordering,
                    connection_id: connection_id,
                    channel_id: channel_id,
                    version: version
                }
            );
        engine::dispatch<T>(param);

        dispatcher::delete_storage<T>();

        event::emit(
            ChannelOpenInit {
                port_id: port_id,
                channel_id: channel_id,
                connection_id: connection_id,
                version: version
            }
        );
    }

    public entry fun channel_open_try<T: key + store + drop>(
        port_id: address,
        channel_state: u8,
        channel_order: u8,
        connection_id: u32,
        counterparty_channel_id: u32,
        version: String,
        counterparty_version: String,
        proof_init: vector<u8>,
        proof_height: u64
    ) acquires IBCStore, Port {
        let port = borrow_global<Port<T>>(get_vault_addr());
        assert!(port.port_id == port_id, E_UNAUTHORIZED);

        let port_id = address_to_string(port_id);

        let client_id = ensure_connection_state(connection_id);

        let expected_channel =
            channel::new(
                CHAN_STATE_INIT,
                channel_order,
                get_counterparty_connection(connection_id),
                counterparty_channel_id,
                counterparty_version
            );

        let err =
            verify_channel_state(
                client_id,
                proof_height,
                proof_init,
                counterparty_channel_id,
                expected_channel
            );
        assert!(err == 0, err);

        let channel_id = generate_channel_identifier();

        event::emit(
            ChannelOpenTry {
                port_id,
                channel_id,
                counterparty_channel_id,
                connection_id,
                version: counterparty_version
            }
        );

        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let channel =
            channel::new(
                channel_state,
                channel_order,
                connection_id,
                counterparty_channel_id,
                version
            );

        smart_table::upsert(&mut store.channels, channel_id, channel);

        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_send_commitment_key(channel_id),
            bcs::to_bytes(&1)
        );

        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_recv_commitment_key(channel_id),
            bcs::to_bytes(&1)
        );

        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_ack_commitment_key(channel_id),
            bcs::to_bytes(&1)
        );

        commit_channel(channel_id, channel);

        let param =
            copyable_any::pack<ChannelOpenTryParams>(
                ChannelOpenTryParams {
                    ordering: channel_order,
                    connection_id: connection_id,
                    channel_id: channel_id,
                    counterparty_channel_id: counterparty_channel_id,
                    version: version,
                    counterparty_version: counterparty_version
                }
            );
        engine::dispatch<T>(param);

        dispatcher::delete_storage<T>();

    }

    public entry fun channel_open_ack<T: key + store + drop>(
        port_id: address,
        channel_id: u32,
        counterparty_version: String,
        counterparty_channel_id: u32,
        proof_try: vector<u8>,
        proof_height: u64
    ) acquires IBCStore, Port {
        let port = borrow_global<Port<T>>(get_vault_addr());
        assert!(port.port_id == port_id, E_UNAUTHORIZED);

        let chan =
            *smart_table::borrow(
                &borrow_global<IBCStore>(get_vault_addr()).channels,
                channel_id
            );

        assert!(channel::state(&chan) == CHAN_STATE_INIT, E_INVALID_CHANNEL_STATE);

        let port_id = address_to_string(port_id);

        let connection_id = channel::connection_id(&chan);

        let client_id = ensure_connection_state(connection_id);

        let expected_channel =
            channel::new(
                CHAN_STATE_TRYOPEN,
                channel::ordering(&chan),
                get_counterparty_connection(connection_id),
                counterparty_channel_id,
                counterparty_version
            );

        let err =
            verify_channel_state(
                client_id,
                proof_height,
                proof_try,
                counterparty_channel_id,
                expected_channel
            );
        assert!(err == 0, err);

        // TODO: Not sure if this upsert is required or not?
        smart_table::upsert(
            &mut borrow_global_mut<IBCStore>(get_vault_addr()).channels,
            channel_id,
            chan
        );
        channel::set_state(&mut chan, CHAN_STATE_OPEN);
        channel::set_version(&mut chan, counterparty_version);
        channel::set_counterparty_channel_id(&mut chan, counterparty_channel_id);

        event::emit(
            ChannelOpenAck {
                port_id,
                channel_id,
                counterparty_channel_id,
                connection_id: connection_id
            }
        );

        commit_channel(channel_id, chan);

        let param =
            copyable_any::pack<ChannelOpenAckParams>(
                ChannelOpenAckParams {
                    channel_id: channel_id,
                    counterparty_channel_id: counterparty_channel_id,
                    counterparty_version: counterparty_version
                }
            );
        engine::dispatch<T>(param);

        dispatcher::delete_storage<T>();
    }

    public entry fun channel_open_confirm<T: key + store + drop>(
        port_id: address,
        channel_id: u32,
        proof_ack: vector<u8>,
        proof_height: u64
    ) acquires IBCStore, Port {
        let port = borrow_global<Port<T>>(get_vault_addr());
        assert!(port.port_id == port_id, E_UNAUTHORIZED);
        let chan =
            *smart_table::borrow(
                &borrow_global<IBCStore>(get_vault_addr()).channels,
                channel_id
            );

        assert!(channel::state(&chan) == CHAN_STATE_TRYOPEN, E_INVALID_CHANNEL_STATE);

        let port_id = address_to_string(port_id);

        let connection_id = channel::connection_id(&chan);

        let client_id = ensure_connection_state(connection_id);

        let expected_channel =
            channel::new(
                CHAN_STATE_OPEN,
                channel::ordering(&chan),
                get_counterparty_connection(connection_id),
                channel_id,
                *channel::version(&chan)
            );

        let err =
            verify_channel_state(
                client_id,
                proof_height,
                proof_ack,
                channel::counterparty_channel_id(&chan),
                expected_channel
            );
        assert!(err == 0, err);

        channel::set_state(&mut chan, CHAN_STATE_OPEN);

        // TODO: Not sure if this upsert is required or not?
        smart_table::upsert(
            &mut borrow_global_mut<IBCStore>(get_vault_addr()).channels,
            channel_id,
            chan
        );

        event::emit(
            ChannelOpenConfirm {
                port_id,
                channel_id,
                counterparty_channel_id: channel::counterparty_channel_id(&chan),
                connection_id: channel::connection_id(&chan)
            }
        );
        commit_channel(channel_id, chan);

        let param =
            copyable_any::pack<ChannelOpenConfirmParams>(
                ChannelOpenConfirmParams { channel_id: channel_id }
            );
        engine::dispatch<T>(param);

        dispatcher::delete_storage<T>();
    }

    // Sends a packet
    public fun send_packet(
        ibc_app: &signer,
        source_port: address,
        source_channel: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        data: vector<u8>
    ): u64 acquires IBCStore {
        authorize_app(ibc_app, source_port);

        if (timeout_timestamp != 0 && timeout_height == 0) {
            abort E_TIMEOUT_MUST_BE_SET
        };

        let channel = ensure_channel_state(source_channel);

        let sequence = generate_packet_sequence(source_channel);

        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let packet =
            packet::new(
                sequence,
                source_channel,
                channel::counterparty_channel_id(&channel),
                data,
                timeout_height,
                timeout_timestamp
            );
        let commitment_key =
            commitment::batch_packets_commitment_key(
                source_channel, commitment::commit_packet(&packet)
            );
        table::upsert(
            &mut store.commitments,
            commitment_key,
            COMMITMENT_MAGIC
        );

        event::emit(
            SendPacket {
                sequence: sequence,
                source_channel: source_channel,
                destination_channel: channel::counterparty_channel_id(&channel),
                data: data,
                timeout_height: timeout_height,
                timeout_timestamp: timeout_timestamp
            }
        );
        sequence
    }

    public fun process_receive<T: key + store + drop>(
        packets: vector<Packet>,
        proof_height: u64,
        proof: vector<u8>,
        intent: bool
    ) acquires IBCStore {
        let l = vector::length(&packets);
        assert!(l > 0, E_NOT_ENOUGH_PACKETS);

        let first_packet = *vector::borrow(&packets, 0);
        let source_channel = packet::source_channel(&first_packet);
        let destination_channel = packet::destination_channel(&first_packet);

        let channel = ensure_channel_state(source_channel);
        let client_id = ensure_connection_state(channel::connection_id(&channel));

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

        let ordering = channel::ordering(&channel);
        let i = 0;
        while (i < l) {
            let packet = *vector::borrow(&packets, i);

            if (packet::timeout_height(&packet) != 0) {
                assert!(
                    block::get_current_block_height() < packet::timeout_height(&packet),
                    E_HEIGHT_TIMEOUT
                );
            };

            let current_timestamp = timestamp::now_seconds() * 1_000_000_000; // 1e9
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

            let already_received = false;

            if (ordering == CHAN_ORDERING_UNORDERED) {
                already_received = set_packet_receive(commitment_key);
            } else if (ordering == CHAN_ORDERING_ORDERED) {
                if (intent) {
                    abort E_CANNOT_INTENT_ORDERED
                };
                set_next_sequence_recv(destination_channel, packet::sequence(&packet));
            };

            if (!already_received) {
                let acknowledgement = vector::empty();
                if (intent) {

                    let param =
                        copyable_any::pack<RecvIntentPacketParams>(
                            RecvIntentPacketParams { packet: packet }
                        );
                    engine::dispatch<T>(param);

                    acknowledgement = dispatcher::get_return_value<T>();

                    dispatcher::delete_storage<T>();
                    event::emit(RecvIntentPacket { packet: packet });
                } else {
                    let param =
                        copyable_any::pack<RecvPacketParams>(
                            RecvPacketParams { packet: packet }
                        );
                    engine::dispatch<T>(param);

                    acknowledgement = dispatcher::get_return_value<T>();

                    dispatcher::delete_storage<T>();
                    event::emit(RecvPacket { packet: packet });
                };
                if (vector::length(&acknowledgement) > 0) {
                    inner_write_acknowledgement(commitment_key, acknowledgement);
                    event::emit(WriteAcknowledgement { packet, acknowledgement });
                };
            };
            i = i + 1;
        }
    }

    /// Receives and processes an IBC packet
    ///
    /// Note that any sanity check failures will result in this function to be aborted in order for caller's
    /// storage to be reverted. This will result in acks won't be able to written.
    public entry fun recv_packet<T: key + store + drop>(
        port_id: address,
        packet_sequences: vector<u64>,
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_datas: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        proof: vector<u8>,
        proof_height: u64
    ) acquires IBCStore, Port {
        let port = borrow_global<Port<T>>(get_vault_addr());
        assert!(port.port_id == port_id, E_UNAUTHORIZED);

        let packets: vector<Packet> = vector::empty();
        let i = 0;
        while (i < vector::length(&packet_sequences)) {
            vector::push_back(
                &mut packets,
                packet::new(
                    *vector::borrow(&packet_sequences, i),
                    *vector::borrow(&packet_source_channels, i),
                    *vector::borrow(&packet_destination_channels, i),
                    *vector::borrow(&packet_datas, i),
                    *vector::borrow(&packet_timeout_heights, i),
                    *vector::borrow(&packet_timeout_timestamps, i)
                )
            );
            i = i + 1;
        };

        process_receive<T>(packets, proof_height, proof, false);
    }

    fun inner_write_acknowledgement(
        commitment_key: vector<u8>, acknowledgement: vector<u8>
    ) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        if (!table::contains(&store.commitments, commitment_key)) {
            abort E_PACKET_NOT_RECEIVED
        };
        let commitment = table::borrow(&store.commitments, commitment_key);
        assert!(
            *commitment == COMMITMENT_MAGIC,
            E_ACK_ALREADY_EXIST
        );
        table::upsert(
            &mut store.commitments,
            commitment_key,
            commitment::commit_ack(acknowledgement)
        );
    }

    public fun write_acknowledgement(
        packet: packet::Packet, acknowledgement: vector<u8>
    ) acquires IBCStore {
        assert!(!vector::is_empty(&acknowledgement), E_ACKNOWLEDGEMENT_IS_EMPTY);

        ensure_channel_state(packet::destination_channel(&packet));

        let commitment_key =
            commitment::batch_receipts_commitment_key(
                packet::destination_channel(&packet),
                commitment::commit_packet(&packet)
            );
        inner_write_acknowledgement(commitment_key, acknowledgement);

        event::emit(WriteAcknowledgement { packet, acknowledgement });
    }

    public entry fun acknowledge_packet<T: key + store + drop>(
        port_id: address,
        packet_sequences: vector<u64>,
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_datas: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        acknowledgements: vector<vector<u8>>,
        proof: vector<u8>,
        proof_height: u64
    ) acquires IBCStore, Port {
        let port = borrow_global<Port<T>>(get_vault_addr());
        assert!(port.port_id == port_id, E_UNAUTHORIZED);

        let packets: vector<Packet> = vector::empty();
        let i = 0;
        while (i < vector::length(&packet_sequences)) {
            vector::push_back(
                &mut packets,
                packet::new(
                    *vector::borrow(&packet_sequences, i),
                    *vector::borrow(&packet_source_channels, i),
                    *vector::borrow(&packet_destination_channels, i),
                    *vector::borrow(&packet_datas, i),
                    *vector::borrow(&packet_timeout_heights, i),
                    *vector::borrow(&packet_timeout_timestamps, i)
                )
            );
            i = i + 1;
        };
        let l = vector::length(&packets);
        assert!(l > 0, E_NOT_ENOUGH_PACKETS);

        let first_packet = *vector::borrow(&packets, 0);
        let source_channel = packet::source_channel(&first_packet);
        let destination_channel = packet::destination_channel(&first_packet);

        let channel = ensure_channel_state(source_channel);
        let client_id = ensure_connection_state(channel::connection_id(&channel));

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
                client_id,
                proof_height,
                proof,
                commitment_key,
                commitment::commit_acks(acknowledgements)
            );

        if (err != 0) {
            abort err
        };

        let ordering = channel::ordering(&channel);
        let i = 0;
        while (i < l) {
            let packet = *vector::borrow(&packets, i);
            let commitment_key =
                commitment::batch_packets_commitment_key(
                    source_channel, commitment::commit_packet(&packet)
                );
            table::remove(
                &mut borrow_global_mut<IBCStore>(get_vault_addr()).commitments,
                commitment_key
            );

            let acknowledgement = *vector::borrow(&acknowledgements, i);
            // onAcknowledgementPacket(...)
            if (ordering == CHAN_ORDERING_ORDERED) {
                set_next_sequence_ack(source_channel, packet::sequence(&packet));
            };

            let param =
                copyable_any::pack<AcknowledgePacketParams>(
                    AcknowledgePacketParams {
                        packet: packet,
                        acknowledgement: acknowledgement
                    }
                );
            engine::dispatch<T>(param);

            dispatcher::delete_storage<T>();

            event::emit(AcknowledgePacket { packet, acknowledgement });

            i = i + 1;
        }
    }

    public entry fun timeout_packet<T: key + store + drop>(
        port_id: address,
        packet_sequence: u64,
        packet_source_channel: u32,
        packet_destination_channel: u32,
        packet_data: vector<u8>,
        packet_timeout_height: u64,
        packet_timeout_timestamp: u64,
        proof: vector<u8>,
        proof_height: u64,
        next_sequence_recv: u64
    ) acquires IBCStore, Port {

        let port = borrow_global<Port<T>>(get_vault_addr());
        assert!(port.port_id == port_id, E_UNAUTHORIZED);

        let packet =
            packet::new(
                packet_sequence,
                packet_source_channel,
                packet_destination_channel,
                packet_data,
                packet_timeout_height,
                packet_timeout_timestamp
            );

        let source_channel = packet::source_channel(&packet);
        let destination_channel = packet::destination_channel(&packet);
        let channel = ensure_channel_state(source_channel);
        let client_id = ensure_connection_state(channel::connection_id(&channel));

        let proof_timestamp =
            light_client::get_timestamp_at_height(client_id, proof_height);
        assert!(proof_timestamp != 0, E_LATEST_TIMESTAMP_NOT_FOUND);

        let ordering = channel::ordering(&channel);

        if (ordering == CHAN_ORDERING_ORDERED) {
            let err =
                verify_commitment(
                    client_id,
                    proof_height,
                    proof,
                    commitment::next_sequence_recv_commitment_key(destination_channel),
                    bcs::to_bytes(&next_sequence_recv)
                );
            assert!(err == 0, err);
        } else if (ordering == CHAN_ORDERING_UNORDERED) {
            let commitment_key =
                commitment::batch_receipts_commitment_key(
                    destination_channel, commitment::commit_packet(&packet)
                );
            let err =
                verify_absent_commitment(client_id, proof_height, proof, commitment_key);
            assert!(err == 0, err);
        } else {
            abort E_UNKNOWN_CHANNEL_ORDERING
        };

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

        if (channel::ordering(&channel) == CHAN_ORDERING_ORDERED) {
            assert!(
                next_sequence_recv > packet::sequence(&packet),
                E_NEXT_SEQUENCE_MUST_BE_GREATER_THAN_TIMEOUT_SEQUENCE
            );
        };
        let commitment_key =
            commitment::batch_packets_commitment_key(
                source_channel, commitment::commit_packet(&packet)
            );
        table::remove(
            &mut borrow_global_mut<IBCStore>(get_vault_addr()).commitments,
            commitment_key
        );

        let param =
            copyable_any::pack<TimeoutPacketParams>(TimeoutPacketParams { packet: packet });
        engine::dispatch<T>(param);

        dispatcher::delete_storage<T>();
        event::emit(TimeoutPacket { packet });
    }

    // ========= UTILS and VIEW functions ========= //

    #[view]
    public fun client_state(client_id: u32): vector<u8> {
        light_client::get_client_state(client_id)
    }

    #[view]
    public fun consensus_state(client_id: u32, revision_height: u64): vector<u8> {
        light_client::get_consensus_state(client_id, revision_height)
    }

    #[view]
    public fun get_vault_addr(): address {
        object::create_object_address(&@ibc, VAULT_SEED)
    }

    // Getter for nextChannelSequence in Commitments
    #[view]
    public fun get_next_channel_sequence(): u64 acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let next_sequence_bytes =
            table::borrow_with_default(
                &store.commitments, b"nextChannelSequence", &bcs::to_bytes<u64>(&0)
            );
        from_bcs::to_u64(*next_sequence_bytes)
    }

    // Getter for nextChannelSequence in Commitments
    #[view]
    public fun get_counterparty_connection(connection_id: u32): u32 acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let connection = smart_table::borrow(&store.connections, connection_id);
        connection_end::counterparty_connection_id(connection)

    }

    fun set_connection(connection_id: u32, connection: ConnectionEnd) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        smart_table::upsert(&mut store.connections, connection_id, connection);
    }

    fun set_channel(channel_id: u32, channel: Channel) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        smart_table::upsert(&mut store.channels, channel_id, channel);
    }

    // Setter for Commitments
    fun set_commitment(key: vector<u8>, value: vector<u8>) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        table::upsert(&mut store.commitments, key, value);
    }

    // Getter for Commitments
    #[view]
    public fun get_commitment(key: vector<u8>): vector<u8> acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let commitment =
            table::borrow_with_default(&store.commitments, key, &vector::empty<u8>());
        *commitment
    }

    fun authorize_app(ibc_app: &signer, port_id: address) {
        assert!(
            object::create_object_address(&port_id, IBC_APP_SEED)
                == signer::address_of(ibc_app),
            E_UNAUTHORIZED
        );
    }

    // Getter for Commitments
    fun get_channel_from_store(channel_id: u32): Channel acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let channel = smart_table::borrow(&store.channels, channel_id);

        *channel
    }

    fun set_next_channel_sequence(sequence: u64) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        table::upsert(
            &mut store.commitments, b"nextChannelSequence", bcs::to_bytes(&sequence)
        );
    }

    fun set_next_sequence_recv(
        destination_channel: u32, received_sequence: u64
    ) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let next_sequence_recv_key =
            commitment::next_sequence_recv_commitment_key(destination_channel);

        let expected_recv_sequence =
            from_bcs::to_u64(
                *table::borrow(&store.commitments, next_sequence_recv_key)
            );

        if (expected_recv_sequence != received_sequence) {
            abort E_PACKET_SEQUENCE_NEXT_SEQUENCE_MISMATCH
        };

        table::upsert(
            &mut store.commitments,
            next_sequence_recv_key,
            bcs::to_bytes<u64>(&(expected_recv_sequence + 1))
        );
    }

    fun set_next_sequence_ack(source_channel: u32, ack_sequence: u64) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let commitment_key = commitment::next_sequence_ack_commitment_key(source_channel);

        let expected_ack_sequence =
            from_bcs::to_u64(*table::borrow(&store.commitments, commitment_key));

        if (expected_ack_sequence != ack_sequence) {
            abort E_PACKET_SEQUENCE_ACK_SEQUENCE_MISMATCH
        };

        table::upsert(
            &mut store.commitments,
            commitment_key,
            bcs::to_bytes<u64>(&(expected_ack_sequence + 1))
        );
    }

    fun set_packet_receive(commitment_key: vector<u8>): bool acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        assert!(
            table::contains(&store.commitments, commitment_key),
            E_CLIENT_NOT_FOUND
        );
        let already_received =
            *table::borrow(&store.commitments, commitment_key) != COMMITMENT_NULL;
        if (!already_received) {
            table::upsert(
                &mut store.commitments,
                commitment_key,
                COMMITMENT_MAGIC
            );
        };
        already_received
    }

    fun generate_packet_sequence(channel_id: u32): u64 acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let commitment_key = commitment::next_sequence_send_commitment_key(channel_id);

        let data = table::borrow(&store.commitments, commitment_key);
        let seq = from_bcs::to_u64(*data);

        table::upsert(
            &mut store.commitments,
            commitment_key,
            bcs::to_bytes<u64>(&(seq + 1))
        );
        seq
    }

    // Function to generate a client identifier
    fun generate_client_identifier(): u32 acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let next_sequence =
            table::borrow_with_default(
                &store.commitments, b"nextClientSequence", &bcs::to_bytes<u64>(&0u64)
            );
        let next_sequence = from_bcs::to_u32(*next_sequence);

        table::upsert(
            &mut store.commitments,
            b"nextClientSequence",
            bcs::to_bytes<u32>(&(next_sequence + 1))
        );
        next_sequence
    }

    fun get_ibc_signer(): signer acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        object::generate_signer_for_extending(&vault.self_ref)
    }

    fun verify_connection_state(
        connection: &ConnectionEnd,
        height: u64,
        proof: vector<u8>,
        connection_id: u32,
        counterparty_connection: ConnectionEnd
    ): u64 {
        light_client::verify_membership(
            connection_end::client_id(connection),
            height,
            proof,
            commitment::connection_commitment_key(connection_id),
            connection_end::encode(&counterparty_connection)
        )
    }

    public fun verify_commitment(
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        path: vector<u8>,
        commitment: vector<u8>
    ): u64 {
        light_client::verify_membership(client_id, height, proof, path, commitment)
    }

    public fun generate_connection_identifier(): u32 acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let next_sequence_bytes =
            table::borrow_with_default(
                &store.commitments,
                b"nextConnectionSequence",
                &bcs::to_bytes<u64>(&0)
            );
        let next_sequence = from_bcs::to_u32(*next_sequence_bytes);
        table::upsert(
            &mut store.commitments,
            b"nextConnectionSequence",
            bcs::to_bytes(&(next_sequence + 1))
        );

        next_sequence
    }

    // Returns connection by `connection_id`. Aborts if the connection does not exist.
    #[view]
    public fun get_connection(connection_id: u32): Option<ConnectionEnd> acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());

        if (!smart_table::contains(&store.connections, connection_id)) {
            option::none<ConnectionEnd>()
        } else {
            option::some<ConnectionEnd>(
                *smart_table::borrow(&store.connections, connection_id)
            )
        }
    }

    #[view]
    public fun get_channel(channel_id: u32): Option<Channel> acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());

        if (!smart_table::contains(&store.channels, channel_id)) {
            option::none<Channel>()
        } else {
            option::some<Channel>(*smart_table::borrow(&store.channels, channel_id))
        }
    }

    fun ordering_to_string(ordering: u8): String {
        let return_val = string::utf8(b"ORDER_INVALID");
        if (ordering == 1) {
            return_val = string::utf8(b"ORDER_UNORDERED");
        } else if (ordering == 2) {
            return_val = string::utf8(b"ORDER_ORDERED");
        };
        return_val
    }

    fun generate_channel_identifier(): u32 acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let next_sequence_bytes =
            table::borrow_with_default(
                &store.commitments,
                b"nextChannelSequence",
                &bcs::to_bytes<u64>(&0)
            );
        let next_sequence = from_bcs::to_u32(*next_sequence_bytes);

        table::upsert(
            &mut store.commitments,
            b"nextChannelSequence",
            bcs::to_bytes(&(next_sequence + 1))
        );
        next_sequence
    }

    fun ensure_connection_state(connection_id: u32): u32 acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let connection = smart_table::borrow(&store.connections, connection_id);
        assert!(
            connection_end::state(connection) == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );

        connection_end::client_id(connection)
    }

    fun encode_channel(channel: Channel): vector<u8> {
        channel::encode(&channel)
    }

    fun encode_connection(connection: ConnectionEnd): vector<u8> {
        connection_end::encode(&connection)
    }

    fun commit_channel(channel_id: u32, channel: Channel) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let key = commitment::channel_commitment_key(channel_id);

        let encoded = encode_channel(channel);
        table::upsert(&mut store.commitments, key, encoded);
    }

    fun commit_connection(connection_id: u32, connection: ConnectionEnd) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let key = commitment::connection_commitment_key(connection_id);

        let encoded = encode_connection(connection);
        table::upsert(&mut store.commitments, key, encoded);
    }

    fun verify_channel_state(
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        channel_id: u32,
        channel: Channel
    ): u64 {
        light_client::verify_membership(
            client_id,
            height,
            proof,
            commitment::channel_commitment_key(channel_id),
            channel::encode(&channel)
        )
    }

    // Ensures that the channel state is open
    fun ensure_channel_state(channel_id: u32): Channel acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let channel = smart_table::borrow(&store.channels, channel_id);

        assert!(channel::state(channel) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);
        *channel
    }

    fun verify_absent_commitment(
        clientId: u32,
        height: u64,
        proof: vector<u8>,
        path: vector<u8>
    ): u64 {
        light_client::verify_non_membership(clientId, height, proof, path)
    }

    fun address_to_string(addr: address): String {
        string_utils::to_string(&bcs::to_bytes(&addr))
    }

    #[test]
    fun test_copyable_any() {
        let channel_id = 35;
        let channel_open_ = ChannelOpenConfirmParams { channel_id };
        let any_packed = copyable_any::pack<ChannelOpenConfirmParams>(channel_open_);

        let type_name_output = copyable_any::type_name(&any_packed);
        assert!(
            *type_name_output == std::type_info::type_name<ChannelOpenConfirmParams>(),
            0
        )
    }

    // #[test(ibc_signer = @ibc)]
    // fun test_get_ibc_signer(ibc_signer: &signer) acquires SignerRef {
    //     init_module(ibc_signer);

    //     std::debug::print(&get_ibc_signer())
    // }

    // #[test]
    // public fun test_address_to_string() {
    //     let addr = @0x0000000e8cb0f6fe55f8b91c16e970a1863552af09b60e6fe1d99808254b0be9;
    //     let str =
    //         utf8(b"0x0000000e8cb0f6fe55f8b91c16e970a1863552af09b60e6fe1d99808254b0be9");

    //     assert!(address_to_string(addr) == str, 1);
    // }

    // #[test]
    // public fun test_to_string() {
    //     let order_unordered: u8 = 1;
    //     let order_ordered: u8 = 2;
    //     let order_invalid: u8 = 3;
    //     // Test case for ORDER_UNORDERED
    //     let order_unordered = ordering_to_string(order_unordered);
    //     assert!(order_unordered == string::utf8(b"ORDER_UNORDERED"), 2001);

    //     // Test case for ORDER_ORDERED
    //     let order_ordered = ordering_to_string(order_ordered);
    //     assert!(order_ordered == string::utf8(b"ORDER_ORDERED"), 2002);

    //     // Test case for invalid order
    //     let order_invalid = ordering_to_string(order_invalid);
    //     assert!(order_invalid == string::utf8(b"ORDER_INVALID"), 2003);
    // }

    // #[test(alice = @ibc)]
    // public fun test_get_counterparty_hops(alice: &signer) acquires IBCStore {
    //     // Initialize IBCStore for testing
    //     init_module(alice);

    //     // Mock connection ID
    //     let connection_id = string::utf8(b"connection-0");

    //     // Prepare counterparty and connection
    //     let counterparty =
    //         connection_end::new_counterparty(
    //             string::utf8(b"counterparty-client"), connection_id, b""
    //         );
    //     let connection =
    //         connection_end::new(
    //             string::utf8(b"client-0"),
    //             vector::empty<connection_end::Version>(),
    //             3,
    //             0,
    //             counterparty
    //         );

    //     // Insert connection into the store
    //     set_connection(connection_id, connection);

    //     // Test get_counterparty_hops function
    //     let hops = get_counterparty_hops(connection_id);
    //     assert!(vector::length(&hops) == 1, 3001);
    //     assert!(*vector::borrow(&hops, 0) == connection_id, 3002);
    // }

    // #[test(alice = @ibc)]
    // public fun test_generate_channel_identifier(alice: &signer) acquires IBCStore {
    //     // Initialize IBCStore for testing
    //     init_module(alice);

    //     // Test generate_channel_identifier function
    //     let _ = generate_channel_identifier();

    //     // Verify the next sequence has incremented
    //     let next_sequence = get_next_channel_sequence();
    //     assert!(next_sequence == 1, 4002);
    // }

    // #[test(alice = @ibc)]
    // public fun test_ensure_connection_state(alice: &signer) acquires IBCStore {
    //     // Initialize IBCStore for testing
    //     init_module(alice);

    //     // Mock connection ID
    //     let connection_id = string::utf8(b"connection-0");

    //     // Prepare counterparty and connection
    //     let counterparty =
    //         connection_end::new_counterparty(
    //             string::utf8(b"counterparty-client"),
    //             connection_id,
    //             vector::empty<u8>()
    //         );
    //     let connection =
    //         connection_end::new(
    //             string::utf8(b"client-0"),
    //             vector::empty(),
    //             3,
    //             0,
    //             counterparty
    //         );

    //     set_connection(connection_id, connection);

    //     // Test ensure_connection_state function
    //     let retrieved_connection_end = ensure_connection_state(connection_id);
    //     assert!(connection_end::state(&retrieved_connection_end) == 3, 5002);
    //     assert!(
    //         *connection_end::client_id(&retrieved_connection_end)
    //             == string::utf8(b"client-0"),
    //         5003
    //     );
    // }

    // #[test(alice = @ibc)]
    // public fun test_ensure_connection_feature(alice: &signer) acquires IBCStore {
    //     // Initialize IBCStore for testing
    //     init_module(alice);

    //     // Mock connection ID
    //     let connection_id = string::utf8(b"connection-0");

    //     // Prepare counterparty and connection
    //     let features = vector::empty<String>();
    //     vector::push_back(&mut features, string::utf8(b"ORDER_ORDERED"));
    //     let version = connection_end::new_version(string::utf8(b"1"), features);
    //     let counterparty =
    //         connection_end::new_counterparty(
    //             string::utf8(b"counterparty-client"),
    //             connection_id,
    //             vector::empty<u8>()
    //         );
    //     let connection =
    //         connection_end::new(
    //             string::utf8(b"client-0"),
    //             vector::singleton(version),
    //             3,
    //             0,
    //             counterparty
    //         );

    //     set_connection(connection_id, connection);

    //     // Test ensure_connection_feature function
    //     let connection_hops = vector::singleton(connection_id);
    //     let order_ordered: u8 = 2;
    //     let (retrieved_connection_id, retrieved_connection_end) =
    //         ensure_connection_feature(connection_hops, order_ordered);
    //     assert!(retrieved_connection_id == connection_id, 6001);
    //     assert!(connection_end::state(&retrieved_connection_end) == 3, 6002);
    //     assert!(
    //         *connection_end::client_id(&retrieved_connection_end)
    //             == string::utf8(b"client-0"),
    //         6003
    //     );
    // }

    // #[
    //     test(
    //         alice = @ibc,
    //         ibc_app = @0xfe1adf0b572c9d480624c86b65345895929a36d8f3c0f7facc67921e7e7c395c
    //     )
    // ]
    // public fun test_channel_open_init(alice: &signer, ibc_app: &signer) acquires IBCStore {
    //     // Initialize IBCStore for testing
    //     init_module(alice);

    //     // Prepare a mock connection and set it in the IBCStore
    //     let client_id = string::utf8(b"client-0");
    //     let connection_id = string::utf8(b"connection-0");
    //     let counterparty =
    //         connection_end::new_counterparty(
    //             string::utf8(b"counterparty-client"),
    //             connection_id,
    //             b""
    //         );
    //     let connection =
    //         connection_end::new(
    //             client_id,
    //             vector::singleton(
    //                 connection_end::new_version(
    //                     string::utf8(b"1"),
    //                     vector::singleton(string::utf8(b"ORDER_ORDERED"))
    //                 )
    //             ),
    //             3, // STATE_OPEN
    //             0,
    //             counterparty
    //         );
    //     set_connection(connection_id, connection);

    //     // Prepare a mock channel
    //     let connection_hops = vector::singleton(connection_id);
    //     let counterparty =
    //         channel::new_counterparty(
    //             string::utf8(b"counterparty-port"), string::utf8(b"")
    //         );
    //     let version = string::utf8(b"1");
    //     let ordering = CHAN_ORDERING_ORDERED;

    //     // Call channel_open_init function

    //     let port_addr = signer::address_of(alice); // ALERT
    //     // PORT_ADDR HERE SHOULD BE @alice
    //     // because the "ibc_app" address is precompiled and assumed port_id will be alice !

    //     let (channel, _) =
    //         channel_open_init(
    //             ibc_app,
    //             port_addr,
    //             connection_hops,
    //             ordering,
    //             counterparty,
    //             version
    //         );
    //     // let channel_id = *channel::chan_id(&channel);ibc_app

    //     // Validate that the channel was added to the store
    //     let port_id = address_to_string(port_addr);
    //     let channel_id = string::utf8(b"channel-0");

    //     let stored_channel = get_channel(port_id, channel_id);

    //     // Validate that the stored channel matches the expected channel
    //     assert!(option::is_some(&stored_channel), 8001);
    //     let stored_channel_val = option::extract(&mut stored_channel);
    //     assert!(channel::state(&stored_channel_val) == CHAN_STATE_INIT, 8002);
    //     assert!(channel::ordering(&stored_channel_val) == CHAN_ORDERING_ORDERED, 8003);
    // }

    // #[
    //     test(
    //         alice = @ibc,
    //         ibc_app = @0xfe1adf0b572c9d480624c86b65345895929a36d8f3c0f7facc67921e7e7c395c
    //     )
    // ]
    // #[expected_failure(abort_code = 1017)]
    // // E_COUNTERPARTY_CHANNEL_NOT_EMPTY
    // public fun test_channel_open_init_non_empty_counterparty_channel_id(
    //     alice: &signer, ibc_app: &signer
    // ) acquires IBCStore {
    //     // Initialize IBCStore for testing
    //     init_module(alice);

    //     // Prepare a mock connection and set it in the IBCStore
    //     let client_id = 0;
    //     let connection_id = 0;
    //     let counterparty =
    //         connection_end::new_counterparty(
    //             1,
    //             connection_id,
    //             b""
    //         );
    //     let connection =
    //         connection_end::new(
    //             client_id,
    //             vector::singleton(
    //                 connection_end::new_version(
    //                     string::utf8(b"1"),
    //                     vector::singleton(string::utf8(b"ORDER_ORDERED"))
    //                 )
    //             ),
    //             3, // STATE_OPEN
    //             0,
    //             counterparty
    //         );
    //     set_connection(connection_id, connection);

    //     // Prepare a mock channel with a non-empty counterparty channel ID
    //     let connection_hops = vector::singleton(connection_id);
    //     let counterparty =
    //         channel::new_counterparty(
    //             string::utf8(b"counterparty-port"), string::utf8(b"channel-1")
    //         ); // Non-empty channel ID
    //     let channel =
    //         channel::new(
    //             CHAN_STATE_INIT,
    //             CHAN_ORDERING_ORDERED,
    //             counterparty,
    //             connection_hops,
    //             string::utf8(b"1")
    //         );

    //     // Insert channel into store
    //     set_channel(string::utf8(b"port-0"), string::utf8(b"channel-0"), channel);

    //     // Attempt to call channel_open_init with a non-empty counterparty channel ID, which should abort with E_COUNTERPARTY_CHANNEL_NOT_EMPTY
    //     let version = string::utf8(b"1");
    //     let port_addr = signer::address_of(alice);
    //     channel_open_init(
    //         ibc_app,
    //         port_addr,
    //         connection_hops,
    //         CHAN_ORDERING_ORDERED,
    //         counterparty,
    //         version
    //     );
    // }

    // #[test(alice = @ibc, ibc_app = @0xfe1adf0b572c9d480624c86b65345895929a36d8f3c0f7facc67921e7e7c395c)]
    // public fun test_channel_open_ack(alice: &signer, ibc_app: &signer) acquires IBCStore, SignerRef {
    //     // Initialize IBCStore for testing
    //     init_module(alice);

    //     let order_ordered: u8 = 2;

    //     // Prepare a mock connection and set it in the IBCStore
    //     let client_id = string::utf8(b"cometbls-0");
    //     let connection_id = string::utf8(b"connection-0");
    //     let counterparty = connection_end::new_counterparty(
    //         string::utf8(b"counterparty-client"),
    //         connection_id,
    //         b"",
    //     );
    //     let connection = connection_end::new(
    //         client_id,
    //         vector::singleton(connection_end::new_version(string::utf8(b"1"), vector::singleton(string::utf8(b"ORDER_ORDERED")))),
    //         3, // STATE_OPEN
    //         0,
    //         counterparty
    //     );
    //     set_connection(connection_id, connection);

    //     // Prepare a mock channel
    //     let connection_hops = vector::singleton(connection_id);
    //     let counterparty = channel::new_counterparty(string::utf8(b"counterparty-port"), string::utf8(b""));
    //     let channel = channel::new(1, order_ordered, counterparty, connection_hops, string::utf8(b"1"));

    //     // Call channel_open_init function to generate a channel ID
    //     let (channel, _) = channel_open_init(ibc_app, signer::address_of(alice), connection_hops, order_ordered, counterparty, string::utf8(b"1"));
    //     let channel_id = string::utf8(b"channel-0"); // Assuming channel-0 was generated

    //     // Prepare mock proof data
    //     let proof_height = height::new(0, 1);
    //     let proof_try = vector::empty<u8>();

    //     let (data1, data2) = light_client::mock_create_client();

    //     create_client(string::utf8(b"cometbls"), data1, data2);

    //     // Call channel_open_ack function
    //     channel_open_ack(
    //         ibc_app,
    //         signer::address_of(alice),
    //         channel_id,
    //         string::utf8(b"counterparty-channel-0"),
    //         string::utf8(b"counterparty-version-0"),
    //         proof_try,
    //         proof_height
    //     );

    //     // Validate that the channel state has been updated to STATE_OPEN
    //     let stored_channel = get_channel_from_store(string::utf8(b"port-0"), channel_id);
    //     assert!(channel::state(&stored_channel) == 3, 9001); // STATE_OPEN
    //     assert!(*channel::version(&stored_channel) == string::utf8(b"counterparty-version-0"), 9002);
    //     assert!(*channel::chan_counterparty_channel_id(&stored_channel) == string::utf8(b"counterparty-channel-0"), 9003);
    // }
}
