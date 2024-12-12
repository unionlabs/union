module only_caller::only_caller {
    use std::option;
    use std::string::{Self};
    use aptos_std::copyable_any;
    use aptos_std::table::{Self, Table};
    use aptos_std::type_info::{Self, TypeInfo};
    use aptos_framework::primary_fungible_store;
    use some_addr::main_app;
    use std::event;
    use some_addr::engine;
    use aptos_framework::dispatchable_fungible_asset;
    use aptos_framework::function_info::FunctionInfo;
    use aptos_framework::fungible_asset::{Self, Metadata};
    use aptos_framework::object::{Self, ExtendRef, Object};
    use std::signer;
    use some_addr::dispatcher;

    public entry fun call_me<T: store>(param1: u32, param2: u32) {
        let param =
            copyable_any::pack<main_app::RandomParam>(
                main_app::create_param(param1, param2)
            );
        engine::dispatch<T>(param);
        dispatcher::delete_storage<T>();
    }
}