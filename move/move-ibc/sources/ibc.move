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

    use std::string_utils;
    use ibc::commitment;
    use ibc::light_client;
    use ibc::height::{Self, Height};
    use ibc::connection_end::{Self, ConnectionEnd};
    use ibc::channel::{Self, Channel};
    use ibc::packet::{Self, Packet};

    const IBC_APP_SEED: vector<u8> = b"union-ibc-app-v1";

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
    const E_INVALID_PROOF: u64 = 1010;
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

    #[event]
    struct ClientCreatedEvent has copy, drop, store {
        client_id: u32,
        client_type: String,
        consensus_height: Height
    }

    #[event]
    struct ClientUpdated has copy, drop, store {
        client_id: u32,
        client_type: String,
        height: Height
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
        version: vector<u8>
    }

    #[event]
    struct ChannelOpenTry has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_channel_id: u32,
        connection_id: u32,
        version: vector<u8>
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
        source_port: String,
        source_channel: String,
        timeout_height: height::Height,
        timeout_timestamp: u64,
        data: vector<u8>
    }

    #[event]
    struct RecvPacket has drop, store {
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

    struct ChannelPort has copy, drop, store {
        port_id: String,
        channel_id: String
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
            commitment::client_state_key(client_id),
            client_state
        );

        let latest_height = light_client::latest_height(client_id);

        table::upsert(
            &mut store.commitments,
            commitment::consensus_state_key(client_id, latest_height),
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
        let version = connection_end::new_version(version_identifier, version_features);
        let counterparty =
            connection_end::new_counterparty(
                counterparty_client_id,
                counterparty_connection_id,
                counterparty_prefix
            );

        assert!(light_client::status(client_id) == 0, E_CLIENT_NOT_ACTIVE);

        let connection_id = generate_connection_identifier();
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let connection =
            connection_end::new(
                client_id,
                vector::empty(),
                CONN_STATE_INIT,
                delay_period,
                counterparty
            );

        if (vector::is_empty(connection_end::version_features(&version))) {
            connection_end::set_versions(&mut connection, get_compatible_versions());
        } else {
            assert!(
                is_supported_version(&get_compatible_versions(), &version),
                E_UNSUPPORTED_VERSION
            );

            connection_end::set_versions(&mut connection, vector[version]);
        };

        smart_table::upsert(&mut store.connections, connection_id, connection);

        update_connection_commitment(store, connection_id, connection);

        event::emit(
            ConnectionOpenInit {
                connection_id: connection_id,
                client_id: client_id,
                counterparty_client_id: *connection_end::conn_counterparty_client_id(
                    &connection
                )
            }
        );
    }

    public entry fun connection_open_try(
        counterparty_client_id: u32,
        counterparty_connection_id: u32,
        counterparty_prefix: vector<u8>,
        delay_period: u64,
        client_id: u32,
        client_state_bytes: vector<u8>,
        counterparty_version_identifiers: vector<String>,
        counterparty_version_features: vector<vector<String>>,
        proof_init: vector<u8>,
        proof_client: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64
    ) acquires IBCStore {
        let counterparty =
            connection_end::new_counterparty(
                counterparty_client_id,
                counterparty_connection_id,
                counterparty_prefix
            );
        let counterparty_versions =
            connection_end::new_versions(
                counterparty_version_identifiers, counterparty_version_features
            );
        let proof_height =
            height::new(proof_height_revision_num, proof_height_revision_height);

        assert!(light_client::status(client_id) == 0, E_CLIENT_NOT_ACTIVE);

        // Generate a new connection identifier
        let connection_id = generate_connection_identifier();

        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        // Retrieve the connection from the store
        let connection =
            smart_table::borrow_mut_with_default(
                &mut store.connections,
                connection_id,
                connection_end::new(
                    client_id,
                    vector[pick_version(
                        &get_compatible_versions(), &counterparty_versions
                    )],
                    CONN_STATE_TRYOPEN,
                    delay_period,
                    counterparty
                )
            );

        // Create the expected connection
        let expected_connection =
            connection_end::new(
                *connection_end::conn_counterparty_client_id(connection),
                counterparty_versions,
                CONN_STATE_INIT,
                delay_period,
                connection_end::new_counterparty(client_id, string::utf8(b""), b"ibc")
            );

        // Verify the connection state
        let err =
            verify_connection_state(
                connection,
                proof_height,
                proof_init,
                *connection_end::counterparty_connection_id(&counterparty),
                expected_connection
            );
        assert!(err == 0, E_INVALID_PROOF);

        let counterparty_client_id =
            connection_end::conn_counterparty_client_id(connection);

        // Verify the client state
        let err =
            verify_client_state(
                connection,
                proof_height,
                commitment::client_state_key(*counterparty_client_id),
                proof_client,
                client_state_bytes
            );
        assert!(err == 0, E_INVALID_PROOF);

        event::emit(
            ConnectionOpenTry {
                connection_id,
                client_id: client_id,
                counterparty_client_id: *connection_end::conn_counterparty_client_id(
                    connection
                ),
                counterparty_connection_id: *connection_end::conn_counterparty_connection_id(
                    connection
                )
            }
        );

        update_connection_commitment(store, connection_id, *connection);
    }

    public entry fun connection_open_ack(
        connection_id: u32,
        client_state_bytes: vector<u8>,
        version_identifier: String,
        version_features: vector<String>,
        proof_try: vector<u8>,
        proof_client: vector<u8>,
        counterparty_connection_id: u32,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64
    ) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let version = connection_end::new_version(version_identifier, version_features);
        let proof_height =
            height::new(proof_height_revision_num, proof_height_revision_height);

        assert!(
            smart_table::contains(&store.connections, connection_id),
            E_CONNECTION_DOES_NOT_EXIST
        );

        let connection = smart_table::borrow_mut(&mut store.connections, connection_id);

        assert!(
            connection_end::state(connection) == CONN_STATE_INIT,
            E_INVALID_CONNECTION_STATE
        );

        assert!(
            is_supported_version(connection_end::versions(connection), &version),
            E_UNSUPPORTED_VERSION
        );

        let expected_counterparty =
            connection_end::new_counterparty(
                *connection_end::client_id(connection),
                connection_id,
                b"ibc"
            );

        let expected_connection =
            connection_end::new(
                *connection_end::conn_counterparty_client_id(connection),
                vector::singleton(version),
                CONN_STATE_TRYOPEN,
                connection_end::delay_period(connection),
                expected_counterparty
            );

        let err =
            verify_connection_state(
                connection,
                proof_height,
                proof_try,
                counterparty_connection_id,
                expected_connection
            );
        assert!(err == 0, err);

        let counterparty_client_id =
            *connection_end::conn_counterparty_client_id(connection);

        let err =
            verify_client_state(
                connection,
                proof_height,
                commitment::client_state_key(counterparty_client_id),
                proof_client,
                client_state_bytes
            );
        assert!(err == 0, err);

        connection_end::set_state(connection, CONN_STATE_OPEN);

        let conn_versions = *connection_end::versions(connection);
        copy_versions(&vector::singleton(version), &mut conn_versions);
        connection_end::set_versions(connection, conn_versions);
        connection_end::set_conn_counterparty_connection_id(
            connection, counterparty_connection_id
        );

        event::emit(
            ConnectionOpenAck {
                connection_id,
                client_id: *connection_end::client_id(connection),
                counterparty_client_id: *connection_end::conn_counterparty_client_id(
                    connection
                ),
                counterparty_connection_id: *connection_end::conn_counterparty_connection_id(
                    connection
                )
            }
        );

        update_connection_commitment(store, connection_id, *connection);
    }

    public entry fun connection_open_confirm(
        connection_id: u32,
        proof_ack: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64
    ) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let proof_height =
            height::new(proof_height_revision_num, proof_height_revision_height);

        let connection = smart_table::borrow_mut(&mut store.connections, connection_id);

        assert!(
            connection_end::state(connection) == CONN_STATE_TRYOPEN,
            E_INVALID_CONNECTION_STATE
        );

        let expected_counterparty =
            connection_end::new_counterparty(
                *connection_end::client_id(connection),
                connection_id,
                b"ibc"
            );

        let expected_connection =
            connection_end::new(
                *connection_end::conn_counterparty_client_id(connection),
                *connection_end::versions(connection),
                CONN_STATE_OPEN,
                connection_end::delay_period(connection),
                expected_counterparty
            );

        let counterparty_conn_id =
            *connection_end::conn_counterparty_connection_id(connection);

        let err =
            verify_connection_state(
                connection,
                proof_height,
                proof_ack,
                counterparty_conn_id,
                expected_connection
            );
        assert!(err == 0, err);

        connection_end::set_state(connection, CONN_STATE_OPEN);

        event::emit(
            ConnectionOpenConfirm {
                connection_id,
                client_id: *connection_end::client_id(connection),
                counterparty_client_id: *connection_end::conn_counterparty_client_id(
                    connection
                ),
                counterparty_connection_id: *connection_end::conn_counterparty_connection_id(
                    connection
                )
            }
        );

        update_connection_commitment(store, connection_id, *connection);
    }

    public entry fun update_client(
        client_id: u32, client_message: vector<u8>
    ) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        assert!(
            table::contains(
                &store.commitments, commitment::client_state_key(client_id)
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
            commitment::client_state_key(client_id),
            client_state
        );

        let i = 0;
        while (i < heights_len) {
            let height = *vector::borrow(&heights, i);

            table::upsert(
                &mut store.commitments,
                commitment::consensus_state_key(client_id, height),
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
                &store.commitments, commitment::client_state_key(client_id)
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

    public fun initialize_channel_sequences(channel_id: u32) acquires IBCStore {
        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_send_key(port_id, channel_id),
            bcs::to_bytes(&1)
        );

        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_recv_key(port_id, channel_id),
            bcs::to_bytes(&1)
        );

        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_ack_key(port_id, channel_id),
            bcs::to_bytes(&1)
        );

    }
    public fun channel_open_init(
        ibc_app: &signer, // this is the caller which should be the `ibc_app`
        port_id: address,
        connection_id: u32,
        ordering: u8,
        version: vector<u8>
    ): (Channel, u64) acquires IBCStore {
        authorize_app(ibc_app, port_id);

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
            commitment::next_sequence_send_key(port_id, channel_id),
            bcs::to_bytes(&1)
        );

        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_recv_key(port_id, channel_id),
            bcs::to_bytes(&1)
        );

        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_ack_key(port_id, channel_id),
            bcs::to_bytes(&1)
        );

        commit_channel(channel_id, channel);

        event::emit(
            ChannelOpenInit {
                port_id: port_id,
                channel_id: channel_id,
                counterparty_port_id: *channel::chan_counterparty_port_id(&channel),
                connection_id: connection_id,
                version: *channel::version(&channel)
            }
        );

        (channel, 0)
    }

    public fun channel_open_try(
        ibc_app: &signer,
        port_id: address,
        channel_state: u8,
        channel_order: u8,
        connection_id: u32,
        counterparty_channel_id: u32,
        version: vector<u8>,
        counterparty_version: vector<u8>,
        proof_init: vector<u8>,
        proof_height: height::Height
    ): (Channel, u64) acquires IBCStore {
        authorize_app(ibc_app, port_id);

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
        assert!(err == 0, E_INVALID_PROOF);

        let channel_id = generate_channel_identifier();

        event::emit(
            ChannelOpenTry {
                port_id,
                channel_id,
                counterparty_channel_id,
                connection_id,
                counterparty_version
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
            commitment::next_sequence_send_key(port_id, channel_id),
            bcs::to_bytes(&1)
        );

        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_recv_key(port_id, channel_id),
            bcs::to_bytes(&1)
        );

        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_ack_key(port_id, channel_id),
            bcs::to_bytes(&1)
        );

        commit_channel(channel_id, channel);

        (channel, 0)
    }

    // TODO(aeryz): should we verify the caller here?
    public fun channel_open_ack(
        ibc_app: &signer, // this is the caller which should be the `ibc_app`
        port_id: address,
        channel_id: u32,
        counterparty_version: vector<u8>,
        counterparty_channel_id: u32,
        proof_try: vector<u8>,
        proof_height: height::Height
    ) acquires IBCStore {
        authorize_app(ibc_app, port_id);

        let chan =
            *smart_table::borrow(
                &borrow_global<IBCStore>(get_vault_addr()).channels,
                channel_id
            );

        assert!(channel::state(&chan) == CHAN_STATE_INIT, E_INVALID_CHANNEL_STATE);

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
        assert!(err == 0, E_INVALID_PROOF);

        // TODO: Not sure if this upsert is required or not?
        smart_table::upsert(
            &mut borrow_global_mut<IBCStore>(get_vault_addr()).channels,
            channel_id,
            channel
        );
        channel::set_state(&mut chan, CHAN_STATE_OPEN);
        channel::set_version(&mut chan, counterparty_version);
        channel::set_chan_counterparty_channel_id(&mut chan, counterparty_channel_id);

        event::emit(
            ChannelOpenAck {
                port_id,
                channel_id,
                counterparty_port_id: *channel::chan_counterparty_port_id(&chan),
                counterparty_channel_id,
                connection_id: *vector::borrow(channel::connection_hops(&chan), 0)
            }
        );

        commit_channel(channel_id, chan);
    }

    public fun channel_open_confirm(
        ibc_app: &signer,
        port_id: address,
        channel_id: u32,
        proof_ack: vector<u8>,
        proof_height: height::Height
    ) acquires IBCStore {
        authorize_app(ibc_app, port_id);

        let chan =
            *smart_table::borrow(
                &borrow_global<IBCStore>(get_vault_addr()).channels,
                channel_id
            );

        assert!(channel::state(&chan) == CHAN_STATE_TRYOPEN, E_INVALID_CHANNEL_STATE);

        let port_id = address_to_string(port_id);

        let client_id = ensure_connection_state(connection_id);

        let expected_channel =
            channel::new(
                CHAN_STATE_OPEN,
                channel_order,
                get_counterparty_connection(connection_id),
                counterparty_channel_id,
                counterparty_version
            );

        let err =
            verify_channel_state(
                client_id,
                proof_height,
                proof_ack,
                counterparty_channel_id,
                expected_channel
            );
        assert!(err == 0, err);

        channel::set_state(&mut chan, CHAN_STATE_OPEN);

        // TODO: Not sure if this upsert is required or not?
        smart_table::upsert(
            &mut borrow_global_mut<IBCStore>(get_vault_addr()).channels,
            channel_id,
            channel
        );

        event::emit(
            ChannelOpenConfirm {
                port_id,
                channel_id,
                counterparty_port_id: *channel::chan_counterparty_port_id(&channel),
                counterparty_channel_id: *channel::chan_counterparty_channel_id(&channel),
                connection_id: *vector::borrow(channel::connection_hops(&channel), 0)
            }
        );
        commit_channel(channel_id, chan);
    }

    // Sends a packet
    public fun send_packet(
        ibc_app: &signer,
        source_port: address,
        source_channel: String,
        timeout_height: height::Height,
        timeout_timestamp: u64,
        data: vector<u8>
    ): u64 acquires IBCStore {
        authorize_app(ibc_app, source_port);

        let source_port = address_to_string(source_port);

        let channel = ensure_channel_state(source_port, source_channel);

        let connection_id = *vector::borrow(channel::connection_hops(&channel), 0);

        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let client_id =
            *connection_end::client_id(
                smart_table::borrow(&store.connections, connection_id)
            );

        let latest_height = light_client::latest_height(client_id);

        assert!(
            height::get_revision_height(&latest_height) != 0, E_LATEST_HEIGHT_NOT_FOUND
        );

        if (!height::is_zero(&timeout_height)) {
            assert!(
                height::lt(&latest_height, &timeout_height), E_INVALID_TIMEOUT_HEIGHT
            );
        };

        let latest_timestamp =
            light_client::get_timestamp_at_height(client_id, latest_height);
        assert!(latest_timestamp != 0, E_LATEST_TIMESTAMP_NOT_FOUND);
        if (timeout_timestamp != 0) {
            assert!(latest_timestamp < timeout_timestamp, E_INVALID_TIMEOUT_TIMESTAMP);
        };

        let packet_sequence =
            from_bcs::to_u64(
                *table::borrow(
                    &store.commitments,
                    commitment::next_sequence_send_key(source_port, source_channel)
                )
            );
        table::upsert(
            &mut store.commitments,
            commitment::next_sequence_send_key(source_port, source_channel),
            bcs::to_bytes(&(packet_sequence + 1))
        );

        table::upsert(
            &mut store.commitments,
            commitment::packet_key(source_port, source_channel, packet_sequence),
            packet::commitment_from_parts(timeout_timestamp, timeout_height, data)
        );

        event::emit(
            SendPacket {
                sequence: packet_sequence,
                source_port,
                source_channel,
                timeout_height,
                timeout_timestamp,
                data
            }
        );

        packet_sequence
    }

    /// Receives and processes an IBC packet
    ///
    /// Note that any sanity check failures will result in this function to be aborted in order for caller's
    /// storage to be reverted. This will result in acks won't be able to written.
    public fun recv_packet(
        ibc_app: &signer,
        port_id: address,
        packet: Packet,
        proof: vector<u8>,
        proof_height: height::Height,
        acknowledgement: vector<u8>
    ) acquires IBCStore {
        authorize_app(ibc_app, port_id);

        let channel =
            ensure_channel_state(
                *packet::destination_port(&packet),
                *packet::destination_channel(&packet)
            );

        let port_id = address_to_string(port_id);
        assert!(port_id == *packet::destination_port(&packet), E_UNAUTHORIZED);

        assert!(
            packet::source_port(&packet)
                == channel::chan_counterparty_port_id(&channel),
            E_SOURCE_AND_COUNTERPARTY_PORT_MISMATCH
        );

        assert!(
            packet::source_channel(&packet)
                == channel::chan_counterparty_channel_id(&channel),
            E_SOURCE_AND_COUNTERPARTY_CHANNEL_MISMATCH
        );

        let connection_hop = *vector::borrow(channel::connection_hops(&channel), 0);
        let store = borrow_global<IBCStore>(get_vault_addr());

        let connection = smart_table::borrow(&store.connections, connection_hop);

        assert!(
            connection_end::state(connection) == CONN_STATE_OPEN,
            E_INVALID_CONNECTION_STATE
        );

        if (height::get_revision_height(&packet::timeout_height(&packet)) != 0) {
            assert!(
                block::get_current_block_height()
                    < height::get_revision_height(&packet::timeout_height(&packet)),
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

        let err =
            verify_commitment(
                connection,
                proof_height,
                proof,
                commitment::packet_key(
                    *packet::source_port(&packet),
                    *packet::source_channel(&packet),
                    packet::sequence(&packet)
                ),
                packet::commitment(&packet)
            );

        assert!(err == 0, err);

        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        if (channel::ordering(&channel) == CHAN_ORDERING_UNORDERED) {
            let receipt_commitment_key =
                commitment::packet_receipt_key(
                    *packet::destination_port(&packet),
                    *packet::destination_channel(&packet),
                    packet::sequence(&packet)
                );
            let receipt =
                table::borrow_with_default(
                    &store.commitments, receipt_commitment_key, &bcs::to_bytes(&0u8)
                );
            assert!(*receipt == bcs::to_bytes(&0u8), E_PACKET_ALREADY_RECEIVED);
            table::upsert(
                &mut store.commitments, receipt_commitment_key, bcs::to_bytes(&1u8)
            );
        } else if (channel::ordering(&channel) == CHAN_ORDERING_ORDERED) { // ORDER_ORDERED
            let expected_recv_sequence =
                from_bcs::to_u64(
                    *table::borrow_with_default(
                        &store.commitments,
                        commitment::next_sequence_recv_key(
                            *packet::destination_port(&packet),
                            *packet::destination_channel(&packet)
                        ),
                        &bcs::to_bytes(&0u64)
                    )
                );
            assert!(
                expected_recv_sequence == packet::sequence(&packet),
                E_PACKET_SEQUENCE_NEXT_SEQUENCE_MISMATCH
            );
            table::upsert(
                &mut store.commitments,
                commitment::next_sequence_recv_key(
                    *packet::destination_port(&packet),
                    *packet::destination_channel(&packet)
                ),
                bcs::to_bytes(&(expected_recv_sequence + 1))
            );
        } else {
            abort E_UNKNOWN_CHANNEL_ORDERING
        };

        if (vector::length(&acknowledgement) > 0) {
            write_acknowledgement(packet, acknowledgement);
        };

        event::emit(RecvPacket { packet: packet });
    }

    public fun write_acknowledgement(
        packet: packet::Packet, acknowledgement: vector<u8>
    ) acquires IBCStore {
        assert!(!vector::is_empty(&acknowledgement), E_ACKNOWLEDGEMENT_IS_EMPTY);

        ensure_channel_state(
            *packet::destination_port(&packet), *packet::destination_channel(&packet)
        );

        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let ack_commitment_key =
            commitment::packet_acknowledgement_key(
                *packet::destination_port(&packet),
                *packet::destination_channel(&packet),
                packet::sequence(&packet)
            );
        let ack_commitment =
            table::borrow_with_default(
                &store.commitments,
                ack_commitment_key,
                &bcs::to_bytes(&0u8)
            );
        assert!(*ack_commitment == bcs::to_bytes(&0u8), E_ACKNOWLEDGEMENT_ALREADY_EXISTS);
        table::upsert(
            &mut store.commitments, ack_commitment_key, hash::sha2_256(acknowledgement)
        );

        event::emit(WriteAcknowledgement { packet, acknowledgement });
    }

    public fun acknowledge_packet(
        ibc_app: &signer,
        port_id: address,
        packet: packet::Packet,
        acknowledgement: vector<u8>,
        proof: vector<u8>,
        proof_height: height::Height
    ) acquires IBCStore {
        authorize_app(ibc_app, port_id);

        let source_port_id = *packet::source_port(&packet);
        let source_channel_id = *packet::source_channel(&packet);

        let port_id = address_to_string(port_id);
        assert!(port_id == *packet::source_port(&packet), E_UNAUTHORIZED);

        let destination_port_id = *packet::destination_port(&packet);
        let destination_channel_id = *packet::destination_channel(&packet);

        let channel = ensure_channel_state(source_port_id, source_channel_id);

        assert!(
            destination_port_id == *channel::chan_counterparty_port_id(&channel),
            E_DESTINATION_AND_COUNTERPARTY_PORT_MISMATCH
        );

        assert!(
            destination_channel_id == *channel::chan_counterparty_channel_id(&channel),
            E_DESTINATION_AND_COUNTERPARTY_CHANNEL_MISMATCH
        );

        let connection =
            ensure_connection_state(
                *vector::borrow(channel::connection_hops(&channel), 0)
            );

        let packet_commitment_key =
            commitment::packet_key(
                source_port_id, source_channel_id, packet::sequence(&packet)
            );
        let expected_packet_commitment = get_commitment(packet_commitment_key);

        assert!(
            !vector::is_empty(&expected_packet_commitment),
            E_PACKET_COMMITMENT_NOT_FOUND
        );

        assert!(
            expected_packet_commitment == packet::commitment(&packet),
            E_INVALID_PACKET_COMMITMENT
        );

        let err =
            verify_commitment(
                &connection,
                proof_height,
                proof,
                commitment::packet_acknowledgement_key(
                    destination_port_id,
                    destination_channel_id,
                    packet::sequence(&packet)
                ),
                hash::sha2_256(acknowledgement)
            );

        if (err != 0) {
            abort err
        };

        if (channel::ordering(&channel) == CHAN_ORDERING_ORDERED) {
            let expected_ack_sequence =
                from_bcs::to_u64(
                    *table::borrow_with_default(
                        &borrow_global<IBCStore>(get_vault_addr()).commitments,
                        commitment::next_sequence_ack_key(
                            source_port_id, source_channel_id
                        ),
                        &bcs::to_bytes(&0u64)
                    )
                );

            assert!(
                expected_ack_sequence == packet::sequence(&packet),
                E_PACKET_SEQUENCE_NEXT_SEQUENCE_MISMATCH
            );

            table::upsert(
                &mut borrow_global_mut<IBCStore>(get_vault_addr()).commitments,
                commitment::next_sequence_ack_key(source_port_id, source_channel_id),
                bcs::to_bytes(&(expected_ack_sequence + 1))
            );
        };

        table::remove(
            &mut borrow_global_mut<IBCStore>(get_vault_addr()).commitments,
            packet_commitment_key
        );

        event::emit(AcknowledgePacket { packet, acknowledgement });
    }

    public fun timeout_packet(
        ibc_app: &signer,
        port_id: address,
        packet: Packet,
        proof: vector<u8>,
        proof_height: height::Height,
        next_sequence_recv: u64
    ) acquires IBCStore {
        authorize_app(ibc_app, port_id);

        let channel_id = *packet::source_channel(&packet);

        let port_id = address_to_string(port_id);
        let channel = ensure_channel_state(port_id, channel_id);

        assert!(
            *packet::destination_port(&packet)
                == *channel::chan_counterparty_port_id(&channel),
            E_DESTINATION_AND_COUNTERPARTY_PORT_MISMATCH
        );
        assert!(
            *packet::destination_channel(&packet)
                == *channel::chan_counterparty_channel_id(&channel),
            E_DESTINATION_AND_COUNTERPARTY_CHANNEL_MISMATCH
        );

        let connection_hop = *vector::borrow(channel::connection_hops(&channel), 0);
        let connection = ensure_connection_state(connection_hop);

        let packet_commitment_key =
            commitment::packet_key(
                *packet::source_port(&packet),
                *packet::source_channel(&packet),
                packet::sequence(&packet)
            );
        let expected_packet_commitment = get_commitment(packet_commitment_key);
        assert!(
            !vector::is_empty(&expected_packet_commitment),
            E_PACKET_COMMITMENT_NOT_FOUND
        );

        let packet_commitment = packet::commitment(&packet);
        assert!(
            expected_packet_commitment == packet_commitment,
            E_INVALID_PACKET_COMMITMENT
        );

        let proof_timestamp =
            light_client::get_timestamp_at_height(
                *connection_end::client_id(&connection),
                proof_height
            );
        assert!(proof_timestamp != 0, E_LATEST_TIMESTAMP_NOT_FOUND);

        if (packet::timeout_timestamp(&packet) != 0) {
            assert!(
                packet::timeout_timestamp(&packet) < proof_timestamp,
                E_TIMESTAMP_TIMEOUT_NOT_REACHED
            );
        };

        if (!height::is_zero(&packet::timeout_height(&packet))) {
            assert!(
                height::lt(&packet::timeout_height(&packet), &proof_height),
                E_TIMEOUT_HEIGHT_NOT_REACHED
            );
        };

        if (channel::ordering(&channel) == CHAN_ORDERING_ORDERED) {
            assert!(
                next_sequence_recv > packet::sequence(&packet),
                E_NEXT_SEQUENCE_MUST_BE_GREATER_THAN_TIMEOUT_SEQUENCE
            );
            let err =
                verify_commitment(
                    &connection,
                    proof_height,
                    proof,
                    commitment::next_sequence_recv_key(
                        *packet::destination_port(&packet),
                        *packet::destination_channel(&packet)
                    ),
                    bcs::to_bytes(&next_sequence_recv)
                );
            assert!(err == 0, err);
            channel::set_state(&mut channel, CHAN_STATE_CLOSED);
        } else if (channel::ordering(&channel) == CHAN_ORDERING_UNORDERED) {
            let err =
                verify_absent_commitment(
                    &connection,
                    proof_height,
                    proof,
                    commitment::packet_receipt_key(
                        *packet::destination_port(&packet),
                        *packet::destination_channel(&packet),
                        packet::sequence(&packet)
                    )
                );
            assert!(err == 0, err);
        } else {
            abort E_UNKNOWN_CHANNEL_ORDERING
        };

        table::remove(
            &mut borrow_global_mut<IBCStore>(get_vault_addr()).commitments,
            packet_commitment_key
        );

        event::emit(TimeoutPacket { packet });
    }

    // ========= UTILS and VIEW functions ========= //

    #[view]
    public fun client_state(client_id: u32): vector<u8> {
        light_client::get_client_state(client_id)
    }

    #[view]
    public fun consensus_state(
        client_id: u32, revision_number: u64, revision_height: u64
    ): vector<u8> {
        light_client::get_consensus_state(
            client_id, height::new(revision_number, revision_height)
        )
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
        smart_table::borrow(&store.connections, connection_id)

    }

    fun set_connection(connection_id: u32, connection: ConnectionEnd) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        smart_table::upsert(&mut store.connections, connection_id, connection);
    }

    fun set_channel(
        port_id: String, channel_id: String, channel: Channel
    ) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let channel_port = ChannelPort { port_id, channel_id };

        smart_table::upsert(&mut store.channels, channel_port, channel);
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
    fun get_channel_from_store(key: String, channel_id: String): Channel acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let channel_port = ChannelPort { port_id: key, channel_id };
        let channel = smart_table::borrow(&store.channels, channel_port);

        *channel
    }

    // Setter for nextChannelSequence in Commitments
    fun set_next_channel_sequence(sequence: u64) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        table::upsert(
            &mut store.commitments, b"nextChannelSequence", bcs::to_bytes(&sequence)
        );
    }

    // Function to generate a client identifier
    fun generate_client_identifier(): u32 acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let next_sequence =
            table::borrow_with_default(
                &store.commitments, b"nextClientSequence", &bcs::to_bytes<u64>(&0u64)
            );
        let next_sequence = from_bcs::to_u64(*next_sequence);

        table::upsert(
            &mut store.commitments,
            b"nextClientSequence",
            bcs::to_bytes<u64>(&(next_sequence + 1))
        );
        next_sequence
    }

    fun get_ibc_signer(): signer acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        object::generate_signer_for_extending(&vault.self_ref)
    }

    fun default_ibc_version(): connection_end::Version {
        connection_end::new_version(
            string::utf8(b"1"),
            vector[string::utf8(b"ORDER_ORDERED"), string::utf8(b"ORDER_UNORDERED")]
        )
    }

    fun set_supported_versions(
        supported_versions: vector<connection_end::Version>,
        dst: &mut vector<connection_end::Version>
    ) {
        assert!(vector::length(dst) == 0, E_VERSION_MUST_BE_UNSET);
        vector::append(dst, supported_versions);
    }

    fun is_supported_version(
        supported_versions: &vector<connection_end::Version>,
        version: &connection_end::Version
    ): bool {
        let (supported_version, found) =
            find_supported_version(supported_versions, version);
        if (found && verify_proposed_version(&supported_version, version)) {
            return true
        };
        false
        // found && verify_proposed_version(&supported_version, version)
    }

    fun get_feature_set_intersection(
        source_features: &vector<String>, counterparty_features: &vector<String>
    ): vector<String> {
        let feature_set = vector::empty<String>();
        let source_len = vector::length(source_features);
        let i = 0;
        while (i < source_len) {
            let feature = vector::borrow(source_features, i);
            if (vector::contains(counterparty_features, feature)) {
                vector::push_back(&mut feature_set, *feature);
            };
            i = i + 1;
        };
        feature_set
    }

    fun pick_version(
        supported_versions: &vector<connection_end::Version>,
        counterparty_versions: &vector<connection_end::Version>
    ): connection_end::Version {
        let supported_len = vector::length(supported_versions);
        let i = 0;
        while (i < supported_len) {
            let supported_version = vector::borrow(supported_versions, i);
            let (counterparty_version, found) =
                find_supported_version(counterparty_versions, supported_version);
            if (found) {
                let feature_set =
                    get_feature_set_intersection(
                        connection_end::version_features(supported_version),
                        connection_end::version_features(&counterparty_version)
                    );
                if (vector::length(&feature_set) > 0) {
                    return connection_end::new_version(
                        *connection_end::version_identifier(supported_version),
                        feature_set
                    )
                };
            };
            i = i + 1;
        };
        abort E_UNSUPPORTED_VERSION
    }

    fun copy_version(
        src: &connection_end::Version, dst: &mut connection_end::Version
    ) {
        connection_end::set_version_identifier(
            dst, *connection_end::version_identifier(src)
        );
        let src_len = vector::length(connection_end::version_features(src));
        let dst_len = vector::length(connection_end::version_features(dst));
        let i = 0;
        let dst_features = connection_end::version_features_mut(dst);
        while (i < src_len) {
            if (i < dst_len) {
                let src_feature = vector::borrow(
                    connection_end::version_features(src), i
                );
                let dst_feature = vector::borrow_mut(dst_features, i);
                *dst_feature = *src_feature;
            } else {
                let src_feature = vector::borrow(
                    connection_end::version_features(src), i
                );
                vector::push_back(dst_features, *src_feature);
            };
            i = i + 1;
        };
        while (i < dst_len) {
            vector::remove(dst_features, i);
            i = i + 1;
        }
    }

    fun copy_versions(
        src: &vector<connection_end::Version>, dst: &mut vector<connection_end::Version>
    ) {
        let src_len = vector::length(src);
        let dst_len = vector::length(dst);
        if (src_len == dst_len) {
            let i = 0;
            while (i < src_len) {
                let src_version = vector::borrow(src, i);
                let dst_version = vector::borrow_mut(dst, i);
                copy_version(src_version, dst_version);
                i = i + 1;
            };
        } else if (src_len > dst_len) {
            let i = 0;
            while (i < dst_len) {
                let src_version = vector::borrow(src, i);
                let dst_version = vector::borrow_mut(dst, i);
                copy_version(src_version, dst_version);
                i = i + 1;
            };
            let j = dst_len;
            while (j < src_len) {
                let src_version = vector::borrow(src, j);
                vector::push_back(dst, *src_version);
                j = j + 1;
            };
        } else {
            let i = 0;
            while (i < src_len) {
                let src_version = vector::borrow(src, i);
                let dst_version = vector::borrow_mut(dst, i);
                copy_version(src_version, dst_version);
                i = i + 1;
            };
            let j = src_len;
            while (j < dst_len) {
                vector::remove(dst, j);
                j = j + 1;
            };
        }
    }

    fun find_supported_version(
        supported_versions: &vector<connection_end::Version>,
        version: &connection_end::Version
    ): (connection_end::Version, bool) {
        let found_version = connection_end::default_version();
        let found = false;
        let len_supported_versions = vector::length(supported_versions);
        let i = 0;
        while (i < len_supported_versions) {
            let v = vector::borrow(supported_versions, i);
            if (connection_end::version_identifier(v)
                == connection_end::version_identifier(version)) {
                found_version = *v;
                found = true;
                break
            };
            i = i + 1;
        };
        (found_version, found)
    }

    fun verify_proposed_version(
        supported_version: &connection_end::Version,
        proposed_version: &connection_end::Version
    ): bool {
        let is_supported = false;
        if (connection_end::version_identifier(supported_version)
            == connection_end::version_identifier(proposed_version)) {
            let len_proposed_version = vector::length(
                connection_end::version_features(proposed_version)
            );
            let i = 0;
            while (i < len_proposed_version) {
                let feature = vector::borrow(
                    connection_end::version_features(proposed_version), i
                );
                is_supported = vector::contains(
                    connection_end::version_features(supported_version), feature
                );
                if (!is_supported) { break };
                i = i + 1;
            };
        };
        is_supported
    }

    fun verify_client_state(
        connection: &ConnectionEnd,
        height: height::Height,
        path: vector<u8>,
        proof: vector<u8>,
        client_state_bytes: vector<u8>
    ): u64 {
        light_client::verify_membership(
            *connection_end::client_id(connection),
            height,
            proof,
            *connection_end::conn_counterparty_key_prefix(connection),
            path,
            client_state_bytes
        )
    }

    fun verify_connection_state(
        connection: &ConnectionEnd,
        height: height::Height,
        proof: vector<u8>,
        connection_id: u32,
        counterparty_connection: ConnectionEnd
    ): u64 {
        light_client::verify_membership(
            *connection_end::client_id(connection),
            height,
            proof,
            *connection_end::conn_counterparty_key_prefix(connection),
            commitment::connection_key(connection_id),
            connection_end::encode_proto(counterparty_connection)
        )
    }

    public fun verify_commitment(
        connection: &ConnectionEnd,
        height: Height,
        proof: vector<u8>,
        path: vector<u8>,
        commitment: vector<u8>
    ): u64 {
        light_client::verify_membership(
            *connection_end::client_id(connection),
            height,
            proof,
            *connection_end::conn_counterparty_key_prefix(connection),
            path,
            commitment
        )
    }

    public fun generate_connection_identifier(): u32 acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let next_sequence_bytes =
            table::borrow_with_default(
                &store.commitments,
                b"nextConnectionSequence",
                &bcs::to_bytes<u64>(&0)
            );
        let next_sequence = from_bcs::to_u64(*next_sequence_bytes);
        table::upsert(
            &mut store.commitments,
            b"nextConnectionSequence",
            bcs::to_bytes(&(next_sequence + 1))
        );

        next_sequence
    }

    public fun update_connection_commitment(
        store: &mut IBCStore, connection_id: u32, connection: ConnectionEnd
    ) {
        let encoded_connection = connection_end::encode_proto(connection);
        let key = commitment::connection_key(connection_id);
        // let hash = hash::sha2_256(encoded_connection);
        table::upsert(&mut store.commitments, key, encoded_connection);
    }

    public fun get_compatible_versions(): vector<connection_end::Version> {
        vector[default_ibc_version()]
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
    public fun get_channel(port_id: String, channel_id: String): Option<Channel> acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());

        if (!smart_table::contains(
            &store.channels,
            ChannelPort { port_id, channel_id }
        )) {
            option::none<Channel>()
        } else {
            option::some<Channel>(
                *smart_table::borrow(&store.channels, ChannelPort { port_id, channel_id })
            )
        }
    }

    #[view]
    public fun get_next_sequence_recv(port_id: String, channel_id: String): u64 acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());

        let seq =
            table::borrow_with_default(
                &store.commitments,
                commitment::next_sequence_recv_key(port_id, channel_id),
                &bcs::to_bytes<u64>(&0)
            );

        from_bcs::to_u64(*seq)
    }

    #[view]
    public fun get_next_sequence_send(port_id: String, channel_id: String): u64 acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());

        let seq =
            table::borrow_with_default(
                &store.commitments,
                commitment::next_sequence_send_key(port_id, channel_id),
                &bcs::to_bytes<u64>(&0)
            );

        from_bcs::to_u64(*seq)
    }

    public fun verify_supported_feature(
        version: &connection_end::Version, feature: String
    ): bool {
        let i = 0;
        while (i < vector::length(connection_end::version_features(version))) {
            if (*vector::borrow(connection_end::version_features(version), i) == feature) {
                return true
            };
            i = i + 1;
        };
        false
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

    fun get_counterparty_hops(connection_id: u32): vector<String> acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let connection = smart_table::borrow(&store.connections, connection_id);
        let hops = vector::empty<String>();
        vector::push_back(
            &mut hops, *connection_end::conn_counterparty_connection_id(connection)
        );
        hops
    }

    fun generate_channel_identifier(): u32 acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let next_sequence_bytes =
            table::borrow_with_default(
                &store.commitments,
                b"nextChannelSequence",
                &bcs::to_bytes<u64>(&0)
            );
        let next_sequence = from_bcs::to_u64(*next_sequence_bytes);

        table::upsert(
            &mut store.commitments,
            b"nextChannelSequence",
            bcs::to_bytes(&new_sequence + 1)
        );
        new_sequence
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

    // TODO(aeryz): borrow instead of copy
    fun ensure_connection_feature(
        connection_hops: vector<String>, ordering: u8
    ): (String, ConnectionEnd) acquires IBCStore {
        assert!(vector::length(&connection_hops) == 1, E_CONN_NOT_SINGLE_HOP);
        let connection_id = *vector::borrow(&connection_hops, 0);
        let connection = ensure_connection_state(connection_id);
        assert!(
            vector::length(connection_end::versions(&connection)) == 1,
            E_CONN_NOT_SINGLE_VERSION
        );
        let version = *vector::borrow(connection_end::versions(&connection), 0);
        assert!(
            verify_supported_feature(&version, ordering_to_string(ordering)),
            E_UNSUPPORTED_FEATURE
        );
        (connection_id, connection)
    }

    fun encode_channel(channel: IBCChannel): vector<u8> {
        channel::encode(channel)
    }

    fun commit_channel(channel_id: u32, channel: IBCChannel) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let key = commitment::channel_commitment_key(channel_id);

        let encoded = encode_channel(channel);
        table::upsert(&mut store.commitments, key, encoded);
    }

    fun update_channel_commitment(port_id: String, channel_id: String) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let channel_port = ChannelPort { port_id: port_id, channel_id };
        let channel = smart_table::borrow(&store.channels, channel_port);

        let encoded = channel::encode_proto(*channel);
        let key = commitment::channel_key(port_id, channel_id);
        table::upsert(&mut store.commitments, key, encoded);
    }

    fun verify_channel_state(
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        channel_id: u32,
        channel: Channel
    ): u64 {
        0
        // TODO: Add verify
        // let client_id = connection_end::client_id(connection);
        // std::debug::print(&string::utf8(b"client_id"));
        // std::debug::print(client_id);
        // std::debug::print(&string::utf8(b"connection"));
        // std::debug::print(connection);
        // let path = commitment::channel_key(port_id, channel_id);
        // light_client::verify_membership(
        //     *connection_end::client_id(connection),
        //     height,
        //     proof,
        //     *connection_end::conn_counterparty_key_prefix(connection),
        //     path,
        //     channel::encode(channel)
        // )
    }

    // Ensures that the channel state is open
    fun ensure_channel_state(port_id: String, channel_id: String): Channel acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let channel_port = ChannelPort { port_id, channel_id };
        let channel = smart_table::borrow(&store.channels, channel_port);

        assert!(channel::state(channel) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);
        *channel
    }

    fun verify_absent_commitment(
        connection: &ConnectionEnd,
        height: height::Height,
        proof: vector<u8>,
        path: vector<u8>
    ): u64 {
        light_client::verify_non_membership(
            *connection_end::client_id(connection),
            height,
            proof,
            *connection_end::conn_counterparty_key_prefix(connection),
            path
        )
    }

    fun address_to_string(addr: address): String {
        string_utils::to_string(&bcs::to_bytes(&addr))
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
