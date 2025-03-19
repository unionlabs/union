// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's aptos subdirectory
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
    use aptos_framework::function_info::FunctionInfo;
    use std::signer;
    use std::vector;
    use aptos_std::smart_table::{Self, SmartTable};
    use aptos_std::table::{Self, Table};
    use aptos_std::aptos_hash;
    use std::from_bcs;
    use std::event;
    use std::bcs;
    use std::object;
    use std::string::{String};
    use std::hash;
    use std::option::{Self, Option};
    use std::string_utils;
    use ibc::commitment;
    use ibc::light_client;
    use ibc::connection_end::{Self, ConnectionEnd};
    use ibc::channel::{Self, Channel};
    use ibc::packet::{Self, Packet};
    use ibc::dispatcher;
    use ibc::create_lens_client_event::{Self, CreateLensClientEvent};

    friend ibc::recv_packet;
    friend ibc::channel_handshake;
    friend ibc::acknowledge_packet;
    friend ibc::timeout_packet;

    const IBC_APP_SEED: vector<u8> = b"ibc-union-app-v1";
    const COMMITMENT_MAGIC: vector<u8> = x"0100000000000000000000000000000000000000000000000000000000000000";

    const CLIENT_TYPE_COMETBLS: vector<u8> = b"cometbls";

    const CHAN_STATE_UNINITIALIZED: u8 = 0;
    const CHAN_STATE_INIT: u8 = 1;
    const CHAN_STATE_TRYOPEN: u8 = 2;
    const CHAN_STATE_OPEN: u8 = 3;
    const CHAN_STATE_CLOSED: u8 = 4;

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
    const E_TIMEOUT_MUST_BE_SET: u64 = 1044;
    const E_PACKET_SEQUENCE_ACK_SEQUENCE_MISMATCH: u64 = 1045;

    #[event]
    struct CreateClient has copy, drop, store {
        client_id: u32,
        client_type: String,
        counterparty_chain_id: String
    }

    #[event]
    struct CreateLensClient has copy, drop, store {
        client_id: u32,
        l2_chain_id: String,
        l1_client_id: u32,
        l2_client_id: u32
    }

    #[event]
    struct UpdateClient has copy, drop, store {
        client_id: u32,
        client_type: String,
        counterparty_height: u64
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
        counterparty_port_id: vector<u8>,
        connection_id: u32,
        version: String
    }

    #[event]
    struct ChannelOpenTry has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_port_id: vector<u8>,
        counterparty_channel_id: u32,
        connection_id: u32,
        version: String
    }

    #[event]
    struct ChannelOpenAck has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_port_id: vector<u8>,
        counterparty_channel_id: u32,
        connection_id: u32
    }

    #[event]
    struct ChannelOpenConfirm has copy, drop, store {
        port_id: String,
        channel_id: u32,
        counterparty_port_id: vector<u8>,
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
    struct RecvIntentPacket has drop, store {
        packet: Packet,
        maker: address,
        maker_msg: vector<u8>
    }

    #[event]
    struct PacketRecv has drop, store {
        packet: Packet,
        maker: address,
        maker_msg: vector<u8>
    }

    #[event]
    struct PacketSend has drop, store {
        source_channel_id: u32,
        destination_channel_id: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64
    }

    #[event]
    struct TimeoutPacket has drop, store {
        packet: Packet
    }

    #[event]
    struct PacketAck has drop, store {
        packet: Packet,
        acknowledgement: vector<u8>,
        maker: address
    }

    #[event]
    struct WriteAck has drop, store {
        packet: Packet,
        acknowledgement: vector<u8>
    }

    #[event]
    struct SubmitMisbehaviour has drop, store {
        client_id: u32,
        client_type: String
    }

    struct Port<phantom T: key + store + drop> has key, copy, drop, store {
        port_id: address
    }

    struct IBCStore has key {
        /// Data that is stored here are verified in the counterparty chain
        /// Using `Table` eases up the proof verification compared to `SmartTable`
        commitments: Table<vector<u8>, vector<u8>>,
        connections: SmartTable<u32, ConnectionEnd>,
        channels: SmartTable<u32, Channel>,
        channel_to_module: SmartTable<u32, address>,
        client_id_to_type: SmartTable<u32, String>
    }

    struct SignerRef has key {
        self_ref: object::ExtendRef
    }

    public(friend) fun get_port_id<T: key + store + drop>(): address acquires Port {
        borrow_global<Port<T>>(get_vault_addr()).port_id
    }

    /// Register a dispatchable ibc application. The IBC apps will register themselves by calling
    /// this function.
    /// WARNING: Type T acts as a witness. Only the module owner should be able to create
    /// an instance of this type.
    ///
    /// The callback function `cb` here will be the single entrypoint of the apps. Which means
    /// `on_recv_packet`, `on_channel_open_init` and etc. callbacks will all invoke the given
    /// callback. And instead of having some function arguments, the callback function will read
    /// the data from the storage interface that is provided by this contract.
    ///
    /// * `ibc_app`: The signer of the calling module
    /// * `cb`: App's callback function. The function needs to have the following signature
    ///         to match the `dispatchable_fungible_asset` spec. Check our example contracts and
    ///         docs to see how it works:
    ///         - `public fun on_packet<T: key>(_store: Object<T>): u64`
    /// * `witness`: The witness where only the owning module can create an instance.
    public fun register_application<T: key + store + drop>(
        ibc_app: &signer, cb: FunctionInfo, witness: T
    ) acquires SignerRef {
        dispatcher::register<T>(
            cb, witness, bcs::to_bytes(&signer::address_of(ibc_app))
        );
        move_to(
            &get_ibc_signer(),
            Port<T> { port_id: signer::address_of(ibc_app) }
        );
    }

    /// Create a client with an initial client and consensus state.
    ///
    /// * `client_type`: Check the `light_client` module for supported client types.
    /// * `client_state`: The initial state of the client. The encoding is defined by the underlying client implementation.
    /// * `consensus_state`: The consensus state at an initial height. The encoding is defined by the underlying client implementation.
    public entry fun create_client(
        sender: &signer,
        client_type: String,
        client_state: vector<u8>,
        consensus_state: vector<u8>
    ) acquires IBCStore, SignerRef {
        create_client_impl(
            client_type,
            client_state,
            consensus_state,
            |client_type, ibc_signer, client_id, client_state_bytes, consensus_state_bytes
            | light_client::create_client(
                sender,
                client_type,
                ibc_signer,
                client_id,
                client_state_bytes,
                consensus_state_bytes
            ),
            |client_type, client_id| light_client::status(client_type, client_id),
            |client_type, client_id| light_client::latest_height(client_type, client_id)
        );
    }

    public(friend) inline fun create_client_impl(
        client_type: String,
        client_state: vector<u8>,
        consensus_state: vector<u8>,
        lc_create_client_fn: |String, &signer, u32, vector<u8>, vector<u8>| (
            vector<u8>, vector<u8>, String, Option<CreateLensClientEvent>
        ),
        lc_status_fn: |String, u32| u64,
        lc_latest_height_fn: |String, u32| u64
    ) acquires IBCStore, SignerRef {
        let client_id = generate_client_identifier();
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let (client_state, consensus_state, counterparty_chain_id, lens_client_event) =
            lc_create_client_fn(
                client_type,
                &get_ibc_signer(),
                client_id,
                client_state,
                consensus_state
            );

        if (option::is_some(&lens_client_event)) {
            let event = option::extract(&mut lens_client_event);
            event::emit(
                CreateLensClient {
                    client_id: create_lens_client_event::client_id(&event),
                    l2_chain_id: create_lens_client_event::l2_chain_id(&event),
                    l1_client_id: create_lens_client_event::l1_client_id(&event),
                    l2_client_id: create_lens_client_event::l2_client_id(&event)
                }
            );
        };

        smart_table::upsert(&mut store.client_id_to_type, client_id, client_type);

        // TODO(aeryz): fetch these status from proper exported consts
        assert!(lc_status_fn(client_type, client_id) == 0, E_CLIENT_NOT_ACTIVE);

        // Update commitments
        table::upsert(
            &mut store.commitments,
            commitment::client_state_commitment_key(client_id),
            client_state
        );

        let latest_height = lc_latest_height_fn(client_type, client_id);

        table::upsert(
            &mut store.commitments,
            commitment::consensus_state_commitment_key(client_id, latest_height),
            consensus_state
        );

        event::emit(CreateClient { client_id, client_type, counterparty_chain_id });
    }

    /// Execute the init phase of the connection handshake.
    ///
    /// * `client_id`: The light client, which will do all the header and membership verifications on this chain.
    /// * `counterparty_client_id`: The light client that runs on the counterparty chain.
    public entry fun connection_open_init(
        client_id: u32, counterparty_client_id: u32
    ) acquires IBCStore {
        let connection_id = generate_connection_identifier();

        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let connection =
            connection_end::new(
                CONN_STATE_INIT,
                client_id,
                counterparty_client_id,
                0
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

    /// Execute the try phase of the connection handshake.
    ///
    /// * `counterparty_client_id`: The light client that runs on the counterparty chain.
    /// * `counterparty_connection_id`: The connection ID that is created during `connection_open_init` on
    ///   the counterparty chain.
    /// * `client_id`: The light client, which will do all the header and membership verifications on this chain.
    /// * `proof_init`: The membership proof of the connection state in the counterparty chain. The encoding is defined
    ///   by the light client (`client_id`).
    /// * `proof_height`: The height at when `proof_init` was generated.
    public entry fun connection_open_try(
        counterparty_client_id: u32,
        counterparty_connection_id: u32,
        client_id: u32,
        proof_init: vector<u8>,
        proof_height: u64
    ) acquires IBCStore {
        connection_open_try_impl(
            counterparty_client_id,
            counterparty_connection_id,
            client_id,
            proof_init,
            proof_height,
            |client_type, client_id, proof_height, proof_init, key, value| {
                light_client::verify_membership(
                    client_type,
                    client_id,
                    proof_height,
                    proof_init,
                    key,
                    value
                )
            }
        )
    }

    inline fun connection_open_try_impl(
        counterparty_client_id: u32,
        counterparty_connection_id: u32,
        client_id: u32,
        proof_init: vector<u8>,
        proof_height: u64,
        lc_verify_membership: |String, u32, u64, vector<u8>, vector<u8>, vector<u8>| u64
    ) acquires IBCStore {
        let client_type = client_id_to_type(client_id);
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
                    counterparty_connection_id
                )
            );

        // Create the expected connection
        let expected_connection =
            connection_end::new(
                CONN_STATE_INIT,
                counterparty_client_id,
                client_id,
                0 // counterparty_connection_id
            );

        // Verify the connection state
        let err =
            lc_verify_membership(
                client_type,
                client_id,
                proof_height,
                proof_init,
                commitment::connection_commitment_key(counterparty_connection_id),
                aptos_hash::keccak256(connection_end::encode(&expected_connection))
            );
        assert!(err == 0, err);

        event::emit(
            ConnectionOpenTry {
                connection_id,
                client_id,
                counterparty_client_id,
                counterparty_connection_id
            }
        );

        commit_connection(connection_id, *connection);
    }

    /// Execute the ack phase of the connection handshake.
    ///
    /// * `connection_id`: The connection ID that is created during `connection_open_init` on
    ///   this chain.
    /// * `counterparty_connection_id`: The connection ID that is created during `connection_open_try` on
    ///   the counterparty chain.
    /// * `proof_try`: The membership proof of the connection state in the counterparty chain. The encoding is defined
    ///   by the light client (`client_id`).
    /// * `proof_height`: The height at when `proof_try` was generated.
    public entry fun connection_open_ack(
        connection_id: u32,
        counterparty_connection_id: u32,
        proof_try: vector<u8>,
        proof_height: u64
    ) acquires IBCStore {
        assert!(
            smart_table::contains(
                &borrow_global<IBCStore>(get_vault_addr()).connections,
                connection_id
            ),
            E_CONNECTION_DOES_NOT_EXIST
        );

        let client_type =
            client_id_to_type(
                connection_end::client_id(
                    smart_table::borrow(
                        &borrow_global<IBCStore>(get_vault_addr()).connections,
                        connection_id
                    )
                )
            );
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

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
                connection_id
            );

        // Verify the connection state
        let err =
            verify_connection_state(
                client_type,
                connection,
                proof_height,
                proof_try,
                counterparty_connection_id,
                expected_connection
            );
        assert!(err == 0, err);

        connection_end::set_state(connection, CONN_STATE_OPEN);
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
                )
            }
        );

        commit_connection(connection_id, *connection);
    }

    /// Execute the confirm phase of the connection handshake.
    ///
    /// * `connection_id`: The connection ID that is created during `connection_open_try` on
    ///   this chain.
    /// * `proof_ack`: The membership proof of the connection state in the counterparty chain. The encoding is defined
    ///   by the light client (`client_id`).
    /// * `proof_height`: The height at when `proof_ack` was generated.
    public entry fun connection_open_confirm(
        connection_id: u32, proof_ack: vector<u8>, proof_height: u64
    ) acquires IBCStore {
        let client_type =
            client_id_to_type(
                connection_end::client_id(
                    smart_table::borrow(
                        &borrow_global<IBCStore>(get_vault_addr()).connections,
                        connection_id
                    )
                )
            );
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

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
                connection_id
            );
        let counterparty_connection_id =
            connection_end::counterparty_connection_id(connection);

        // Verify the connection state
        let err =
            verify_connection_state(
                client_type,
                connection,
                proof_height,
                proof_ack,
                counterparty_connection_id,
                expected_connection
            );
        assert!(err == 0, err);

        connection_end::set_state(connection, CONN_STATE_OPEN);

        event::emit(
            ConnectionOpenConfirm {
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

    /// Update the light client with id `client_id` using `client_message`.
    ///
    /// * `client_id`: The light client that will be updated.
    /// * `client_message`: The light client defined update data. It's the caller's responsibility to gather and encode
    /// the client update data. The light client just needs to make sure altering this data can NEVER make it
    /// transition to an invalid state.
    public entry fun update_client(
        client_id: u32, client_message: vector<u8>
    ) acquires IBCStore {
        let client_type = client_id_to_type(client_id);
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        assert!(
            table::contains(
                &store.commitments,
                commitment::client_state_commitment_key(client_id)
            ),
            E_CLIENT_NOT_FOUND
        );

        if (light_client::check_for_misbehaviour(client_type, client_id, client_message)) {
            event::emit(SubmitMisbehaviour { client_id, client_type: client_type });
            return
        };

        let (client_state, consensus_states, heights) =
            light_client::update_client(client_type, client_id, client_message);

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
                UpdateClient { client_id, client_type, counterparty_height: height }
            );

            i = i + 1;
        };
    }

    /// Report a misbehaviour to the client such as the target chain being forked, a finalized state being reverted, etc.
    /// The light clients are expected to freeze themselves if the misbehaviour is valid. Freezing means the light
    /// client will no longer accept any updates. Hence packet relaying after that point will not be possible using that
    /// client.
    ///
    /// * `client_id`: The light client which will verify and act upon the misbehaviour.
    /// * `misbehaviour`: Light client defined misbehaviour data. It's the responsibility of the caller to gather and encode
    ///   the correct data. The light client MUST detect any invalid misbehaviors and ignore those.
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

        let client_type = client_id_to_type(client_id);

        light_client::report_misbehaviour(client_type, client_id, misbehaviour);

        event::emit(SubmitMisbehaviour { client_id, client_type });
    }

    /// Execute the init phase of the channel handshake. `T` is the witness type of the target module that is
    /// previously been registered to this contract.
    ///
    /// * `port_id`: The address of the IBC app on this chain that will use this channel.
    /// * `counterparty_port_id`: The port ID of the IBC app that runs on the counterparty chain.
    /// * `connection_id`: The ID of the connection that this channel will use. The light client that is used
    ///   during the connection handshake will be used to verify all the packets flowing through this channel.
    /// * `version`: The version of the channel. Note that this must be the same in both ends of the channel.
    public fun channel_open_init<T: key + store + drop>(
        port_id: address,
        counterparty_port_id: vector<u8>,
        connection_id: u32,
        version: String
    ): (u32, u32) acquires IBCStore, Port {
        let port = borrow_global<Port<T>>(get_vault_addr());
        assert!(port.port_id == port_id, E_UNAUTHORIZED);

        ensure_connection_state(connection_id);

        let channel_id = generate_channel_identifier();

        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        smart_table::upsert(&mut store.channel_to_module, channel_id, port_id);

        let port_id = address_to_string(port_id);

        let channel =
            channel::new(
                CHAN_STATE_INIT,
                connection_id,
                0,
                counterparty_port_id,
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

        event::emit(
            ChannelOpenInit {
                port_id: port_id,
                counterparty_port_id: counterparty_port_id,
                channel_id: channel_id,
                connection_id: connection_id,
                version: version
            }
        );

        (channel_id, connection_id)
    }

    /// Execute the try phase of the channel handshake. `T` is the witness type of the target module that is
    /// previously been registered to this contract.
    ///
    /// * `port_id`: The address of the IBC app on this chain that will use this channel.
    /// * `connection_id`: The ID of the connection that this channel will use. The light client that is used
    ///   during the connection handshake will be used to verify all the packets flowing through this channel.
    /// * `counterparty_channel_id`: The channel ID of on the counterparty chain that we want to connect to.
    /// * `counterparty_port_id`: The port ID of the IBC app that runs on the counterparty chain.
    /// * `version`: The version of the channel. Note that this must be the same in both ends of the channel.
    /// * `counterparty_version`: The version of the channel. Note that this must be the same in both ends of the channel.
    /// * `proof_init`: The membership proof of the channel state in the counterparty chain. The encoding is defined
    ///   by the light client (`client_id`).
    /// * `proof_height`: The height at when `proof_init` was generated.
    public fun channel_open_try<T: key + store + drop>(
        port_id: address,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String,
        counterparty_version: String,
        proof_init: vector<u8>,
        proof_height: u64
    ): u32 acquires IBCStore, Port {
        let port = borrow_global<Port<T>>(get_vault_addr());
        assert!(port.port_id == port_id, E_UNAUTHORIZED);

        let client_id = ensure_connection_state(connection_id);

        let client_type = client_id_to_type(client_id);

        let expected_channel =
            channel::new(
                CHAN_STATE_INIT,
                get_counterparty_connection(connection_id),
                0,
                bcs::to_bytes(&port_id),
                counterparty_version
            );

        let err =
            verify_channel_state(
                client_type,
                client_id,
                proof_height,
                proof_init,
                counterparty_channel_id,
                expected_channel
            );
        assert!(err == 0, err);

        let channel_id = generate_channel_identifier();

        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        smart_table::upsert(&mut store.channel_to_module, channel_id, port_id);

        let port_id = address_to_string(port_id);

        event::emit(
            ChannelOpenTry {
                port_id,
                channel_id,
                counterparty_channel_id,
                counterparty_port_id,
                connection_id,
                version: counterparty_version
            }
        );

        let channel =
            channel::new(
                CHAN_STATE_TRYOPEN,
                connection_id,
                counterparty_channel_id,
                counterparty_port_id,
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

        channel_id
    }

    /// Execute the ack phase of the channel handshake. `T` is the witness type of the target module that is
    /// previously been registered to this contract.
    ///
    /// * `port_id`: The address of the IBC app on this chain that will use this channel.
    /// * `channel_id`: The ID of the channel on this chain.
    /// * `counterparty_version`: The version of the channel. Note that this must be the same in both ends of the channel.
    /// * `counterparty_channel_id`: The channel ID of on the counterparty chain that we want to connect to.
    /// * `proof_try`: The membership proof of the channel state in the counterparty chain. The encoding is defined
    ///   by the light client (`client_id`).
    /// * `proof_height`: The height at when `proof_try` was generated.
    public fun channel_open_ack<T: key + store + drop>(
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
                get_counterparty_connection(connection_id),
                channel_id,
                bcs::to_bytes(&port.port_id),
                counterparty_version
            );

        let client_type = client_id_to_type(client_id);

        let err =
            verify_channel_state(
                client_type,
                client_id,
                proof_height,
                proof_try,
                counterparty_channel_id,
                expected_channel
            );
        assert!(err == 0, err);

        channel::set_state(&mut chan, CHAN_STATE_OPEN);
        channel::set_version(&mut chan, counterparty_version);
        channel::set_counterparty_channel_id(&mut chan, counterparty_channel_id);
        smart_table::upsert(
            &mut borrow_global_mut<IBCStore>(get_vault_addr()).channels,
            channel_id,
            chan
        );

        event::emit(
            ChannelOpenAck {
                port_id,
                channel_id,
                counterparty_channel_id,
                counterparty_port_id: *channel::counterparty_port_id(&chan),
                connection_id
            }
        );

        commit_channel(channel_id, chan);
    }

    /// Execute the confirm phase of the channel handshake. `T` is the witness type of the target module that is
    /// previously been registered to this contract.
    ///
    /// * `port_id`: The address of the IBC app on this chain that will use this channel.
    /// * `channel_id`: The ID of the channel on this chain.
    /// * `proof_ack`: The membership proof of the channel state in the counterparty chain. The encoding is defined
    ///   by the light client (`client_id`).
    /// * `proof_height`: The height at when `proof_ack` was generated.
    public fun channel_open_confirm<T: key + store + drop>(
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
                get_counterparty_connection(connection_id),
                channel_id,
                *channel::counterparty_port_id(&chan),
                *channel::version(&chan)
            );

        let client_type = client_id_to_type(client_id);

        let err =
            verify_channel_state(
                client_type,
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

        commit_channel(channel_id, chan);

        event::emit(
            ChannelOpenConfirm {
                port_id,
                channel_id,
                counterparty_channel_id: channel::counterparty_channel_id(&chan),
                counterparty_port_id: *channel::counterparty_port_id(&chan),
                connection_id: channel::connection_id(&chan)
            }
        );
    }

    /// Used for sending a packet to the counterparty chain. Note that this doesn't send the packet directly, it prepares the packet
    /// and emits a `PacketSend` event such that it's being picked up by a relayer.
    ///
    /// * `ibc_app`: The signer of the calling contract.
    /// * `source_port`: The address of the calling contract.
    /// * `source_channel`: The source channel that will be used for sending this packet.
    /// * `timeout_height`: The height in the COUNTERPARTY chain when this packet will time out. `0` means none, but note that both
    ///   `timeout_height` and `timeout_timestamp` cannot be `0`.
    /// * `timeout_timestamp`: The timestamp when this packet will time out. `0` means none, but note that both `timeout_height`
    ///   and `timeout_timestamp` cannot be `0`.
    /// * `data`: The app defined arbitrary data that will be relayed to the counterparty chain as is.
    public fun send_packet(
        ibc_app: &signer,
        source_port: address,
        source_channel: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        data: vector<u8>
    ): packet::Packet acquires IBCStore {
        authorize_app(ibc_app, source_port);

        if (timeout_timestamp != 0 && timeout_height == 0) {
            abort E_TIMEOUT_MUST_BE_SET
        };

        let channel = ensure_channel_state(source_channel);

        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let packet =
            packet::new(
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
            PacketSend {
                source_channel_id: source_channel,
                destination_channel_id: channel::counterparty_channel_id(&channel),
                data: data,
                timeout_height: timeout_height,
                timeout_timestamp: timeout_timestamp
            }
        );

        packet
    }

    /// Write an acknowledgement to the IBC store. The acknowledgement is written automatically after the a packet is received
    /// if the app sets an acknowledgement. But this function can be used when an asynchronous acknowledgement is needed.
    ///
    /// * `packet`: The packet that will be acknowledged.
    /// * `acknowledgement`: The acknowledgement that is defined by the IBC app.
    public fun write_acknowledgement(
        packet: packet::Packet, acknowledgement: vector<u8>
    ) acquires IBCStore {
        assert!(!vector::is_empty(&acknowledgement), E_ACKNOWLEDGEMENT_IS_EMPTY);

        ensure_channel_state(packet::destination_channel_id(&packet));

        let commitment_key =
            commitment::batch_receipts_commitment_key(
                packet::destination_channel_id(&packet),
                commitment::commit_packet(&packet)
            );
        inner_write_acknowledgement(commitment_key, packet, acknowledgement);
    }

    public(friend) fun inner_write_acknowledgement(
        commitment_key: vector<u8>, packet: Packet, acknowledgement: vector<u8>
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

        event::emit(WriteAck { packet, acknowledgement });
    }

    public(friend) fun timeout_packet<T: key + store + drop>(
        port_id: address,
        packet_source_channel: u32,
        packet_destination_channel: u32,
        packet_data: vector<u8>,
        packet_timeout_height: u64,
        packet_timeout_timestamp: u64,
        proof: vector<u8>,
        proof_height: u64,
        _next_sequence_recv: u64
    ): Packet acquires IBCStore, Port {
        let port = borrow_global<Port<T>>(get_vault_addr());
        assert!(port.port_id == port_id, E_UNAUTHORIZED);

        let packet =
            packet::new(
                packet_source_channel,
                packet_destination_channel,
                packet_data,
                packet_timeout_height,
                packet_timeout_timestamp
            );

        let source_channel = packet::source_channel_id(&packet);
        let destination_channel = packet::destination_channel_id(&packet);
        let channel = ensure_channel_state(source_channel);
        let client_id = ensure_connection_state(channel::connection_id(&channel));
        let client_type = client_id_to_type(client_id);

        let proof_timestamp =
            light_client::get_timestamp_at_height(client_type, client_id, proof_height);
        assert!(proof_timestamp != 0, E_LATEST_TIMESTAMP_NOT_FOUND);

        let commitment_key =
            commitment::batch_receipts_commitment_key(
                destination_channel, commitment::commit_packet(&packet)
            );
        let err =
            verify_absent_commitment(
                client_type,
                client_id,
                proof_height,
                proof,
                commitment_key
            );
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
        table::remove(
            &mut borrow_global_mut<IBCStore>(get_vault_addr()).commitments,
            commitment_key
        );

        event::emit(TimeoutPacket { packet });

        packet
    }

    // Initializes the IBCStore resource in the signer's account
    fun init_module(account: &signer) {
        assert!(
            signer::address_of(account) == @ibc, E_NOT_ENOUGH_PERMISSIONS_TO_INITIALIZE
        );
        let vault_constructor_ref = &object::create_named_object(account, VAULT_SEED);
        let vault_signer = &object::generate_signer(vault_constructor_ref);

        let store = IBCStore {
            commitments: table::new(),
            connections: smart_table::new(),
            channels: smart_table::new(),
            channel_to_module: smart_table::new(),
            client_id_to_type: smart_table::new()
        };

        move_to(vault_signer, store);

        move_to(
            vault_signer,
            SignerRef { self_ref: object::generate_extend_ref(vault_constructor_ref) }
        );
    }

    // ========= UTILS and VIEW functions ========= //

    #[view]
    public fun client_id_to_type(client_id: u32): String acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        *smart_table::borrow(&store.client_id_to_type, client_id)
    }

    #[view]
    public fun get_module(channel_id: u32): address acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        *smart_table::borrow(&store.channel_to_module, channel_id)
    }

    #[view]
    public fun client_state(client_id: u32): vector<u8> acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        light_client::get_client_state(
            *smart_table::borrow(&store.client_id_to_type, client_id), client_id
        )
    }

    #[view]
    public fun consensus_state(client_id: u32, revision_height: u64): vector<u8> acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        light_client::get_consensus_state(
            *smart_table::borrow(&store.client_id_to_type, client_id),
            client_id,
            revision_height
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

    public(friend) fun remove_commitment(key: vector<u8>) acquires IBCStore {
        table::remove(
            &mut borrow_global_mut<IBCStore>(get_vault_addr()).commitments,
            key
        );
    }

    // Setter for Commitments
    public(friend) fun set_commitment(key: vector<u8>, value: vector<u8>) acquires IBCStore {
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

    // Function to generate a client identifier
    public(friend) fun generate_client_identifier(): u32 acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());

        let next_sequence =
            table::borrow_with_default(
                &store.commitments, b"nextClientSequence", &bcs::to_bytes<u32>(&1)
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
        client_type: String,
        connection: &ConnectionEnd,
        height: u64,
        proof: vector<u8>,
        connection_id: u32,
        counterparty_connection: ConnectionEnd
    ): u64 {
        light_client::verify_membership(
            client_type,
            connection_end::client_id(connection),
            height,
            proof,
            commitment::connection_commitment_key(connection_id),
            aptos_hash::keccak256(connection_end::encode(&counterparty_connection))
        )
    }

    public(friend) fun verify_commitment(
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        path: vector<u8>,
        commitment: vector<u8>
    ): u64 acquires IBCStore {
        let client_type = client_id_to_type(client_id);
        light_client::verify_membership(
            client_type,
            client_id,
            height,
            proof,
            path,
            commitment
        )
    }

    fun generate_connection_identifier(): u32 acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let next_sequence_bytes =
            table::borrow_with_default(
                &store.commitments,
                b"nextConnectionSequence",
                &bcs::to_bytes<u32>(&1)
            );
        let next_sequence = from_bcs::to_u32(*next_sequence_bytes);
        table::upsert(
            &mut store.commitments,
            b"nextConnectionSequence",
            bcs::to_bytes<u32>(&(next_sequence + 1))
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

    fun generate_channel_identifier(): u32 acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let next_sequence_bytes =
            table::borrow_with_default(
                &store.commitments,
                b"nextChannelSequence",
                &bcs::to_bytes<u32>(&1)
            );
        let next_sequence = from_bcs::to_u32(*next_sequence_bytes);

        table::upsert(
            &mut store.commitments,
            b"nextChannelSequence",
            bcs::to_bytes(&(next_sequence + 1))
        );
        next_sequence
    }

    public(friend) fun ensure_connection_state(connection_id: u32): u32 acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        assert!(
            smart_table::contains(&store.connections, connection_id),
            E_CONNECTION_DOES_NOT_EXIST
        );
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

    fun commit_channel(channel_id: u32, channel: Channel) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let key = commitment::channel_commitment_key(channel_id);

        let encoded = encode_channel(channel);
        table::upsert(&mut store.commitments, key, encoded);
    }

    fun commit_connection(connection_id: u32, connection: ConnectionEnd) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(get_vault_addr());
        let key = commitment::connection_commitment_key(connection_id);

        let encoded = aptos_hash::keccak256(connection_end::encode(&connection));
        table::upsert(&mut store.commitments, key, encoded);
    }

    fun verify_channel_state(
        client_type: String,
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        channel_id: u32,
        channel: Channel
    ): u64 {
        light_client::verify_membership(
            client_type,
            client_id,
            height,
            proof,
            commitment::channel_commitment_key(channel_id),
            aptos_hash::keccak256(channel::encode(&channel))
        )
    }

    // Ensures that the channel state is open
    public(friend) fun ensure_channel_state(channel_id: u32): Channel acquires IBCStore {
        let store = borrow_global<IBCStore>(get_vault_addr());
        let channel = smart_table::borrow(&store.channels, channel_id);

        assert!(channel::state(channel) == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);
        *channel
    }

    fun verify_absent_commitment(
        client_type: String,
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        path: vector<u8>
    ): u64 {
        light_client::verify_non_membership(client_type, client_id, height, proof, path)
    }

    fun address_to_string(addr: address): String {
        string_utils::to_string(&bcs::to_bytes(&addr))
    }

    public(friend) fun emit_recv_packet(
        packet: Packet, maker: address, maker_msg: vector<u8>
    ) {
        event::emit(PacketRecv { packet, maker, maker_msg })
    }

    public(friend) fun emit_recv_intent_packet(
        packet: Packet, maker: address, maker_msg: vector<u8>
    ) {
        event::emit(RecvIntentPacket { packet, maker, maker_msg })
    }

    public(friend) fun emit_acknowledge_packet(
        packet: Packet, acknowledgement: vector<u8>, maker: address
    ) {
        event::emit(PacketAck { packet, acknowledgement, maker });
    }

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

    #[test_only]
    use std::string;

    #[test_only]
    const CLIENT_TYPE: vector<u8> = b"client";

    #[test_only]
    public(friend) fun init_module_for_tests(account: &signer) {
        init_module(account)
    }

    // Client identifier creation starts from 1 and increases one by one
    #[test(alice = @ibc)]
    fun test_generate_client_identifier(alice: &signer) acquires IBCStore {
        init_module_for_tests(alice);

        assert!(generate_client_identifier() == 1, 1);
        assert!(generate_client_identifier() == 2, 1);
    }

    #[test(alice = @ibc)]
    fun create_client_works(alice: &signer) acquires IBCStore, SignerRef {
        init_module_for_tests(alice);

        let counterparty_chain_id = string::utf8(b"union");
        let client_state = vector[1, 2, 3];
        let consensus_state = vector[1, 2, 3];
        let event = create_lens_client_event::new(1, string::utf8(b"hello"), 2, 3);

        create_client_impl(
            string::utf8(CLIENT_TYPE),
            client_state,
            consensus_state,
            |_1, _2, _3, _4, _5| (
                client_state, consensus_state, counterparty_chain_id, option::some(event)
            ),
            |_s, _s2| 0,
            |_s, _s2| 10
        );

        assert!(
            get_commitment(commitment::client_state_commitment_key(1)) == client_state,
            1
        );

        assert!(
            get_commitment(commitment::consensus_state_commitment_key(1, 10))
                == consensus_state,
            1
        );

        assert!(
            event::was_event_emitted(
                &CreateClient {
                    client_id: 1,
                    client_type: string::utf8(CLIENT_TYPE),
                    counterparty_chain_id
                }
            ),
            1
        );

        assert!(
            event::was_event_emitted(
                &CreateLensClient {
                    client_id: create_lens_client_event::client_id(&event),
                    l2_chain_id: create_lens_client_event::l2_chain_id(&event),
                    l1_client_id: create_lens_client_event::l1_client_id(&event),
                    l2_client_id: create_lens_client_event::l2_client_id(&event)
                }
            ),
            1
        );
    }

    #[test(alice = @ibc)]
    #[expected_failure(abort_code = E_CLIENT_NOT_ACTIVE)]
    fun create_client_fails_inactive_client(alice: &signer) acquires IBCStore, SignerRef {
        init_module_for_tests(alice);

        let counterparty_chain_id = string::utf8(b"union");
        let client_state = vector[1, 2, 3];
        let consensus_state = vector[1, 2, 3];
        let event = create_lens_client_event::new(1, string::utf8(b"hello"), 2, 3);

        create_client_impl(
            string::utf8(CLIENT_TYPE),
            client_state,
            consensus_state,
            |_1, _2, _3, _4, _5| (
                client_state, consensus_state, counterparty_chain_id, option::some(event)
            ),
            // returning nonzero status which means the client is not active
            |_s, _s2| 1,
            |_s, _s2| 10
        );
    }

    #[test_only]
    fun prepare_client() acquires IBCStore, SignerRef {
        create_client_impl(
            string::utf8(b"client"),
            vector[1, 2],
            vector[1, 2],
            |_1, _2, _3, _4, _5| (
                vector[1, 2], vector[1, 2], string::utf8(b"chain"), option::none()
            ),
            // returning nonzero status which means the client is not active
            |_s, _s2| 0,
            |_s, _s2| 10
        );

    }

    #[test(alice = @ibc)]
    fun connection_open_init_works(alice: &signer) acquires IBCStore, SignerRef {
        init_module_for_tests(alice);
        prepare_client();

        let client_id = 1;
        let counterparty_client_id = 2;

        connection_open_init(client_id, counterparty_client_id);

        let connection =
            connection_end::new(
                CONN_STATE_INIT,
                client_id,
                counterparty_client_id,
                0
            );

        assert!(get_connection(1) == option::some(connection), 1);

        assert!(
            get_commitment(commitment::connection_commitment_key(1))
                == aptos_hash::keccak256(connection_end::encode(&connection)),
            1
        );

        assert!(
            event::was_event_emitted(
                &ConnectionOpenInit { connection_id: 1, client_id, counterparty_client_id }
            ),
            1
        );
    }

    #[test(alice = @ibc)]
    fun connection_open_try_works(alice: &signer) acquires IBCStore, SignerRef {
        init_module_for_tests(alice);
        prepare_client();

        let client_id = 1;
        let counterparty_client_id = 2;
        let counterparty_connection_id = 3;
        let proof_init = vector[1, 2];
        let proof_height = 10;
        let counterparty_connection =
            connection_end::encode(
                &connection_end::new(
                    CONN_STATE_INIT,
                    counterparty_client_id,
                    client_id,
                    0
                )
            );

        connection_open_try_impl(
            counterparty_client_id,
            counterparty_connection_id,
            client_id,
            proof_init,
            proof_height,
            |client_type, client_id_, proof_height_, proof_init_, key, value| {
                assert!(client_type == string::utf8(CLIENT_TYPE), 1);
                assert!(client_id_ == client_id, 1);
                assert!(proof_height == proof_height_, 1);
                assert!(proof_init == proof_init_, 1);
                assert!(
                    key
                        == commitment::connection_commitment_key(
                            counterparty_connection_id
                        ),
                    1
                );
                assert!(
                    value == aptos_hash::keccak256(counterparty_connection),
                    1
                );

                0
            }
        );

        let connection =
            connection_end::new(
                CONN_STATE_TRYOPEN,
                client_id,
                counterparty_client_id,
                counterparty_connection_id
            );

        assert!(get_connection(1) == option::some(connection), 1);

        assert!(
            get_commitment(commitment::connection_commitment_key(1))
                == aptos_hash::keccak256(connection_end::encode(&connection)),
            1
        );

        assert!(
            event::was_event_emitted(
                &ConnectionOpenTry {
                    connection_id: 1,
                    client_id,
                    counterparty_client_id,
                    counterparty_connection_id
                }
            ),
            1
        );
    }

    #[test(alice = @ibc)]
    #[expected_failure(location = Self, abort_code = 1)]
    fun connection_open_try_fails_with_membership_failure(
        alice: &signer
    ) acquires IBCStore, SignerRef {
        init_module_for_tests(alice);
        prepare_client();

        connection_open_try_impl(0, 0, 1, vector[1, 2], 1, |_0, _1, _2, _3, _4, _5| { 1 });
    }
}
