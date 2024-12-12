module some_addr::main_app {
    use std::option;
    use std::string::{Self};
    use aptos_std::copyable_any;
    use aptos_std::table::{Self, Table};
    use aptos_std::type_info::{Self, TypeInfo};
    use aptos_framework::primary_fungible_store;

    use std::event;
    use aptos_framework::dispatchable_fungible_asset;
    use aptos_framework::function_info::FunctionInfo;
    use aptos_framework::fungible_asset::{Self, Metadata};
    use aptos_framework::object::{Self, ExtendRef, Object};
    use std::signer;


    use some_addr::engine;

    #[event]
    struct TEST_EVENT has drop, store {
        u1: u32,
        s1: string::String
    }

    public fun get_u1(
        param: &RandomParam
    ): u32 {
        param.u1
    }

    public fun get_u2(
        param: &RandomParam
    ): u32 {
        param.u2
    }

    struct RandomParam has copy, drop, store {
        u1: u32,
        u2: u32,
    }

    public fun create_param(u1: u32, u2: u32): RandomParam {
        RandomParam {
            u1: u1,
            u2: u2
        }
    }

    public entry fun call_me<T: store>(param1: u32, param2: u32) {
        let param =
            copyable_any::pack<RandomParam>(
                RandomParam {
                    u1: param1,
                    u2: param2
                }
            );
        engine::dispatch<T>(param);
    }

    public entry fun call_me2<T: store>() {
        let val = std::type_info::type_name<T>();
        event::emit(
            TEST_EVENT { u1: 4, s1: val }
        );
    }


    public entry fun call_me4<T>() {
        let val = std::type_info::type_name<T>();
        event::emit(
            TEST_EVENT { u1: 4, s1: val }
        );
    }

    public entry fun call_me3() {
        event::emit(
            TEST_EVENT { u1: 4, s1: string::utf8(b"Hello") }
        );
    }

}