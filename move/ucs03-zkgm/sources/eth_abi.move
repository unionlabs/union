module zkgm::ethabi {
    use std::bcs;
    use std::vector;
    use std::string::{Self, String};
    use std::from_bcs;
    const ZERO_32_BYTES: vector<u8> = vector[
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
    ];

    public fun encode_string(buf: &mut vector<u8>, str: &String) {
        encode_bytes(buf, string::bytes(str))
    }

    public fun encode_bytes(buf: &mut vector<u8>, bytes: &vector<u8>) {
        let len = vector::length(bytes);
        let len_bytes = bcs::to_bytes(&(len as u256));
        vector::reverse(&mut len_bytes); // Reverse the bytes to big-endian

        vector::append(buf, len_bytes);
        vector::append(buf, *bytes);

        // Calculate padding to align to 32 bytes
        let padding_len = (32 - (len % 32)) % 32;
        let padding = vector::empty<u8>();
        let i = 0;
        while (i < padding_len) {
            vector::push_back(&mut padding, 0);
            i = i + 1;
        };
        // Append the padding
        vector::append(buf, padding);
    }

    public fun encode_bytes32(buf: &mut vector<u8>, bytes: &vector<u8>) {
        let len = vector::length(bytes);
        vector::append(buf, *bytes);
        // Calculate padding to align to 32 bytes
        let padding_len = (32 - (len % 32)) % 32;
        let padding = vector::empty<u8>();
        let i = 0;
        while (i < padding_len) {
            vector::push_back(&mut padding, 0);
            i = i + 1;
        };
        // Append the padding
        vector::append(buf, padding);
    }

    public fun decode_bytes32(buf: &vector<u8>, index: &mut u64): vector<u8> {
        // Decode the length of the bytes array
        // let len_bytes = vector::slice(buf, *index, *index + 32); // Extract the next 32 bytes for length
        // vector::reverse(&mut len_bytes); // Convert to big-endian format
        // let len: u64 = (from_bcs::to_u256(len_bytes) as u64); // Convert the length bytes to u64
        // *index = *index + 32; // Move the index forward after reading the length
        // Decode the actual bytes
        let len: u64 = 32;
        let byte_data = vector::slice(buf, *index, *index + len); // Extract the bytes of the given length
        *index = *index + len; // Move the index forward after reading the byte data

        // Skip padding to align to 32-byte boundary
        let padding_len = (32 - (len % 32)) % 32;
        *index = *index + padding_len; // Adjust the index to skip the padding

        byte_data // Return the decoded bytes
    }

    public fun decode_bytes(buf: &vector<u8>, index: &mut u64): vector<u8> {
        // Decode the length of the bytes array
        let len_bytes = vector::slice(buf, *index, *index + 32); // Extract the next 32 bytes for length
        vector::reverse(&mut len_bytes); // Convert to big-endian format
        let len: u64 = (from_bcs::to_u256(len_bytes) as u64); // Convert the length bytes to u64
        *index = *index + 32; // Move the index forward after reading the length
        // Decode the actual bytes
        let byte_data = vector::slice(buf, *index, *index + len); // Extract the bytes of the given length
        *index = *index + len; // Move the index forward after reading the byte data

        // Skip padding to align to 32-byte boundary
        let padding_len = (32 - (len % 32)) % 32;
        *index = *index + padding_len; // Adjust the index to skip the padding

        byte_data // Return the decoded bytes
    }

    public fun decode_bytes_from_offset(buf: &vector<u8>, index: &mut u64): vector<u8> {
        let i = *index;
        let offset = (decode_uint(buf, &mut i) as u64);
        *index = *index + 32;
        decode_bytes(buf, &mut offset)
    }

    public fun decode_string(buf: &vector<u8>, index: &mut u64): String {
        string::utf8(decode_bytes(buf, index))
    }

    public fun decode_string_from_offset(buf: &vector<u8>, index: &mut u64): String {
        let i = *index;
        let offset = (decode_uint(buf, &mut i) as u64);
        *index = *index + 32;
        decode_string(buf, &mut offset)
    }

    public fun encode_address(buf: &mut vector<u8>, addr: address) {
        let sender_bytes = bcs::to_bytes(&addr);
        vector::append(buf, sender_bytes);
    }

    public fun encode_uint<T: copy + store + drop>(
        buf: &mut vector<u8>, data: T
    ) {
        // Create a 32-byte vector filled with zeros (u256 is 32 bytes)
        let padded_bytes = vector[
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
        ];

        let data_bytes = bcs::to_bytes(&data);

        let data_len = vector::length(&data_bytes);

        // Copy the data bytes into the last part of the 32-byte padded vector
        let i = 0;
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
        let padded_bytes = vector::slice(buf, *index, *index + 32);

        // Reverse the vector to little-endian format
        let reversed_bytes = padded_bytes;
        vector::reverse(&mut reversed_bytes);

        // Move the index forward after reading 32 bytes
        *index = *index + 32;

        // TODO: Can't convert to T directly because of the endianness we get problems.
        let result = from_bcs::to_u256(reversed_bytes);

        result
    }

    public fun decode_u8(buf: &vector<u8>, index: &mut u64): u8 {
        let padded_bytes = vector::slice(buf, *index, *index + 1);

        *index = *index + 1;

        from_bcs::to_u8(padded_bytes)
    }

    public inline fun encode_vector<T: copy>(
        buf: &mut vector<u8>,
        vec: &vector<T>,
        encode_fn: |&mut vector<u8>, &T|
    ) {
        let len = vector::length(vec);
        encode_uint<u64>(buf, len);

        let i = 0;
        while (i < len) {
            let item = vector::borrow(vec, i);
            encode_fn(buf, item);
            i = i + 1;
        };

        // // Calculate padding to align to 32 bytes
        // let padding_len = (32 - (len % 32)) % 32;
        // if (padding_len > 0) {
        //     let padding = vector::empty<u8>();
        //     let j = 0;
        //     while (j < padding_len) {
        //         vector::push_back(&mut padding, 0);
        //         j = j + 1;
        //     };
        //     // Append the padding
        //     vector::append(buf, padding);
        // }
    }

    /// encode array of dynamic-sized data (string[], SomeDynStruct[])
    public inline fun encode_dyn_array<T: copy>(
        buf: &mut vector<u8>,
        vec: &vector<T>,
        encode_fn: |&mut vector<u8>, &T|
    ) {
        let rest_buf = vector::empty();

        let i = 0;
        let len = vector::length(vec);
        encode_uint(buf, len);

        while (i < len) {
            encode_uint(buf, len * 32 + vector::length(&rest_buf));
            encode_fn(&mut rest_buf, vector::borrow(vec, i));
            i = i + 1;
        };

        vector::append(buf, rest_buf);
    }

    public inline fun decode_vector<T>(
        buf: &vector<u8>,
        index: &mut u64,
        decode_fn: |&vector<u8>, &mut u64| T
    ): vector<T> {
        let vec_len = (decode_uint(buf, index) as u64); // Decode the length of the vector

        let result = vector::empty<T>();
        let i = 0;
        while (i < vec_len) {
            let item = decode_fn(buf, index); // Call the specific decoding function
            vector::push_back(&mut result, item);
            i = i + 1;
        };

        // // Calculate padding length and adjust the index to skip padding bytes
        // let padding_len = (32 - (vec_len % 32)) % 32;
        // *index = *index + padding_len; // Skip the padding bytes
        result
    }

    // Decoding an Ethereum address (20 bytes)
    public fun decode_address(buf: &vector<u8>, index: &mut u64): address {
        // Read the 20 bytes representing the address
        let addr_bytes = vector::slice(buf, *index, *index + 32);
        *index = *index + 32; // Move the index forward

        // Convert back to address using BCS deserialization
        from_bcs::to_address(addr_bytes)
    }

    #[test]
    public fun test_encode_decode_string() {
        let some_variable: vector<u8> = vector[0x31, 0x31, 0x31, 0x31];
        let some_str = string::utf8(b"encode string encode string");

        encode_string(&mut some_variable, &some_str);

        let decoded_str = decode_string(&some_variable, &mut 4); // idx is 4, first 4 byte is garbage

        assert!(decoded_str == some_str, 1);
    }

    #[test]
    public fun test_encode_decode_address() {
        let some_variable: vector<u8> = vector[0x31, 0x31, 0x31, 0x31];

        let addr1 = @0x1111111111111111111111111111111111111111;
        let addr2 = @0x0000000000000000000000000000000000000033;

        encode_address(&mut some_variable, addr1);
        encode_address(&mut some_variable, addr2);

        let idx = 4;
        let decoded_addr1 = decode_address(&some_variable, &mut idx);
        let decoded_addr2 = decode_address(&some_variable, &mut idx);

        assert!(decoded_addr1 == addr1, 1);
        assert!(decoded_addr2 == addr2, 1);
    }

    #[test]
    public fun test_encode_decode_uint() {
        let some_variable: vector<u8> = vector[0x31, 0x31, 0x31, 0x31];

        let data: u8 = 4;
        let data2: u32 = 444;
        let data3: u128 = 1444223;

        encode_uint<u8>(&mut some_variable, data);
        encode_uint<u32>(&mut some_variable, data2);
        encode_uint<u128>(&mut some_variable, data3);

        let idx = 4;
        let decoded_data: u8 = (decode_uint(&some_variable, &mut idx) as u8);
        let decoded_data2: u32 = (decode_uint(&some_variable, &mut idx) as u32);
        let decoded_data3: u128 = (decode_uint(&some_variable, &mut idx) as u128);

        assert!(decoded_data == data, 1);
        assert!(decoded_data2 == data2, 1);
        assert!(decoded_data3 == data3, 1);
    }

    #[test]
    public fun test_encode_decode_vector() {
        let some_variable: vector<u8> = vector[0x31, 0x31, 0x31, 0x31];

        let vector_test_variable: vector<u8> = vector[0x41, 0x51, 0x61];

        let vector_test_variable2: vector<address> = vector[
            @0x1111111111111111111111111111111111111111,
            @0x0000000000000000000000000000000000000033
        ];

        encode_vector<u8>(
            &mut some_variable,
            &vector_test_variable,
            |some_variable, data| {
                encode_uint<u8>(some_variable, *data);
            }
        );

        encode_vector<address>(
            &mut some_variable,
            &vector_test_variable2,
            |some_variable, data| {
                encode_address(some_variable, *data);
            }
        );

        // Now, let's decode the vectors and verify correctness
        let idx: u64 = 4; // Start index (skip the first 4 bytes of garbage)

        // Decode the u8 vector
        let decoded_u8_vector =
            decode_vector<u8>(
                &some_variable,
                &mut idx,
                |buf, index| {
                    (decode_uint(buf, index) as u8)
                }
            );

        // Decode the address vector
        let decoded_address_vector =
            decode_vector<address>(
                &some_variable,
                &mut idx,
                |buf, index| { decode_address(buf, index) }
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
