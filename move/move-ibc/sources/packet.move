module ibc::packet {
    use std::string::{Self, String, utf8};
    use std::vector;
    use std::option::{Self, Option};
    use std::hash;
    use std::bcs;

    use ibc::height::{Self, Height};
    use ibc::proto_utils;
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
        // TODO: Implmenet this - abi.encode(packet)
        vector::empty()
    }

    public fun commit_acks(acks: vector<vector<u8>>): vector<u8> {
        // TODO: Implement this - merge_ack(abi.encode(acks))
        vector::empty()
    }

    public fun merge_ack(ack: vector<u8>): vector<u8> {
        // TODO: Implement this
        // COMMITMENT_MAGIC | (ack & x"00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")
        vector::empty()
    }

    public fun commit_ack_memory(ack: vector<u8>): vector<u8> {
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

    // public fun encode_proto(packet: Packet): vector<u8> {
    //     let buf = vector::empty();

    //     if (packet.sequence != 0) {
    //         vector::append(&mut buf, proto_utils::encode_u64(1, packet.sequence));
    //     };

    //     if (!string::is_empty(&packet.source_port)) {
    //         vector::append(&mut buf, proto_utils::encode_string(2, packet.source_port));
    //     };

    //     if (!string::is_empty(&packet.source_channel)) {
    //         vector::append(
    //             &mut buf, proto_utils::encode_string(3, packet.source_channel)
    //         );
    //     };

    //     if (!string::is_empty(&packet.destination_port)) {
    //         vector::append(
    //             &mut buf, proto_utils::encode_string(4, packet.destination_port)
    //         );
    //     };

    //     if (!string::is_empty(&packet.destination_channel)) {
    //         vector::append(
    //             &mut buf, proto_utils::encode_string(5, packet.destination_channel)
    //         );
    //     };

    //     if (!vector::is_empty(&packet.data)) {
    //         vector::append(&mut buf, proto_utils::encode_bytes(6, packet.data));
    //     };

    //     let height = height::encode_proto(packet.timeout_height);
    //     if (!vector::is_empty(&height)) {
    //         vector::append(&mut buf, proto_utils::encode_prefix(7, 2));
    //         vector::append(
    //             &mut buf, proto_utils::encode_varint(vector::length(&height))
    //         );
    //         vector::append(&mut buf, height);
    //     };

    //     if (packet.timeout_timestamp != 0) {
    //         vector::append(
    //             &mut buf, proto_utils::encode_u64(8, packet.timeout_timestamp)
    //         );
    //     };

    //     buf
    // }

    // public fun decode_proto(buf: vector<u8>): Option<Packet> {
    //     if (vector::is_empty(&buf)) {
    //         return option::none()
    //     };
    //     let cursor = 0;
    //     let packet = default();
    //     while (cursor < vector::length(&buf)) {
    //         let (tag, wire_type, advance, err) = proto_utils::decode_prefix(
    //             &buf, cursor
    //         );
    //         if (err != 0) {
    //             return option::none()
    //         };
    //         cursor = cursor + advance;
    //         let n_read =
    //             if (tag == 1) {
    //                 let (num, advance, err) =
    //                     proto_utils::decode_varint(wire_type, &buf, cursor);
    //                 if (err != 0) {
    //                     return option::none()
    //                 };
    //                 packet.sequence = num;
    //                 advance
    //             } else if (tag == 2) {
    //                 let (str, advance) =
    //                     proto_utils::decode_string(wire_type, &buf, cursor);
    //                 if (option::is_none(&str)) {
    //                     return option::none()
    //                 };
    //                 packet.source_port = option::extract(&mut str);
    //                 advance
    //             } else if (tag == 3) {
    //                 let (str, advance) =
    //                     proto_utils::decode_string(wire_type, &buf, cursor);
    //                 if (option::is_none(&str)) {
    //                     return option::none()
    //                 };
    //                 packet.source_channel = option::extract(&mut str);
    //                 advance
    //             } else if (tag == 4) {
    //                 let (str, advance) =
    //                     proto_utils::decode_string(wire_type, &buf, cursor);
    //                 if (option::is_none(&str)) {
    //                     return option::none()
    //                 };
    //                 packet.destination_port = option::extract(&mut str);
    //                 advance
    //             } else if (tag == 5) {
    //                 let (str, advance) =
    //                     proto_utils::decode_string(wire_type, &buf, cursor);
    //                 if (option::is_none(&str)) {
    //                     return option::none()
    //                 };
    //                 packet.destination_channel = option::extract(&mut str);
    //                 advance
    //             } else if (tag == 6) {
    //                 let (bytes, advance) =
    //                     proto_utils::decode_bytes(wire_type, &buf, cursor);
    //                 if (option::is_none(&bytes)) {
    //                     return option::none()
    //                 };
    //                 packet.data = option::extract(&mut bytes);
    //                 advance
    //             } else if (tag == 7) {
    //                 let (len, advance, err) =
    //                     proto_utils::decode_nested_len(wire_type, &buf, cursor);
    //                 if (err != 0) {
    //                     return option::none()
    //                 };
    //                 cursor = cursor + advance;
    //                 let (n_read, err) =
    //                     height::decode_proto(&buf, cursor, len, &mut packet.timeout_height);
    //                 if (err != 0 || n_read != len) {
    //                     return option::none()
    //                 };
    //                 len
    //             } else if (tag == 8) {
    //                 let (num, advance, err) =
    //                     proto_utils::decode_varint(wire_type, &buf, cursor);
    //                 if (err != 0) {
    //                     return option::none()
    //                 };
    //                 packet.timeout_timestamp = num;
    //                 advance
    //             } else {
    //                 return option::none()
    //             };
    //         cursor = cursor + n_read;
    //     };

    //     option::some(packet)
    // }

    // #[test]
    // fun test_packet_proto_complete() {
    //     let encoded_packet =
    //         x"08011206706f72742d311a096368616e6e656c2d312206706f72742d322a096368616e6e656c2d323204010203043a040801100a4064";

    //     let packet = Packet {
    //         sequence: 1,
    //         source_port: utf8(b"port-1"),
    //         source_channel: utf8(b"channel-1"),
    //         destination_port: utf8(b"port-2"),
    //         destination_channel: utf8(b"channel-2"),
    //         data: x"01020304",
    //         timeout_height: height::new(1, 10),
    //         timeout_timestamp: 100
    //     };

    //     let res = encode_proto(packet);

    //     assert!(encoded_packet == res, 0);

    //     let decoded = decode_proto(encoded_packet);

    //     assert!(option::extract(&mut decoded) == packet, 1);
    // }

    // #[test]
    // fun test_packet_proto_incomplete() {
    //     let encoded_packet =
    //         x"1206706f72742d312206706f72742d322a096368616e6e656c2d323204010203043a02100a";

    //     let packet = Packet {
    //         sequence: 0,
    //         source_port: utf8(b"port-1"),
    //         source_channel: utf8(b""),
    //         destination_port: utf8(b"port-2"),
    //         destination_channel: utf8(b"channel-2"),
    //         data: x"01020304",
    //         timeout_height: height::new(0, 10),
    //         timeout_timestamp: 0
    //     };

    //     let res = encode_proto(packet);

    //     assert!(encoded_packet == res, 0);

    //     let decoded = decode_proto(encoded_packet);

    //     assert!(option::extract(&mut decoded) == packet, 1);
    // }
}
