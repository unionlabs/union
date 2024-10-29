module ibc::dynamic_dispatch_app {
    use aptos_framework::object::{Self, Object};
    use std::option;
    use ibc::dispatcher;
    use std::string;
    use ibc::packet::{Self, Packet};

    struct TestProof has drop, store {}
    
    const ON_RECV_PACKET: u8 = 0;
    const ON_ACKNOWLEDGE_PACKET: u8 = 1;
    const ON_TIMEOUT_PACKET: u8 = 2;
    const ON_CHANNEL_OPEN_INIT: u8 = 3;
    const ON_CHANNEL_OPEN_TRY: u8 = 4;
    const ON_CHANNEL_OPEN_ACK: u8 = 5;
    const ON_CHANNEL_OPEN_CONFIRM: u8 = 6;
    const ON_CHANNEL_CLOSE_INIT: u8 = 7;
    const ON_CHANNEL_CLOSE_CONFIRM: u8 = 8;

    struct DynamicDispatchParam has copy, store, drop, key {
        function_type: u8, // Identifier for the function type
        packet: option::Option<Packet>, // Consolidates packet parameters
        acknowledgement: option::Option<vector<u8>>,
        proof: option::Option<vector<u8>>,
        proof_height: option::Option<u64>,
        connection_id: option::Option<u32>,
        ordering: option::Option<u8>,
        version: option::Option<vector<u8>>,
        channel_state: option::Option<u8>,
        channel_order: option::Option<u8>,
        counterparty_channel_id: option::Option<u32>,
        counterparty_version: option::Option<vector<u8>>,
        proof_init: option::Option<vector<u8>>,
        channel_id: option::Option<u32>,
        proof_ack: option::Option<vector<u8>>,
        port_id: option::Option<string::String>,
        channel_id_str: option::Option<string::String>
    }

    public(friend) fun new_dynamic_dispatch_param(
        function_type: u8,
        packet: option::Option<Packet>,
        acknowledgement: option::Option<vector<u8>>,
        proof: option::Option<vector<u8>>,
        proof_height: option::Option<u64>,
        connection_id: option::Option<u32>,
        ordering: option::Option<u8>,
        version: option::Option<vector<u8>>,
        channel_state: option::Option<u8>,
        channel_order: option::Option<u8>,
        counterparty_channel_id: option::Option<u32>,
        counterparty_version: option::Option<vector<u8>>,
        proof_init: option::Option<vector<u8>>,
        channel_id: option::Option<u32>,
        proof_ack: option::Option<vector<u8>>,
        port_id: option::Option<string::String>,
        channel_id_str: option::Option<string::String>
    ): DynamicDispatchParam {
        DynamicDispatchParam {
            function_type,
            packet,
            acknowledgement,
            proof,
            proof_height,
            connection_id,
            ordering,
            version,
            channel_state,
            channel_order,
            counterparty_channel_id,
            counterparty_version,
            proof_init,
            channel_id,
            proof_ack,
            port_id,
            channel_id_str
        }
    }

    // Functions with the "on_" prefix for each specific operation
    public fun on_recv_packet(packet: Packet) {
        std::debug::print(&string::utf8(b"on_recv_packet called."));
        // Implementation for handling on_recv_packet
    }

    public fun on_acknowledge_packet(
        packet: Packet, acknowledgement: vector<u8>
    ) {
        std::debug::print(&string::utf8(b"on_acknowledge_packet called."));
        // Implementation for handling acknowledge_packet
    }

    public fun on_timeout_packet(_packet: DynamicDispatchParam) {
        // Implementation for handling timeout_packet
    }

    public fun on_channel_open_init(
        connection_id: u32, ordering: u8, version: vector<u8>
    ) {
        // Implementation for handling channel_open_init
    }

    public fun on_channel_open_try(
        connection_id: u32, proof_init: vector<u8>, proof_height: u64
    ) {
        // Implementation for handling channel_open_try
    }

    public fun on_channel_open_ack(
        channel_id: u32, counterparty_version: vector<u8>
    ) {
        // Implementation for handling channel_open_ack
    }

    public fun on_channel_open_confirm(
        channel_id: u32, proof_ack: vector<u8>
    ) {
        // Implementation for handling channel_open_confirm
    }

    public fun on_channel_close_init(
        port_id: string::String, channel_id_str: string::String
    ) {
        // Implementation for handling channel_close_init
    }

    public fun on_channel_close_confirm(
        port_id: string::String, channel_id_str: string::String
    ) {
        // Implementation for handling channel_close_confirm
    }

    friend ibc::sample_ibc;

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

    public fun on_packet<T: key>(_store: Object<T>): u64 {
        let value: DynamicDispatchParam = dispatcher::retrieve(new_proof());
        if (value.function_type == ON_RECV_PACKET) {
            let pack = option::extract(&mut value.packet);
            on_recv_packet(pack);
        } else if(value.function_type == ON_ACKNOWLEDGE_PACKET) {
            std::debug::print(&string::utf8(b"Its on on_acknowledge_packet"));
            let pack = option::extract(&mut value.packet);
            let acknowledgement = option::extract(&mut value.acknowledgement);
            std::debug::print(&string::utf8(b"packet is:"));
            std::debug::print(&pack);
            std::debug::print(&string::utf8(b"acknowledgement is:"));
            std::debug::print(&acknowledgement);
            on_acknowledge_packet(pack, acknowledgement);
        };
        0
    }
}
