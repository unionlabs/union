module ibc::packet {
    use std::vector;
    use ibc::ethabi;

    const COMMITMENT_MAGIC: vector<u8> = x"0100000000000000000000000000000000000000000000000000000000000000";
    const COMMITMENT_NULL: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";
    struct Packet has copy, store, drop, key {
        sequence: u64,
        source_channel: u32,
        destination_channel: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64
    }

    public fun sequence(packet: &Packet): u64 {
        packet.sequence
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

    public fun commit_packet(packet: &Packet): vector<u8> {
        encode(packet)
    }

    public fun commit_packets(packets: &vector<Packet>): vector<u8> {
        let buf = vector::empty();
        let rest_buf = vector::empty();

        let i = 0;
        let packets_len = vector::length(packets);
        ethabi::encode_uint(&mut buf, packets_len);
        while (i < packets_len) {
            ethabi::encode_uint(
                &mut buf,
                packets_len * 32 + vector::length(&rest_buf)
            );
            vector::append(&mut rest_buf, encode(vector::borrow(packets, i)));
            i = i + 1;
        };

        vector::append(&mut buf, rest_buf);

        buf
    }

    public fun commit_acks(_acks: vector<vector<u8>>): vector<u8> {
        // TODO: Implement this - merge_ack(abi.encode(acks))
        vector::empty()
    }

    public fun merge_ack(_ack: vector<u8>): vector<u8> {
        // TODO: Implement this
        // COMMITMENT_MAGIC | (ack & x"00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")
        vector::empty()
    }

    public fun commit_ack_memory(_ack: vector<u8>): vector<u8> {
        // TODO: Implement this - merge_ack(abi.encode(acks))
        vector::empty()
    }

    public fun new(
        sequence: u64,
        source_channel: u32,
        destination_channel: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64
    ): Packet {
        Packet {
            sequence,
            source_channel,
            destination_channel,
            data,
            timeout_height,
            timeout_timestamp
        }
    }

    public fun default(): Packet {
        new(0, 0, 0, vector::empty(), 0, 0)
    }

    public fun encode(packet: &Packet): vector<u8> {
        let buf = vector::empty();

        ethabi::encode_uint(&mut buf, packet.sequence);
        ethabi::encode_uint(&mut buf, packet.source_channel);
        ethabi::encode_uint(&mut buf, packet.destination_channel);
        // offset of `data`
        ethabi::encode_uint(&mut buf, 6 * 32);
        ethabi::encode_uint(&mut buf, packet.timeout_height);
        ethabi::encode_uint(&mut buf, packet.timeout_timestamp);
        ethabi::encode_bytes(&mut buf, &packet.data);

        buf
    }

    #[test]
    fun test_encode_packet() {
        let buf =
            x"00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c800000000000000000000000000000000000000000000000000000000000000030102030000000000000000000000000000000000000000000000000000000000";

        let packet = new(1, 2, 3, x"010203", 100, 200);

        // std::debug::print(&encode(&packet)); // == buf, 1);
        assert!(encode(&packet) == buf, 1);
    }

    #[test]
    fun test_encode_packets() {
        let buf =
            x"000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c80000000000000000000000000000000000000000000000000000000000000003010203000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c80000000000000000000000000000000000000000000000000000000000000003010203000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c800000000000000000000000000000000000000000000000000000000000000030102030000000000000000000000000000000000000000000000000000000000";

        let packet = new(1, 2, 3, x"010203", 100, 200);
        let packets = vector::empty();
        vector::push_back(&mut packets, packet);
        vector::push_back(&mut packets, packet);
        vector::push_back(&mut packets, packet);

        assert!(commit_packets(&packets) == buf, 1);
    }
}
