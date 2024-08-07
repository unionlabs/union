module IBC::Core {
    use std::signer;
    use std::vector;
    use std::error;
    use std::debug;
    use aptos_std::smart_table::{Self as SmartTable, SmartTable};
    use aptos_framework::event;
    use aptos_framework::account::{Self, SignerCapability};
    use aptos_framework::aptos_account;
    use std::bcs;
    use aptos_framework::object;
    use aptos_std::string::{Self, String};

    use aptos_std::string_utils;
    use aptos_std::any::{Self, Any};
    use aptos_std::from_bcs;
    use IBC::IBCCommitment;
    use IBC::LightClient;
    use IBC::height;
    use IBC::IBCModule;

    const SEED: vector<u8> = b"Move Seed Example";
    const VAULT_SEED: vector<u8> = b"Vault Seed Example";
    const E_CLIENT_ALREADY_EXISTS: u64 = 1001;
    const E_CLIENT_IMPL_NOT_FOUND: u64 = 1002;
    const E_LIGHT_CLIENT_CALL_FAILED: u64 = 1003;
    const ERR_SWAP_NOT_INITIALIZED: u64 = 1004;
    const ERR_NOT_ENOUGH_PERMISSIONS_TO_INITIALIZE: u64 = 1005;
    const ERR_VERSION_MUST_BE_UNSET: u64 = 1006;
    const ERR_UNSUPPORTED_VERSION: u64 = 1007;
    const ERR_INVALID_CONNECTION_STATE: u64 = 1008;
    const E_CONNECTION_ALREADY_EXISTS: u64 = 1009;
    const E_INVALID_PROOF: u64 = 1010;
    const E_CONN_NOT_SINGLE_HOP: u64 = 1011;
    const E_CONN_NOT_SINGLE_VERSION: u64 = 1012;
    const E_UNSUPPORTED_FEATURE: u64 = 1013;
    const E_CAPABILITY_ALREADY_CLAIMED: u64 = 1014;
    const E_PORT_ID_MUST_BE_LOWERCASE: u64 = 1015;
    const E_INVALID_CHANNEL_STATE: u64 = 1016;
    const E_COUNTERPARTY_CHANNEL_NOT_EMPTY: u64 = 1017;
    
    struct ClientCreatedEvent has copy, drop, store {
        client_id: String,
    }

    
    struct ConnectionOpenInit has copy, drop, store {
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
    }    

    struct ChannelOpenInit has copy, drop, store {
        port_id: String,
        channel_id: String,
        counterparty_port_id: String,
        connection_id: String,
        version: String
    }

    struct ChannelOpenAck has copy, drop, store {
        port_id: String,
        channel_id: String,
        counterparty_port_id: String,
        counterparty_channel_id: String,
        connection_id: String
    }
    
    struct ConnectionOpenTry has copy, drop, store {
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
        counterparty_connection_id: String,
    }

    struct ConnectionOpenAck has copy, drop, store {
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
        counterparty_connection_id: String,
    }

    struct ConnectionOpenConfirm has copy, drop, store {
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
        counterparty_connection_id: String,
    }

    struct ChannelPort has copy, drop, store {
        port_id: String,
        channel_id: String,
    }

    
    // Resource to hold the global state
    struct IBCStore has key {
        client_created_events: event::EventHandle<ClientCreatedEvent>,
        connection_open_init_events: event::EventHandle<ConnectionOpenInit>, 
        channel_open_init_events: event::EventHandle<ChannelOpenInit>, 
        channel_open_ack_events: event::EventHandle<ChannelOpenAck>,   
        connection_open_try_events: event::EventHandle<ConnectionOpenTry>, 
        connection_open_ack_events: event::EventHandle<ConnectionOpenAck>,
        connection_open_confirm_events: event::EventHandle<ConnectionOpenConfirm>,
        client_impls: SmartTable<String, address>,
        client_registry: SmartTable<String, address>,
        commitments: SmartTable<vector<u8>, vector<u8>>,
        connections: SmartTable<String, ConnectionEnd>,
        channels: SmartTable<ChannelPort, IbcCoreChannelV1Channel>, 
        capabilities: SmartTable<String, address>,
    }


    // Sample setter for `capabilities`
    public fun set_capability(capability_id: String, addr: address) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        SmartTable::upsert(&mut store.capabilities, capability_id, addr);
    }

    struct SignerRef has key {
        self_ref: object::ExtendRef,
    }


    #[view]
    public fun get_vault_addr(): address {
        object::create_object_address(&@IBC, VAULT_SEED)
    }

    public fun set_connection(connection_id: String, connection: ConnectionEnd) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        SmartTable::upsert(&mut store.connections, connection_id, connection);
    }

    // Setter for Commitments
    public fun set_commitment(key: vector<u8>, value: vector<u8>) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        SmartTable::upsert(&mut store.commitments, key, value);
    }

    // Getter for Commitments
    public fun get_commitment(key: vector<u8>): vector<u8> acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let commitment = SmartTable::borrow_with_default(&store.commitments, key, &vector::empty<u8>());
        *commitment
    }

    // Getter for Commitments
    public fun get_channel_from_store(key: String, channel_id: String): IbcCoreChannelV1Channel acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let channel_port = ChannelPort{port_id:key, channel_id};
        let channel = SmartTable::borrow(&store.channels, channel_port);

        *channel
    }

    // Setter for nextChannelSequence in Commitments
    public fun set_next_channel_sequence(sequence: u64) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        SmartTable::upsert(&mut store.commitments, b"nextChannelSequence", bcs::to_bytes(&sequence));
    }

    // Getter for nextChannelSequence in Commitments
    public fun get_next_channel_sequence(): u64 acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let next_sequence_bytes = SmartTable::borrow_with_default(&store.commitments, b"nextChannelSequence", &bcs::to_bytes<u64>(&0));
        from_bcs::to_u64(*next_sequence_bytes)
    }


    // Initializes the IBCStore resource in the signer's account
    public fun create_ibc_store(account: &signer)  {
        assert!(signer::address_of(account) == @IBC, ERR_NOT_ENOUGH_PERMISSIONS_TO_INITIALIZE);
        let vault_constructor_ref = &object::create_named_object(account, VAULT_SEED);
        let vault_signer = &object::generate_signer(vault_constructor_ref);

        let (resource_signer, resource_signer_cap) = account::create_resource_account(account, SEED);
        let store = IBCStore {
            client_registry: SmartTable::new(),
            commitments: SmartTable::new(),
            client_impls: SmartTable::new(),
            connections: SmartTable::new(),
            channels: SmartTable::new<ChannelPort, IbcCoreChannelV1Channel>(),
            capabilities: SmartTable::new(),
            client_created_events: account::new_event_handle(&resource_signer),
            connection_open_init_events: account::new_event_handle(&resource_signer), 
            channel_open_init_events: account::new_event_handle(&resource_signer), 
            channel_open_ack_events: account::new_event_handle(&resource_signer), 
            connection_open_try_events: account::new_event_handle(&resource_signer),
            connection_open_ack_events: account::new_event_handle(&resource_signer),
            connection_open_confirm_events: account::new_event_handle(&resource_signer),
        };

        move_to(vault_signer, store);

        move_to(vault_signer, SignerRef {
            self_ref: object::generate_extend_ref(vault_constructor_ref)
        });

        let addr = get_vault_addr();
    }

    // Function to generate a client identifier
    public fun generate_client_identifier(client_type: String): String acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let next_sequence = SmartTable::borrow_with_default(&store.commitments, b"nextClientSequence", &bcs::to_bytes<u64>(&0u64));

        let next_sequence = from_bcs::to_u64(*next_sequence);

        let next_sequence_str = string_utils::to_string(&next_sequence);

        let next_sequence = next_sequence + 1;

        // Constructing the identifier string using append
        let identifier = client_type;
        string::append_utf8(&mut identifier, b"-");
        string::append(&mut identifier, next_sequence_str);

        SmartTable::upsert(&mut store.commitments, b"nextClientSequence", bcs::to_bytes<u64>(&next_sequence));

        identifier
    }


    // // Function to create a client based on the provided message
    public fun create_client(
        client_type: String,
        client_state: Any,
        consensus_state: Any,
        relayer: address
    ): String  acquires IBCStore, SignerRef {
        let client_id = generate_client_identifier(client_type);
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let client_state_bytes = bcs::to_bytes<Any>(&client_state);
        let status_code = IBC::LightClient::create_client(
            &get_ibc_signer(),
            client_id, 
            client_state, 
            consensus_state
        );
    
        // Check if the client was created successfully
        assert!(status_code == 0, status_code);

        // Update commitments
        SmartTable::upsert(&mut store.commitments, IBCCommitment::client_state_commitment_key(client_id), client_state_bytes);

        // SmartTable::upsert(
        //     &mut store.commitments,
        //     IBCCommitment::consensus_state_commitment_key(client_id, update.height, 1),
        //     msg.consensus_state.data
        // ); 

        event::emit_event(&mut  store.client_created_events, ClientCreatedEvent {
            client_id
        });

        client_id

    }

    public fun get_ibc_signer(): signer acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        object::generate_signer_for_extending(&vault.self_ref)
    }

    struct Version has copy, store, drop, key {
        identifier: String,
        features: vector<String>,
    }

    struct IbcCoreChannelV1Packet has copy, store, drop, key {
        sequence: u64,
        source_port: String,
        source_channel: String,
        destination_port: String,
        destination_channel: String,
        data: vector<u8>,
        timeout_height: height::Height,
        timeout_timestamp: u64,
    }


    /*
    IbcCoreChannelV1GlobalEnums {
        //enum definition
        // Solidity enum definitions
        enum State {
            STATE_UNINITIALIZED_UNSPECIFIED,
            STATE_INIT,
            STATE_TRYOPEN,
            STATE_OPEN,
            STATE_CLOSED
        }
        */
    struct IbcCoreChannelV1Channel has copy, store, drop, key {
        state: u8, //STATE_UNINITIALIZED_UNSPECIFIED, STATE_INIT, STATE_TRYOPEN, STATE_OPEN, STATE_CLOSED
        ordering: u8, //STATE_UNINITIALIZED_UNSPECIFIED, STATE_INIT, STATE_TRYOPEN, STATE_OPEN, STATE_CLOSED
        counterparty: IbcCoreChannelV1Counterparty,
        connection_hops: vector<String>,
        version: String
    }


    struct ConnectionEnd has copy, store, drop {
        client_id: String,
        versions: vector<Version>,
        state: u64,
        delay_period: u64,
        counterparty: IbcCoreConnectionV1Counterparty,
    }

    struct IbcCoreConnectionV1Counterparty has copy, store, drop {
        client_id: String,
        connection_id: String,
        prefix: IbcCoreCommitmentV1MerklePrefix,
    }

    struct IbcCoreChannelV1Counterparty has copy, store, drop {
        port_id: String,
        channel_id: String,
    }

    struct IbcCoreCommitmentV1MerklePrefix has copy, store, drop {
        key_prefix: vector<u8>,
    }

    public fun new_version(identifier: String, features: vector<String>): Version {
        Version { identifier, features }
    }

    public fun new_merkleprefix(key_prefix: vector<u8>): IbcCoreCommitmentV1MerklePrefix {
        IbcCoreCommitmentV1MerklePrefix { key_prefix }
    }

    public fun get_version_identifier(version: &Version): String {
        version.identifier
    }

    public fun get_version_features(version: &Version): vector<String> {
        version.features
    }

    public fun new_connection_counterparty(client_id: String, connection_id: String, prefix: IbcCoreCommitmentV1MerklePrefix): IbcCoreConnectionV1Counterparty {
        IbcCoreConnectionV1Counterparty { client_id, connection_id, prefix }
    }
    
    public fun new_channel_counterparty(port_id: String, channel_id: String): IbcCoreChannelV1Counterparty {
        IbcCoreChannelV1Counterparty { port_id, channel_id }
    } 

    public fun get_connection_counterparty_client_id(counterparty: &IbcCoreConnectionV1Counterparty): String {
        counterparty.client_id
    }

    public fun get_connection_counterparty_connection_id(counterparty: &IbcCoreConnectionV1Counterparty): String {
        counterparty.connection_id
    }

    // ConnectionEnd-related functions
    public fun new_connection_end(
        client_id: String,
        versions: vector<Version>,
        state: u64,
        delay_period: u64,
        counterparty: IbcCoreConnectionV1Counterparty
    ): ConnectionEnd {
        ConnectionEnd {
            client_id,
            versions,
            state,
            delay_period,
            counterparty,
        }
    }

    // ConnectionEnd-related functions
    public fun new_channel(
        state: u8,
        ordering: u8,
        counterparty: IbcCoreChannelV1Counterparty,
        connection_hops: vector<String>,
        version: String
    ): IbcCoreChannelV1Channel {
        IbcCoreChannelV1Channel {
            state,
            ordering,
            counterparty,
            connection_hops,
            version
        }
    }

    public fun get_connection_end(
        connection_end: ConnectionEnd
    ): (String,
        vector<Version>,
        u64,
        u64,
        IbcCoreConnectionV1Counterparty) {
            (connection_end.client_id, connection_end.versions, connection_end.state, connection_end.delay_period, connection_end.counterparty)
    }

    public fun get_channel(
        channel: &IbcCoreChannelV1Channel
    ): (u8, u8, IbcCoreChannelV1Counterparty, vector<String>, String) {
        (channel.state, channel.ordering, channel.counterparty, channel.connection_hops, channel.version)
    }


    public fun get_channel_counterparty(
        counterparty: IbcCoreChannelV1Counterparty
    ): (String, String) {
        (counterparty.port_id, counterparty.channel_id)
    }



    public fun get_connection_client_id(connection: &ConnectionEnd): String {
        connection.client_id
    }

    public fun get_connection_versions(connection: &ConnectionEnd): vector<Version> {
        connection.versions
    }

    public fun get_connection_state(connection: &ConnectionEnd): u64 {
        connection.state
    }

    public fun get_connection_delay_period(connection: &ConnectionEnd): u64 {
        connection.delay_period
    }

    public fun get_connection_counterparty(connection: &ConnectionEnd): IbcCoreConnectionV1Counterparty {
        connection.counterparty
    }

     public fun default_ibc_version(): Version {
        Version {
            identifier: string::utf8(b"1"),
            features: vector<String>[string::utf8(b"ORDER_ORDERED"), string::utf8(b"ORDER_UNORDERED")],
        }
    }

    public fun set_supported_versions(
        supported_versions: vector<Version>,
        dst: &mut vector<Version>
    ) {
        assert!(vector::length(dst) == 0, ERR_VERSION_MUST_BE_UNSET);
        vector::append(dst, supported_versions);
    }

    public fun is_supported_version(
        supported_versions: &vector<Version>,
        version: &Version
    ): bool {
        let (supported_version, found) = find_supported_version(supported_versions, version);
        if(found && verify_proposed_version(&supported_version, version)) {
            return true;
        };
        false
        // found && verify_proposed_version(&supported_version, version)
    }



    public fun contains(elem: &String, set: &vector<String>): bool {
        let hashedElem = IBCCommitment::keccak256(*elem);
        let set_len = vector::length(set);
        let i = 0;
        while (i < set_len) {
            let item = IBCCommitment::keccak256(*vector::borrow(set, i));
            if (item == hashedElem) {
                return true;
            };
            i = i + 1;
        };
        false
    }


    public fun get_feature_set_intersection(
        source_features: &vector<String>,
        counterparty_features: &vector<String>
    ): vector<String> {
        let feature_set = vector::empty<String>();
        let source_len = vector::length(source_features);
        let i = 0;
        while (i < source_len) {
            let feature = vector::borrow(source_features, i);
            if (contains(feature, counterparty_features)) {
                vector::push_back(&mut feature_set, *feature);
            };
            i = i + 1;
        };
        feature_set
    }


    public fun pick_version(
        supported_versions: &vector<Version>,
        counterparty_versions: &vector<Version>
    ): Version  {
        let supported_len = vector::length(supported_versions);
        let i = 0;
        while (i < supported_len) {
            let supported_version = vector::borrow(supported_versions, i);
            let (counterparty_version, found) = find_supported_version(counterparty_versions, supported_version);
            if (found) {
                let feature_set = get_feature_set_intersection(&supported_version.features, &counterparty_version.features);
                if (vector::length(&feature_set) > 0) {
                    return Version {
                        identifier: supported_version.identifier,
                        features: feature_set,
                    };
                };
            };
            i = i + 1;
        };
        abort(ERR_UNSUPPORTED_VERSION);
        new_version(string::utf8(b""), vector::empty<String>()) //TODO: why i need to return :D it will revert anyway
        // this language is so random
    }
    public fun copy_version(src: &Version, dst: &mut Version) {
        dst.identifier = src.identifier;
        let src_len = vector::length(&src.features);
        let dst_len = vector::length(&dst.features);
        let i = 0;
        while (i < src_len) {
            if (i < dst_len) {
                let src_feature = vector::borrow(&src.features, i);
                let dst_feature = vector::borrow_mut(&mut dst.features, i);
                *dst_feature = *src_feature;
            } else {
                let src_feature = vector::borrow(&src.features, i);
                vector::push_back(&mut dst.features, *src_feature);
            };
            i = i + 1;
        };
        while (i < dst_len) {
            vector::remove(&mut dst.features, i);
            i = i + 1;
        }
    }

    public fun copy_versions(src: &vector<Version>, dst: &mut vector<Version>) {
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


    public fun find_supported_version(
        supported_versions: &vector<Version>,
        version: &Version
    ): (Version, bool) {
        let found_version = Version {
            identifier: string::utf8(b""),
            features: vector::empty<String>(),
        };
        let found = false;
        let len_supported_versions = vector::length(supported_versions);
        let i = 0;
        while(i < len_supported_versions) {
            let v = vector::borrow(supported_versions, i);
            let data1 = string::utf8(bcs::to_bytes(&v.identifier));
            let data2 = string::utf8(bcs::to_bytes(&version.identifier));
            if (string::utf8(bcs::to_bytes(&v.identifier)) == string::utf8(bcs::to_bytes(&version.identifier))) {
                found_version = *v;
                found = true;
                // debug::print(&string::utf8(b"my string log"));
                break;
            };
            i = i + 1;
        };
        (found_version, found)
    }

    public fun verify_proposed_version(
        supported_version: &Version,
        proposed_version: &Version
    ): bool {
        let is_supported = false;
        if(string::utf8(bcs::to_bytes(&supported_version.identifier)) == string::utf8(bcs::to_bytes(&proposed_version.identifier))) {
            let len_proposed_version = vector::length(&proposed_version.features);
            let i = 0;
            while(i < len_proposed_version) {
                let feature = vector::borrow(&proposed_version.features, i);
                is_supported = vector::contains(&supported_version.features, feature);
                if(!is_supported) {
                    break;
                };
                i = i + 1;
            };
        };
        is_supported
    }

    public fun verify_client_state(
        connection: &ConnectionEnd,
        height: height::Height,
        path: vector<u8>,
        proof: Any,
        client_state_bytes: vector<u8>
    ): bool {
        let (result, error_code) = IBC::LightClient::verify_membership(
            connection.client_id,
            height,
            proof,
            connection.counterparty.prefix.key_prefix,
            path,
            client_state_bytes
        );
        assert!(error_code == 0, ERR_INVALID_CONNECTION_STATE);
        true
    }

    public fun verify_connection_state(
        connection: &ConnectionEnd,
        height: height::Height,
        proof: Any,
        connection_id: String,
        counterparty_connection: ConnectionEnd
    ): bool {
        let path = IBCCommitment::connection_commitment_key(connection_id);
        let encoded_counterparty_connection = bcs::to_bytes(&counterparty_connection);
        let (result, error_code) = IBC::LightClient::verify_membership(
            connection.client_id,
            height,
            proof,
            connection.counterparty.prefix.key_prefix,
            bcs::to_bytes(&IBCCommitment::connection_path(connection_id)),
            bcs::to_bytes(&encode(&counterparty_connection))
        );
        assert!(error_code == 0, ERR_INVALID_CONNECTION_STATE);
        true
    }

    public fun generate_connection_identifier(): String acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let next_sequence_bytes = SmartTable::borrow_with_default(
            &store.commitments,
            b"nextConnectionSequence",
            &bcs::to_bytes<u64>(&0)
        );
        let next_sequence = from_bcs::to_u64(*next_sequence_bytes);
        let identifier = string::utf8(b"connection-");
        string::append(&mut identifier, string_utils::to_string(&next_sequence));
        let new_sequence = next_sequence + 1;
        SmartTable::upsert(
            &mut store.commitments,
            b"nextConnectionSequence",
            bcs::to_bytes(&new_sequence)
        );
        identifier
    }



    // TODO: Implement the encode function for Channel
    // originally, defined under: IbcCoreChannelV1Channel.encode
    public fun encode_channel(channel: &IbcCoreChannelV1Channel): String {
        // Placeholder implementation for encoding a connection
        string::utf8(b"")
    }

    // TODO: Implement the encode function for ConnectionEnd
    // originally, defined under: IbcCoreConnectionV1ConnectionEnd.encode
    public fun encode(connection: &ConnectionEnd): String {
        // Placeholder implementation for encoding a connection
        string::utf8(b"")
    }
    public fun update_connection_commitment(store: &mut IBCStore, connection_id: String) {
        // let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let default_counterparty_prefix = IBCCommitment::keccak256(string::utf8(b"prefix"));
        let default_counterparty = new_connection_counterparty(
            string::utf8(b"counterparty-client"),
            string::utf8(b"connection-0"),
            new_merkleprefix(default_counterparty_prefix)
        );

        let connection = SmartTable::borrow_with_default(
            &store.connections,
            connection_id,
            &new_connection_end(
                string::utf8(b"client_id"),
                vector::empty<Version>(),
                0,
                0,
                default_counterparty
            )
        );

        let encoded_connection = encode(connection);
        let key = IBCCommitment::connection_commitment_key(connection_id);
        let hash = IBCCommitment::keccak256(encoded_connection);
        SmartTable::upsert(&mut store.commitments, key, hash);
    }
    public fun connection_open_init(
        client_id: String,
        version: Version,
        counterparty: IbcCoreConnectionV1Counterparty,
        delay_period: u64,
        relayer: address
    ): String acquires IBCStore {

        let connection_id = generate_connection_identifier();
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let connection = new_connection_end(
            client_id,
            vector::empty<Version>(),
            0,
            delay_period,
            counterparty
        );

        if (get_connection_state(&connection) != 0) {
            abort(E_CONNECTION_ALREADY_EXISTS);
        };

        if (vector::length(&get_version_features(&version)) > 0) {
            if (!is_supported_version(&get_compatible_versions(), &version)) {
                abort(ERR_UNSUPPORTED_VERSION);
            };
            vector::push_back(&mut connection.versions, version);
        } else {
            set_supported_versions(get_compatible_versions(), &mut connection.versions);
        };

        connection.state = 1; // STATE_INIT
        connection.delay_period = delay_period;
        connection.counterparty = counterparty;
        SmartTable::upsert(&mut store.connections, connection_id, connection);

        update_connection_commitment(store, connection_id);

        event::emit_event(&mut store.connection_open_init_events, ConnectionOpenInit {
            connection_id: connection_id,
            client_id: client_id,
            counterparty_client_id: counterparty.client_id
        });

        connection_id
    }

    public fun get_compatible_versions(): vector<Version> {
        let versions = vector::empty<Version>();
        vector::push_back(&mut versions, default_ibc_version());
        versions
    }

    #[view]
    public fun get_connection(connection_id: String): ConnectionEnd acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let connection = SmartTable::borrow_with_default(
            &store.connections,
            connection_id,
            &new_connection_end(
                string::utf8(b""),
                vector::empty<Version>(),
                0,
                0,
                new_connection_counterparty(string::utf8(b""), string::utf8(b""), new_merkleprefix(vector::empty<u8>()))
            )
        );
        *connection
    }

    // Getter function to retrieve a connection commitment by its ID
    #[view]
    public fun get_connection_commitment(connection_id: String): vector<u8> acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let key = IBCCommitment::connection_commitment_key(connection_id);
        let commitment = SmartTable::borrow_with_default(
            &store.commitments,
            key,
            &vector::empty<u8>()
        );
        *commitment
    }

    public fun connection_open_try(
        counterparty: IbcCoreConnectionV1Counterparty,
        delay_period: u64,
        client_id: String,
        client_state_bytes: vector<u8>,
        counterparty_versions: vector<Version>,
        proof_init: Any,
        proof_client: Any,
        proof_consensus: vector<u8>,
        proof_height: height::Height,
        consenseus_height: height::Height,
        relayer: address
    ):String acquires IBCStore {
        // Generate a new connection identifier
        let connection_id = generate_connection_identifier();

        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        // Retrieve the connection from the store
        let existing_connection = SmartTable::borrow_with_default(
            &store.connections,
            connection_id,
            &new_connection_end(
                string::utf8(b""),
                vector::empty<Version>(),
                0,
                0,
                new_connection_counterparty(string::utf8(b""), string::utf8(b""), new_merkleprefix(vector::empty<u8>()))
            )
        );

        // Create a mutable copy of the connection
        let connection = *existing_connection;

        // Check if the connection is already initialized
        if (get_connection_state(&connection) != 0) {
            abort(E_CONNECTION_ALREADY_EXISTS);
        };

        // Set the client ID and versions
        connection.client_id = client_id;
        let version = pick_version(&get_compatible_versions(), &counterparty_versions);
        vector::push_back(&mut connection.versions, version);

        // Set the state, delay period, and counterparty
        connection.state = 2; // STATE_TRYOPEN
        connection.delay_period = delay_period;
        connection.counterparty = counterparty;

        SmartTable::upsert(&mut store.connections, connection_id, connection);

        // Create the expected connection
        let expected_connection = new_connection_end(
            counterparty.client_id,
            counterparty_versions,
            1, // STATE_INIT
            delay_period,
            new_connection_counterparty(client_id, string::utf8(b""), new_merkleprefix(IBCCommitment::keccak256(string::utf8(b"ibc"))))
        );

        // Verify the connection state
        if (!verify_connection_state(
            &connection,
            proof_height,
            proof_init,
            counterparty.connection_id,
            expected_connection
        )) {
            abort(E_INVALID_PROOF);
        };

        // Verify the client state
        if (!verify_client_state(
            &connection,
            proof_height,
            IBCCommitment::client_state_commitment_key(connection.counterparty.client_id),
            proof_client,
            client_state_bytes
        )) {
            abort(E_INVALID_PROOF);
        };

        // Update the connection commitment
        update_connection_commitment(store, connection_id);

        // Emit the event
        event::emit_event(&mut store.connection_open_try_events, ConnectionOpenTry {
            connection_id,
            client_id: client_id,
            counterparty_client_id: counterparty.client_id,
            counterparty_connection_id: counterparty.connection_id
        });

        connection_id
    }

    public fun connection_open_ack(
        connection_id: String,
        client_state_bytes: vector<u8>,
        version: Version,
        proof_try: Any,
        proof_client: Any,
        proof_consensus: vector<u8>,
        counterparty_connection_id: String,
        proof_height: height::Height,
        consenseus_height: height::Height,
        relayer: address
    ) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let existing_connection = SmartTable::borrow_with_default(
            &store.connections,
            connection_id,
            &new_connection_end(
                string::utf8(b""),
                vector::empty<Version>(),
                0,
                0,
                new_connection_counterparty(string::utf8(b""), string::utf8(b""), new_merkleprefix(vector::empty<u8>()))
            )
        );

        let connection = *existing_connection;

        if (get_connection_state(&connection) != 1) { // STATE_INIT
            abort(ERR_INVALID_CONNECTION_STATE);
        };

        if (!is_supported_version(&connection.versions, &version)) {
            abort(ERR_UNSUPPORTED_VERSION);
        };

        let expected_counterparty = new_connection_counterparty(
            connection.client_id,
            connection_id,
            new_merkleprefix(IBCCommitment::keccak256(string::utf8(b"ibc")))
        );

        let expected_connection = new_connection_end(
            connection.counterparty.client_id,
            vector::singleton(version),
            2, // STATE_TRYOPEN
            connection.delay_period,
            expected_counterparty
        );

        if (!verify_connection_state(
            &connection,
            proof_height,
            proof_try,
            counterparty_connection_id,
            expected_connection
        )) {
            abort(E_INVALID_PROOF);
        };

        if (!verify_client_state(
            &connection,
            proof_height,
            IBCCommitment::client_state_commitment_key(connection.counterparty.client_id),
            proof_client,
            client_state_bytes
        )) {
            abort(E_INVALID_PROOF);
        };

        connection.state = 3; // STATE_OPEN
        copy_versions(&vector::singleton(version), &mut connection.versions);
        connection.counterparty.connection_id = counterparty_connection_id;

        SmartTable::upsert(&mut store.connections, connection_id, connection);

        update_connection_commitment(store, connection_id);

        event::emit_event(&mut store.connection_open_ack_events, ConnectionOpenAck {
            connection_id,
            client_id: connection.client_id,
            counterparty_client_id: connection.counterparty.client_id,
            counterparty_connection_id: connection.counterparty.connection_id
        });
    }

    public fun connection_open_confirm(
        connection_id: String,
        proof_ack: Any,
        proof_height: height::Height,
        relayer: address
    ) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let existing_connection = SmartTable::borrow_with_default(
            &store.connections,
            connection_id,
            &new_connection_end(
                string::utf8(b""),
                vector::empty<Version>(),
                0,
                0,
                new_connection_counterparty(string::utf8(b""), string::utf8(b""), new_merkleprefix(vector::empty<u8>()))
            )
        );

        let connection = *existing_connection;

        if (get_connection_state(&connection) != 2) { // STATE_TRYOPEN
            abort(ERR_INVALID_CONNECTION_STATE);
        };

        let expected_counterparty = new_connection_counterparty(
            connection.client_id,
            connection_id,
            new_merkleprefix(IBCCommitment::keccak256(string::utf8(b"ibc")))
        );

        let expected_connection = new_connection_end(
            connection.counterparty.client_id,
            connection.versions,
            3, // STATE_OPEN
            connection.delay_period,
            expected_counterparty
        );

        if (!verify_connection_state(
            &connection,
            proof_height,
            proof_ack,
            connection.counterparty.connection_id,
            expected_connection
        )) {
            abort(E_INVALID_PROOF);
        };

        connection.state = 3; // STATE_OPEN
        SmartTable::upsert(&mut store.connections, connection_id, connection);

        update_connection_commitment(store, connection_id);

        event::emit_event(&mut store.connection_open_confirm_events, ConnectionOpenConfirm {
            connection_id,
            client_id: connection.client_id,
            counterparty_client_id: connection.counterparty.client_id,
            counterparty_connection_id: connection.counterparty.connection_id
        });
    }

    public fun verify_supported_feature(version: &Version, feature: String): bool {
        let h = IBCCommitment::keccak256(feature);
        let i = 0;
        while (i < vector::length(&version.features)) {
            if (IBCCommitment::keccak256(*vector::borrow(&version.features, i)) == h) {
                return true;
            };
            i = i + 1;
        };
        false
    }

    public fun to_string(ordering: u8): String {
        let return_val = string::utf8(b"ORDER_INVALID");
        if (ordering == 1) { // Order.ORDER_UNORDERED
            return_val = string::utf8(b"ORDER_UNORDERED");
        } else if (ordering == 2) { // Order.ORDER_ORDERED
            return_val = string::utf8(b"ORDER_ORDERED");
        };
        return_val
    }


    public fun get_counterparty_hops(connection_id: String): vector<String> acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let connection = SmartTable::borrow_with_default(
            &store.connections,
            connection_id,
            &new_connection_end(
                string::utf8(b""),
                vector::empty<Version>(),
                0,
                0,
                new_connection_counterparty(string::utf8(b""), string::utf8(b""), new_merkleprefix(vector::empty<u8>()))
            )
        );
        let hops = vector::empty<String>();
        vector::push_back(&mut hops, connection.counterparty.connection_id);
        hops
    }

    public fun generate_channel_identifier(): String acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let next_sequence_bytes = SmartTable::borrow_with_default(
            &store.commitments,
            b"nextChannelSequence",
            &bcs::to_bytes<u64>(&0)
        );
        let next_sequence = from_bcs::to_u64(*next_sequence_bytes);
        let identifier = string::utf8(b"channel-");
        string::append(&mut identifier, string_utils::to_string(&next_sequence));
        let new_sequence = next_sequence + 1;
        SmartTable::upsert(
            &mut store.commitments,
            b"nextChannelSequence",
            bcs::to_bytes(&new_sequence)
        );
        identifier
    }

    public fun ensure_connection_state(connection_id: String): ConnectionEnd acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let connection = SmartTable::borrow_with_default(
            &store.connections,
            connection_id,
            &new_connection_end(
                string::utf8(b""),
                vector::empty<Version>(),
                0,
                0,
                new_connection_counterparty(string::utf8(b""), string::utf8(b""), new_merkleprefix(vector::empty<u8>()))
            )
        );
        if (connection.state != 3) { // STATE_OPEN
            abort(ERR_INVALID_CONNECTION_STATE);
        };
        *connection
    }
    public fun ensure_connection_feature(connection_hops: vector<String>, ordering: u8): (String, ConnectionEnd) acquires IBCStore {
        if (vector::length(&connection_hops) != 1) {
            abort(E_CONN_NOT_SINGLE_HOP);
        };
        let connection_id = *vector::borrow(&connection_hops, 0);
        let connection = ensure_connection_state(connection_id);
        if (vector::length(&connection.versions) != 1) {
            abort(E_CONN_NOT_SINGLE_VERSION);
        };
        let version = *vector::borrow(&connection.versions, 0);
        if (!verify_supported_feature(&version, to_string(ordering))) {
            abort(E_UNSUPPORTED_FEATURE);
        };
        (connection_id, connection)
    }

    // TODO: Here the implementation returns false if there is any uppercase character in the string
    // eg: "portId" will return false but "port-0" will return true. Is this correct way to check?
    public fun is_lowercase(s: &String): bool {
        let bytes = bcs::to_bytes(s);
        let i = 1; // First byte is the length of the string
        while (i < vector::length(&bytes)) {
            let byte = *vector::borrow(&bytes, i);
            // if (byte < 0x61 || byte > 0x7A) { // ASCII values for 'a' and 'z'
            //     return false;
            // };
            if (byte > 0x40 && byte < 0x5b) {
                return false;
            };
            i = i + 1;
        };
        true
    }
    
    public fun update_channel_commitment(port_id: String, channel_id: String) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let channel_port = ChannelPort{port_id:port_id, channel_id};
        let channel = SmartTable::borrow(
            &store.channels,
            channel_port
        );

        let hash = IBCCommitment::keccak256(encode_channel(channel));
        let key = IBCCommitment::channel_commitment_key(port_id, channel_id);
        SmartTable::upsert(&mut store.commitments, key, hash);
    }


    public fun verify_channel_state(
        connection: &ConnectionEnd,
        height: height::Height,
        proof: Any,
        port_id: String,
        channel_id: String,
        channel_bytes: vector<u8>
    ): bool {
        let path = IBCCommitment::channel_commitment_key(port_id, channel_id);
        let (result, error_code) = IBC::LightClient::verify_membership(
            connection.client_id,
            height,
            proof,
            connection.counterparty.prefix.key_prefix,
            path,
            channel_bytes
        );
        assert!(error_code == 0, E_INVALID_PROOF);
        true
    }

    public fun claim_capability(name: String, addr: address) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let default_addr: address = @0x0;
        let existing_addr = SmartTable::borrow_with_default(&store.capabilities, name, &default_addr);
        if (*existing_addr != default_addr) {
            abort E_CAPABILITY_ALREADY_CLAIMED;
        };
        SmartTable::upsert(&mut store.capabilities, name, addr);
    }

    public fun create_new_table(): SmartTable<String, SmartTable<String, IbcCoreChannelV1Channel>> {
        let channel_table = SmartTable::new<String, SmartTable<String, IbcCoreChannelV1Channel>>();
        channel_table
    }
    public fun channel_open_init(
        msg_port_id: String,
        msg_channel: IbcCoreChannelV1Channel,
        relayer: address
    ): String acquires IBCStore {
        // string::utf8(b"test return")
        if (!is_lowercase(&msg_port_id)) {
            abort E_PORT_ID_MUST_BE_LOWERCASE;
        };

        let (connection_id, _) = ensure_connection_feature(
            msg_channel.connection_hops, 
            msg_channel.ordering
        );

        if (msg_channel.state != 1) { // STATE_INIT
            abort E_INVALID_CHANNEL_STATE;
        };

        if (string::length(&msg_channel.counterparty.channel_id) != 0) {
            abort E_COUNTERPARTY_CHANNEL_NOT_EMPTY;
        };

        let channel_id = generate_channel_identifier();

        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let channel_port = ChannelPort{port_id:msg_port_id, channel_id};
        SmartTable::upsert(&mut store.channels, channel_port, msg_channel);
        
        // string::utf8(b"test return")

        SmartTable::upsert(
            &mut store.commitments,
            IBCCommitment::next_sequence_send_commitment_key(msg_port_id, channel_id),
            bcs::to_bytes(&1)
        );

        SmartTable::upsert(
            &mut store.commitments,
            IBCCommitment::next_sequence_recv_commitment_key(msg_port_id, channel_id),
            bcs::to_bytes(&1)
        );

        SmartTable::upsert(
            &mut store.commitments,
            IBCCommitment::next_sequence_ack_commitment_key(msg_port_id, channel_id),
            bcs::to_bytes(&1)
        );

        // TODO: Is it problem to emit event before updating the commitment?

        event::emit_event(&mut store.channel_open_init_events, ChannelOpenInit {
            port_id: msg_port_id,
            channel_id: channel_id,
            counterparty_port_id: msg_channel.counterparty.port_id,
            connection_id: connection_id,
            version: msg_channel.version
        });

        update_channel_commitment(msg_port_id, channel_id);

        // Hardcoded call to IBCModule::on_chan_open_init
        let result = IBCModule::on_chan_open_init(
            msg_channel.ordering,
            msg_channel.connection_hops,
            msg_port_id,
            channel_id,
            msg_channel.counterparty.port_id,
            msg_channel.counterparty.channel_id,
            msg_channel.version,
        );

        claim_capability(
            IBCCommitment::channel_capability_path(msg_port_id, channel_id),
            relayer
        );

        channel_id
    }

    public fun channel_open_ack(
        port_id: String,
        channel_id: String,
        counterparty_channel_id: String,
        counterparty_version: String,
        proof_try: Any,
        proof_height: height::Height
    ) acquires IBCStore {
        // Retrieve the channel from the store
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let channel_port = ChannelPort { port_id, channel_id };
        let channel = SmartTable::borrow_mut(&mut store.channels, channel_port);

        if (channel.state != 1) { // STATE_INIT
            abort E_INVALID_CHANNEL_STATE;
        };

        event::emit_event(&mut store.channel_open_ack_events, ChannelOpenAck {
            port_id,
            channel_id,
            counterparty_port_id: channel.counterparty.port_id,
            counterparty_channel_id,
            connection_id: *vector::borrow(&channel.connection_hops, 0)
        });

        let connection = ensure_connection_state(*vector::borrow(&channel.connection_hops, 0));

        let expected_counterparty = new_channel_counterparty(port_id, channel_id);
        let expected_channel = new_channel(
            2, // STATE_TRYOPEN
            channel.ordering,
            expected_counterparty,
            get_counterparty_hops(*vector::borrow(&channel.connection_hops, 0)),
            counterparty_version
        );

        if (!verify_channel_state(
            &connection,
            proof_height,
            proof_try,
            channel.counterparty.port_id,
            counterparty_channel_id,
            bcs::to_bytes(&expected_channel)
        )) {
            abort E_INVALID_PROOF;
        };

        channel.state = 3; // STATE_OPEN
        channel.version = counterparty_version;
        channel.counterparty.channel_id = counterparty_channel_id;

        update_channel_commitment(port_id, channel_id);

        IBCModule::on_chan_open_ack(
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version
        );
    }
}   
