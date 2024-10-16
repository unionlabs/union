module ping_pong::ibc {
    use std::event;
    use std::timestamp;
    use std::object;
    use std::signer;
    use std::string::{Self, String, utf8};
    use ibc::ibc;
    use std::vector;
    use std::bcs;
    use std::from_bcs;
    use ibc::height;
    use ibc::channel;
    use ibc::packet::{Self, Packet};

    const ACK_SUCCESS: vector<u8> = b"\x01";
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
        channel_id: String,
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

        let pp = PingPong { channel_id: string::utf8(b""), seconds_before_timeout: 100000 };
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
        if (string::length(&pp.channel_id) == 0) {
            abort ERR_NO_CHANNEL
        };

        ibc::send_packet(
            &get_signer(),
            get_self_address(),
            pp.channel_id,
            height::default(), // no height timeout
            (std::timestamp::now_seconds() + pp.seconds_before_timeout) * 1_000_000_000,
            encode_packet(
                &PingPongPacket {
                    ping
                    // counterparty_timeout,
                }
            )
        );
    }

    public entry fun recv_packet(
        packet_sequence: u64,
        packet_source_port: String,
        packet_source_channel: String,
        packet_destination_port: String,
        packet_destination_channel: String,
        packet_data: vector<u8>,
        packet_timeout_revision_num: u64,
        packet_timeout_revision_height: u64,
        packet_timeout_timestamp: u64,
        proof: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64
    ) acquires PingPong, SignerRef {
        let pp_packet = decode_packet(&packet_data);
        event::emit(RingEvent { ping: pp_packet.ping });

        // let local_timeout = pp_packet.counterparty_timeout;

        pp_packet.ping = !pp_packet.ping;
        // pp_packet.counterparty_timeout = timestamp::now_seconds() * 1_000_000_000 + borrow_global<PingPong>(get_vault_addr()).timeout;

        initiate(pp_packet.ping);

        ibc::recv_packet(
            &get_signer(),
            get_self_address(),
            packet::new(
                packet_sequence,
                packet_source_port,
                packet_source_channel,
                packet_destination_port,
                packet_destination_channel,
                packet_data,
                height::new(
                    packet_timeout_revision_num,
                    packet_timeout_revision_height
                ),
                packet_timeout_timestamp
            ),
            proof,
            height::new(proof_height_revision_num, proof_height_revision_height),
            vector[1]
        );
    }

    public entry fun acknowledge_packet(
        packet_sequence: u64,
        packet_source_port: String,
        packet_source_channel: String,
        packet_destination_port: String,
        packet_destination_channel: String,
        packet_data: vector<u8>,
        packet_timeout_revision_num: u64,
        packet_timeout_revision_height: u64,
        packet_timeout_timestamp: u64,
        acknowledgement: vector<u8>,
        proof: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64
    ) acquires SignerRef {
        ibc::acknowledge_packet(
            &get_signer(),
            get_self_address(),
            packet::new(
                packet_sequence,
                packet_source_port,
                packet_source_channel,
                packet_destination_port,
                packet_destination_channel,
                packet_data,
                height::new(
                    packet_timeout_revision_num,
                    packet_timeout_revision_height
                ),
                packet_timeout_timestamp
            ),
            acknowledgement,
            proof,
            height::new(proof_height_revision_num, proof_height_revision_height)
        );
        event::emit(AcknowledgedEvent {});
    }

    public fun timeout_packet(_packet: Packet) {
        event::emit(TimedOutEvent {});
    }

    public entry fun channel_open_init(
        connection_hops: vector<String>,
        ordering: u8,
        counterparty_port_id: String,
        counterparty_channel_id: String,
        version: String
    ) acquires PingPong, SignerRef {
        // TODO(aeryz): save the channel here
        ibc::channel_open_init(
            &get_signer(),
            get_self_address(),
            connection_hops,
            ordering,
            channel::new_counterparty(counterparty_port_id, counterparty_channel_id),
            version
        );
        if (string::length(
            &borrow_global<PingPong>(get_vault_addr()).channel_id
        ) != 0) {
            abort ERR_ONLY_ONE_CHANNEL
        };
    }

    public entry fun channel_open_try(
        connection_hops: vector<String>,
        ordering: u8,
        counterparty_port_id: String,
        counterparty_channel_id: String,
        counterparty_version: String,
        version: String,
        proof_init: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64
    ) acquires PingPong, SignerRef {
        // TODO(aeryz): save the channel here
        ibc::channel_open_try(
            &get_signer(),
            get_self_address(),
            connection_hops,
            ordering,
            channel::new_counterparty(counterparty_port_id, counterparty_channel_id),
            counterparty_version,
            version,
            proof_init,
            height::new(proof_height_revision_num, proof_height_revision_height)
        );

        if (string::length(
            &borrow_global<PingPong>(get_vault_addr()).channel_id
        ) != 0) {
            abort ERR_ONLY_ONE_CHANNEL
        };
    }

    public entry fun channel_open_ack(
        channel_id: String,
        counterparty_channel_id: String,
        counterparty_version: String,
        proof_try: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64
    ) acquires PingPong, SignerRef {
        // Store the channel_id
        ibc::channel_open_ack(
            &get_signer(),
            get_self_address(),
            channel_id,
            counterparty_channel_id,
            counterparty_version,
            proof_try,
            height::new(proof_height_revision_num, proof_height_revision_height)
        );
        borrow_global_mut<PingPong>(get_vault_addr()).channel_id = channel_id;
    }

    public entry fun channel_open_confirm(
        channel_id: String,
        proof_ack: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64
    ) acquires PingPong, SignerRef {
        ibc::channel_open_confirm(
            &get_signer(),
            get_self_address(),
            channel_id,
            proof_ack,
            height::new(proof_height_revision_num, proof_height_revision_height)
        );

        borrow_global_mut<PingPong>(get_vault_addr()).channel_id = channel_id;
    }

    public entry fun channel_close_init(
        _port_id: String, _channel_id: String
    ) {
        abort ERR_INFINITE_GAME
    }

    public entry fun channel_close_confirm(
        _port_id: String, _channel_id: String
    ) {
        abort ERR_INFINITE_GAME
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

    #[test]
    public fun test_encode() {
        let packet = PingPongPacket { ping: true, counterparty_timeout: 1000 };
        let encoded = encode_packet(&packet);
        let decoded = decode_packet(&encoded);

        assert!(decoded.ping == packet.ping, 1);
        assert!(decoded.counterparty_timeout == packet.counterparty_timeout, 2);
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
