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

module ibc::packet {
    use std::vector;
    use sui::object::{Self, UID};
    use ibc::ethabi;

    public struct Packet has copy, store, drop {
        source_channel_id: u32,
        destination_channel_id: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64
    }


    public fun source_channel_id(packet: &Packet): u32 {
        packet.source_channel_id
    }

    public fun destination_channel_id(packet: &Packet): u32 {
        packet.destination_channel_id
    }

    public fun data(packet: &Packet): &vector<u8> {
        &packet.data
    }

    public fun timeout_timestamp(packet: &Packet): u64 {
        packet.timeout_timestamp
    }

    public fun timeout_height(packet: &Packet): u64 {
        packet.timeout_height
    }

    public fun new(
        source_channel_id: u32,
        destination_channel_id: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64
    ): Packet {
        Packet {
            source_channel_id,
            destination_channel_id,
            data,
            timeout_height,
            timeout_timestamp
        }
    }

    public fun default(): Packet {
        new(0, 0, vector::empty(), 0, 0)
    }

    public fun encode(packet: &Packet): vector<u8> {
        let mut buf = vector::empty();

        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u32>(&mut buf, packet.source_channel_id);
        ethabi::encode_uint<u32>(&mut buf, packet.destination_channel_id);
        ethabi::encode_uint<u32>(&mut buf, 5 * 0x20);
        ethabi::encode_uint<u64>(&mut buf, packet.timeout_height);
        ethabi::encode_uint<u64>(&mut buf, packet.timeout_timestamp);
        ethabi::encode_bytes(&mut buf, &packet.data);
        buf
    }

    #[test]
    fun test_encode_packet() {
        let output = x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c8000000000000000000000000000000000000000000000000000000000000007968656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6400000000000000";
        let source_channel_id = 2;
        let destination_channel_id = 3;
        let data = b"hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world";
        let timeout_height = 100;
        let timeout_timestamp = 200;
        let packet =
            new(
                source_channel_id,
                destination_channel_id,
                data,
                timeout_height,
                timeout_timestamp
            );

        assert!(encode(&packet) == output, 1);
    } 

}