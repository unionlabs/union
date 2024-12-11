module some_addr::main_app {
    use std::option;
    use std::string;
    use aptos_std::copyable_any;
    use aptos_std::table::{Self, Table};
    use aptos_std::type_info::{Self, TypeInfo};
    use aptos_framework::primary_fungible_store;

    use aptos_framework::dispatchable_fungible_asset;
    use aptos_framework::function_info::FunctionInfo;
    use aptos_framework::fungible_asset::{Self, Metadata};
    use aptos_framework::object::{Self, ExtendRef, Object};
    use std::signer;


    use some_addr::engine;


    struct RandomParam has copy, drop, store {
        u1: u32,
        u2: u32,
    }

    public entry fun call_me<T: key+ store+ drop>() {
        let param =
            copyable_any::pack<RandomParam>(
                RandomParam {
                    u1: 3,
                    u2: 5
                }
            );
        engine::dispatch<T>(param);
    }

}