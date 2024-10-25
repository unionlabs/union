module ibc::dynamic_dispatch_app {
    use aptos_framework::object::{Self, Object};
    use std::option;
    use ibc::dispatcher;
    use std::string;
    struct TestProof has drop, store {}

    friend ibc::sample_ibc;

    public(friend) fun new_proof(): TestProof {
        TestProof {}
    }

    struct SignerRef has key {
        self_ref: object::ExtendRef,
        self_address: address
    }

    public fun on_recv_packet<T: key>(
        _store: Object<T>
    ): u64 {
        let value: u64 = dispatcher::retrieve(new_proof());
        std::debug::print(&string::utf8(b"incoming value to on_recv_packet is:"));
        std::debug::print(&value);
        321 // Just for testing
    }

}