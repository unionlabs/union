module my_app_addr::test_app {
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

    fun init_module(publisher: &signer) {
        dispatcher::init_module_for_testing(publisher);
        let cb = function_info::new_function_info(
            publisher,
            string::utf8(b"test_app"),
            string::utf8(b"on_recv_packet"),
        );
        dispatcher::register(cb, new_proof(), bcs::to_bytes(&signer::address_of(publisher)));
    }


    public fun on_recv_packet<T: key>(
        _store: Object<T>
    ): u64 {

        // let value: u64 = dispatcher::retrieve(new_proof());
        // std::debug::print(&string::utf8(b"incoming value to on_recv_packet is:"));
        // std::debug::print(&value);
        // assert!(value == verify_value(), 0);
        321 // Just for testing
    }



}