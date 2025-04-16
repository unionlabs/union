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

module zkgm::fungible_asset_order {
    use zkgm::zkgm_ethabi;

    use std::string::String;
    use std::vector;

    public struct FungibleAssetOrder has copy, drop, store {
        sender: vector<u8>,
        receiver: vector<u8>,
        base_token: vector<u8>,
        base_amount: u256,
        base_token_symbol: String,
        base_token_name: String,
        base_token_path: u256,
        quote_token: vector<u8>,
        quote_amount: u256
    }

    public fun new(
        sender: vector<u8>,
        receiver: vector<u8>,
        base_token: vector<u8>,
        base_amount: u256,
        base_token_symbol: String,
        base_token_name: String,
        base_token_path: u256,
        quote_token: vector<u8>,
        quote_amount: u256
    ): FungibleAssetOrder {
        FungibleAssetOrder {
            sender,
            receiver,
            base_token,
            base_amount,
            base_token_symbol,
            base_token_name,
            base_token_path,
            quote_token,
            quote_amount
        }
    }

    public fun sender(order: &FungibleAssetOrder): &vector<u8> {
        &order.sender
    }

    public fun receiver(order: &FungibleAssetOrder): &vector<u8> {
        &order.receiver
    }

    public fun base_token(order: &FungibleAssetOrder): &vector<u8> {
        &order.base_token
    }

    public fun base_amount(order: &FungibleAssetOrder): u256 {
        order.base_amount
    }

    public fun base_token_symbol(order: &FungibleAssetOrder): &String {
        &order.base_token_symbol
    }

    public fun base_token_name(order: &FungibleAssetOrder): &String {
        &order.base_token_name
    }

    public fun base_token_path(order: &FungibleAssetOrder): u256 {
        order.base_token_path
    }

    public fun quote_token(order: &FungibleAssetOrder): &vector<u8> {
        &order.quote_token
    }

    public fun quote_amount(order: &FungibleAssetOrder): u256 {
        order.quote_amount
    }

    public fun encode(order: &FungibleAssetOrder): vector<u8> {
        let mut buf = vector::empty();

        let mut sender = vector::empty();
        zkgm_ethabi::encode_bytes(&mut sender, &order.sender);
        let mut receiver = vector::empty();
        zkgm_ethabi::encode_bytes(&mut receiver, &order.receiver);
        let mut base_token = vector::empty();
        zkgm_ethabi::encode_bytes(&mut base_token, &order.base_token);
        let mut base_token_symbol = vector::empty();
        zkgm_ethabi::encode_string(&mut base_token_symbol, &order.base_token_symbol);
        let mut base_token_name = vector::empty();
        zkgm_ethabi::encode_string(&mut base_token_name, &order.base_token_name);
        let mut quote_token = vector::empty();
        zkgm_ethabi::encode_bytes(&mut quote_token, &order.quote_token);

        let mut dyn_offset = 0x20 * 9;
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
        // base_token_symbol offset
        zkgm_ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + vector::length(&base_token_symbol);
        // base_token_name offset
        zkgm_ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + vector::length(&base_token_name);
        zkgm_ethabi::encode_uint<u256>(&mut buf, order.base_token_path);
        // quote_token offset
        zkgm_ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        zkgm_ethabi::encode_uint<u256>(&mut buf, order.quote_amount);

        vector::append(&mut buf, sender);
        vector::append(&mut buf, receiver);
        vector::append(&mut buf, base_token);
        vector::append(&mut buf, base_token_symbol);
        vector::append(&mut buf, base_token_name);
        vector::append(&mut buf, quote_token);

        buf
    }

    public fun decode(buf: &vector<u8>): FungibleAssetOrder {
        let mut index = 0;
        FungibleAssetOrder {
            sender: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            receiver: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            base_token: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            base_amount: zkgm_ethabi::decode_uint(buf, &mut index),
            base_token_symbol: zkgm_ethabi::decode_string_from_offset(buf, &mut index),
            base_token_name: zkgm_ethabi::decode_string_from_offset(buf, &mut index),
            base_token_path: zkgm_ethabi::decode_uint(buf, &mut index),
            quote_token: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            quote_amount: zkgm_ethabi::decode_uint(buf, &mut index)
        }
    }

//     #[test]
//     fun test_encode_fungible_asset_order() {
//         let encoded =
//             x"0000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000000fa000000000000000000000000000000000000000000000000000000000000000441414141000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004424242420000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000044343434300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000568656c6c6f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005776f726c6400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000634444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444440000000000000000000000000000000000000000000000000000000000";
//         let packet = decode(&encoded);
//         let expected_packet = FungibleAssetOrder {
//             sender: b"AAAA",
//             receiver: b"BBBB",
//             base_token: b"CCCC",
//             base_amount: 100,
//             base_token_symbol: string::utf8(b"hello"),
//             base_token_name: string::utf8(b"world"),
//             base_token_path: 0,
//             quote_token: b"DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD",
//             quote_amount: 250
//         };

//         assert!(packet == expected_packet, 1);
//         assert!(encode(&packet) == encoded, 1);
//     }
}
