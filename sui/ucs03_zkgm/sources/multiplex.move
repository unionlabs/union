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

module zkgm::multiplex {
    use zkgm::zkgm_ethabi;

    use std::vector;

    public struct Multiplex has copy, drop, store {
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
        let mut buf = vector::empty();

        let mut sender = vector::empty();
        zkgm_ethabi::encode_bytes(&mut sender, &multiplex.sender);
        let mut contract_address = vector::empty();
        zkgm_ethabi::encode_bytes(&mut contract_address, &multiplex.contract_address);
        let mut contract_calldata = vector::empty();
        zkgm_ethabi::encode_bytes(&mut contract_calldata, &multiplex.contract_calldata);

        let mut dyn_offset = 0x20 * 4;
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
        let mut index = 0;
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
        let mut buf = vector::empty<u8>();
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
