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
    use std::from_bcs;
    use ibc::height;
    use ibc::channel;
    use ibc::packet::{Self, Packet};

    struct PingPongProof has drop, store {}
    
    public(friend) fun new_ping_pong_proof(): PingPongProof {
        PingPongProof {}
    }

    friend ibc::sample_ibc;


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

    struct DynamicDispatchParam has copy, store, drop, key {
        recv_packet_param: option::Option<RecvPacketParams>,
        acknowledge_packet_param: option::Option<AcknowledgePacketParams>,
        timeout_packet_param: option::Option<TimeoutPacketParams>,
        channel_open_init_param: option::Option<ChannelOpenInitParams>,
        channel_open_try_param: option::Option<ChannelOpenTryParams>,
        channel_open_ack_param: option::Option<ChannelOpenAckParams>,
        channel_open_confirm_param: option::Option<ChannelOpenConfirmParams>,
        channel_close_init_param: option::Option<ChannelCloseInitParams>,
        channel_close_confirm_param: option::Option<ChannelCloseConfirmParams>
    }

    public(friend) fun new_dynamic_dispatch_param(
        recv_packet_param: option::Option<RecvPacketParams>,
        acknowledge_packet_param: option::Option<AcknowledgePacketParams>,
        timeout_packet_param: option::Option<TimeoutPacketParams>,
        channel_open_init_param: option::Option<ChannelOpenInitParams>,
        channel_open_try_param: option::Option<ChannelOpenTryParams>,
        channel_open_ack_param: option::Option<ChannelOpenAckParams>,
        channel_open_confirm_param: option::Option<ChannelOpenConfirmParams>,
        channel_close_init_param: option::Option<ChannelCloseInitParams>,
        channel_close_confirm_param: option::Option<ChannelCloseConfirmParams>
    ): DynamicDispatchParam {
        DynamicDispatchParam {
            recv_packet_param,
            acknowledge_packet_param,
            timeout_packet_param,
            channel_open_init_param,
            channel_open_try_param,
            channel_open_ack_param,
            channel_open_confirm_param,
            channel_close_init_param,
            channel_close_confirm_param
        }
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
        counterparty_version: vector<u8>,
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
        let value: DynamicDispatchParam = dispatcher::retrieve(new_ping_pong_proof());

        if (option::is_some(&value.recv_packet_param)){
            let pack = option::extract(&mut value.recv_packet_param).packet;
            on_recv_packet(pack);
        } else if (option::is_some(&value.acknowledge_packet_param)) {
            let pack = option::extract(&mut value.acknowledge_packet_param).packet;
            let acknowledgement = option::extract(&mut value.acknowledge_packet_param).acknowledgement;
            on_acknowledge_packet(pack, acknowledgement);
        } else if (option::is_some(&value.timeout_packet_param)) {
            let pack = option::extract(&mut value.timeout_packet_param).packet;
            on_timeout_packet(pack);
        } else if (option::is_some(&value.channel_open_init_param)) {
            let ordering = option::extract(&mut value.channel_open_init_param).ordering;
            let connection_id = option::extract(&mut value.channel_open_init_param).connection_id;
            let channel_id = option::extract(&mut value.channel_open_init_param).channel_id;
            let version = option::extract(&mut value.channel_open_init_param).version;
            on_channel_open_init(ordering, connection_id, channel_id, version);
        } else if (option::is_some(&value.channel_open_try_param)) {
            let ordering = option::extract(&mut value.channel_open_try_param).ordering;
            let connection_id = option::extract(&mut value.channel_open_try_param).connection_id;
            let channel_id = option::extract(&mut value.channel_open_try_param).channel_id;
            let counterparty_channel_id = option::extract(&mut value.channel_open_try_param).counterparty_channel_id;
            let version = option::extract(&mut value.channel_open_try_param).version;
            let counterparty_version = option::extract(&mut value.channel_open_try_param).counterparty_version;
            on_channel_open_try(
                ordering,
                connection_id,
                channel_id,
                counterparty_channel_id,
                version,
                counterparty_version
            );
        } else if (option::is_some(&value.channel_open_ack_param)) {
            let channel_id = option::extract(&mut value.channel_open_ack_param).channel_id;
            let counterparty_version = option::extract(&mut value.channel_open_ack_param).counterparty_version;
            let counterparty_version = option::extract(&mut value.channel_open_ack_param).counterparty_version;
            on_channel_open_ack(channel_id, counterparty_version, counterparty_version);
        } else if (option::is_some(&value.channel_open_confirm_param)) {
            let channel_id = option::extract(&mut value.channel_open_confirm_param).channel_id;
            on_channel_open_confirm(channel_id);
        } else if (option::is_some(&value.channel_close_init_param)) {
            let channel_id = option::extract(&mut value.channel_close_init_param).channel_id;
            on_channel_close_init(channel_id);
        } else if (option::is_some(&value.channel_close_confirm_param)) {
            let channel_id = option::extract(&mut value.channel_close_confirm_param).channel_id;
            on_channel_close_confirm(channel_id);
        } else {
            std::debug::print(&string::utf8(b"Invalid function type. under ping_pong_app.move!!!!"));
            // abort 0
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
