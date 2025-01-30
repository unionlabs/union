module zkgm::fungible_asset_order_ack {
    use zkgm::ethabi;

    use std::vector;

    struct FungibleAssetOrderAck has copy, drop, store {
        fill_type: u256,
        market_maker: vector<u8>
    }

    public fun new(fill_type: u256, market_maker: vector<u8>): FungibleAssetOrderAck {
        FungibleAssetOrderAck { fill_type, market_maker }
    }

    public fun fill_type(order: &FungibleAssetOrderAck): u256 {
        order.fill_type
    }

    public fun market_maker(order: &FungibleAssetOrderAck): &vector<u8> {
        &order.market_maker
    }

    public fun encode(ack: &FungibleAssetOrderAck): vector<u8> {
        let buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u256>(&mut buf, ack.fill_type);

        let version_offset = 0x40;
        ethabi::encode_uint<u32>(&mut buf, version_offset);

        ethabi::encode_vector<u8>(
            &mut buf,
            &ack.market_maker,
            |some_variable, data| {
                ethabi::encode_uint<u8>(some_variable, *data);
            }
        );
        buf
    }

    public fun decode(buf: &vector<u8>): FungibleAssetOrderAck {
        let index = 0x20;
        let fill_type = ethabi::decode_uint(buf, &mut index);
        index = index + 0x20;
        let market_maker =
            ethabi::decode_vector<u8>(
                buf,
                &mut index,
                |buf, index| {
                    (ethabi::decode_uint(buf, index) as u8)
                }
            );

        FungibleAssetOrderAck { fill_type: fill_type, market_maker: market_maker }
    }

    #[test]
    fun test_encode_decode_asset_transfer_ack() {
        let output =
            x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000007157f2addb00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000006f";
        let ack_data = FungibleAssetOrderAck {
            fill_type: 7788909223344,
            market_maker: b"hellloo"
        };

        let ack_bytes = encode(&ack_data);
        assert!(ack_bytes == output, 0);

        let ack_data_decoded = decode(&ack_bytes);
        assert!(ack_data_decoded.fill_type == 7788909223344, 1);
        assert!(ack_data_decoded.market_maker == b"hellloo", 3);
    }
}
