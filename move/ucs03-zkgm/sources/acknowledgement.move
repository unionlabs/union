module zkgm::acknowledgement {
    use zkgm::ethabi;

    use std::vector;

    struct Acknowledgement has copy, drop, store {
        tag: u256,
        inner_ack: vector<u8>
    }

    public fun new(tag: u256, inner_ack: vector<u8>): Acknowledgement {
        Acknowledgement {
            tag,
            inner_ack
        }        
    }

    public fun tag(ack: &Acknowledgement): u256 {
        ack.tag
    }

    public fun inner_ack(ack: &Acknowledgement): &vector<u8> {
        &ack.inner_ack
    }

    public fun encode(ack: &Acknowledgement): vector<u8> {
        let buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u256>(&mut buf, ack.tag);

        let version_offset = 0x40;
        ethabi::encode_uint<u32>(&mut buf, version_offset);

        ethabi::encode_vector<u8>(
            &mut buf,
            &ack.inner_ack,
            |some_variable, data| {
                ethabi::encode_uint<u8>(some_variable, *data);
            }
        );

        buf
    }

    public fun decode(buf: &vector<u8>): Acknowledgement {
        let index = 0x20;
        let tag = ethabi::decode_uint(buf, &mut index);
        index = index + 0x20;
        let inner_ack =
            ethabi::decode_vector<u8>(
                buf,
                &mut index,
                |buf, index| {
                    (ethabi::decode_uint(buf, index) as u8)
                }
            );

        Acknowledgement { tag: tag, inner_ack: inner_ack }
    }

    #[test]
    fun test_encode_decode_ack() {
        let output =
            x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000007157f2addb00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000006f";
        let ack_data = Acknowledgement { tag: 7788909223344, inner_ack: b"hellloo" };

        let ack_bytes = encode(&ack_data);
        assert!(ack_bytes == output, 0);

        let ack_data_decoded = decode(&ack_bytes);
        assert!(ack_data_decoded.tag == 7788909223344, 1);
        assert!(ack_data_decoded.inner_ack == b"hellloo", 3);
    }

}
