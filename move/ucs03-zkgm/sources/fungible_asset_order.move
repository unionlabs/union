module zkgm::fungible_asset_order {
    use zkgm::ethabi;

    use std::string::{Self, String};
    use std::vector;

    struct FungibleAssetOrder has copy, drop, store {
        sender: vector<u8>,
        receiver: vector<u8>,
        base_token: vector<u8>,
        base_amount: u256,
        base_token_symbol: String,
        base_token_name: String,
        base_token_path: u256,
        quote_token: vector<u8>,
        quote_amount: u256,
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
        quote_amount: u256,
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
            quote_amount,
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
        let buf = vector::empty();

        let sender = vector::empty();
        ethabi::encode_bytes(&mut sender, &order.sender);
        let receiver = vector::empty();
        ethabi::encode_bytes(&mut receiver, &order.receiver);
        let base_token = vector::empty();
        ethabi::encode_bytes(&mut base_token, &order.base_token);
        let base_token_symbol = vector::empty();
        ethabi::encode_string(&mut base_token_symbol, &order.base_token_symbol);
        let base_token_name = vector::empty();
        ethabi::encode_string(&mut base_token_name, &order.base_token_name);
        let quote_token = vector::empty();
        ethabi::encode_bytes(&mut quote_token, &order.quote_token);

        let dyn_offset = 0x20 * 9;
        // sender offset
        ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + vector::length(&sender);
        // receiver offset
        ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + vector::length(&receiver);
        // base_token offset
        ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        ethabi::encode_uint<u256>(&mut buf, order.base_amount);
        dyn_offset = dyn_offset + vector::length(&base_token);
        // base_token_symbol offset
        ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + vector::length(&base_token_symbol);
        // base_token_name offset
        ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        dyn_offset = dyn_offset + vector::length(&base_token_name);
        ethabi::encode_uint<u256>(&mut buf, order.base_token_path);
        // quote_token offset
        ethabi::encode_uint<u64>(&mut buf, dyn_offset);
        ethabi::encode_uint<u256>(&mut buf, order.quote_amount);

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
            sender: ethabi::decode_bytes_from_offset(buf, &mut index),
            receiver: ethabi::decode_bytes_from_offset(buf, &mut index),
            base_token: ethabi::decode_bytes_from_offset(buf, &mut index),
            base_amount: ethabi::decode_uint(buf, &mut index),
            base_token_symbol: ethabi::decode_string_from_offset(buf, &mut index),
            base_token_name: ethabi::decode_string_from_offset(buf, &mut index),
            base_token_path: ethabi::decode_uint(buf, &mut index),
            quote_token: ethabi::decode_bytes_from_offset(buf, &mut index),
            quote_amount: ethabi::decode_uint(buf, &mut index)
        }
    }

    #[test]
    fun test_encode_fungible_asset_order() {
        let encoded = x"0000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000000fa000000000000000000000000000000000000000000000000000000000000000441414141000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004424242420000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000044343434300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000568656c6c6f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005776f726c6400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000634444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444440000000000000000000000000000000000000000000000000000000000";
        let packet = decode(&encoded);
        let expected_packet = FungibleAssetOrder {
            sender: b"AAAA",
            receiver: b"BBBB",
            base_token: b"CCCC",
            base_amount: 100,
            base_token_symbol: string::utf8(b"hello"),
            base_token_name: string::utf8(b"world"),
            base_token_path: 0,
            quote_token: b"DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD",
            quote_amount: 250
        };

        assert!(packet == expected_packet, 1);
        assert!(encode(&packet) == encoded, 1);
    }
}
