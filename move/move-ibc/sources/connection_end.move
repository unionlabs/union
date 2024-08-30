module IBC::connection_end {
    use std::string::{Self, String};
    use std::vector;
    use IBC::proto_utils;
    use std::option::{Option, Self};

    struct Version has copy, store, drop, key {
        identifier: String,
        features: vector<String>,
    }
    
    struct ConnectionEnd has copy, store, drop {
        client_id: String,
        versions: vector<Version>,
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

    public fun new(
        client_id: String,
        versions: vector<Version>,
        state: u64,
        delay_period: u64,
        counterparty: Counterparty,
    ): ConnectionEnd {
        ConnectionEnd {
            client_id,
            versions,
            state,
            delay_period,
            counterparty,
        }
    }

    public fun new_counterparty(
        client_id: String,
        connection_id: String,
        prefix: vector<u8>,
    ): Counterparty {
        Counterparty {
            client_id,
            connection_id,
            prefix: MerklePrefix {
                key_prefix: prefix,
            },
        }
    }

    public fun new_version(
        identifier: String,
        features: vector<String>,
    ): Version {
        Version {
            identifier,
            features,
        }
    }

    public fun new_versions(
        identifiers: vector<String>,
        features: vector<vector<String>>,
    ): vector<Version> {        
        let i = 0;
        let versions = vector::empty();
        while (i < vector::length(&identifiers)) {
            let ident = *vector::borrow(&identifiers, i);
            let f = *vector::borrow(&features, i);
            vector::push_back(&mut versions, new_version(ident, f));
            i = i + 1;
        };
        versions
    }

    public fun delay_period(connection_end: &ConnectionEnd): u64 {
        connection_end.delay_period
    }

    public fun versions(connection_end: &ConnectionEnd): &vector<Version> {
        &connection_end.versions
    }

    public fun state(connection_end: &ConnectionEnd): u64 {
        connection_end.state
    }

    public fun set_state(connection_end: &mut ConnectionEnd, state: u64) {
        connection_end.state = state;
    }

    public fun set_versions(connection_end: &mut ConnectionEnd, versions: vector<Version>) {
        connection_end.versions = versions;
    }

    public fun client_id(connection_end: &ConnectionEnd): &String {
        &connection_end.client_id
    }

    public fun conn_counterparty_client_id(connection_end: &ConnectionEnd): &String {
        &connection_end.counterparty.client_id
    }

    public fun conn_counterparty_connection_id(connection_end: &ConnectionEnd): &String {
        &connection_end.counterparty.connection_id
    }

    public fun set_conn_counterparty_connection_id(connection_end: &mut ConnectionEnd, connection_id: String) {
        connection_end.counterparty.connection_id = connection_id;
    }

    public fun conn_counterparty_key_prefix(connection_end: &ConnectionEnd): &vector<u8> {
        &connection_end.counterparty.prefix.key_prefix
    }

    public fun counterparty_connection_id(counterparty: &Counterparty): &String {
        &counterparty.connection_id
    }

    public fun counterparty_client_id(counterparty: &Counterparty): &String {
        &counterparty.client_id
    }

    public fun version_features(version: &Version): &vector<String> {
        &version.features
    }

    public fun version_features_mut(version: &mut Version): &mut vector<String> {
        &mut version.features
    }

    public fun version_identifier(version: &Version): &String {
        &version.identifier
    }

    public fun version_identifier_mut(version: &mut Version): &mut String {
        &mut version.identifier
    }

    public fun set_version_features(version: &mut Version, features: vector<String>) {
        version.features = features;
    } 

    public fun set_version_identifier(version: &mut Version, identifier: String) {
        version.identifier = identifier;
    }

    public fun default(): ConnectionEnd {
        ConnectionEnd {
            client_id: string::utf8(b""),
            versions: vector<Version>[],
            state: 0,
            delay_period: 0,
            counterparty: Counterparty {
                client_id: string::utf8(b""),
                connection_id: string::utf8(b""),
                prefix: MerklePrefix {
                    key_prefix: vector::empty(),
                }
            }
        }
    }

    public fun default_version(): Version {
        Version {
            identifier: string::utf8(b""),
            features: vector::empty(),
        }
    }

    fun decode_version(buf: &vector<u8>, cursor: u64, len: u64, version: &mut Version): (u64, u64) {
        let first_pos = cursor;
        while (cursor - first_pos < len) {
            let (tag, wire_type, advance, err) = proto_utils::decode_prefix(buf, cursor);
            if (err != 0) {
                return (0, err)
            };
            cursor = cursor + advance;
            let advance = if (tag == 1) {
                let (str, advance) = proto_utils::decode_string(wire_type, buf, cursor);
                if (option::is_none(&str)) {
                    return (0, 1)
                };
                version.identifier = option::extract(&mut str);
                advance
            } else if (tag == 2) {
                let (str, advance) = proto_utils::decode_string(wire_type, buf, cursor);
                if (option::is_none(&str)) {
                    return (0, 1)
                };
                vector::push_back(&mut version.features, option::extract(&mut str));
                advance
            } else {
                return (0, 1)
            };
            cursor = cursor + advance;
        };
        
        (cursor - first_pos, 0)
    }

    fun decode_merkle_prefix(buf: &vector<u8>, cursor: u64, len: u64, prefix: &mut MerklePrefix): (u64, u64) {
        let first_pos = cursor;
        while (cursor - first_pos < len) {
            let (tag, wire_type, advance, err) = proto_utils::decode_prefix(buf, cursor);
            if (err != 0) {
                return (0, err)
            };
            cursor = cursor + advance;
            let advance = if (tag == 1) {
                let (bytes, advance) = proto_utils::decode_bytes(wire_type, buf, cursor);
                if (option::is_none(&bytes)) {
                    return (0, 1)
                };
                prefix.key_prefix = option::extract(&mut bytes);
                advance
            } else {
                return (0, 1)
            };
            cursor = cursor + advance;
        };
        
        (cursor - first_pos, 0)
    }

    fun decode_counterparty(buf: &vector<u8>, cursor: u64, len: u64, counterparty: &mut Counterparty): (u64, u64) {
        let first_pos = cursor;
        while (cursor - first_pos < len) {
            let (tag, wire_type, advance, err) = proto_utils::decode_prefix(buf, cursor);
            if (err != 0) {
                return (0, err)
            };
            cursor = cursor + advance;
            let advance = if (tag == 1) {
                let (str, advance) = proto_utils::decode_string(wire_type, buf, cursor);
                if (option::is_none(&str)) {
                    return (0, 1)
                };
                counterparty.client_id = option::extract(&mut str);
                advance
            } else if (tag == 2) {
                let (str, advance) = proto_utils::decode_string(wire_type, buf, cursor);
                if (option::is_none(&str)) {
                    return (0, 1)
                };
                counterparty.connection_id = option::extract(&mut str);
                advance
                
            } else if (tag == 3) {
                let (len, advance, err) = proto_utils::decode_nested_len(wire_type, buf, cursor);
                if (err != 0) {
                    return (0, err)
                };
                cursor = cursor + advance;
                let (n_read, err) = decode_merkle_prefix(buf, cursor, len, &mut counterparty.prefix);
                if (err != 0 || n_read != len) {
                    return (0, err)
                };
                len
            } else {
                return (0, 1)
            };
            cursor = cursor + advance;
        };
        
        (cursor - first_pos, 0)
    }

    public fun decode_proto(buf: vector<u8>): Option<ConnectionEnd> {
        if (vector::is_empty(&buf)) {
            return option::none()
        };
        let cursor = 0;
        let connection_end = default();
        while (cursor < vector::length(&buf)) {
            let (tag, wire_type, advance, err) = proto_utils::decode_prefix(&buf, cursor);
            if (err != 0) {
                return option::none()
            };
            cursor = cursor + advance;
            let n_read = if (tag == 1) {
                let (str, advance) = proto_utils::decode_string(wire_type, &buf, cursor);
                if (option::is_none(&str)) {
                    return option::none()
                };
                connection_end.client_id = option::extract(&mut str);
                advance
            } else if (tag == 2) {                
                let (len, advance, err) = proto_utils::decode_nested_len(wire_type, &buf, cursor);
                if (err != 0) {
                    return option::none()
                };
                cursor = cursor + advance;
                let version = default_version();
                let (n_read, err) = decode_version(&buf, cursor, len, &mut version);
                if (err != 0 || n_read != len) {
                    return option::none()
                };
                vector::push_back(&mut connection_end.versions, version);
                len
            } else if (tag == 3) {
                let (num, advance, err) = proto_utils::decode_varint(wire_type, &buf, cursor);
                if (err != 0) {
                    return option::none()
                };
                connection_end.state = num;
                advance
            } else if (tag == 4) {
                let (num, advance, err) = proto_utils::decode_varint(wire_type, &buf, cursor);
                if (err != 0) {
                    return option::none()
                };
                connection_end.delay_period = num;
                advance
            } else if (tag == 5) {
                let (len, advance, err) = proto_utils::decode_nested_len(wire_type, &buf, cursor);
                if (err != 0) {
                    return option::none()
                };
                cursor = cursor + advance;
                let (n_read, err) = decode_counterparty(&buf, cursor, len, &mut connection_end.counterparty);
                if (err != 0 || n_read != len) {
                    return option::none()
                };
                len
            } else {
                return option::none()
            };
            cursor = cursor + n_read;
        };

        option::some(connection_end)
    }

    public fun encode_proto(end: ConnectionEnd): vector<u8> {
        let buf = vector::empty();

        if (!string::is_empty(&end.client_id)) {
            vector::append(&mut buf, proto_utils::encode_string(1, end.client_id));
        };

        if (!vector::is_empty(&end.versions)) {
            let i = 0;
            while (i < vector::length(&end.versions)) {
                let version = encode_proto_version(*vector::borrow(&end.versions, i));
                vector::append(&mut buf, proto_utils::encode_prefix(2, 2));
                vector::append(&mut buf, proto_utils::encode_varint(vector::length(&version)));
                vector::append(&mut buf, version);
                i = i + 1;
            };
        };

        if (end.state != 0) {
            vector::append(&mut buf, proto_utils::encode_u64(3, end.state));
        };

        if (end.delay_period != 0) {
            vector::append(&mut buf, proto_utils::encode_u64(4, end.delay_period));
        };

        let counterparty = encode_proto_counterparty(end.counterparty);
        if (!vector::is_empty(&counterparty)) {
            vector::append(&mut buf, proto_utils::encode_prefix(5, 2));
            vector::append(&mut buf, proto_utils::encode_varint(vector::length(&counterparty)));
            vector::append(&mut buf, counterparty);
        };
        
        buf
    }

    fun encode_proto_version(value: Version): vector<u8> {
        let buf = proto_utils::encode_string(1, value.identifier);
        let i = 0;
        while (i < vector::length(&value.features)) {
            vector::append(&mut buf, proto_utils::encode_string(2, *vector::borrow(&value.features, i)));
            i = i + 1;
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
    fun test_proto() {
        let conn_end = ConnectionEnd {
            client_id: string::utf8(b"cometbls-1"),
            versions: vector<Version>[
                Version {
                    identifier: string::utf8(b"first_version"),
                    features: vector<String>[string::utf8(b"first_1"), string::utf8(b"first_2")],
                },
                Version {
                    identifier: string::utf8(b"second_version"),
                    features: vector<String>[string::utf8(b"second_1"), string::utf8(b"second_2")],
                },
            ],
            state: 3,
            delay_period: 100,
            counterparty: Counterparty {
                client_id: string::utf8(b"08-wasm-0"),
                connection_id: string::utf8(b"connection-0"),
                prefix: MerklePrefix {
                    key_prefix: x"",
                }
            },
        };

        let res = encode_proto(conn_end);

        let conn = option::extract(&mut decode_proto(res));
        assert!(conn == conn_end,  0)
    }
}
