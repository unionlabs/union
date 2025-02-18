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

module ibc::move_in_cosmos_client {
    use std::signer;
    use aptos_std::vector;
    use aptos_std::aptos_hash::keccak256;
    use std::event;
    use std::object;
    use aptos_std::copyable_any;
    use std::bcs;
    use std::string::{String};
    use std::from_bcs;
    use aptos_std::smart_table::{Self, SmartTable};
    use ibc::mpt_verifier;
    use ibc::cometbls_lc;
    use ibc::commitment;
    const EVM_IBC_COMMITMENT_SLOT: u8 = 0;

    // Errors
    const ERR_NOT_IBC: u64 = 1001;
    const ERR_TRUSTED_CONSENSUS_STATE_NOT_FOUND: u64 = 1002;
    const ERR_CLIENT_FROZEN: u64 = 1003;
    const ERR_INVALID_L1_PROOF: u64 = 1004;
    const ERR_INVALID_INITIAL_CONSENSUS_STATE: u64 = 1005;
    const ERR_UNSUPPORTED: u64 = 1006;

    const VAULT_SEED: vector<u8> = b"IBC_STATE_LENS_SEED";

    /// Client State
    struct ClientState has copy, drop, store {
        l2_chain_id: String,
        l1_client_id: u32,
        l2_client_id: u32,
        l2_latest_height: u64,
        timestamp_offset: u16,
        state_root_offset: u16,
        storage_root_offset: u16
    }

    /// Consensus State
    struct ConsensusState has copy, drop, store {
        timestamp: u64,
        state_root: vector<u8>,
        storage_root: vector<u8>
    }

    /// Header
    struct Header has copy, drop, store {
        l1_height: u64,
        l2_height: u64,
        l2_inclusion_proof: vector<u8>,
        l2_consensus_state: vector<u8>
    }

    /// ConsensusStateUpdate
    struct ConsensusStateUpdate has copy, drop, store {
        client_state_commitment: vector<u8>,
        consensus_state_commitment: vector<u8>,
        height: u64
    }

    struct ConsensusStatesTuple has copy, drop, store {
        client_id: u32,
        l2_height: u64
    }

    // Events
    #[event]
    struct CreateLensClient has copy, drop, store {
        client_id: u32,
        l1_client_id: u32,
        l2_client_id: u32,
        l2_chain_id: String
    }

    /// Storage for the module
    struct EvmInCosmosStorage has key {
        client_states: SmartTable<u32, ClientState>,
        consensus_states: SmartTable<ConsensusStatesTuple, ConsensusState>
    }

    struct SignerRef has key {
        self_ref: object::ExtendRef,
        self_address: address
    }

    #[view]
    public fun get_vault_addr(): address {
        object::create_object_address(&@ibc, VAULT_SEED)
    }

    public fun get_signer(): signer acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        object::generate_signer_for_extending(&vault.self_ref)
    }

    public fun get_self_address(): address acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        vault.self_address
    }

    /// Initialize the module storage
    fun init_module(account: &signer) {
        assert!(signer::address_of(account) == @ibc, ERR_NOT_IBC);
        let storage = EvmInCosmosStorage {
            client_states: smart_table::new(),
            consensus_states: smart_table::new()
        };

        let vault_constructor_ref = &object::create_named_object(account, VAULT_SEED);
        let vault_signer = &object::generate_signer(vault_constructor_ref);

        move_to(
            vault_signer,
            SignerRef {
                self_ref: object::generate_extend_ref(vault_constructor_ref),
                self_address: signer::address_of(account)
            }
        );

        move_to(vault_signer, storage);
    }

    /// Create a new client
    public fun create_client(
        account: &signer,
        client_id: u32,
        client_state_bcs: copyable_any::Any, //vector<u8>,
        consensus_state_bcs: copyable_any::Any //vector<u8>
    ): ConsensusStateUpdate acquires EvmInCosmosStorage {
        assert!(signer::address_of(account) == @ibc, ERR_NOT_IBC);
        let store = borrow_global_mut<EvmInCosmosStorage>(get_vault_addr());

        let client_state = copyable_any::unpack<ClientState>(client_state_bcs);
        let consensus_state = copyable_any::unpack<ConsensusState>(consensus_state_bcs);

        assert!(client_state.l2_latest_height > 0, ERR_INVALID_INITIAL_CONSENSUS_STATE);
        assert!(consensus_state.timestamp > 0, ERR_INVALID_INITIAL_CONSENSUS_STATE);

        let cons_state_tuple = ConsensusStatesTuple {
            client_id: client_id,
            l2_height: client_state.l2_latest_height
        };
        smart_table::upsert(
            &mut store.consensus_states,
            cons_state_tuple,
            consensus_state
        );

        let client_state_commitment = keccak256(bcs::to_bytes(&client_state));

        smart_table::upsert(&mut store.client_states, client_id, client_state);

        let consensus_state_commitment = keccak256(bcs::to_bytes(&consensus_state));

        event::emit(
            CreateLensClient {
                client_id: client_id,
                l1_client_id: client_state.l1_client_id,
                l2_client_id: client_state.l2_client_id,
                l2_chain_id: client_state.l2_chain_id
            }
        );
        ConsensusStateUpdate {
            client_state_commitment,
            consensus_state_commitment,
            height: client_state.l2_latest_height
        }
    }

    public fun misbehaviour(
        _client_id: u32, _client_msg_bytes: vector<u8>
    ) {
        abort ERR_UNSUPPORTED
    }

    public fun is_frozen(client_id: u32): bool {
        cometbls_lc::is_frozen(client_id)
    }

    /// Verify membership in the trie
    public fun verify_membership(
        client_id: u32,
        height: u64,
        proof: &vector<u8>,
        path: &vector<u8>,
        value: &vector<u8>
    ): bool acquires EvmInCosmosStorage {
        if (is_frozen(client_id)) {
            abort ERR_CLIENT_FROZEN
        };
        let store = borrow_global<EvmInCosmosStorage>(get_vault_addr());
        let appended_path = vector::empty<u8>();
        vector::append(&mut appended_path, *path);
        vector::append(&mut appended_path, bcs::to_bytes(&0));

        let slot = keccak256(appended_path);
        let cons_state_tuple = ConsensusStatesTuple {
            client_id: client_id,
            l2_height: height
        };

        let cons_state: ConsensusState =
            *smart_table::borrow(&store.consensus_states, cons_state_tuple);
        let storage_root = cons_state.storage_root;

        let (is_exist, proven_value) =
            mpt_verifier::verify_trie_value(proof, &keccak256(slot), storage_root);

        is_exist
            && (
                mpt_verifier::encode_uint(mpt_verifier::load_u256_big_endian(value))
                    == keccak256(proven_value)
            )
    }

    /// Verify non-membership in the trie
    public fun verify_non_membership(
        client_id: u32,
        height: u64,
        proof: &vector<u8>,
        path: &vector<u8>
    ): bool acquires EvmInCosmosStorage {
        if (is_frozen(client_id)) {
            abort ERR_CLIENT_FROZEN
        };
        let store = borrow_global<EvmInCosmosStorage>(get_vault_addr());
        let appended_path = vector::empty<u8>();
        vector::append(&mut appended_path, *path);
        vector::append(&mut appended_path, bcs::to_bytes(&0));

        let slot = keccak256(appended_path);
        let cons_state_tuple = ConsensusStatesTuple {
            client_id: client_id,
            l2_height: height
        };

        let cons_state: ConsensusState =
            *smart_table::borrow(&store.consensus_states, cons_state_tuple);
        let storage_root = cons_state.storage_root;

        let (is_exist, _proven_value) =
            mpt_verifier::verify_trie_value(proof, &keccak256(slot), storage_root);

        !is_exist
    }

    /// Update the client with a new header
    public fun update_client(
        _account: &signer, // TODO: not sure if this is needed
        client_id: u32,
        header_bcs: copyable_any::Any
    ): ConsensusStateUpdate acquires EvmInCosmosStorage {
        let store = borrow_global_mut<EvmInCosmosStorage>(get_vault_addr());

        let header = copyable_any::unpack<Header>(header_bcs);

        let client_state = smart_table::borrow_mut(&mut store.client_states, client_id);

        let proof =
            commitment::consensus_state_commitment_key(
                client_state.l2_client_id, header.l2_height
            );

        if (cometbls_lc::verify_membership(
            client_state.l1_client_id,
            header.l1_height,
            header.l2_inclusion_proof,
            proof,
            keccak256(header.l2_consensus_state)
        ) != 0) { // TODO: its returning u64 not bool
            abort ERR_INVALID_L1_PROOF
        };

        let raw_l2_consensus_state = &header.l2_consensus_state;
        // TODO: Not sure about the below
        let l2_timestamp =
            from_bcs::to_u64(
                vector::slice(
                    raw_l2_consensus_state,
                    (client_state.timestamp_offset as u64),
                    (client_state.timestamp_offset as u64) + 8
                )
            );
        let l2_state_root = vector::slice(
            raw_l2_consensus_state,
            (client_state.state_root_offset as u64),
            (client_state.state_root_offset as u64) + 32
        );
        let l2_storage_root = vector::slice(
            raw_l2_consensus_state,
            (client_state.storage_root_offset as u64),
            (client_state.storage_root_offset as u64) + 32
        );

        if (header.l2_height > client_state.l2_latest_height) {
            client_state.l2_latest_height = header.l2_height;
        };

        let consensus_state = ConsensusState {
            timestamp: l2_timestamp,
            state_root: l2_state_root,
            storage_root: l2_storage_root
        };
        let cons_state_tuple = ConsensusStatesTuple {
            client_id: client_id,
            l2_height: header.l2_height
        };
        smart_table::upsert(
            &mut store.consensus_states,
            cons_state_tuple,
            consensus_state
        );

        let client_state_commitment = keccak256(bcs::to_bytes(client_state));
        let consensus_state_commitment =
            keccak256(bcs::to_bytes<ConsensusState>(&consensus_state));

        ConsensusStateUpdate {
            client_state_commitment,
            consensus_state_commitment,
            height: header.l2_height
        }
    }
}
