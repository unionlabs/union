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

module ibc::ethabi {
    use sui::bcs;
    use std::vector;
    use std::string::{Self, String};

    const ZERO_32_BYTES: vector<u8> = vector[
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
    ];

    public fun encode_string(buf: &mut vector<u8>, str: &String) {
        encode_bytes(buf, string::bytes(str))
    }

    public fun encode_bytes(buf: &mut vector<u8>, bytes: &vector<u8>) {
        let len = vector::length(bytes);
        let mut len_bytes = bcs::to_bytes(&(len as u256));
        vector::reverse(&mut len_bytes); // Reverse the bytes to big-endian

        vector::append(buf, len_bytes);
        vector::append(buf, *bytes);

        // Calculate padding to align to 32 bytes
        let padding_len = (32 - (len % 32)) % 32;
        let mut padding = vector::empty<u8>();
        let mut i = 0;
        while (i < padding_len) {
            vector::push_back(&mut padding, 0);
            i = i + 1;
        };
        // Append the padding
        vector::append(buf, padding);
    }

    public fun encode_address(buf: &mut vector<u8>, addr: address) {
        let sender_bytes = bcs::to_bytes(&addr);
        vector::append(buf, sender_bytes);
    }


    public fun vector_slice(buf: &vector<u8>, start: u64, end: u64): vector<u8> {
        let mut sliced = vector::empty<u8>();
        let mut i = start;
        while (i < end) {
            vector::push_back(&mut sliced, buf[i]);
            i = i + 1;
        };
        sliced
    }

    public fun encode_uint<T: copy + store + drop>(
        buf: &mut vector<u8>, data: T
    ) {
        // Create a 32-byte vector filled with zeros (u256 is 32 bytes)
        let mut padded_bytes = vector[
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
        ];

        let data_bytes = bcs::to_bytes(&data);

        let data_len = vector::length(&data_bytes);

        // Copy the data bytes into the last part of the 32-byte padded vector
        let mut i = 0;
        while (i < data_len) {
            *vector::borrow_mut(&mut padded_bytes, i) = *vector::borrow(&data_bytes, i);
            i = i + 1;
        };

        // Reverse the vector to make it big-endian
        vector::reverse(&mut padded_bytes);

        // Append the padded bytes to the output buffer
        vector::append(buf, padded_bytes);
    }

    public fun decode_uint(buf: &vector<u8>, index: &mut u64): u256 {
        // Extract the 32 bytes starting from the current index
        let padded_bytes = vector_slice(buf, *index, *index + 32);

        // Reverse the vector to little-endian format
        let mut reversed_bytes = padded_bytes;
        vector::reverse(&mut reversed_bytes);

        // Move the index forward after reading 32 bytes
        *index = *index + 32;

        // TODO: Can't convert to T directly because of the endianness we get problems.
        let result = bcs::new(reversed_bytes).peel_u256();

        result
    }

    public fun encode_u8(buf: &mut vector<u8>, data: u8) {
        let u8_data = bcs::to_bytes(&(data as u8));
        vector::append(buf, u8_data);
    }

    public fun decode_u8(buf: &vector<u8>, index: &mut u64): u8 {
        let padded_bytes = vector_slice(buf, *index, *index + 1);
        // let padded_bytes = vector::slice(buf, *index, *index + 1);

        *index = *index + 1;
        bcs::new(padded_bytes).peel_u8()
    }


    public macro fun encode_vector<$T>(
        $buf: &mut vector<u8>,
        $vec: &vector<$T>,
        $encode_fn: |&mut vector<u8>, &$T|
    ) {
        let len = vector::length($vec);
        encode_uint<u64>($buf, len);  // encode length as u64

        let mut i = 0;
        while (i < len) {
            let item = vector::borrow($vec, i);
            $encode_fn($buf, item);  // apply the encoding lambda to each item
            i = i + 1;
        };

        // Padding for 32-byte alignment
        let padding_len = (32 - (len % 32)) % 32;
        if (padding_len > 0) {
            let mut padding = vector::empty<u8>();
            let mut j = 0;
            while (j < padding_len) {
                vector::push_back(&mut padding, 0);
                j = j + 1;
            };
            vector::append($buf, padding);
        }
    }

    public macro fun decode_vector<$T>(
        $buf: &vector<u8>,
        $index: &mut u64,
        $decode_fn: |&vector<u8>, &mut u64| -> $T
    ): vector<$T> {
        let vec_len = (decode_uint($buf, $index) as u64); // Decode the length of the vector

        let mut result = vector::empty<$T>();
        let mut i = 0;
        while (i < vec_len) {
            let item = $decode_fn($buf, $index); // Call the specific decoding function
            vector::push_back(&mut result, item);
            i = i + 1;
        };

        // Calculate padding length and adjust the index to skip padding bytes
        let padding_len = (32 - (vec_len % 32)) % 32;
        *$index = *$index + padding_len; // Skip the padding bytes
        result
    }

    /// encode array of dynamic-sized data (string[], SomeDynStruct[])
    public macro fun encode_dyn_array<$T: copy>(
        $buf: &mut vector<u8>,
        $vec: &vector<$T>,
        $encode_fn: |&mut vector<u8>, &$T|
    ) {
        let mut rest_buf = vector::empty();

        let mut i = 0;
        let len = vector::length($vec);
        encode_uint($buf, len);

        while (i < len) {
            encode_uint($buf, len * 32 + vector::length(&rest_buf));
            $encode_fn(&mut rest_buf, vector::borrow($vec, i));
            i = i + 1;
        };

        vector::append($buf, rest_buf);
    }

    public fun decode_string(buf: &vector<u8>, index: &mut u64): String {
        // Read the first 32 bytes to get the length of the string
        let mut len_bytes = vector_slice(buf, *index, *index + 32);

        vector::reverse(&mut len_bytes); // Reverse the bytes to big-endian
        let str_len: u256 = bcs::new(len_bytes).peel_u256();

        *index = *index + 32; // Move the index forward after reading the length

        // // Read the actual string bytes
        let str_bytes = vector_slice(buf, *index, *index + (str_len as u64));
        *index = *index + (str_len as u64); // Move the index forward after reading the string

        // Calculate padding to skip (align to 32-byte boundary)
        let padding_len = (32 - ((str_len as u64) % 32)) % 32;
        *index = *index + padding_len; // Skip the padding bytes

        // Convert the string bytes back to a String
        string::utf8(str_bytes)
    }

    // Decoding an Ethereum address (20 bytes)
    public fun decode_address(buf: &vector<u8>, index: &mut u64): address {
        // Read the 20 bytes representing the address
        let addr_bytes = vector_slice(buf, *index, *index + 32);
        *index = *index + 32; // Move the index forward

        // Convert back to address using BCS deserialization
        bcs::new(addr_bytes).peel_address()
    }

    #[test]
    public fun test_encode_decode_string() {
        let mut some_variable: vector<u8> = vector[0x31, 0x31, 0x31, 0x31];
        let some_str = string::utf8(b"encode string encode string");

        encode_string(&mut some_variable, &some_str);

        let decoded_str = decode_string(&some_variable, &mut 4); // idx is 4, first 4 byte is garbage

        assert!(decoded_str == some_str, 1);
    }

    #[test]
    public fun test_encode_decode_address() {
        let mut some_variable: vector<u8> = vector[0x31, 0x31, 0x31, 0x31];

        let addr1 = @0x1111111111111111111111111111111111111111;
        let addr2 = @0x0000000000000000000000000000000000000033;

        encode_address(&mut some_variable, addr1);
        encode_address(&mut some_variable, addr2);

        let mut idx = 4;
        let decoded_addr1 = decode_address(&some_variable, &mut idx);
        let decoded_addr2 = decode_address(&some_variable, &mut idx);

        assert!(decoded_addr1 == addr1, 1);
        assert!(decoded_addr2 == addr2, 1);
    }

    #[test]
    public fun test_encode_decode_uint() {
        let mut some_variable: vector<u8> = vector[0x31, 0x31, 0x31, 0x31];

        let data: u8 = 4;
        let data2: u32 = 444;
        let data3: u128 = 1444223;

        encode_uint<u8>(&mut some_variable, data);
        encode_uint<u32>(&mut some_variable, data2);
        encode_uint<u128>(&mut some_variable, data3);

        let mut idx = 4;
        let decoded_data: u8 = (decode_uint(&some_variable, &mut idx) as u8);
        let decoded_data2: u32 = (decode_uint(&some_variable, &mut idx) as u32);
        let decoded_data3: u128 = (decode_uint(&some_variable, &mut idx) as u128);

        assert!(decoded_data == data, 1);
        assert!(decoded_data2 == data2, 1);
        assert!(decoded_data3 == data3, 1);
    }

    #[test]
    public fun test_encode_decode_vector() {
        let mut some_variable: vector<u8> = vector[0x31, 0x31, 0x31, 0x31];

        let vector_test_variable: vector<u8> = vector[0x41, 0x51, 0x61];

        let vector_test_variable2: vector<address> = vector[
            @0x1111111111111111111111111111111111111111,
            @0x0000000000000000000000000000000000000033
        ];

        encode_vector!<u8>(
            &mut some_variable,
            &vector_test_variable,
            |some_variable, data| {
                encode_uint<u8>(some_variable, *data);
            }
        );

        encode_vector!<address>(
            &mut some_variable,
            &vector_test_variable2,
            |some_variable, data| {
                encode_address(some_variable, *data);
            }
        );

        // Now, let's decode the vectors and verify correctness
        let mut idx: u64 = 4; // Start index (skip the first 4 bytes of garbage)

        // Decode the u8 vector
        let decoded_u8_vector =
            decode_vector!<u8>(
                &some_variable,
                &mut idx,
                |buf, index| {decode_uint(buf, index) as u8}
            );

        // Decode the address vector
        let decoded_address_vector =
            decode_vector!<address>(
                &some_variable,
                &mut idx,
                |buf, index| {decode_address(buf, index)}
            );

        assert!(*vector::borrow(&decoded_u8_vector, 0) == 0x41, 1);
        assert!(*vector::borrow(&decoded_u8_vector, 1) == 0x51, 1);
        assert!(*vector::borrow(&decoded_u8_vector, 2) == 0x61, 1);

        assert!(
            *vector::borrow(&decoded_address_vector, 0)
                == @0x1111111111111111111111111111111111111111,
            1
        );
        assert!(
            *vector::borrow(&decoded_address_vector, 1) ==
            @0x0000000000000000000000000000000000000033,
            1
        );

    }
}