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

module zkgm::token_order {
    use zkgm::zkgm_ethabi;

    public struct TokenOrderV2 has copy, drop, store {
        sender: vector<u8>,
        receiver: vector<u8>,
        base_token: vector<u8>,
        base_amount: u256,
        quote_token: vector<u8>,
        quote_amount: u256,
        kind: u8,
        metadata: vector<u8>,
    }

    public fun new(
        sender: vector<u8>,
        receiver: vector<u8>,
        base_token: vector<u8>,
        base_amount: u256,
        quote_token: vector<u8>,
        quote_amount: u256,
        kind: u8,
        metadata: vector<u8>,
    ): TokenOrderV2 {
        TokenOrderV2 {
            sender,
            receiver,
            base_token,
            base_amount,
            quote_token,
            quote_amount,
            kind,
            metadata,
        }
    }

    public fun sender(order: &TokenOrderV2): &vector<u8> {
        &order.sender
    }

    public fun receiver(order: &TokenOrderV2): &vector<u8> {
        &order.receiver
    }

    public fun base_token(order: &TokenOrderV2): &vector<u8> {
        &order.base_token
    }

    public fun base_amount(order: &TokenOrderV2): u256 {
        order.base_amount
    }

    public fun quote_token(order: &TokenOrderV2): &vector<u8> {
        &order.quote_token
    }

    public fun quote_amount(order: &TokenOrderV2): u256 {
        order.quote_amount
    }

    public fun kind(order: &TokenOrderV2): u8 {
        order.kind
    }

    public fun metadata(order: &TokenOrderV2): &vector<u8> {
        &order.metadata
    }

    public fun encode(order: &TokenOrderV2): vector<u8> {
        let mut buf = vector::empty();

        let mut sender = vector::empty();
        zkgm_ethabi::encode_bytes(&mut sender, &order.sender);
        let mut receiver = vector::empty();
        zkgm_ethabi::encode_bytes(&mut receiver, &order.receiver);
        let mut base_token = vector::empty();
        zkgm_ethabi::encode_bytes(&mut base_token, &order.base_token);
         let mut quote_token = vector::empty();
        zkgm_ethabi::encode_bytes(&mut quote_token, &order.quote_token);
        let mut metadata = vector::empty();
        zkgm_ethabi::encode_bytes(&mut metadata, &order.metadata);

        let mut dyn_offset = 0x20 * 8;
        // sender offset
        zkgm_ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + vector::length(&sender);
        // receiver offset
        zkgm_ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + vector::length(&receiver);
        // base_token offset
        zkgm_ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        zkgm_ethabi::encode_uint<u256>(&mut buf, order.base_amount);
        dyn_offset = dyn_offset + vector::length(&base_token);
        // quote_token offset
        zkgm_ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        zkgm_ethabi::encode_uint<u256>(&mut buf, order.quote_amount);
        dyn_offset = dyn_offset + vector::length(&quote_token);

        zkgm_ethabi::encode_uint<u8>(&mut buf, order.kind);
        // metadata offset
        zkgm_ethabi::encode_uint<u64>(&mut buf, dyn_offset);

        vector::append(&mut buf, sender);
        vector::append(&mut buf, receiver);
        vector::append(&mut buf, base_token);
        vector::append(&mut buf, quote_token);
        vector::append(&mut buf, metadata);

        buf
    }

    public fun decode(buf: &vector<u8>): TokenOrderV2 {
        let mut index = 0;
        TokenOrderV2 {
            sender: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            receiver: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            base_token: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            base_amount: zkgm_ethabi::decode_uint(buf, &mut index),
            quote_token: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            quote_amount: zkgm_ethabi::decode_uint(buf, &mut index),
            kind: zkgm_ethabi::decode_uint(buf, &mut index) as u8,
            metadata: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
        }
    }

    #[test]
    fun test_encode_decode() {
        let encoded = x"000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000000fa000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000002abcd0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003111111000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000032222220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000344444400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020dafe631cbe5cf0e5dfb68f6b5f6ed2ebaa139a6807b47c9402d7bce691ee732d";
        let packet = decode(&encoded);
        let expected_packet = TokenOrderV2 {
            sender: x"abcd",
            receiver: x"111111",
            base_token: x"222222",
            base_amount: 100,
            quote_token: x"444444",
            quote_amount: 250,
            kind: 1,
            metadata: x"000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020dafe631cbe5cf0e5dfb68f6b5f6ed2ebaa139a6807b47c9402d7bce691ee732d",
        };
        std::debug::print(&std::string::utf8(b"DECODED PACKET METADATA IS:"));
        std::debug::print(&packet);
        assert!(packet == expected_packet, 1);
        let test_encoded = encode(&packet);
        std::debug::print(&std::string::utf8(b"RE-ENCODED PACKET IS:"));
        std::debug::print(&test_encoded);
        assert!(test_encoded== encoded, 1);
    }
}

