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

module ibc::channel {
    use std::option::{Self, Option};
    use std::string::{Self, String};
    use std::vector;
    use ibc::ethabi;

    const CHAN_STATE_UNINITIALIZED: u8 = 0;
    const CHAN_STATE_INIT: u8 = 1;
    const CHAN_STATE_TRYOPEN: u8 = 2;
    const CHAN_STATE_OPEN: u8 = 3;
    const CHAN_STATE_CLOSED: u8 = 4;

    const CHAN_ORDERING_NONE: u8 = 0;
    const CHAN_ORDERING_UNORDERED: u8 = 1;
    const CHAN_ORDERING_ORDERED: u8 = 2;

    const E_PACKET_VERSION_LENGTH_EXCEEDS_MAX: u64 = 1;

    public struct Channel has copy, store, drop {
        state: u8,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String
    }

    // Getters
    public fun state(channel: &Channel): u8 {
        channel.state
    }

    public fun connection_id(channel: &Channel): u32 {
        channel.connection_id
    }

    public fun counterparty_port_id(channel: &Channel): &vector<u8> {
        &channel.counterparty_port_id
    }

    public fun counterparty_channel_id(channel: &Channel): u32 {
        channel.counterparty_channel_id
    }

    public fun version(channel: &Channel): &String {
        &channel.version
    }

    // Setters
    public fun set_state(channel: &mut Channel, new_state: u8) {
        channel.state = new_state;
    }

    public fun set_counterparty_port_id(channel: &mut Channel, new_counterparty_port_id: vector<u8>) {
        channel.counterparty_port_id = new_counterparty_port_id;
    }

    public fun set_connection_id(
        channel: &mut Channel, new_connection_id: u32
    ) {
        channel.connection_id = new_connection_id;
    }

    public fun set_counterparty_channel_id(
        channel: &mut Channel, new_id: u32
    ) {
        channel.counterparty_channel_id = new_id;
    }

    public fun set_version(channel: &mut Channel, new_version: String) {
        channel.version = new_version;
    }

    // Encode and decode functions
    public fun encode(channel: &Channel): vector<u8> {
        // TODO: test this
        let mut buf = vector::empty<u8>();

        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u8>(&mut buf, channel.state);
        ethabi::encode_uint<u32>(&mut buf, channel.connection_id);
        ethabi::encode_uint<u32>(&mut buf, channel.counterparty_channel_id);
        ethabi::encode_uint<u32>(&mut buf, 5 * 0x20);

        let version_offset =
            (((vector::length(&channel.counterparty_port_id) - 1) / 0x20) as u32);
        ethabi::encode_uint<u32>(&mut buf, (7 + version_offset) * 0x20);
        ethabi::encode_bytes(&mut buf, &channel.counterparty_port_id);

        let version_length = string::length(&channel.version);
        ethabi::encode_uint<u64>(&mut buf, version_length);

        let mut i = 32 - version_length;
        vector::append(&mut buf, *string::bytes(&channel.version));
        while (i > 0) {
            vector::push_back(&mut buf, 0);
            i = i - 1;
        };

        buf
    }

    // TODO: Do we need this?
    // public fun decode(buf: vector<u8>): Option<Channel> {
    //     let mut index = 0;

    //     let state = (ethabi::decode_uint(&buf, &mut index) as u8);
    //     let ordering = (ethabi::decode_uint(&buf, &mut index) as u8);
    //     let connection_id = (ethabi::decode_uint(&buf, &mut index) as u32);
    //     let counterparty_connection_id = (ethabi::decode_uint(&buf, &mut index) as u32);

    //     let mut i = index;
    //     while (i < index + 32) {
    //         let char = *vector::borrow(&buf, i);

    //         if (char == 0) { break };

    //         i = i + 1;
    //     };

    //     let version = ethabi::vector_slice(&buf, index, i);

    //     option::some(
    //         new(
    //             state,
    //             ordering,
    //             connection_id,
    //             counterparty_connection_id,
    //             version
    //         )
    //     )
    // }

    // Constructor
    public fun new(
        state: u8,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String
    ): Channel {
        assert!(string::length(&version) <= 32, E_PACKET_VERSION_LENGTH_EXCEEDS_MAX);

        Channel {
            state,
            connection_id,
            counterparty_channel_id,
            counterparty_port_id,
            version
        }
    }

    // Default function
    public fun default(): Channel {
        new(0, 0, 0, vector::empty(), string::utf8(b""))
    }

    #[test]
    public fun test_encode_channel() {
        let buf =
            x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000044141414100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b75637330312d72656c6179000000000000000000000000000000000000000000";

        let channel = new(2, 1, 2, b"AAAA", string::utf8(b"ucs01-relay"));

        let encoded = encode(&channel);

        assert!(buf == encoded, 1);
    }
}