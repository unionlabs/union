module ibc::packet {
    use std::vector;
    use ibc::ethabi;

    struct Packet has copy, store, drop, key {
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
        let buf = vector::empty();

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

    #[test]
    fun test_encode_packet() {
        let buf =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000009041414242434344444141424243434444414142424343444441414242434344444141424243434444414142424343444441414242434344444141424243434444414142424343444441414242434344444141424243434444414142424343444441414242434344444141424243434444414142424343444441414242434344444141424243434444414142424343444400000000000000000000000000000000";

        let packet =
            new(
                100,
                20,
                b"AABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDDAABBCCDD",
                10,
                20
            );

        assert!(encode(&packet) == buf, 1);
    }
}
