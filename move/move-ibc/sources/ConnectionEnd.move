module IBC::connection_end {
    use std::string::{Self, String};
    use std::vector;
    use IBC::proto_utils;
    
    struct ConnectionEnd has copy, store, drop {
        client_id: String,
        // versions: vector<Version>,
        state: u64,
        delay_period: u64,
        counterparty: Counterparty,
    }

    struct Counterparty has copy, store, drop {
        client_id: String,
        connection_id: String,
        prefix: MerklePrefix,
    }

    struct MerklePrefix has copy, store, drop {
        key_prefix: vector<u8>,
    }

    public fun encode_proto(end: ConnectionEnd): vector<u8> {
        let buf = vector::empty();

        if (!string::is_empty(&end.client_id)) {
            vector::append(&mut buf, proto_utils::encode_string(1, end.client_id));
        };

        if (end.state != 0) {
            vector::append(&mut buf, proto_utils::encode_u64(2, end.state));
        };

        if (end.delay_period != 0) {
            vector::append(&mut buf, proto_utils::encode_u64(3, end.delay_period));
        };

        let counterparty = encode_proto_counterparty(end.counterparty);        
        if (!vector::is_empty(&counterparty)) {
            vector::append(&mut buf, proto_utils::encode_prefix(4, 2));
            vector::append(&mut buf, proto_utils::encode_varint(vector::length(&counterparty)));
            vector::append(&mut buf, counterparty);
        };
        
        buf
    }

    fun encode_proto_counterparty(value: Counterparty): vector<u8> {
        let buf = proto_utils::encode_string(1, value.client_id);
        vector::append(&mut buf, proto_utils::encode_string(2, value.connection_id));
        let merkle_prefix = encode_merkle_prefix(value.prefix);
        // nested merkle prefix tag
        vector::append(&mut buf, proto_utils::encode_prefix(3, 2));
        // nested merkle prefix total length
        vector::append(&mut buf, proto_utils::encode_varint(vector::length(&merkle_prefix)));
        // nested merkle prefix encode
        vector::append(&mut buf, merkle_prefix);
        buf
    }

    fun encode_merkle_prefix(value: MerklePrefix): vector<u8> {
        proto_utils::encode_bytes(1, value.key_prefix)
    }

    #[test]
    fun proto() {
        let encoded_s = x"0a0a636f6d6574626c732d311003186422200a0930382d7761736d2d30120c636f6e6e656374696f6e2d301a050a03010203";

        let res = encode_proto(ConnectionEnd {
            client_id: string::utf8(b"cometbls-1"),
            // versions: vector<Version>,
            state: 3,
            delay_period: 100,
            counterparty: Counterparty {
                client_id: string::utf8(b"08-wasm-0"),
                connection_id: string::utf8(b"connection-0"),
                prefix: MerklePrefix {
                    key_prefix: x"010203",
                }
            },
        });

        std::debug::print(&encoded_s);
        std::debug::print(&res);

        assert(res == encoded_s, 0);
    }
}
