// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's sui subdirectory                      
//                       The Licensed Work is (c) 2024 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
// 

// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.

// Notice

// Business Source License 1.1

// Terms

// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.

// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.

// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.

// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.

// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.

// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.

// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).

// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

module ping_pong::ibc {
    use sui::event;
    // use std::timestamp;
    use std::string::{Self, String, utf8};
    use ibc::ibc;
    use std::vector;
    use sui::clock;
    use std::bcs;
    use ibc::channel::{Self, Channel};
    use ibc::packet::{Self, Packet};

    const ACK_SUCCESS: vector<u8> = b"\x01";
    const ERR_ONLY_ONE_CHANNEL: u64 = 2001;
    const ERR_INVALID_ACK: u64 = 2002;
    const ERR_NO_CHANNEL: u64 = 2003;
    const ERR_INFINITE_GAME: u64 = 2004;

    const IBC_APP_SEED: vector<u8> = b"union-ibc-app-v1";

    #[event]
    public struct RingEvent has copy, drop, store {
        ping: bool
    }

    #[event]
    public struct TimedOutEvent has copy, drop, store {}

    #[event]
    public struct AcknowledgedEvent has copy, drop, store {}


    public struct PingPongPacket has copy, store, drop {
        ping: bool
        // counterparty_timeout: u64,
    }

    public struct PingPong has key {
        id: UID,
        channel_id: u32,
        seconds_before_timeout: u64
    }

    fun init(ctx: &mut TxContext) {
        let id = object::new(ctx);
        transfer::share_object(PingPong {
            id: id,
            channel_id: 0,
            seconds_before_timeout: 100000
        });
    }

    public fun encode_packet(packet: &PingPongPacket): vector<u8> {
        let mut buf = vector::empty<u8>();

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
        ibc_store: &mut ibc::IBCStore,
        pp_store: &mut PingPong,
        clock: &clock::Clock,
        ping: bool
    ) {
        if (pp_store.channel_id == 0) {
            abort ERR_NO_CHANNEL
        };

    
        ibc::send_packet(
            ibc_store,
            pp_store.channel_id,
            0, // no height timeout
            (clock::timestamp_ms(clock) + pp_store.seconds_before_timeout) * 1_000_000,
            encode_packet(
                &PingPongPacket {
                    ping
                    // counterparty_timeout,
                }
            )
        );
    }


    public entry fun recv_packet(
        ibc_store: &mut ibc::IBCStore,
        pp_store: &mut PingPong,
        clock: &clock::Clock,
        packet_source_channel: u32,
        packet_destination_channel: u32,
        packet_data: vector<u8>,
        packet_timeout_height: u64,
        packet_timeout_timestamp: u64,
        proof: vector<u8>,
        proof_height: u64
    ) {
        let mut pp_packet = decode_packet(&packet_data);
        event::emit(RingEvent { ping: pp_packet.ping });

        // let local_timeout = pp_packet.counterparty_timeout;

        pp_packet.ping = !pp_packet.ping;

        initiate(ibc_store, pp_store, clock, pp_packet.ping);

        let packet =
            packet::new(
                packet_source_channel,
                packet_destination_channel,
                packet_data,
                packet_timeout_height,
                packet_timeout_timestamp
            );

        ibc::recv_packet(
            ibc_store,
            clock,
            vector[packet],
            proof,
            proof_height,
            vector[1]
        );
    }

    public entry fun acknowledge_packet(
        ibc_store: &mut ibc::IBCStore,
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_datas: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        acknowledgements: vector<vector<u8>>,
        proof: vector<u8>,
        proof_height: u64
    ) {
        let mut packets: vector<Packet> = vector::empty();
        let mut i = 0;
        while (i < vector::length(&packet_source_channels)) {
            vector::push_back(
                &mut packets,
                packet::new(
                    *vector::borrow(&packet_source_channels, i),
                    *vector::borrow(&packet_destination_channels, i),
                    *vector::borrow(&packet_datas, i),
                    *vector::borrow(&packet_timeout_heights, i),
                    *vector::borrow(&packet_timeout_timestamps, i)
                )
            );
            i = i + 1;
        };
        ibc::acknowledge_packet(
            ibc_store,
            packets,
            acknowledgements,
            proof,
            proof_height
        );
        event::emit(AcknowledgedEvent {});
    }

    public fun timeout_packet(_packet: Packet) {
        event::emit(TimedOutEvent {});
    }

    public entry fun channel_open_init(
        ibc_store: &mut ibc::IBCStore,
        pp_store: &mut PingPong,
        counterparty_port_id: vector<u8>,
        connection_id: u32,
        version: String
    ) {
        // TODO(aeryz): save the channel here
        ibc::channel_open_init(
            ibc_store,
            utf8(b"@ping_pong"), // TODO: Do we need this port_id?
            counterparty_port_id,
            connection_id,
            version
        );
        if (pp_store.channel_id != 0) {
            abort ERR_ONLY_ONE_CHANNEL
        };
    }

    public entry fun channel_open_try(
        ibc_store: &mut ibc::IBCStore,
        pp_store: &mut PingPong,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String,
        counterparty_version: String,
        proof_init: vector<u8>,
        proof_height: u64
    ) {
        // TODO(aeryz): save the channel here
        ibc::channel_open_try(
            ibc_store,
            utf8(b"@ping_pong"), // TODO: Do we need this port_id?
            connection_id,
            counterparty_channel_id,
            counterparty_port_id,
            version,
            counterparty_version,
            proof_init,
            proof_height
        );

        if (pp_store.channel_id != 0) {
            abort ERR_ONLY_ONE_CHANNEL
        };
    }

    public entry fun channel_open_ack(
        ibc_store: &mut ibc::IBCStore,
        pp_store: &mut PingPong,
        channel_id: u32,
        counterparty_version: String,
        counterparty_channel_id: u32,
        proof_try: vector<u8>,
        proof_height: u64
    ) {
        // Store the channel_id
        ibc::channel_open_ack(
            ibc_store,
            utf8(b"@ping_pong"), // TODO: Do we need this port_id?
            channel_id,
            counterparty_version,
            counterparty_channel_id,
            proof_try,
            proof_height
        );
        pp_store.channel_id = channel_id;
    }

    public entry fun channel_open_confirm(
        ibc_store: &mut ibc::IBCStore,
        pp_store: &mut PingPong,
        channel_id: u32, proof_ack: vector<u8>, proof_height: u64
    ) {
        ibc::channel_open_confirm(
            ibc_store,
            utf8(b"@ping_pong"), // TODO: Do we need this port_id?
            channel_id,
            proof_ack,
            proof_height
        );

        pp_store.channel_id = channel_id;
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

}