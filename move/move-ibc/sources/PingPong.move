module IBCModuleAddr::PingPong {
    use aptos_framework::event;
    use aptos_framework::timestamp;
    use std::object;
    use std::signer;
    use aptos_std::string::{Self, String};
    use IBC::Core;
    use std::vector;
    use std::bcs;
    use aptos_std::from_bcs;
    use aptos_std::any::{Any};
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
    }

    fun init_module(deployer: &signer) {
        assert!(signer::address_of(deployer) == @IBCModuleAddr, 1);
        let vault_constructor_ref = &object::create_named_object(deployer, VAULT_SEED);
        let vault_signer = &object::generate_signer(vault_constructor_ref);
        move_to(vault_signer, SignerRef {
            self_ref: object::generate_extend_ref(vault_constructor_ref)
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

        Core::send_packet(  
            &get_signer(),
            pp.channel_id,
            height::default(), // no height timeout
            local_timeout,
            encode_packet(&packet)
        );
    }

    public fun recv_packet(
        msg_port_id: String,
        msg_channel_id: String,
        packet: Packet,
        msg_proof: Any,
        msg_proof_height: height::Height,
    ) acquires PingPong, SignerRef {
        let pp_packet = decode_packet(packet::data(&packet));
        event::emit(RingEvent { ping: pp_packet.ping });

        let local_timeout = pp_packet.counterparty_timeout;

        pp_packet.ping = !pp_packet.ping;
        pp_packet.counterparty_timeout = timestamp::now_seconds() + borrow_global<PingPong>(@0x1).timeout;

        initiate(pp_packet, local_timeout);

        Core::recv_packet(
            &get_signer(),
            msg_port_id,
            msg_channel_id,
            packet,
            msg_proof,
            msg_proof_height,
            vector[1]
        );
    }

    public fun acknowledge_packet(
        packet: packet::Packet,
        acknowledgement: vector<u8>,
        proof: Any,
        proof_height: height::Height
    ) acquires SignerRef {
        Core::acknowledge_packet(&get_signer(), packet, acknowledgement, proof, proof_height);
        event::emit(AcknowledgedEvent {});
    }

    public fun timeout_packet(
        _packet: Packet
    )  {
        event::emit(TimedOutEvent {});
    }

    public fun chan_open_init(
        port_id: String,
        connection_hops: vector<String>,
        ordering: u8,
        counterparty: channel::Counterparty,
        version: String,
    ) acquires PingPong, SignerRef {
        // TODO(aeryz): save the channel here
        Core::channel_open_init(
            &get_signer(),
            port_id,
            connection_hops,
            ordering,
            counterparty,
            version,
        );
        if (string::length(&borrow_global<PingPong>(@0x1).channel_id) != 0) {
            abort ERR_ONLY_ONE_CHANNEL
        };
    }

    public fun chan_open_try(
        port_id: String,
        connection_hops: vector<String>,
        ordering: u8,
        counterparty: channel::Counterparty,
        counterparty_version: String,
        version: String,
        proof_init: Any,
        proof_height: height::Height,
    ) acquires PingPong, SignerRef {
        // TODO(aeryz): save the channel here
        Core::channel_open_try(
            &get_signer(),
            port_id,
            connection_hops,
            ordering,
            counterparty,
            counterparty_version,
            version,
            proof_init,
            proof_height,
        );

        if (string::length(&borrow_global<PingPong>(@0x1).channel_id) != 0) {
            abort ERR_ONLY_ONE_CHANNEL
        };
    }

    public fun chan_open_ack(
        port_id: String,
        channel_id: String,
        counterparty_channel_id: String,
        counterparty_version: String,
        proof_try: Any,
        proof_height: height::Height
    ) acquires PingPong {
        // Store the channel_id
        let (_,channel_id,_,_) = Core::channel_open_ack(
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
            proof_try,
            proof_height
        );
        borrow_global_mut<PingPong>(@0x1).channel_id = channel_id;
    }

    public fun chan_open_confirm(
        port_id: String,
        channel_id: String,
        proof_ack: Any,
        proof_height: height::Height
    ) acquires PingPong {
        // Store the channel_id
        let (_,channel_id) = Core::channel_open_confirm(
            port_id,
            channel_id,
            proof_ack,
            proof_height
        );

        borrow_global_mut<PingPong>(@0x1).channel_id = channel_id;
    }

    public fun chan_close_init(
        _port_id: String,
        _channel_id: String
    ) {
        abort ERR_INFINITE_GAME
    }

    public fun chan_close_confirm(
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
        object::create_object_address(&@IBCModuleAddr, VAULT_SEED)
    }

    public fun get_signer(): signer acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        object::generate_signer_for_extending(&vault.self_ref)
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
}
