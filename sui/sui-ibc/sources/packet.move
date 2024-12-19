module ibc::packet {
    use std::vector;
    use sui::object::{Self, UID};
    use ibc::ethabi;

    public struct Packet has copy, store, drop {
        source_channel: u32,
        destination_channel: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64
    }


    public fun source_channel(packet: &Packet): u32 {
        packet.source_channel
    }

    public fun destination_channel(packet: &Packet): u32 {
        packet.destination_channel
    }

    public fun data(packet: &Packet): &vector<u8> {
        &packet.data
    }

    public fun timeout_timestamp(packet: &Packet): u64 {
        packet.timeout_timestamp
    }

    public fun timeout_height(packet: &Packet): u64 {
        packet.timeout_height
    }

    public fun new(
        source_channel: u32,
        destination_channel: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64
    ): Packet {
        Packet {
            source_channel,
            destination_channel,
            data,
            timeout_height,
            timeout_timestamp
        }
    }

    public fun default(): Packet {
        new(0, 0, vector::empty(), 0, 0)
    }

    public fun encode(packet: &Packet): vector<u8> {
        let mut buf = vector::empty();

        // TODO(aeryz): figure out why this happens
        ethabi::encode_uint(&mut buf, 0x20);
        ethabi::encode_uint(&mut buf, packet.source_channel);
        ethabi::encode_uint(&mut buf, packet.destination_channel);
        // offset of `data`
        ethabi::encode_uint(&mut buf, 5 * 32);
        ethabi::encode_uint(&mut buf, packet.timeout_height);
        ethabi::encode_uint(&mut buf, packet.timeout_timestamp);
        ethabi::encode_bytes(&mut buf, &packet.data);
    
        buf
    }

    // #[test]
    // fun test_packet_creation() {
    //     let sequence = 1;
    //     let source_channel = 2;
    //     let destination_channel = 3;
    //     let data = vector[10, 20, 30];
    //     let timeout_height = 100;
    //     let timeout_timestamp = 200;
    //     let packet =
    //         new(
    //             sequence,
    //             source_channel,
    //             destination_channel,
    //             data,
    //             timeout_height,
    //             timeout_timestamp
    //         );

    //     assert!(sequence(&packet) == sequence, 1);
    //     assert!(source_channel(&packet) == source_channel, 2);
    //     assert!(destination_channel(&packet) == destination_channel, 3);
    //     assert!(data(&packet) == data, 4);
    //     assert!(timeout_height(&packet) == timeout_height, 5);
    //     assert!(timeout_timestamp(&packet) == timeout_timestamp, 6);
    //     let data = data(&packet);
    //     assert!(data[0] == 10, 7);
    //     assert!(data[1] == 20, 8);
    //     assert!(data[2] == 30, 9);
    // }

    // #[test]
    // fun test_encode_packet() {
    //     let buf =
    //         x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000009041414242434344444141424243434444414142424343444441414242434344444141424243434444414142424343444441414242434344444141424243434444414142424343444441414242434344444141424243434444414142424343444441414242434344444141424243434444414142424343444441414242434344444141424243434444414142424343444400000000000000000000000000000000";

    //     let packet =
    //         new(
    //             100,
    //             20,
    //             b"AABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDD",
    //             10,
    //             20
    //         );

    //     assert!(encode(&packet) == buf, 1);
    // }
}