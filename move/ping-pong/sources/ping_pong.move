module ping_pong::ibc {
    use std::event;
    use std::timestamp;
    use std::object;
    use std::signer;
    use std::string::{Self, String, utf8};
    use IBC::ibc;
    use std::vector;
    use std::bcs;
    use std::from_bcs;
    use IBC::height;
    use IBC::channel;
    use IBC::packet::{Self, Packet};

    const ACK_SUCCESS: vector<u8> = b"\x01";
    const ERR_ONLY_ONE_CHANNEL: u64 = 2001;
    const ERR_INVALID_ACK: u64 = 2002;
    const ERR_NO_CHANNEL: u64 = 2003;
    const ERR_INFINITE_GAME: u64 = 2004;

    const VAULT_SEED: vector<u8> = b"Vault Seed Example";

    #[event]
    struct RingEvent has copy, drop, store {
        ping: bool,
    }

    #[event]
    struct TimedOutEvent has copy, drop, store {}

    #[event]
    struct AcknowledgedEvent has copy, drop, store {}

    struct PingPongPacket has copy, store, drop {
        ping: bool,
        counterparty_timeout: u64,
    }

    struct PingPong has key {
        channel_id: String,
        revision_number: u64,
        timeout: u64,
    }

    struct SignerRef has key {
        self_ref: object::ExtendRef,
        self_address: address
    }

    fun init_module(deployer: &signer) {
        assert!(signer::address_of(deployer) == @ping_pong, 1);
        let vault_constructor_ref = &object::create_named_object(deployer, VAULT_SEED);
        let vault_signer = &object::generate_signer(vault_constructor_ref);
        move_to(vault_signer, SignerRef {
            self_ref: object::generate_extend_ref(vault_constructor_ref),
            self_address: signer::address_of(deployer),
        });
    }

    public fun encode_packet(packet: &PingPongPacket): vector<u8> {
        let buf = vector::empty<u8>();

        // Convert `ping` to u8 (1 if true, 0 if false) and append to buffer
        let ping: u8 = if (packet.ping) { 1 } else { 0 };
        vector::push_back(&mut buf, ping);

        // Convert `counterparty_timeout` to bytes and append to buffer
        let counterparty_timeout_bytes = bcs::to_bytes(&packet.counterparty_timeout);
        vector::append(&mut buf, counterparty_timeout_bytes);

        buf
    }


    public fun decode_packet(data: &vector<u8>): PingPongPacket {
        // Extract the `ping` value
        let ping = *vector::borrow(data, 0) == 1;

        // Extract the `counterparty_timeout` bytes and convert them back to `u64`
        let counterparty_timeout_bytes = vector::slice(data, 1, 9);
        let counterparty_timeout = from_bcs::to_u64(counterparty_timeout_bytes);

        PingPongPacket {
            ping,
            counterparty_timeout,
        }
    }


    public fun initiate(
        packet: PingPongPacket,
        local_timeout: u64
    ) acquires PingPong, SignerRef {
        let pp = borrow_global<PingPong>(@0x1); // assuming @0x1 is the address of the PingPong instance
        if (string::length(&pp.channel_id) == 0) {
            abort ERR_NO_CHANNEL
        };

        ibc::send_packet(  
            &get_signer(),
            pp.channel_id,
            height::default(), // no height timeout
            local_timeout,
            encode_packet(&packet)
        );
    }

    // public entry fun recv_packet(
    //     channel_id: String,
    //     packet: Packet,
    //     proof: vector<u8>,
    //     proof_height_revision_num: u64,
    //     proof_height_revision_height: u64,
    // ) acquires PingPong, SignerRef {
    //     let pp_packet = decode_packet(packet::data(&packet));
    //     event::emit(RingEvent { ping: pp_packet.ping });

    //     let local_timeout = pp_packet.counterparty_timeout;

    //     pp_packet.ping = !pp_packet.ping;
    //     pp_packet.counterparty_timeout = timestamp::now_seconds() + borrow_global<PingPong>(@0x1).timeout;

    //     initiate(pp_packet, local_timeout);

    //     ibc::recv_packet(
    //         &get_signer(),
    //         utf8(b""),
    //         channel_id,
    //         packet,
    //         proof,
    //         height::new(proof_height_revision_num, proof_height_revision_height),
    //         vector[1]
    //     );
    // }


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
        proof_height_revision_height: u64,
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
                    packet_timeout_revision_height,
                ),
                packet_timeout_timestamp,
            ),
            acknowledgement,
            proof,
            height::new(proof_height_revision_num, proof_height_revision_height),
        );
        event::emit(AcknowledgedEvent {});
    }

    public fun timeout_packet(
        _packet: Packet
    )  {
        event::emit(TimedOutEvent {});
    }

    public entry fun channel_open_init(
        connection_hops: vector<String>,
        ordering: u8,
        counterparty_port_id: String,
        counterparty_channel_id: String,
        version: String,
    ) acquires PingPong, SignerRef {
        // TODO(aeryz): save the channel here
        ibc::channel_open_init(
            &get_signer(),
            get_self_address(),
            connection_hops,
            ordering,
            channel::new_counterparty(counterparty_port_id, counterparty_channel_id),
            version,
        );
        if (string::length(&borrow_global<PingPong>(@0x1).channel_id) != 0) {
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
        proof_height_revision_height: u64,
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
            height::new(proof_height_revision_num, proof_height_revision_height),
        );

        if (string::length(&borrow_global<PingPong>(@0x1).channel_id) != 0) {
            abort ERR_ONLY_ONE_CHANNEL
        };
    }

    public entry fun channel_open_ack(
        channel_id: String,
        counterparty_channel_id: String,
        counterparty_version: String,
        proof_try: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) acquires PingPong, SignerRef {
        // Store the channel_id
        ibc::channel_open_ack(
            &get_signer(),
            get_self_address(),
            channel_id,
            counterparty_channel_id,
            counterparty_version,
            proof_try,
            height::new(proof_height_revision_num, proof_height_revision_height),
        );
        borrow_global_mut<PingPong>(@0x1).channel_id = channel_id;
    }

    public fun channel_open_confirm(
        channel_id: String,
        proof_ack: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) acquires PingPong, SignerRef {
        ibc::channel_open_confirm(
            &get_signer(),
            get_self_address(),
            channel_id,
            proof_ack,
            height::new(proof_height_revision_num, proof_height_revision_height),
        );

        borrow_global_mut<PingPong>(@0x1).channel_id = channel_id;
    }

    public entry fun chan_close_init(
        _port_id: String,
        _channel_id: String
    ) {
        abort ERR_INFINITE_GAME
    }

    public entry fun chan_close_confirm(
        _port_id: String,
        _channel_id: String
    ) {
        abort ERR_INFINITE_GAME
    }

    public fun initialize(
        account: &signer,
        revision_number: u64,
        timeout: u64
    ) {
        let pp = PingPong {
            channel_id: string::utf8(b""),
            revision_number,
            timeout
        };
        move_to(account, pp);
    }

    #[view]
    public fun get_vault_addr(): address {
        object::create_object_address(&@ping_pong, VAULT_SEED)
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
        let packet = PingPongPacket {
            ping: true,
            counterparty_timeout: 1000,
        };
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
}
