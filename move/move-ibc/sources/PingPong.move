module IBCModuleAddr::PingPong {
    use std::signer;
    use std::vector;
    use aptos_framework::event;
    use aptos_framework::account::{Self as AptosAccount};
    use aptos_framework::timestamp;
    use aptos_std::string::{Self, String};
    use IBC::Core;
    use aptos_std::any::{Self, Any};
    use IBC::height;
    use std::bcs;

    const ACK_SUCCESS: vector<u8> = b"\x01";
    const ERR_ONLY_ONE_CHANNEL: u64 = 2001;
    const ERR_INVALID_ACK: u64 = 2002;
    const ERR_NO_CHANNEL: u64 = 2003;
    const ERR_INFINITE_GAME: u64 = 2004;

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

    public fun encode_packet(packet: &PingPongPacket): Any {
        any::pack<PingPongPacket>(*packet)
        // bcs::to_bytes(packet)
    }

    public fun decode_packet(data: Any): PingPongPacket {
        any::unpack<PingPongPacket>(data)
        // bcs::from_bytes<PingPongPacket>(data)
    }

    public fun initiate(
        packet: PingPongPacket,
        local_timeout: u64
    ) acquires PingPong {
        let pp = borrow_global<PingPong>(@0x1); // assuming @0x1 is the address of the PingPong instance
        if (string::length(&pp.channel_id) == 0) {
            abort ERR_NO_CHANNEL
        }

        // TODO: send_packet here
        // Core::send_packet(  
        //     &pp.channel_id,
        //     0, // no height timeout
        //     local_timeout,
        //     encode_packet(&packet)
        // );
    }

    public fun on_recv_packet(
        packet: Core::IbcCoreChannelV1Packet,
        proof: Any,
        proof_height: height::Height
    ) {
        // Here we'll call on_recv_packet of ibc module and it will return packet

        // // let pp_packet = decode_packet(&packet.data);
        // let ping = true;
        // event::emit(RingEvent { ping: pp_packet.ping });

        // let local_timeout = pp_packet.counterparty_timeout;

        // pp_packet.ping = !pp_packet.ping;
        // pp_packet.counterparty_timeout = timestamp::now_seconds() + borrow_global<PingPong>(@0x1).timeout;

        // initiate(pp_packet, local_timeout);
    }

    public fun on_acknowledgement_packet(
        packet: Core::IbcCoreChannelV1Packet,
        acknowledgement: vector<u8>
    ) {
        if (acknowledgement != ACK_SUCCESS) {
            abort ERR_INVALID_ACK;
        };
        event::emit(AcknowledgedEvent {});
    }

    public fun on_timeout_packet(
        packet: Core::IbcCoreChannelV1Packet
    )  {
        event::emit(TimedOutEvent {});
    }

    public fun on_chan_open_init(
        port_id: String,
        channel_id: String
    ) acquires PingPong {
        if (string::length(&borrow_global<PingPong>(@0x1).channel_id) != 0) {
            abort ERR_ONLY_ONE_CHANNEL;
        };
        // Store the channel_id
        borrow_global_mut<PingPong>(@0x1).channel_id = channel_id;
    }

    public fun on_chan_open_try(
        port_id: String,
        channel_id: String
    ) acquires PingPong {
        if (string::length(&borrow_global<PingPong>(@0x1).channel_id) != 0) {
            abort ERR_ONLY_ONE_CHANNEL;
        };
        // Store the channel_id
        borrow_global_mut<PingPong>(@0x1).channel_id = channel_id;
    }

    public fun on_chan_open_ack(
        port_id: String,
        channel_id: String
    ) acquires PingPong {
        // Store the channel_id
        borrow_global_mut<PingPong>(@0x1).channel_id = channel_id;
    }

    public fun on_chan_open_confirm(
        port_id: String,
        channel_id: String,
        proof_ack: Any,
        proof_height: height::Height
    ) acquires PingPong {
        // Store the channel_id
        borrow_global_mut<PingPong>(@0x1).channel_id = channel_id;
        Core::channel_open_confirm(port_id, channel_id, proof_ack, proof_height); // Call the IBC function
    }

    public fun on_chan_close_init(
        port_id: String,
        channel_id: String
    ) {
        abort ERR_INFINITE_GAME;
    }

    public fun on_chan_close_confirm(
        port_id: String,
        channel_id: String
    ) {
        abort ERR_INFINITE_GAME;
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
}
