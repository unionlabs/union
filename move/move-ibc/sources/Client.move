module ContractClientAddress::Client {

    use std::signer;
    use std::vector;
    use std::error;
    use std::table;
    use std::debug;
    // use std::string::String;
    use aptos_std::smart_table::{Self as SmartTable, SmartTable};
    use aptos_std::string::{Self as StringModule, String};
    use aptos_std::string_utils;

    const E_CLIENT_ALREADY_EXISTS: u64 = 1001;
    const E_CLIENT_IMPL_NOT_FOUND: u64 = 1002;

    // Resource to hold the global state
    struct IBCStore has key {
        // client_types: SmartTable<String, vector<u8>>,
        client_impls: SmartTable<String, address>,
        client_registry: SmartTable<String, address>,
        commitments: SmartTable<vector<u8>, u256>,
    }

    // Struct representing the message to create a client
    struct MsgCreateClient has drop {
        client_type: String,
        client_state_bytes: String,
        consensus_state_bytes: String,
        relayer: address,
    }

    // Initializes the IBCStore resource in the signer's account
    public fun create_ibc_store(account: &signer) {
        let store = IBCStore {
            client_registry: SmartTable::new(),
            commitments: SmartTable::new(),
            // client_types: SmartTable::new(),
            client_impls: SmartTable::new(),
        };
        move_to(account, store);
    }

    // Function to register a client type
    public fun register_client(account: &signer, client_type: String, client_address: address) acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(signer::address_of(account));

        let curr_registry = SmartTable::borrow_with_default(&store.client_registry, client_type, &@0x0);
        if (*curr_registry != @0x0) {
            abort E_CLIENT_ALREADY_EXISTS;
        };

        // Register the new client type
        SmartTable::add(&mut store.client_registry, client_type, client_address);
    }

    public fun get_client(account: &signer, client_type: String): address acquires IBCStore {
        let store = borrow_global<IBCStore>(signer::address_of(account));

        let client_address = SmartTable::borrow_with_default(&store.client_registry, client_type, &@0x0);
        if (*client_address == @0x0) {
            abort E_CLIENT_IMPL_NOT_FOUND;
        };

        *client_address
    }

    // Function to generate a client identifier
    public fun generate_client_identifier(account: &signer, client_type: String): String acquires IBCStore {
        let store = borrow_global_mut<IBCStore>(signer::address_of(account));

        let next_sequence = SmartTable::borrow_with_default(&store.commitments, b"nextClientSequence", &0);
        
         // Convert the next sequence to a string
        let next_sequence_str = string_utils::to_string(next_sequence);

        // Format the identifier as "client_type-next_sequence"
        let format_str = b"{}-{}";
        let identifier = string_utils::format2(&format_str, client_type, next_sequence_str);
        // let identifier = string_utils::to_string(&identifier);

        SmartTable::upsert(&mut store.commitments, b"nextClientSequence", *next_sequence + 1);

        identifier
    }

    // Function to create a client based on the provided message
    public fun create_client(account: &signer, msg: MsgCreateClient) acquires IBCStore {
        let client_impl = get_client(account, msg.client_type);
        let clientId = generate_client_identifier(account, msg.client_type);
        
        let store = borrow_global_mut<IBCStore>(signer::address_of(account));
        // SmartTable::upsert(&mut store.client_types, clientId, msg.client_type);
        SmartTable::upsert(&mut store.client_impls, clientId, client_impl);


        // TODO: Call createClient functiom from lightclient.
        // Continue with further steps...
    }

    
}
