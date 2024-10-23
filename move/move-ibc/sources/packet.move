module ibc::packet {
    use std::vector;

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

    public fun commit_packet(_packet: &Packet): vector<u8> {
        // TODO: Implmenet this - abi.encode(packet)
        vector::empty()
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

    // public fun commitment(packet: &Packet): vector<u8> {
    //     commitment_from_parts(
    //         packet.timeout_timestamp, packet.timeout_height, packet.data
    //     )
    // }

    // public fun commitment_from_parts(
    //     timeout_timestamp: u64, timeout_height: Height, data: vector<u8>
    // ): vector<u8> {
    //     let buf = bcs::to_bytes(&timeout_timestamp);
    //     // bcs encodes little endian by default but we want big endian
    //     vector::reverse(&mut buf);

    //     let rev_num = bcs::to_bytes(&height::get_revision_number(&timeout_height));
    //     vector::reverse(&mut rev_num);
    //     vector::append(&mut buf, rev_num);

    //     let rev_height = bcs::to_bytes(&height::get_revision_height(&timeout_height));
    //     vector::reverse(&mut rev_height);
    //     vector::append(&mut buf, rev_height);

    //     vector::append(&mut buf, hash::sha2_256(data));

    //     hash::sha2_256(buf)
    // }

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
}
