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
    use 0x1::IBCCommitment;
    use IBC::LightClient;
    use IBC::height;
    

    const SEED: vector<u8> = b"Move Seed Example";
    const VAULT_SEED: vector<u8> = b"Vault Seed Example";
    const E_CLIENT_ALREADY_EXISTS: u64 = 1001;
    const E_CLIENT_IMPL_NOT_FOUND: u64 = 1002;
    const E_LIGHT_CLIENT_CALL_FAILED: u64 = 1003;
    const ERR_SWAP_NOT_INITIALIZED: u64 = 1004;
    const ERR_NOT_ENOUGH_PERMISSIONS_TO_INITIALIZE: u64 = 1005;
    
    struct ClientCreatedEvent has copy, drop, store {
        client_id: String,
    }

    // Resource to hold the global state
    struct IBCStore has key {
        client_created_events: event::EventHandle<ClientCreatedEvent>,
        client_impls: SmartTable<String, address>,
        client_registry: SmartTable<String, address>,
        commitments: SmartTable<vector<u8>, vector<u8>>
    }

    struct SignerRef has key {
        self_ref: object::ExtendRef,
    }



    // // Struct representing the message to create a client
    // struct MsgCreateClient has drop {
    //     client_type: String,
    //     client_state: Any,
    //     consensus_state: Any,
    //     relayer: address,
    // }

    // Public factory function to create MsgCreateClient
    // public fun new_msg_create_client(
    //     client_type: String, 
    //     client_state: Any, 
    //     consensus_state: Any, 
    //     relayer: address
    // ): (
    //     client_type: String,
    //     client_state: Any,
    //     consensus_state: Any,
    //     relayer: address,) {
    //     MsgCreateClient {
    //         client_type,
    //         client_state,
    //         consensus_state,
    //         relayer,
    //     }
    // }

    #[view]
    public fun get_vault_addr(): address {
        object::create_object_address(&@IBC, VAULT_SEED)
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
            client_created_events: account::new_event_handle(&resource_signer),
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
            get_ibc_signer(),
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

    //connection handshake
    //channel handshake
}
