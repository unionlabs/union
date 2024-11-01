module ibc::ping_pong_app {
    use aptos_framework::object::{Self, Object};
    use std::option;
    use ibc::dispatcher;
    use std::event;
    use std::timestamp;
    use std::signer;
    use std::string::{Self, String, utf8};
    use ibc::ibc;
    use std::vector;
    use std::bcs;
    use aptos_framework::function_info;
    use std::from_bcs;
    use ibc::height;
    use ibc::channel;
    use ibc::packet::{Self, Packet};
    use ibc::ibc_dispatch;

    struct PingPongProof has drop, store, key{}
    
    public(friend) fun new_ping_pong_proof(): PingPongProof {
        PingPongProof {}
    }


    const ACK_SUCCESS: vector<u8> = b"1";
    const ON_RECV_PACKET: u8 = 0;
    const ON_ACKNOWLEDGE_PACKET: u8 = 1;
    const ON_TIMEOUT_PACKET: u8 = 2;
    const ON_CHANNEL_OPEN_INIT: u8 = 3;
    const ON_CHANNEL_OPEN_TRY: u8 = 4;
    const ON_CHANNEL_OPEN_ACK: u8 = 5;
    const ON_CHANNEL_OPEN_CONFIRM: u8 = 6;
    const ON_CHANNEL_CLOSE_INIT: u8 = 7;
    const ON_CHANNEL_CLOSE_CONFIRM: u8 = 8;

    const ERR_ONLY_ONE_CHANNEL: u64 = 2001;
    const ERR_INVALID_ACK: u64 = 2002;
    const ERR_NO_CHANNEL: u64 = 2003;
    const ERR_INFINITE_GAME: u64 = 2004;

    const IBC_APP_SEED: vector<u8> = b"union-ibc-app-v1";

    #[event]
    struct RingEvent has copy, drop, store {
        ping: bool
    }

    #[event]
    struct TimedOutEvent has copy, drop, store {}

    #[event]
    struct AcknowledgedEvent has copy, drop, store {}

    struct PingPongPacket has copy, store, drop {
        ping: bool
        // counterparty_timeout: u64,
    }

    struct PingPong has key {
        channel_id: u32,
        seconds_before_timeout: u64
    }

    struct SignerRef has key {
        self_ref: object::ExtendRef,
        self_address: address
    }

    fun init_module(deployer: &signer) {
        assert!(signer::address_of(deployer) == @ping_pong, 1);
        let vault_constructor_ref = &object::create_named_object(deployer, IBC_APP_SEED);
        let vault_signer = &object::generate_signer(vault_constructor_ref);

        let pp = PingPong { channel_id: 0, seconds_before_timeout: 100000 };
        move_to(vault_signer, pp);

        move_to(
            vault_signer,
            SignerRef {
                self_ref: object::generate_extend_ref(vault_constructor_ref),
                self_address: signer::address_of(deployer)
            }
        );

        let cb =
            function_info::new_function_info(
                deployer,
                string::utf8(b"ping_pong_app"),
                string::utf8(b"on_packet")
            );

        ibc_dispatch::register_application<PingPongProof>(deployer, cb, new_ping_pong_proof());
    }

    public fun encode_packet(packet: &PingPongPacket): vector<u8> {
        let buf = vector::empty<u8>();

        // 31 bytes of left padding
        vector::append(
            &mut buf,
            x"00000000000000000000000000000000000000000000000000000000000000"
        );
        if (packet.ping) {
            vector::push_back(&mut buf, 1);
        } else {
            vector::push_back(&mut buf, 0);
        };

        // // 24 bytes of left padding (u256 - u64)
        // vector::append(&mut buf, x"000000000000000000000000000000000000000000000000");
        // let counterparty_timeout_bytes = bcs::to_bytes(&packet.counterparty_timeout);
        // // we want big-endian
        // vector::reverse(&mut counterparty_timeout_bytes);
        // vector::append(&mut buf, counterparty_timeout_bytes);

        buf
    }

    public fun decode_packet(data: &vector<u8>): PingPongPacket {
        // bool is left padded [0u8; 32], so we check the last element
        std::debug::print(data);
        let ping = *vector::borrow(data, 31) == 1;

        // // u64 is left padded [0u8; 32], rightmost 8 bytes are used to define u64
        // let counterparty_timeout_bytes = vector::slice(data, 56, 64);
        // // we parse it as little endian
        // vector::reverse(&mut counterparty_timeout_bytes);
        // let counterparty_timeout = from_bcs::to_u64(counterparty_timeout_bytes);

        PingPongPacket {
            ping
            // counterparty_timeout,
        }
    }

    public entry fun initiate(
        ping: bool
        // counterparty_timeout: u64,
        // local_timeout: u64
    ) acquires PingPong, SignerRef {
        let pp = borrow_global<PingPong>(get_vault_addr());
        if (pp.channel_id == 0) {
            abort ERR_NO_CHANNEL
        };

        ibc::send_packet(
            &get_signer(),
            get_self_address(),
            pp.channel_id,
            0, // no height timeout
            (std::timestamp::now_seconds() + pp.seconds_before_timeout) * 1_000_000_000,
            encode_packet(
                &PingPongPacket {
                    ping
                    // counterparty_timeout,
                }
            )
        );
    }

    #[view]
    public fun get_vault_addr(): address {
        object::create_object_address(&@ping_pong, IBC_APP_SEED)
    }

    public fun get_signer(): signer acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        object::generate_signer_for_extending(&vault.self_ref)
    }

    public fun get_self_address(): address acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        vault.self_address
    }

    struct RecvPacketParams has copy, drop, store {
        packet: Packet
    }

    struct AcknowledgePacketParams has copy, drop, store {
        packet: Packet,
        acknowledgement: vector<u8>
    }

    struct TimeoutPacketParams has copy, drop, store {
        packet: Packet
    }

    struct ChannelOpenInitParams has copy, drop, store {
        ordering: u8,
        connection_id: u32,
        channel_id: u32,
        version: vector<u8>
    }

    struct ChannelOpenTryParams has copy, drop, store {
        ordering: u8,
        connection_id: u32,
        channel_id: u32,
        counterparty_channel_id: u32,
        version: vector<u8>,
        counterparty_version: vector<u8>
    }

    struct ChannelOpenAckParams has copy, drop, store {
        channel_id: u32,
        counterparty_channel_id: u32,
        counterparty_version: vector<u8>,
    }

    struct ChannelOpenConfirmParams has copy, drop, store {
        channel_id: u32
    }

    struct ChannelCloseInitParams has copy, drop, store {
        channel_id: u32
    }

    struct ChannelCloseConfirmParams has copy, drop, store {
        channel_id: u32
    }

    // Functions with the "on_" prefix for each specific operation
    public fun on_recv_packet(packet: Packet): vector<u8> acquires PingPong, SignerRef {
        std::debug::print(&string::utf8(b"on_recv_packet called."));

        let packet_data = packet::data(&packet);
        std::debug::print(&string::utf8(b"packet_data is:"));
        std::debug::print(packet_data);
        let pp_packet = decode_packet(packet_data);
        event::emit(RingEvent { ping: pp_packet.ping });

        pp_packet.ping = !pp_packet.ping;

        initiate(pp_packet.ping);

        dispatcher::set_return_value<PingPongProof, ibc_dispatch::DynamicDispatchParam>(new_ping_pong_proof(), ACK_SUCCESS);

        ACK_SUCCESS
    }

    public fun on_acknowledge_packet(
        packet: Packet, acknowledgement: vector<u8>
    ) {
        if (acknowledgement != ACK_SUCCESS) {
            abort ERR_INVALID_ACK
        };
        event::emit(AcknowledgedEvent {});
    }

    public fun on_timeout_packet(_packet: Packet) {
        event::emit(TimedOutEvent {});
    }

    public fun on_channel_open_init(
        _ordering: u8,
        _connection_id: u32,
        _channel_id: u32,
        _version: vector<u8>
    ) acquires PingPong {
        if (borrow_global<PingPong>(get_vault_addr()).channel_id != 0) {
            abort ERR_ONLY_ONE_CHANNEL
        };
    }

    public fun on_channel_open_try(
        _ordering: u8,
        _connection_id: u32,
        _channel_id: u32,
        _counterparty_channel_id: u32,
        _version: vector<u8>,
        _counterparty_version: vector<u8>
    ) acquires PingPong {
        if (borrow_global<PingPong>(get_vault_addr()).channel_id != 0) {
            abort ERR_ONLY_ONE_CHANNEL
        };
    }

    public fun on_channel_open_ack(
        channel_id: u32,
        _counterparty_channel_id: u32,
        _counterparty_version: vector<u8>
    ) acquires PingPong {
        borrow_global_mut<PingPong>(get_vault_addr()).channel_id = channel_id;
    }

    public fun on_channel_open_confirm(channel_id: u32) acquires PingPong {
        borrow_global_mut<PingPong>(get_vault_addr()).channel_id = channel_id;
    }

    public fun on_channel_close_init(channel_id: u32) {
        abort ERR_INFINITE_GAME
    }

    public fun on_channel_close_confirm(channel_id: u32) {
        abort ERR_INFINITE_GAME
    }

    public fun on_packet<T: key>(_store: Object<T>): u64 acquires PingPong, SignerRef {
        // Get the current value of `return_value`
        // let other_data = dispatcher::get_return_value<PingPongProof, ibc_dispatch::DynamicDispatchParam>();
        // std::debug::print(&string::utf8(b"other_data before modification: "));
        // std::debug::print(&other_data);

        // // Modify `other_data` using the setter
        // dispatcher::set_return_value<PingPongProof, ibc_dispatch::DynamicDispatchParam>(new_ping_pong_proof(), b"new data");

        // // Get and print the modified `other_data`
        // let modified_other_data = dispatcher::get_return_value<PingPongProof, ibc_dispatch::DynamicDispatchParam>();
        // std::debug::print(&string::utf8(b"other_data after modification: "));
        // std::debug::print(&modified_other_data);

        // Access the `data` field through the getter
        let value: ibc_dispatch::DynamicDispatchParam = dispatcher::get_data(new_ping_pong_proof());

        // other_data = b"asdasd";
        // std::debug::print(&string::utf8(b"other_data after overwrite is: "));
        // std::debug::print(other_data);
        // let other_data: vector<u8> = dispatcher::retrieve_storage<PingPongProof, DynamicDispatchParam>(new_ping_pong_proof());
        // other_data = b"asdasd";
        if (option::is_some(&ibc_dispatch::recv_packet_param(&value))) {
            let recv_param = option::extract(&mut ibc_dispatch::recv_packet_param(&value));
            let pack = ibc_dispatch::get_packet_from_recv_param(&recv_param);
            on_recv_packet(*pack);
        } else if (option::is_some(&ibc_dispatch::acknowledge_packet_param(&value))) {
            let ack_param = option::extract(&mut ibc_dispatch::acknowledge_packet_param(&value));
            let pack = ibc_dispatch::get_packet_from_ack_param(&ack_param);
            let acknowledgement = ibc_dispatch::get_acknowledgement_from_ack_param(&ack_param);
            on_acknowledge_packet(*pack, *acknowledgement);

        } else if (option::is_some(&ibc_dispatch::timeout_packet_param(&value))) {
            let timeout_param = option::extract(&mut ibc_dispatch::timeout_packet_param(&value));
            let pack = ibc_dispatch::get_packet_from_timeout_param(&timeout_param);
            on_timeout_packet(*pack);

        } else if (option::is_some(&ibc_dispatch::channel_open_init_param(&value))) {
            let init_param = option::extract(&mut ibc_dispatch::channel_open_init_param(&value));
            let ordering = ibc_dispatch::get_ordering_from_channel_open_init_param(&init_param);
            let connection_id = ibc_dispatch::get_connection_id_from_channel_open_init_param(&init_param);
            let channel_id = ibc_dispatch::get_channel_id_from_channel_open_init_param(&init_param);
            let version = ibc_dispatch::get_version_from_channel_open_init_param(&init_param);
            on_channel_open_init(ordering, connection_id, channel_id, *version);

        } else if (option::is_some(&ibc_dispatch::channel_open_try_param(&value))) {
            let try_param = option::extract(&mut ibc_dispatch::channel_open_try_param(&value));
            let ordering = ibc_dispatch::get_ordering_from_channel_open_try_param(&try_param);
            let connection_id = ibc_dispatch::get_connection_id_from_channel_open_try_param(&try_param);
            let channel_id = ibc_dispatch::get_channel_id_from_channel_open_try_param(&try_param);
            let counterparty_channel_id = ibc_dispatch::get_counterparty_channel_id_from_channel_open_try_param(&try_param);
            let version = ibc_dispatch::get_version_from_channel_open_try_param(&try_param);
            let counterparty_version = ibc_dispatch::get_counterparty_version_from_channel_open_try_param(&try_param);
            on_channel_open_try(ordering, connection_id, channel_id, counterparty_channel_id, *version, *counterparty_version);

        } else if (option::is_some(&ibc_dispatch::channel_open_ack_param(&value))) {
            let ack_param = option::extract(&mut ibc_dispatch::channel_open_ack_param(&value));
            let channel_id = ibc_dispatch::get_channel_id_from_channel_open_ack_param(&ack_param);
            let counterparty_version = ibc_dispatch::get_counterparty_version_from_channel_open_ack_param(&ack_param);
            let counterparty_channel_id = ibc_dispatch::get_counterparty_channel_id_from_channel_open_ack_param(&ack_param);
            on_channel_open_ack(channel_id, counterparty_channel_id, *counterparty_version);

        } else if (option::is_some(&ibc_dispatch::channel_open_confirm_param(&value))) {
            let confirm_param = option::extract(&mut ibc_dispatch::channel_open_confirm_param(&value));
            let channel_id = ibc_dispatch::get_channel_id_from_channel_open_confirm_param(&confirm_param);
            on_channel_open_confirm(channel_id);

        } else if (option::is_some(&ibc_dispatch::channel_close_init_param(&value))) {
            let close_init_param = option::extract(&mut ibc_dispatch::channel_close_init_param(&value));
            let channel_id = ibc_dispatch::get_channel_id_from_channel_close_init_param(&close_init_param);
            on_channel_close_init(channel_id);

        } else if (option::is_some(&ibc_dispatch::channel_close_confirm_param(&value))) {
            let close_confirm_param = option::extract(&mut ibc_dispatch::channel_close_confirm_param(&value));
            let channel_id = ibc_dispatch::get_channel_id_from_channel_close_confirm_param(&close_confirm_param);
            on_channel_close_confirm(channel_id);

        } else {
            std::debug::print(&string::utf8(b"Invalid function type. under ping_pong_app.move!!!!"));
        };

        0
    }

    #[test(deployer = @ping_pong)]
    public fun test_signer(deployer: &signer) acquires SignerRef {
        std::debug::print(deployer);
        init_module(deployer);
        std::debug::print(&get_signer());
    }

    #[test(framework = @0x1)]
    public fun test_decode(framework: &signer) {
        std::timestamp::set_time_has_started_for_testing(framework);
        let encoded =
            x"000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000003e8";

        let decoded = decode_packet(&encoded);
    }
}
