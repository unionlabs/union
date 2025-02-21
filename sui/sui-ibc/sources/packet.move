module ibc::packet {
    use std::vector;
    use sui::object::{Self, UID};
    use ibc::ethabi;

    public struct Packet has copy, store, drop {
        source_channel_id: u32,
        destination_channel_id: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64
    }


    public fun source_channel_id(packet: &Packet): u32 {
        packet.source_channel_id
    }

    public fun destination_channel_id(packet: &Packet): u32 {
        packet.destination_channel_id
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
        source_channel_id: u32,
        destination_channel_id: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64
    ): Packet {
        Packet {
            source_channel_id,
            destination_channel_id,
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

        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u32>(&mut buf, packet.source_channel_id);
        ethabi::encode_uint<u32>(&mut buf, packet.destination_channel_id);
        ethabi::encode_uint<u32>(&mut buf, 5 * 0x20);
        ethabi::encode_uint<u64>(&mut buf, packet.timeout_height);
        ethabi::encode_uint<u64>(&mut buf, packet.timeout_timestamp);
        ethabi::encode_bytes(&mut buf, &packet.data);
        buf
    }

    #[test]
    fun test_encode_packet() {
        let output = x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c8000000000000000000000000000000000000000000000000000000000000007968656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6400000000000000";
        let source_channel_id = 2;
        let destination_channel_id = 3;
        let data = b"hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world";
        let timeout_height = 100;
        let timeout_timestamp = 200;
        let packet =
            new(
                source_channel_id,
                destination_channel_id,
                data,
                timeout_height,
                timeout_timestamp
            );

        assert!(encode(&packet) == output, 1);
    } 

}