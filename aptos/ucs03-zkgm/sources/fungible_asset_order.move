// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's aptos subdirectory                      
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

    struct FungibleAssetOrder has copy, drop, store {
        sender: vector<u8>,
        receiver: vector<u8>,
        base_token: vector<u8>,
        base_amount: u256,
        base_token_symbol: String,
        base_token_name: String,
        base_token_decimals: u8,
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
        base_token_decimals: u8,
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
            base_token_decimals,
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

    public fun base_token_decimals(order: &FungibleAssetOrder): u8 {
        order.base_token_decimals
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
        let buf = vector::empty();

        let sender = vector::empty();
        zkgm_ethabi::encode_bytes(&mut sender, &order.sender);
        let receiver = vector::empty();
        zkgm_ethabi::encode_bytes(&mut receiver, &order.receiver);
        let base_token = vector::empty();
        zkgm_ethabi::encode_bytes(&mut base_token, &order.base_token);
        let base_token_symbol = vector::empty();
        zkgm_ethabi::encode_string(&mut base_token_symbol, &order.base_token_symbol);
        let base_token_name = vector::empty();
        zkgm_ethabi::encode_string(&mut base_token_name, &order.base_token_name);
        let quote_token = vector::empty();
        zkgm_ethabi::encode_bytes(&mut quote_token, &order.quote_token);

        let dyn_offset = 0x20 * 10;
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
        zkgm_ethabi::encode_uint<u8>(&mut buf, order.base_token_decimals);
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
        let index = 0;
        FungibleAssetOrder {
            sender: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            receiver: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            base_token: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            base_amount: zkgm_ethabi::decode_uint(buf, &mut index),
            base_token_symbol: zkgm_ethabi::decode_string_from_offset(buf, &mut index),
            base_token_name: zkgm_ethabi::decode_string_from_offset(buf, &mut index),
            base_token_decimals: (zkgm_ethabi::decode_uint(buf, &mut index) as u8),
            base_token_path: zkgm_ethabi::decode_uint(buf, &mut index),
            quote_token: zkgm_ethabi::decode_bytes_from_offset(buf, &mut index),
            quote_amount: zkgm_ethabi::decode_uint(buf, &mut index)
        }
    }

    #[test]
    fun test_encode_fungible_asset_order() {
        use std::string;

        let encoded =
            x"000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000000c800000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c000000000000000000000000000000000000000000000000000000000000000c80000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000204d8a66ece11f6352224942bd1dabc456b4bb5316124f02b9a7b6292ad61f77770000000000000000000000000000000000000000000000000000000000000040756e696f6e31677968347464377639366d7563723465616b7364326d7367306a76306d636e396135796a38357678356c376874793374753970737178736a79320000000000000000000000000000000000000000000000000000000000000004414e414d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000963616e696d616e616d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020feec3232793d275bfc623cc14f7306904b080746752fefce94e87dfe0bcf4962";
        let packet = decode(&encoded);
        let expected_packet = FungibleAssetOrder {
            sender: x"756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
            receiver: x"4d8a66ece11f6352224942bd1dabc456b4bb5316124f02b9a7b6292ad61f7777",
            base_token: x"756e696f6e31677968347464377639366d7563723465616b7364326d7367306a76306d636e396135796a38357678356c376874793374753970737178736a7932",
            base_amount: 200,
            base_token_symbol: string::utf8(b"ANAM"),
            base_token_name: string::utf8(b"canimanam"),
            base_token_path: 0,
            quote_token: x"feec3232793d275bfc623cc14f7306904b080746752fefce94e87dfe0bcf4962",
            quote_amount: 200,
            decimals: 8
        };

        assert!(packet == expected_packet, 1);
        assert!(encode(&packet) == encoded, 1);
    }
}
