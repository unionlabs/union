module ibc::dispatcher {
    use std::option;
    use std::string;

    use aptos_std::table::{Self, Table};
    use aptos_std::type_info::{Self, TypeInfo};
    use aptos_framework::primary_fungible_store;

    use aptos_framework::dispatchable_fungible_asset;
    use aptos_framework::function_info::FunctionInfo;
    use aptos_framework::fungible_asset::{Self, Metadata};
    use aptos_framework::object::{Self, ExtendRef, Object};
    use std::signer;

    friend ibc::engine;
    const DISPATCHER_APP_SEED: vector<u8> = b"union-ibc-dispatcher-v1";

    struct Dispatcher has key {
        /// Tracks the input type to the dispatch handler.
        dispatcher: Table<TypeInfo, Object<Metadata>>,
        /// Used to store temporary data for dispatching.
        obj_ref: ExtendRef
    }

    /// Store the data to dispatch here.
    struct Storage<phantom P, T> has drop, key {
        data: T
    }

    /// Register a `T` to callback. Providing an instance of `T` guarantees that only the
    /// originating module can call `register` for that type.
    public fun register<T: drop>(callback: FunctionInfo, _proof: T) acquires Dispatcher {
        let constructor_ref =
            object::create_named_object(&storage_signer(), DISPATCHER_APP_SEED);

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
        table::add(&mut dispatcher.dispatcher, type_info::type_of<T>(), metadata);
    }

    /// Insert into this module as the callback needs to retrieve and avoid a cyclical dependency:
    /// engine -> storage and then engine -> callback -> storage
    public(friend) fun insert<P: store, T: store>(data: T): Object<Metadata> acquires Dispatcher {
        move_to(&storage_signer(), Storage<P, T> { data });

        let typeinfo = type_info::type_of<P>();
        let dispatcher = borrow_global<Dispatcher>(get_vault_addr());
        let type_info = *table::borrow(&dispatcher.dispatcher, typeinfo);

        type_info
    }

    public fun retrieve<P: drop, T: copy + drop + store>(_proof: P): T acquires Dispatcher, Storage {
        move_from<Storage<P, T>>(storage_address()).data
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
                dispatcher: table::new(),
                obj_ref: object::generate_extend_ref(constructor_ref)
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
        object::create_object_address(&@ibc, DISPATCHER_APP_SEED)
    }

    public fun storage_signer(): signer acquires Dispatcher {
        let vault = borrow_global<Dispatcher>(get_vault_addr());
        object::generate_signer_for_extending(&vault.obj_ref)
    }

    public fun init_module_for_testing(publisher: &signer) {
        init_module(publisher);
    }
}
