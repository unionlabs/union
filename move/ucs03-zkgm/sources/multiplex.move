module zkgm::multiplex {
    use zkgm::ethabi;

    use std::vector;

    struct Multiplex has copy, drop, store {
        sender: vector<u8>,
        eureka: bool,
        contract_address: vector<u8>,
        contract_calldata: vector<u8>
    }

    public fun sender(multiplex: &Multiplex): &vector<u8> {
        &multiplex.sender
    }

    public fun eureka(multiplex: &Multiplex): bool {
        multiplex.eureka
    }

    public fun contract_address(multiplex: &Multiplex): &vector<u8> {
        &multiplex.contract_address
    }

    public fun contract_calldata(multiplex: &Multiplex): &vector<u8> {
        &multiplex.contract_calldata
    }

    public fun encode(multiplex: &Multiplex): vector<u8> {
        let buf = vector::empty();

        let sender = vector::empty();
        ethabi::encode_bytes(&mut sender, &multiplex.sender);
        let contract_address = vector::empty();
        ethabi::encode_bytes(&mut contract_address, &multiplex.contract_address);
        let contract_calldata = vector::empty();
        ethabi::encode_bytes(&mut contract_calldata, &multiplex.contract_calldata);

        let dyn_offset = 0x20 * 4;
        // sender offset
        ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + vector::length(&sender);
        if (multiplex.eureka) {
            ethabi::encode_uint<u8>(&mut buf, 1);
        } else {
            ethabi::encode_uint<u8>(&mut buf, 0);
        };
        // contract address offset
        ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + vector::length(&contract_address);
        // contract calldata offset
        ethabi::encode_uint<u64>(&mut buf, dyn_offset);

        vector::append(&mut buf, sender);
        vector::append(&mut buf, contract_address);
        vector::append(&mut buf, contract_calldata);

        buf
    }

    public fun decode(buf: &vector<u8>): Multiplex {
        let index = 0;
        Multiplex {
            sender: ethabi::decode_bytes_from_offset(buf, &mut index),
            eureka: if (ethabi::decode_uint(buf, &mut index) == 0) {
                false
            } else {
                true
            },
            contract_address: ethabi::decode_bytes_from_offset(buf, &mut index),
            contract_calldata: ethabi::decode_bytes_from_offset(buf, &mut index),            
        }
    }

    #[test]
    fun test_encode_multiplex() {
        let encoded = x"00000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000045414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c4242424242424242424242424242424242424242424242424242424242424242424242424242424242424242000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000114444444444444444444444444444444444000000000000000000000000000000";
        let multiplex = decode(&encoded);
        let expected_multiplex = Multiplex {
            sender: b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            eureka: false,
            contract_address: b"BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
            contract_calldata: b"DDDDDDDDDDDDDDDDD"
        };
        assert!(multiplex == expected_multiplex, 1);
        assert!(encode(&expected_multiplex) == encoded, 1);
    }
}
