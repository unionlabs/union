module zkgm::multiplex {
    use zkgm::zkgm_ethabi;

    use std::vector;

    struct Multiplex has copy, drop, store {
        sender: vector<u8>,
        eureka: bool,
        contract_address: vector<u8>,
        contract_calldata: vector<u8>
    }

    public fun new(
        sender: vector<u8>,
        eureka: bool,
        contract_address: vector<u8>,
        contract_calldata: vector<u8>
    ): Multiplex {
        Multiplex { sender, eureka, contract_address, contract_calldata }
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
        zkgm_ethabi::encode_bytes(&mut sender, &multiplex.sender);
        let contract_address = vector::empty();
        zkgm_ethabi::encode_bytes(&mut contract_address, &multiplex.contract_address);
        let contract_calldata = vector::empty();
        zkgm_ethabi::encode_bytes(&mut contract_calldata, &multiplex.contract_calldata);

        let dyn_offset = 0x20 * 4;
        // sender offset
        zkgm_ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + vector::length(&sender);
        if (multiplex.eureka) {
            zkgm_ethabi::encode_uint<u8>(&mut buf, 1);
        } else {
            zkgm_ethabi::encode_uint<u8>(&mut buf, 0);
        };
        // contract address offset
        zkgm_ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + vector::length(&contract_address);
        // contract calldata offset
        zkgm_ethabi::encode_uint<u64>(&mut buf, dyn_offset);

        vector::append(&mut buf, sender);
        vector::append(&mut buf, contract_address);
        vector::append(&mut buf, contract_calldata);

        buf
    }

    public fun decode(buf: &vector<u8>): Multiplex {
        let index = 0;
        Multiplex {
            sender: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            eureka: if (zkgm_ethabi::decode_uint(buf, &mut index) == 0) { false }
            else { true },
            contract_address: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            contract_calldata: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index)
        }
    }

    public fun encode_multiplex_sender_and_calldata(
        sender: vector<u8>, contract_calldata: vector<u8>
    ): vector<u8> {
        let buf = vector::empty<u8>();
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x20);
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x40);
        let length_of_first = vector::length(&sender);
        zkgm_ethabi::encode_uint<u64>(&mut buf, ((length_of_first / 32) * 0x20) + 0x80);
        zkgm_ethabi::encode_bytes(&mut buf, &sender);
        zkgm_ethabi::encode_bytes(&mut buf, &contract_calldata);
        buf
    }

    #[test]
    fun test_encode_multiplex() {
        let encoded =
            x"00000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000045414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141414141000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c4242424242424242424242424242424242424242424242424242424242424242424242424242424242424242000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000114444444444444444444444444444444444000000000000000000000000000000";
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

    #[test]
    fun test_encode_multiplex_sender_and_calldata() {
        let output =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000000d23078354233384461366137303163353638353435644366634230334663423837356635366265646443343078354233384461366137303163353638353435644366634230334663423837356635366265646443343078354233384461366137303163353638353435644366634230334663423837356635366265646443343078354233384461366137303163353638353435644366634230334663423837356635366265646443343078354233384461366137303163353638353435644366634230334663423837356635366265646443340000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010e4578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c44617461000000000000000000000000000000000000";

        let ack_bytes =
            encode_multiplex_sender_and_calldata(
                b"0x5B38Da6a701c568545dCfcB03FcB875f56beddC40x5B38Da6a701c568545dCfcB03FcB875f56beddC40x5B38Da6a701c568545dCfcB03FcB875f56beddC40x5B38Da6a701c568545dCfcB03FcB875f56beddC40x5B38Da6a701c568545dCfcB03FcB875f56beddC4",
                b"ExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallData"
            );
        assert!(ack_bytes == output, 0);
    }
}
