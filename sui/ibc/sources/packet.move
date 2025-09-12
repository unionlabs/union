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
    use ibc::ethabi;
    use sui::address;
    use sui::bcs;

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

    // you don't wanna know
    public struct PacketBcs has drop {
        source_channel_id: address,
        destination_channel_id: address,
        data_offset: address,
        timeout_height: address,
        timeout_timestamp: address,
        vec_size_offset: vector<u8>,
        data: vector<u8>,
        data_padding: vector<u8>,                 
    }

    /// This is a big hack to efficiently encode in ethabi. The struct here is formed in a way that the encoding will
    /// have the same alignment and the length as the ethabi encoding. But it still requires some modifications, so
    /// make sure to call `apply_post_encoding_adjustments` after encoding this.
    public fun to_bcs_repr(packet: &Packet): PacketBcs {
        let data_len = packet.data.length();
        let mod = data_len % 32;
        PacketBcs {
            source_channel_id: address::from_u256(packet.source_channel_id as u256),
            destination_channel_id: address::from_u256(packet.destination_channel_id as u256),
            data_offset: address::from_u256(0x20 * 5),
            // this is a nasty hack to temporarily save `packet_data`'s last element when the packet data is well aligned
            // with 32 bytes. This is needed due to the fact that when the data is aligned, there will be an extra length
            // prefix coming from `data_padding`. In that case, we pop it from the data and temporarily save it here.
            timeout_height: if (mod == 0) {
                address::from_u256((*packet.data.borrow(data_len - 1)) as u256)
            } else {
                address::from_u256(0)
            },
            timeout_timestamp: address::from_u256(packet.timeout_timestamp as u256),
            vec_size_offset: {
                let data_len = packet.data.length();

                // The bcs encoding uses ULEB encoded integers for the length prefix.
                // We support the data which has a maximum of u32::MAX length and we construct the
                // `vec_size_offset` based on how many bytes the bcs encoding will use. For example,
                // if the ULEB-encoded length is going to be 1 byte, we use 30 bytes of zeroes. This is
                // because when this data is encoded, since it's a vector, it will be length prefixed using 1 byte
                // and the rest of the empty byte(s) will be used as the length prefix of the actual data.
                // The length-prefix of the `vec_size_offset` will be removed later when `apply_post_encoding_adjustments`
                // is called.
                if (data_len <= 0x7F) {
                    // 30 bytes
                    x"000000000000000000000000000000000000000000000000000000000000"
                } else if (data_len > 0x7F && data_len <= 0x3F_FF) {
                    x"0000000000000000000000000000000000000000000000000000000000"
                } else if (data_len > 0x3F_FF && data_len <= 0x1F_FF_FF) {
                    x"00000000000000000000000000000000000000000000000000000000"
                } else if (data_len > 0x1F_FF_FF && data_len <= 0x0F_FF_FF_FF) {
                    x"000000000000000000000000000000000000000000000000000000"
                } else if (data_len > 0x0F_FF_FF_FF && data_len <= 0xFF_FF_FF_FF) {
                    x"0000000000000000000000000000000000000000000000000000"
                } else {
                    abort 1
                }
            },
            data: {
                if (packet.data.length() % 32 == 0) {
                    let mut packet_data = packet.data;
                    packet_data.pop_back();
                    packet_data
                } else {
                    packet.data
                }
            },
            data_padding: {
                let mut offset = vector::empty();
                let len = packet.data.length();
                let rem = len % 32;

                // we don't want rem == 31 case since the length prefix will already occupy a single byte
                if (rem > 0 && rem < 31) {
                    let mut i = 1;
                    while (i < (32 - rem)) {
                        offset.push_back(0);
                        i = i + 1;
                    };
                };
                offset
            }
        }
    }

    /// Applies the necessary modifications to efficiently turn the bcs encoded `PacketBcs` into
    /// ethabi encoded `Packet`.
    ///
    /// - `encoded`: bcs encoded data that contains or is `PacketBcs`
    /// - `data_len`: length of the packet data
    /// - `offset`: offset in the encoded data where this packet encoding starts (check `PacketCommitmentBcs`)
    public fun apply_post_encoding_adjustments(encoded: &mut vector<u8>, data_len: u64, offset: u64) {
        // `vec_size_offset` is a vector, hence the encoding prefixes its length. We get rid of that since we need all zeroes
        // before the actual data length prefix.
        *encoded.borrow_mut(offset + 0x20 * 5) = 0;     

        // Write the appopriate length in place and clear the extra byte that might be coming from the ULEB representation
        if (data_len <= 0xFF) {
            *encoded.borrow_mut(offset + 0xc0 - 1) = data_len as u8;
            if (data_len >= 0x7F) {
                *encoded.borrow_mut(offset + 0xc0 - 2) = 0;
            };
        } else if (data_len > 0xFF && data_len <= 0xFF_FF) {
            let data_len_encoded = bcs::to_bytes(&address::from_u256(data_len as u256));
            *encoded.borrow_mut(offset + 0xc0 - 1) = data_len_encoded[31];
            *encoded.borrow_mut(offset + 0xc0 - 2) = data_len_encoded[30];
            if (data_len > 0x3F_FF) {
                *encoded.borrow_mut(offset + 0xc0 - 3) = 0;
            };
        } else if (data_len > 0xFF_FF && data_len <= 0xFF_FF_FF) {
            let data_len_encoded = bcs::to_bytes(&address::from_u256(data_len as u256));
            *encoded.borrow_mut(offset + 0xc0 - 1) = data_len_encoded[31];
            *encoded.borrow_mut(offset + 0xc0 - 2) = data_len_encoded[30];
            *encoded.borrow_mut(offset + 0xc0 - 3) = data_len_encoded[29];
            if (data_len > 0x1F_FF_FF) {
                *encoded.borrow_mut(offset + 0xc0 - 4) = 0;
            };
        } else if (data_len > 0xFF_FF_FF && data_len <= 0xFF_FF_FF_FF) {
            let data_len_encoded = bcs::to_bytes(&address::from_u256(data_len as u256));
            *encoded.borrow_mut(offset + 0xc0 - 1) = data_len_encoded[31];
            *encoded.borrow_mut(offset + 0xc0 - 2) = data_len_encoded[30];
            *encoded.borrow_mut(offset + 0xc0 - 3) = data_len_encoded[29];
            *encoded.borrow_mut(offset + 0xc0 - 4) = data_len_encoded[28];
            if (data_len > 0x0F_FF_FF_FF) {
                *encoded.borrow_mut(offset + 0xc0 - 5) = 0;
            };
        };

        let data_offset = data_len % 32;
        if (data_offset == 0) {
            // if the data_len is 0, then it means the last byte of the data is written to `timeout_height` and the
            // last byte of the serialized data is the unnecessary length prefix (0) coming from the length prefix
            // of `data_padding`. Since it's already 0, we can just swap.
            encoded.swap(offset + 0x7f, offset + (0x20 * 6) + data_len - 1);
        } else {
            // else, the data is postfixed with the length of the `data_padding` + zeroes. And the length prefix of
            // the `data_data_offset` is at the end of the data.
            *encoded.borrow_mut(offset + (0x20 * 6) + data_len) = 0;
        };
    }

    public fun encoded_length(packet: &PacketBcs): u64 {
        let mut len = 0x20 * 6 + packet.data.length();
        let rem = len % 32;
        if (rem != 0) {
            len = len + (32 - rem);
        };
        std::debug::print(&(len % 32));
        len
    }

    // you don't wanna know
    public fun encode(packet: &Packet): vector<u8> {
        let mut encoded = bcs::to_bytes(&packet.to_bcs_repr());

        apply_post_encoding_adjustments(&mut encoded, packet.data.length(), 0);

        encoded
    }

    // public fun encode(packet: &Packet): vector<u8> {
    //     let mut buf = vector::empty();

    //     ethabi::encode_uint<u32>(&mut buf, packet.source_channel_id);
    //     ethabi::encode_uint<u32>(&mut buf, packet.destination_channel_id);
    //     ethabi::encode_uint<u32>(&mut buf, 5 * 0x20);
    //     ethabi::encode_uint<u64>(&mut buf, packet.timeout_height);
    //     ethabi::encode_uint<u64>(&mut buf, packet.timeout_timestamp);
    //     ethabi::encode_bytes(&mut buf, &packet.data);
    //     buf
    // }
    //
    #[test]
    fun test_encode_packet() {
        let output = x"0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c8000000000000000000000000000000000000000000000000000000000000007968656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6400000000000000";
        let source_channel_id = 2;
        let destination_channel_id = 3;
        let data = b"hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world";
        let timeout_height = 0;
        let timeout_timestamp = 200;
        let packet =
            new(
                source_channel_id,
                destination_channel_id,
                data,
                timeout_height,
                timeout_timestamp
            );

        let mut encoded = packet.encode();
        apply_post_encoding_adjustments(&mut encoded, data.length(), 0);

        assert!(encoded == output, 1);
    }
}
