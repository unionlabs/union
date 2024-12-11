module some_addr::dynamic_dispatch_app {
    use aptos_framework::object::{Self, Object};
    use std::option;
    use some_addr::dispatcher;
    use std::signer;
    use aptos_framework::function_info;
    use aptos_framework::function_info::FunctionInfo;
    use std::bcs;

    use std::string;
    use some_addr::engine;
    struct TestProof has drop, store {}


    public(friend) fun new_proof(): TestProof {
        TestProof {}
    }

    struct SignerRef has key {
        self_ref: object::ExtendRef,
        self_address: address
    }

    public fun verify_value(): u64 {
        4444444444321 // Just for testing
    }

    public fun register_application<T: key + store + drop>(
        ibc_app: &signer, cb: FunctionInfo, type: T
    ) {
        dispatcher::register<T>(cb, type, bcs::to_bytes(&signer::address_of(ibc_app)));

    }

}