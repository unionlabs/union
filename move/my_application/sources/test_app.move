module my_app_addr::test_app {
    use aptos_framework::object::{Self, Object};
    use std::option;
    use some_addr::dispatcher;
    use std::signer;
    // use some_addr::RandomParam;
    use aptos_framework::function_info;
    use aptos_framework::function_info::FunctionInfo;
    use std::bcs;
    use std::event;
    use some_addr::main_app;

    use aptos_std::copyable_any;
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

    #[event]
    struct TEST_EVENT has drop, store {
        u1: u32,
        s1: string::String,
        s2: string::String
    }



    public fun on_recv_packet<T: key>(
        _store: Object<T>
    ): u64 {
        let value: copyable_any::Any = dispatcher::get_data(new_proof());
        let type_name_output = *copyable_any::type_name(&value);


        let value: main_app::RandomParam = copyable_any::unpack<main_app::RandomParam>(value);

        let u1 = main_app::get_u1(&value);

        if(u1 == 1) {
            abort 4
        };

         event::emit(
            TEST_EVENT { u1: u1, s1: type_name_output , s2: string::utf8(b"calling from onrecvpacket dispatcher worked?") }
        );



        // let value: u64 = dispatcher::retrieve(new_proof());
        // std::debug::print(&string::utf8(b"incoming value to on_recv_packet is:"));
        // std::debug::print(&value);
        // assert!(value == verify_value(), 0);
        321 // Just for testing
    }




}