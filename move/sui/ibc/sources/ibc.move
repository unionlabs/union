/*
/// Module: ibc
module ibc::ibc;
*/

module ibc::ibc {
    use std::string::{String, utf8};
    use sui::table::{Self, Table};
    use sui::object::{Self, UID};
    use sui::transfer;
    use sui::bcs;
    use sui::event;

    use ibc::cometbls_light_client::{Self, Client};
    use ibc::commitment;
    use ibc::height::{Self, Height};
    use ibc::connection_end::{Self, ConnectionEnd, Version};

    const CONN_STATE_UNSPECIFIED: u64 = 0;
    const CONN_STATE_INIT: u64 = 1;
    const CONN_STATE_TRYOPEN: u64 = 2;
    const CONN_STATE_OPEN: u64 = 3;

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
    public struct ClientCreatedEvent has copy, drop {
        client_id: String,
        client_type: String,
        consensus_height: Height,
    }

    #[event]
    public struct ConnectionOpenInit has copy, drop {
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
    }

    #[event]
    public struct ConnectionOpenTry has copy, drop {
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
        counterparty_connection_id: String,
    }

    #[event]
    public struct ConnectionOpenAck has copy, drop {
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
        counterparty_connection_id: String,
    }

    #[event]
    public struct ConnectionOpenConfirm has copy, drop {
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
        counterparty_connection_id: String,
    }

    #[event]
    public struct ChannelOpenConfirm has copy, drop {
        port_id: String,
        channel_id: String,
        counterparty_port_id: String,
        counterparty_channel_id: String,
        connection_id: String
    }

    // Resource to hold the global state
    public struct IBCStore has key {
        id: UID,
        client_impls: Table<String, address>,
        client_registry: Table<String, address>,
        commitments: Table<vector<u8>, vector<u8>>,
        clients: Table<String, Client>,
        connections: Table<String, ConnectionEnd>,
        // channels: Table<ChannelPort, Channel>, 
    }

    fun init(ctx: &mut TxContext) {
        // transfer::transfer(CreatorCapability {
        //     id: object::new(ctx),
        // }, tx_context::sender(ctx))

        transfer::share_object(IBCStore {
            id: object::new(ctx),
            client_registry: table::new(ctx),
            commitments: table::new(ctx),
            client_impls: table::new(ctx),
            clients: table::new(ctx),
            connections: table::new(ctx),
            // channels: smart_table::new<ChannelPort, Channel>(),
        });
        // TODO(aeryz): init `nextClientSequence`
    }

    /// Create a client with an initial client and consensus state
    public entry fun create_client(
        ibc_store: &mut IBCStore,
        client_type: String,
        client_state: vector<u8>,
        consensus_state: vector<u8>,
        ctx: &mut TxContext,
    ) {
        assert!(client_type.bytes() == &b"cometbls", E_UNKNOWN_CLIENT_TYPE);

        let client_id = ibc_store.generate_client_identifier(client_type);

        let client = cometbls_light_client::create_client(
            client_id,
            client_state,
            consensus_state,
            ctx,
        );

        // TODO(aeryz): fetch these status from proper exported consts
        assert!(client.status() == 0, E_CLIENT_NOT_ACTIVE);

        ibc_store.commitments.add(commitment::client_state_key(client_id), client_state);

        let latest_height = client.latest_height();

        ibc_store.commitments.add(commitment::consensus_state_key(client_id, latest_height), consensus_state);

        ibc_store.clients.add(client_id, client);

        event::emit(
            ClientCreatedEvent {
                client_id,
                client_type,
                consensus_height: latest_height,
            },
        )
    }

    public entry fun connection_open_init(
        ibc_store: &mut IBCStore,
        client_id: String,
        version_identifier: String,
        version_features: vector<String>,
        counterparty_client_id: String,
        counterparty_connection_id: String,
        counterparty_prefix: vector<u8>,
        delay_period: u64
    ) {
        let version = connection_end::new_version(version_identifier, version_features);
        let counterparty = connection_end::new_counterparty(counterparty_client_id, counterparty_connection_id, counterparty_prefix);

        assert!(ibc_store.clients.borrow(client_id).status() == 0, E_CLIENT_NOT_ACTIVE);

        let connection_id = ibc_store.generate_connection_identifier();

        let mut connection = connection_end::new(
            client_id,
            vector::empty(),
            CONN_STATE_INIT,
            delay_period,
            counterparty
        );

        if (version.version_features().is_empty()) {
            connection.set_versions(get_compatible_versions());
        } else {
            assert!(is_supported_version(&get_compatible_versions(), &version), E_UNSUPPORTED_VERSION);

            connection.set_versions(vector[version]);
        };

        ibc_store.connections.add(connection_id, connection);

        update_connection_commitment(ibc_store, connection_id, connection);

        event::emit(
            ConnectionOpenInit {
                connection_id,
                client_id,
                counterparty_client_id: *connection.conn_counterparty_client_id(),
            }
        )
    }

    
    public entry fun connection_open_try(
        ibc_store: &mut IBCStore,
        counterparty_client_id: String,
        counterparty_connection_id: String,
        counterparty_prefix: vector<u8>,
        delay_period: u64,
        client_id: String,
        client_state_bytes: vector<u8>,
        counterparty_version_identifiers: vector<String>,
        counterparty_version_features: vector<vector<String>>,
        proof_init: vector<u8>,
        proof_client: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) { 
        let counterparty = connection_end::new_counterparty(counterparty_client_id, counterparty_connection_id, counterparty_prefix);
        let counterparty_versions = connection_end::new_versions(counterparty_version_identifiers, counterparty_version_features);
        let proof_height = height::new(proof_height_revision_num, proof_height_revision_height);

        let connection_id = ibc_store.generate_connection_identifier();

        let connection = connection_end::new(
            client_id,
            vector[pick_version(&get_compatible_versions(), &counterparty_versions)],
            CONN_STATE_TRYOPEN,
            delay_period,
            counterparty
        );

        let client = ibc_store.clients.borrow(client_id);
        assert!(client.status() == 0, E_CLIENT_NOT_ACTIVE);

        // Create the expected connection
        let expected_connection = connection_end::new(
            *connection.conn_counterparty_client_id(),
            counterparty_versions,
            CONN_STATE_INIT,
            delay_period,
            connection_end::new_counterparty(client_id, utf8(b""), b"ibc")
        );

        // Verify the connection state
        let err = verify_connection_state(
            client,
            &connection,
            proof_height,
            proof_init,
            *connection_end::counterparty_connection_id(&counterparty),
            expected_connection
        ); 
        assert!(err == 0, E_INVALID_PROOF);

        let counterparty_client_id = connection.conn_counterparty_client_id();

        // Verify the client state
        let err = verify_client_state(
            client,
            &connection,
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
                counterparty_client_id: *connection.conn_counterparty_client_id(),
                counterparty_connection_id: *connection.conn_counterparty_connection_id(),
            },
        );

        ibc_store.connections.add(connection_id, connection);
        ibc_store.update_connection_commitment(connection_id, connection);
    }

    public entry fun connection_open_ack(
        ibc_store: &mut IBCStore,
        connection_id: String,
        client_state_bytes: vector<u8>,
        version_identifier: String,
        version_features: vector<String>,
        proof_try: vector<u8>,
        proof_client: vector<u8>,
        counterparty_connection_id: String,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) {
        let version = connection_end::new_version(version_identifier, version_features);
        let proof_height = height::new(proof_height_revision_num, proof_height_revision_height);

        assert!(ibc_store.connections.contains(connection_id), E_CONNECTION_DOES_NOT_EXIST);

        let mut connection = ibc_store.connections.borrow_mut(
            connection_id,
        );

        let client = ibc_store.clients.borrow(*connection.client_id());
        assert!(client.status() == 0, E_CLIENT_NOT_ACTIVE);

        assert!(connection.state() == CONN_STATE_INIT, E_INVALID_CONNECTION_STATE);

        assert!(is_supported_version(connection.versions(), &version), E_UNSUPPORTED_VERSION);

        let expected_counterparty = connection_end::new_counterparty(
            *connection_end::client_id(connection),
            connection_id,
            b"ibc",
        );

        let expected_connection = connection_end::new(
            *connection.conn_counterparty_client_id(),
            vector::singleton(version),
            CONN_STATE_TRYOPEN,
            connection.delay_period(),
            expected_counterparty
        );

        let err = verify_connection_state(
            client,
            connection,
            proof_height,
            proof_try,
            counterparty_connection_id,
            expected_connection
        );
        assert!(err == 0, err);

        let counterparty_client_id = *connection.conn_counterparty_client_id();

        let err = verify_client_state(
            client,
            connection,
            proof_height,
            commitment::client_state_key(counterparty_client_id),
            proof_client,
            client_state_bytes
        );
        assert!(err == 0, err);

        connection.set_state(CONN_STATE_OPEN);

        let mut conn_versions = *connection.versions();
        copy_versions(&vector::singleton(version), &mut conn_versions);
        connection_end::set_versions(connection, conn_versions);
        connection_end::set_conn_counterparty_connection_id(connection, counterparty_connection_id);

        event::emit(
            ConnectionOpenAck {
                connection_id,
                client_id: *connection_end::client_id(connection),
                counterparty_client_id: *connection_end::conn_counterparty_client_id(connection),
                counterparty_connection_id: *connection_end::conn_counterparty_connection_id(connection),
            },
        );

        ibc_store.update_connection_commitment(connection_id, *connection);   
    }

    public entry fun connection_open_confirm(
        ibc_store: &mut IBCStore,
        connection_id: String,
        proof_ack: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) {
        let proof_height = height::new(proof_height_revision_num, proof_height_revision_height);

        let mut connection = ibc_store.connections.borrow_mut(
            connection_id,
        );

        let client = ibc_store.clients.borrow(*connection.client_id());
        assert!(client.status() == 0, E_CLIENT_NOT_ACTIVE);

        if (connection.state() != CONN_STATE_TRYOPEN) {
            abort E_INVALID_CONNECTION_STATE
        };

        let expected_counterparty = connection_end::new_counterparty(
            *connection_end::client_id(connection),
            connection_id,
            b"ibc",
        );

        let expected_connection = connection_end::new(
            *connection.conn_counterparty_client_id(),
            *connection.versions(),
            CONN_STATE_OPEN,
            connection.delay_period(),
            expected_counterparty
        );

        let counterparty_conn_id = *connection.conn_counterparty_connection_id();

        let err = verify_connection_state(
            client,
            connection,
            proof_height,
            proof_ack,
            counterparty_conn_id,
            expected_connection
        );
        assert!(err == 0, E_INVALID_PROOF);

        connection.set_state(CONN_STATE_OPEN);

        event::emit(
            ConnectionOpenConfirm {
                connection_id,
                client_id: *connection.client_id(),
                counterparty_client_id: *connection.conn_counterparty_client_id(),
                counterparty_connection_id: *connection.conn_counterparty_connection_id(),
            },
        );

        ibc_store.update_connection_commitment(connection_id, *connection);
    }

    public entry fun update_client(ibc_store: &mut IBCStore, client_id: String, client_message: vector<u8>) {
        let client = ibc_store.clients.borrow(client_id);

        let (client_state, consensus_states, heights) = LightClient::update_client(
            client_id,
            client_message
        );

        let heights_len = vector::length(&heights); 

        assert!(
            !vector::is_empty(&consensus_states) 
                && !vector::is_empty(&heights) 
                && heights_len == vector::length(&consensus_states), 
            E_INVALID_UPDATE
        );
        
        table::upsert(&mut store.commitments, IBCCommitment::client_state_key(client_id), client_state);

        let i = 0;
        while (i < heights_len) {
            let height = *vector::borrow(&heights, i);

            table::upsert(
                &mut store.commitments,
                IBCCommitment::consensus_state_key(client_id, height),
                hash::sha2_256(*vector::borrow(&consensus_states, i))
            );

            event::emit(ClientUpdated {
                client_id,
                // NOTE: This is currently enforced, if/when we refactor to be more general across clients then this will need to be modified accordingly
                client_type: string::utf8(CLIENT_TYPE_COMETBLS),
                height,
            });

            i = i + 1;
        };
    }

    // Function to generate a client identifier
    fun generate_client_identifier(ibc_store: &mut IBCStore, mut client_type: String): String {
        let next_sequence = ibc_store.commitments.borrow(b"nextClientSequence");
        let next_sequence = bcs::new(*next_sequence).peel_u64();

        ibc_store.commitments.add(b"nextClientSequence", bcs::to_bytes<u64>(&(next_sequence + 1)));

        client_type.append_utf8(b"-");
        client_type.append(next_sequence.to_string());
        client_type
    }

    fun generate_connection_identifier(store: &mut IBCStore): String {
        // TODO(aeryz): init this 
        let next_sequence_bytes = store.commitments.borrow(
            b"nextConnectionSequence",
        );
        let next_sequence = bcs::new(*next_sequence_bytes).peel_u64();
        store.commitments.add(
            b"nextConnectionSequence",
            bcs::to_bytes(&(next_sequence + 1))
        );

        let mut connection_id = utf8(b"connection-");
        connection_id.append(next_sequence.to_string());
        connection_id
    }

    fun get_compatible_versions(): vector<connection_end::Version> {
        vector[default_ibc_version()]
    }

    fun default_ibc_version(): connection_end::Version {
        connection_end::new_version(
            utf8(b"1"),
            vector[utf8(b"ORDER_ORDERED"), utf8(b"ORDER_UNORDERED")],
        )
    }

    fun set_supported_versions(
        supported_versions: vector<connection_end::Version>,
        versions: &mut vector<connection_end::Version>
    ) {
        assert!(versions.is_empty(), E_VERSION_MUST_BE_UNSET);
        versions.append(supported_versions);
    }

    fun is_supported_version(
        supported_versions: &vector<connection_end::Version>,
        version: &connection_end::Version
    ): bool {
        let (supported_version, found) = find_supported_version(supported_versions, version);
        found && verify_proposed_version(&supported_version, version)
    }

    fun copy_version(src: &connection_end::Version, dst: &mut connection_end::Version) {
        dst.set_version_identifier(*src.version_identifier());
        let src_len = src.version_features().length();
        let dst_len = dst.version_features().length();
        let mut i = 0;
        let mut dst_features = dst.version_features_mut();
        while (i < src_len) {
            if (i < dst_len) {
                let src_feature = src.version_features()[i];
                let mut dst_feature = dst_features.borrow_mut(i);
                *dst_feature = src_feature;
            } else {
                let src_feature = src.version_features()[i];
                dst_features.push_back(src_feature);
            };
            i = i + 1;
        };
        while (i < dst_len) {
            dst_features.remove(i);
            i = i + 1;
        }
    }

    fun copy_versions(src: &vector<connection_end::Version>, dst: &mut vector<connection_end::Version>) {
        let src_len = src.length();
        let dst_len = dst.length();
        if (src_len == dst_len) {
            let mut i = 0;
            while (i < src_len) {
                let src_version = src.borrow(i);
                let dst_version = dst.borrow_mut(i);
                copy_version(src_version, dst_version);
                i = i + 1;
            };
        } else if (src_len > dst_len) {
            let mut i = 0;
            while (i < dst_len) {
                let src_version = src.borrow(i);
                let dst_version = dst.borrow_mut(i);
                copy_version(src_version, dst_version);
                i = i + 1;
            };
            let mut j = dst_len;
            while (j < src_len) {
                let src_version = src.borrow(j);
                dst.push_back(*src_version);
                j = j + 1;
            };
        } else {
            let mut i = 0;
            while (i < src_len) {
                let src_version = src.borrow(i);
                let dst_version = dst.borrow_mut(i);
                copy_version(src_version, dst_version);
                i = i + 1;
            };
            let mut j = src_len;
            while (j < dst_len) {
                dst.remove(j);
                j = j + 1;
            };
        }
    }

    fun find_supported_version(
        supported_versions: &vector<connection_end::Version>,
        version: &connection_end::Version
    ): (connection_end::Version, bool) {
        let mut found_version = connection_end::default_version();
        let mut found = false;
        let len_supported_versions = supported_versions.length();
        let mut i = 0;
        while(i < len_supported_versions) {
            let v = supported_versions[i];
            if (v.version_identifier() == version.version_identifier()) {
                found_version = v;
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
        let mut is_supported = false;
        if(supported_version.version_identifier() == proposed_version.version_identifier()) {
            let len_proposed_version = proposed_version.version_features().length();
            let mut i = 0;
            while(i < len_proposed_version) {
                let feature = proposed_version.version_features()[i];
                is_supported = supported_version.version_features().contains(&feature);
                if(!is_supported) {
                    break
                };
                i = i + 1;
            };
        };
        is_supported
    }

    fun pick_version(
        supported_versions: &vector<connection_end::Version>,
        counterparty_versions: &vector<connection_end::Version>
    ): connection_end::Version  {
        let supported_len = supported_versions.length();
        let mut i = 0;
        while (i < supported_len) {
            let supported_version = supported_versions[i];
            let (counterparty_version, found) = find_supported_version(counterparty_versions, &supported_version);
            if (found) {
                let feature_set = get_feature_set_intersection(supported_version.version_features(), counterparty_version.version_features());
                if (feature_set.length() > 0) {
                    return connection_end::new_version(*supported_version.version_identifier(), feature_set)
                };
            };
            i = i + 1;
        };
        abort E_UNSUPPORTED_VERSION
    }

    fun get_feature_set_intersection(
        source_features: &vector<String>,
        counterparty_features: &vector<String>
    ): vector<String> {
        let mut feature_set = vector::empty();
        let source_len = source_features.length();
        let mut i = 0;
        while (i < source_len) {
            let feature = source_features[i];
            if (counterparty_features.contains(&feature)) {
                feature_set.push_back(feature);
            };
            i = i + 1;
        };
        feature_set
    }

    fun update_connection_commitment(store: &mut IBCStore, connection_id: String, connection: ConnectionEnd) {
        let encoded_connection = connection.encode_proto();
        let key = commitment::connection_key(connection_id);
        store.commitments.add(key, encoded_connection);
    }

    fun verify_connection_state(
        client: &Client,
        connection: &ConnectionEnd,
        height: height::Height,
        proof: vector<u8>,
        connection_id: String,
        counterparty_connection: ConnectionEnd
    ): u64 {
        client.verify_membership(
            height,
            proof,
            *connection.conn_counterparty_key_prefix(),
            commitment::connection_key(connection_id),
            counterparty_connection.encode_proto()
        )
    }

    fun verify_client_state(
        client: &Client,
        connection: &ConnectionEnd,
        height: height::Height,
        path: vector<u8>,
        proof: vector<u8>,
        client_state_bytes: vector<u8>
    ): u64 {
        client.verify_membership(
            height,
            proof,
            *connection.conn_counterparty_key_prefix(),
            path,
            client_state_bytes
        )
    }
}

