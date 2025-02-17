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

module zkgm::dispatcher_zkgm {
    use std::option;
    use std::string;
    use aptos_std::copyable_any;
    use aptos_std::type_info::{Self};
    use aptos_std::smart_table::{Self, SmartTable};

    use std::event;
    use aptos_framework::dispatchable_fungible_asset;
    use aptos_framework::function_info::FunctionInfo;
    use aptos_framework::fungible_asset::{Self, Metadata};
    use aptos_framework::object::{Self, ExtendRef, Object};
    const DISPATCHER_APP_SEED: vector<u8> = b"union-zkgm-dispatcher-v1";

    struct Storage has drop, key, store {
        data: copyable_any::Any,
        return_value: vector<u8>
    }

    #[event]
    struct LOG has copy, drop, store {
        incoming_addr: address
    }

    struct Dispatcher has key {
        /// Tracks the input type to the dispatch handler.
        dispatcher: SmartTable<address, Object<Metadata>>,
        /// Used to store temporary data for dispatching.
        obj_ref: ExtendRef,
        storage: SmartTable<address, Storage>
    }

    /// Register a `T` to callback. Providing an instance of `T` guarantees that only the
    /// originating module can call `register` for that type.
    public fun register<T: drop>(
        callback: FunctionInfo, _proof: T, some_seed: vector<u8>
    ) acquires Dispatcher {
        let constructor_ref = object::create_named_object(&storage_signer(), some_seed);

        let metadata =
            fungible_asset::add_fungibility(
                &constructor_ref,
                option::none(),
                string::utf8(b"dis"),
                string::utf8(b"dis"),
                0,
                string::utf8(b""),
                string::utf8(b"")
            );

        fungible_asset::create_store(&constructor_ref, metadata); //its required to create store for fungible asset

        dispatchable_fungible_asset::register_dispatch_functions(
            &constructor_ref,
            option::none(),
            option::none(),
            option::some(callback)
        );

        let dispatcher = borrow_global_mut<Dispatcher>(get_vault_addr());

        let type_info_addr = type_info::account_address(&type_info::type_of<T>());

        event::emit(LOG { incoming_addr: type_info_addr });
        smart_table::upsert(&mut dispatcher.dispatcher, type_info_addr, metadata);
    }

    /// Insert into this module as the callback needs to retrieve and avoid a cyclical dependency:
    /// engine -> storage and then engine -> callback -> storage
    public fun insert(
        data: copyable_any::Any, return_value: vector<u8>, type_info_addr: address
    ): Object<Metadata> acquires Dispatcher {
        // move_to(
        //     &storage_signer(),
        //     Storage<P> { data, return_value }
        // );
        let dispatcher = borrow_global_mut<Dispatcher>(get_vault_addr());

        let storage_val = Storage { data, return_value };
        smart_table::upsert(&mut dispatcher.storage, type_info_addr, storage_val);
        let type_info = *smart_table::borrow(&dispatcher.dispatcher, type_info_addr);

        type_info
    }

    // public fun delete_storage<P: store>() acquires Dispatcher, Storage {
    //     move_from<Storage<P>>(storage_address());
    // }

    // Getter for `data`
    public fun get_data<P: drop>(_proof: P): copyable_any::Any acquires Dispatcher {
        let dispatcher = borrow_global<Dispatcher>(get_vault_addr());
        let storage_val =
            smart_table::borrow(
                &dispatcher.storage,
                type_info::account_address(&type_info::type_of<P>())
            );
        storage_val.data
    }

    // Getter for `return_value`
    public fun get_return_value(type_info_addr: address): vector<u8> acquires Dispatcher {
        let dispatcher = borrow_global<Dispatcher>(get_vault_addr());
        let storage_val = smart_table::borrow(&dispatcher.storage, type_info_addr);
        storage_val.return_value
    }

    // Setter for `return_value`
    public fun set_return_value<P: drop>(_proof: P, new_value: vector<u8>) acquires Dispatcher {
        let type_info_addr = type_info::account_address(&type_info::type_of<P>());
        let dispatcher = borrow_global_mut<Dispatcher>(get_vault_addr());
        smart_table::borrow_mut(&mut dispatcher.storage, type_info_addr).return_value = new_value;

        // borrow_global_mut<Storage<P>>(storage_address()).return_value = new_value;
    }

    /// Prepares the dispatch table.
    fun init_module(publisher: &signer) {
        let constructor_ref = &object::create_named_object(
            publisher, DISPATCHER_APP_SEED
        );
        let vault_signer = &object::generate_signer(constructor_ref);

        move_to(
            vault_signer,
            Dispatcher {
                dispatcher: smart_table::new(),
                obj_ref: object::generate_extend_ref(constructor_ref),
                storage: smart_table::new()
            }
        );
    }

    inline fun storage_address(): address acquires Dispatcher {
        // object::address_from_extend_ref(&borrow_global<Dispatcher>(get_vault_addr()).obj_ref)
        object::address_from_extend_ref(
            &borrow_global<Dispatcher>(get_vault_addr()).obj_ref
        )
    }

    #[view]
    public fun get_vault_addr(): address {
        object::create_object_address(&@zkgm, DISPATCHER_APP_SEED)
    }

    public fun storage_signer(): signer acquires Dispatcher {
        let vault = borrow_global<Dispatcher>(get_vault_addr());
        object::generate_signer_for_extending(&vault.obj_ref)
    }

    public fun init_module_for_testing(publisher: &signer) {
        init_module(publisher);
    }
}
