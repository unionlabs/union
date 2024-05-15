// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.23;

library GoogleProtobufDuration {
    //struct definition
    struct Data {
        int64 Seconds;
        int32 nanos;
    }

    // Decoder section

    /**
     * @dev The main decoder for memory
     * @param bs The bytes array to be decoded
     * @return The decoded struct
     */
    function decode(bytes memory bs) internal pure returns (Data memory) {
        (Data memory x,) = _decode(32, bs, bs.length);
        return x;
    }

    /**
     * @dev The main decoder for storage
     * @param self The in-storage struct
     * @param bs The bytes array to be decoded
     */
    function decode(Data storage self, bytes memory bs) internal {
        (Data memory x,) = _decode(32, bs, bs.length);
        store(x, self);
    }

    // inner decoder

    /**
     * @dev The decoder for internal usage
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param sz The number of bytes expected
     * @return The decoded struct
     * @return The number of bytes decoded
     */
    function _decode(
        uint256 p,
        bytes memory bs,
        uint256 sz
    ) internal pure returns (Data memory, uint256) {
        Data memory r;
        uint256[3] memory counters;
        uint256 fieldId;
        ProtoBufRuntime.WireType wireType;
        uint256 bytesRead;
        uint256 offset = p;
        uint256 pointer = p;
        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer += _read_Seconds(pointer, bs, r, counters);
            } else if (fieldId == 2) {
                pointer += _read_nanos(pointer, bs, r, counters);
            } else {
                if (wireType == ProtoBufRuntime.WireType.Fixed64) {
                    uint256 size;
                    (, size) = ProtoBufRuntime._decode_fixed64(pointer, bs);
                    pointer += size;
                }
                if (wireType == ProtoBufRuntime.WireType.Fixed32) {
                    uint256 size;
                    (, size) = ProtoBufRuntime._decode_fixed32(pointer, bs);
                    pointer += size;
                }
                if (wireType == ProtoBufRuntime.WireType.Varint) {
                    uint256 size;
                    (, size) = ProtoBufRuntime._decode_varint(pointer, bs);
                    pointer += size;
                }
                if (wireType == ProtoBufRuntime.WireType.LengthDelim) {
                    uint256 size;
                    (, size) = ProtoBufRuntime._decode_lendelim(pointer, bs);
                    pointer += size;
                }
            }
        }
        return (r, sz);
    }

    // field readers

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @param counters The counters for repeated fields
     * @return The number of bytes decoded
     */
    function _read_Seconds(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[3] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (int64 x, uint256 sz) = ProtoBufRuntime._decode_int64(p, bs);
        if (isNil(r)) {
            counters[1] += 1;
        } else {
            r.Seconds = x;
            if (counters[1] > 0) counters[1] -= 1;
        }
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @param counters The counters for repeated fields
     * @return The number of bytes decoded
     */
    function _read_nanos(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[3] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (int32 x, uint256 sz) = ProtoBufRuntime._decode_int32(p, bs);
        if (isNil(r)) {
            counters[2] += 1;
        } else {
            r.nanos = x;
            if (counters[2] > 0) counters[2] -= 1;
        }
        return sz;
    }

    // Encoder section

    /**
     * @dev The main encoder for memory
     * @param r The struct to be encoded
     * @return The encoded byte array
     */
    function encode(Data memory r) internal pure returns (bytes memory) {
        bytes memory bs = new bytes(_estimate(r));
        uint256 sz = _encode(r, 32, bs);
        assembly {
            mstore(bs, sz)
        }
        return bs;
    }

    // inner encoder

    /**
     * @dev The encoder for internal usage
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 pointer = p;

        if (r.Seconds != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_int64(r.Seconds, pointer, bs);
        }
        if (r.nanos != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_int32(r.nanos, pointer, bs);
        }
        return pointer - offset;
    }

    // nested encoder

    /**
     * @dev The encoder for inner struct
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode_nested(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        //
        // First encoded `r` into a temporary array, and encode the actual size used.
        // Then copy the temporary array into `bs`.
        //
        uint256 offset = p;
        uint256 pointer = p;
        bytes memory tmp = new bytes(_estimate(r));
        uint256 tmpAddr = ProtoBufRuntime.getMemoryAddress(tmp);
        uint256 bsAddr = ProtoBufRuntime.getMemoryAddress(bs);
        uint256 size = _encode(r, 32, tmp);
        pointer += ProtoBufRuntime._encode_varint(size, pointer, bs);
        ProtoBufRuntime.copyBytes(tmpAddr + 32, bsAddr + pointer, size);
        pointer += size;
        delete tmp;
        return pointer - offset;
    }

    // estimator

    /**
     * @dev The estimator for a struct
     * @param r The struct to be encoded
     * @return The number of bytes encoded in estimation
     */
    function _estimate(Data memory r) internal pure returns (uint256) {
        uint256 e;
        e += 1 + ProtoBufRuntime._sz_int64(r.Seconds);
        e += 1 + ProtoBufRuntime._sz_int32(r.nanos);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.Seconds != 0) {
            return false;
        }

        if (r.nanos != 0) {
            return false;
        }

        return true;
    }

    //store function
    /**
     * @dev Store in-memory struct to storage
     * @param input The in-memory struct
     * @param output The in-storage struct
     */
    function store(Data memory input, Data storage output) internal {
        output.Seconds = input.Seconds;
        output.nanos = input.nanos;
    }

    //utility functions
    /**
     * @dev Return an empty struct
     * @return r The empty struct
     */
    function nil() internal pure returns (Data memory r) {
        assembly {
            r := 0
        }
    }

    /**
     * @dev Test whether a struct is empty
     * @param x The struct to be tested
     * @return r True if it is empty
     */
    function isNil(Data memory x) internal pure returns (bool r) {
        assembly {
            r := iszero(x)
        }
    }
}

//library Duration

library GoogleProtobufTimestamp {
    //struct definition
    struct Data {
        int64 secs;
        int64 nanos;
    }

    // Decoder section

    /**
     * @dev The main decoder for memory
     * @param bs The bytes array to be decoded
     * @return The decoded struct
     */
    function decode(bytes memory bs) internal pure returns (Data memory) {
        (Data memory x,) = _decode(32, bs, bs.length);
        return x;
    }

    /**
     * @dev The main decoder for storage
     * @param self The in-storage struct
     * @param bs The bytes array to be decoded
     */
    function decode(Data storage self, bytes memory bs) internal {
        (Data memory x,) = _decode(32, bs, bs.length);
        store(x, self);
    }

    // inner decoder

    /**
     * @dev The decoder for internal usage
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param sz The number of bytes expected
     * @return The decoded struct
     * @return The number of bytes decoded
     */
    function _decode(
        uint256 p,
        bytes memory bs,
        uint256 sz
    ) internal pure returns (Data memory, uint256) {
        Data memory r;
        uint256[3] memory counters;
        uint256 fieldId;
        ProtoBufRuntime.WireType wireType;
        uint256 bytesRead;
        uint256 offset = p;
        uint256 pointer = p;
        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer += _read_secs(pointer, bs, r, counters);
            } else if (fieldId == 2) {
                pointer += _read_nanos(pointer, bs, r, counters);
            } else {
                if (wireType == ProtoBufRuntime.WireType.Fixed64) {
                    uint256 size;
                    (, size) = ProtoBufRuntime._decode_fixed64(pointer, bs);
                    pointer += size;
                }
                if (wireType == ProtoBufRuntime.WireType.Fixed32) {
                    uint256 size;
                    (, size) = ProtoBufRuntime._decode_fixed32(pointer, bs);
                    pointer += size;
                }
                if (wireType == ProtoBufRuntime.WireType.Varint) {
                    uint256 size;
                    (, size) = ProtoBufRuntime._decode_varint(pointer, bs);
                    pointer += size;
                }
                if (wireType == ProtoBufRuntime.WireType.LengthDelim) {
                    uint256 size;
                    (, size) = ProtoBufRuntime._decode_lendelim(pointer, bs);
                    pointer += size;
                }
            }
        }
        return (r, sz);
    }

    // field readers

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @param counters The counters for repeated fields
     * @return The number of bytes decoded
     */
    function _read_secs(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[3] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (int64 x, uint256 sz) = ProtoBufRuntime._decode_int64(p, bs);
        if (isNil(r)) {
            counters[1] += 1;
        } else {
            r.secs = x;
            if (counters[1] > 0) counters[1] -= 1;
        }
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @param counters The counters for repeated fields
     * @return The number of bytes decoded
     */
    function _read_nanos(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[3] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (int64 x, uint256 sz) = ProtoBufRuntime._decode_int64(p, bs);
        if (isNil(r)) {
            counters[2] += 1;
        } else {
            r.nanos = x;
            if (counters[2] > 0) counters[2] -= 1;
        }
        return sz;
    }

    // Encoder section

    /**
     * @dev The main encoder for memory
     * @param r The struct to be encoded
     * @return The encoded byte array
     */
    function encode(Data memory r) internal pure returns (bytes memory) {
        bytes memory bs = new bytes(_estimate(r));
        uint256 sz = _encode(r, 32, bs);
        assembly {
            mstore(bs, sz)
        }
        return bs;
    }

    // inner encoder

    /**
     * @dev The encoder for internal usage
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 pointer = p;

        if (r.secs != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_int64(r.secs, pointer, bs);
        }
        if (r.nanos != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_int64(r.nanos, pointer, bs);
        }
        return pointer - offset;
    }

    // nested encoder

    /**
     * @dev The encoder for inner struct
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode_nested(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        /**
         * First encoded `r` into a temporary array, and encode the actual size used.
         * Then copy the temporary array into `bs`.
         */
        uint256 offset = p;
        uint256 pointer = p;
        bytes memory tmp = new bytes(_estimate(r));
        uint256 tmpAddr = ProtoBufRuntime.getMemoryAddress(tmp);
        uint256 bsAddr = ProtoBufRuntime.getMemoryAddress(bs);
        uint256 size = _encode(r, 32, tmp);
        pointer += ProtoBufRuntime._encode_varint(size, pointer, bs);
        ProtoBufRuntime.copyBytes(tmpAddr + 32, bsAddr + pointer, size);
        pointer += size;
        delete tmp;
        return pointer - offset;
    }

    // estimator

    /**
     * @dev The estimator for a struct
     * @param r The struct to be encoded
     * @return The number of bytes encoded in estimation
     */
    function _estimate(Data memory r) internal pure returns (uint256) {
        uint256 e;
        e += 1 + ProtoBufRuntime._sz_int64(r.secs);
        e += 1 + ProtoBufRuntime._sz_int64(r.nanos);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.secs != 0) {
            return false;
        }

        if (r.nanos != 0) {
            return false;
        }

        return true;
    }

    //store function
    /**
     * @dev Store in-memory struct to storage
     * @param input The in-memory struct
     * @param output The in-storage struct
     */
    function store(Data memory input, Data storage output) internal {
        output.secs = input.secs;
        output.nanos = input.nanos;
    }

    //utility functions
    /**
     * @dev Return an empty struct
     * @return r The empty struct
     */
    function nil() internal pure returns (Data memory r) {
        assembly {
            r := 0
        }
    }

    /**
     * @dev Test whether a struct is empty
     * @param x The struct to be tested
     * @return r True if it is empty
     */
    function isNil(Data memory x) internal pure returns (bool r) {
        assembly {
            r := iszero(x)
        }
    }
}

//library Timestamp

/**
 * @title Runtime library for ProtoBuf serialization and/or deserialization.
 * All ProtoBuf generated code will use this library.
 */
library ProtoBufRuntime {
    // Types defined in ProtoBuf
    enum WireType {
        Varint,
        Fixed64,
        LengthDelim,
        StartGroup,
        EndGroup,
        Fixed32
    }
    // Constants for bytes calculation

    uint256 constant WORD_LENGTH = 32;
    uint256 constant HEADER_SIZE_LENGTH_IN_BYTES = 4;
    uint256 constant BYTE_SIZE = 8;
    uint256 constant REMAINING_LENGTH =
        WORD_LENGTH - HEADER_SIZE_LENGTH_IN_BYTES;
    string constant OVERFLOW_MESSAGE = "length overflow";

    //Storages
    /**
     * @dev Encode to storage location using assembly to save storage space.
     * @param location The location of storage
     * @param encoded The encoded ProtoBuf bytes
     */
    function encodeStorage(
        bytes storage location,
        bytes memory encoded
    ) internal {
        /**
         * This code use the first four bytes as size,
         * and then put the rest of `encoded` bytes.
         */
        uint256 length = encoded.length;
        uint256 firstWord;
        uint256 wordLength = WORD_LENGTH;
        uint256 remainingLength = REMAINING_LENGTH;

        assembly {
            firstWord := mload(add(encoded, wordLength))
        }
        firstWord = (firstWord >> (BYTE_SIZE * HEADER_SIZE_LENGTH_IN_BYTES))
            | (length << (BYTE_SIZE * REMAINING_LENGTH));

        assembly {
            sstore(location.slot, firstWord)
        }

        if (length > REMAINING_LENGTH) {
            length -= REMAINING_LENGTH;
            for (uint256 i; i < ceil(length, WORD_LENGTH); i++) {
                assembly {
                    let offset := add(mul(i, wordLength), remainingLength)
                    let slotIndex := add(i, 1)
                    sstore(
                        add(location.slot, slotIndex),
                        mload(add(add(encoded, wordLength), offset))
                    )
                }
            }
        }
    }

    /**
     * @dev Decode storage location using assembly using the format in `encodeStorage`.
     * @param location The location of storage
     * @return The encoded bytes
     */
    function decodeStorage(bytes storage location)
        internal
        view
        returns (bytes memory)
    {
        /**
         * This code is to decode the first four bytes as size,
         * and then decode the rest using the decoded size.
         */
        uint256 firstWord;
        uint256 remainingLength = REMAINING_LENGTH;
        uint256 wordLength = WORD_LENGTH;

        assembly {
            firstWord := sload(location.slot)
        }

        uint256 length = firstWord >> (BYTE_SIZE * REMAINING_LENGTH);
        bytes memory encoded = new bytes(length);

        assembly {
            mstore(add(encoded, remainingLength), firstWord)
        }

        if (length > REMAINING_LENGTH) {
            length -= REMAINING_LENGTH;
            for (uint256 i; i < ceil(length, WORD_LENGTH); i++) {
                assembly {
                    let offset := add(mul(i, wordLength), remainingLength)
                    let slotIndex := add(i, 1)
                    mstore(
                        add(add(encoded, wordLength), offset),
                        sload(add(location.slot, slotIndex))
                    )
                }
            }
        }
        return encoded;
    }

    /**
     * @dev Fast memory copy of bytes using assembly.
     * @param src The source memory address
     * @param dest The destination memory address
     * @param len The length of bytes to copy
     */
    function copyBytes(uint256 src, uint256 dest, uint256 len) internal pure {
        if (len == 0) {
            return;
        }

        // Copy word-length chunks while possible
        for (; len > WORD_LENGTH; len -= WORD_LENGTH) {
            assembly {
                mstore(dest, mload(src))
            }
            dest += WORD_LENGTH;
            src += WORD_LENGTH;
        }

        // Copy remaining bytes
        uint256 mask = 256 ** (WORD_LENGTH - len) - 1;
        assembly {
            let srcpart := and(mload(src), not(mask))
            let destpart := and(mload(dest), mask)
            mstore(dest, or(destpart, srcpart))
        }
    }

    /**
     * @dev Use assembly to get memory address.
     * @param r The in-memory bytes array
     * @return The memory address of `r`
     */
    function getMemoryAddress(bytes memory r) internal pure returns (uint256) {
        uint256 addr;
        assembly {
            addr := r
        }
        return addr;
    }

    /**
     * @dev Implement Math function of ceil
     * @param a The denominator
     * @param m The numerator
     * @return r The result of ceil(a/m)
     */
    function ceil(uint256 a, uint256 m) internal pure returns (uint256 r) {
        return (a + m - 1) / m;
    }

    // Decoders
    /**
     * This section of code `_decode_(u)int(32|64)`, `_decode_enum` and `_decode_bool`
     * is to decode ProtoBuf native integers,
     * using the `varint` encoding.
     */

    /**
     * @dev Decode integers
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded integer
     * @return The length of `bs` used to get decoded
     */
    function _decode_uint32(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint32, uint256) {
        (uint256 varint, uint256 sz) = _decode_varint(p, bs);
        return (uint32(varint), sz);
    }

    /**
     * @dev Decode integers
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded integer
     * @return The length of `bs` used to get decoded
     */
    function _decode_uint64(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint64, uint256) {
        (uint256 varint, uint256 sz) = _decode_varint(p, bs);
        return (uint64(varint), sz);
    }

    /**
     * @dev Decode integers
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded integer
     * @return The length of `bs` used to get decoded
     */
    function _decode_int32(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int32, uint256) {
        (uint256 varint, uint256 sz) = _decode_varint(p, bs);
        int32 r;
        assembly {
            r := varint
        }
        return (r, sz);
    }

    /**
     * @dev Decode integers
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded integer
     * @return The length of `bs` used to get decoded
     */
    function _decode_int64(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int64, uint256) {
        (uint256 varint, uint256 sz) = _decode_varint(p, bs);
        int64 r;
        assembly {
            r := varint
        }
        return (r, sz);
    }

    /**
     * @dev Decode enum
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded enum's integer
     * @return The length of `bs` used to get decoded
     */
    function _decode_enum(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int64, uint256) {
        return _decode_int64(p, bs);
    }

    /**
     * @dev Decode enum
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded boolean
     * @return The length of `bs` used to get decoded
     */
    function _decode_bool(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bool, uint256) {
        (uint256 varint, uint256 sz) = _decode_varint(p, bs);
        if (varint == 0) {
            return (false, sz);
        }
        return (true, sz);
    }

    /**
     * This section of code `_decode_sint(32|64)`
     * is to decode ProtoBuf native signed integers,
     * using the `zig-zag` encoding.
     */

    /**
     * @dev Decode signed integers
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded integer
     * @return The length of `bs` used to get decoded
     */
    function _decode_sint32(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int32, uint256) {
        (int256 varint, uint256 sz) = _decode_varints(p, bs);
        return (int32(varint), sz);
    }

    /**
     * @dev Decode signed integers
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded integer
     * @return The length of `bs` used to get decoded
     */
    function _decode_sint64(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int64, uint256) {
        (int256 varint, uint256 sz) = _decode_varints(p, bs);
        return (int64(varint), sz);
    }

    /**
     * @dev Decode string
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded string
     * @return The length of `bs` used to get decoded
     */
    function _decode_string(
        uint256 p,
        bytes memory bs
    ) internal pure returns (string memory, uint256) {
        (bytes memory x, uint256 sz) = _decode_lendelim(p, bs);
        return (string(x), sz);
    }

    /**
     * @dev Decode bytes array
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded bytes array
     * @return The length of `bs` used to get decoded
     */
    function _decode_bytes(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes memory, uint256) {
        return _decode_lendelim(p, bs);
    }

    /**
     * @dev Decode ProtoBuf key
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded field ID
     * @return The decoded WireType specified in ProtoBuf
     * @return The length of `bs` used to get decoded
     */
    function _decode_key(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256, WireType, uint256) {
        (uint256 x, uint256 n) = _decode_varint(p, bs);
        WireType typeId = WireType(x & 7);
        uint256 fieldId = x / 8;
        return (fieldId, typeId, n);
    }

    /**
     * @dev Decode ProtoBuf varint
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded unsigned integer
     * @return The length of `bs` used to get decoded
     */
    function _decode_varint(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256, uint256) {
        /**
         * Read a byte.
         * Use the lower 7 bits and shift it to the left,
         * until the most significant bit is 0.
         * Refer to https://developers.google.com/protocol-buffers/docs/encoding
         */
        uint256 x = 0;
        uint256 sz = 0;
        uint256 length = bs.length + WORD_LENGTH;
        assembly {
            let b := 0x80
            p := add(bs, p)
            for {} eq(0x80, and(b, 0x80)) {} {
                if eq(lt(sub(p, bs), length), 0) {
                    mstore(
                        0,
                        0x08c379a000000000000000000000000000000000000000000000000000000000
                    ) //error function selector
                    mstore(4, 32)
                    mstore(36, 15)
                    mstore(
                        68,
                        0x6c656e677468206f766572666c6f770000000000000000000000000000000000
                    ) // length overflow in hex
                    revert(0, 83)
                }
                let tmp := mload(p)
                let pos := 0
                for {} and(eq(0x80, and(b, 0x80)), lt(pos, 32)) {} {
                    if eq(lt(sub(p, bs), length), 0) {
                        mstore(
                            0,
                            0x08c379a000000000000000000000000000000000000000000000000000000000
                        ) //error function selector
                        mstore(4, 32)
                        mstore(36, 15)
                        mstore(
                            68,
                            0x6c656e677468206f766572666c6f770000000000000000000000000000000000
                        ) // length overflow in hex
                        revert(0, 83)
                    }
                    b := byte(pos, tmp)
                    x := or(x, shl(mul(7, sz), and(0x7f, b)))
                    sz := add(sz, 1)
                    pos := add(pos, 1)
                    p := add(p, 0x01)
                }
            }
        }
        return (x, sz);
    }

    /**
     * @dev Decode ProtoBuf zig-zag encoding
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded signed integer
     * @return The length of `bs` used to get decoded
     */
    function _decode_varints(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int256, uint256) {
        /**
         * Refer to https://developers.google.com/protocol-buffers/docs/encoding
         */
        (uint256 u, uint256 sz) = _decode_varint(p, bs);
        int256 s;
        assembly {
            s := xor(shr(1, u), add(not(and(u, 1)), 1))
        }
        return (s, sz);
    }

    /**
     * @dev Decode ProtoBuf fixed-length encoding
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded unsigned integer
     * @return The length of `bs` used to get decoded
     */
    function _decode_uintf(
        uint256 p,
        bytes memory bs,
        uint256 sz
    ) internal pure returns (uint256, uint256) {
        /**
         * Refer to https://developers.google.com/protocol-buffers/docs/encoding
         */
        uint256 x = 0;
        uint256 length = bs.length + WORD_LENGTH;
        assert(p + sz <= length);
        assembly {
            let i := 0
            p := add(bs, p)
            let tmp := mload(p)
            for {} lt(i, sz) {} {
                x := or(x, shl(mul(8, i), byte(i, tmp)))
                p := add(p, 0x01)
                i := add(i, 1)
            }
        }
        return (x, sz);
    }

    /**
     * `_decode_(s)fixed(32|64)` is the concrete implementation of `_decode_uintf`
     */
    function _decode_fixed32(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint32, uint256) {
        (uint256 x, uint256 sz) = _decode_uintf(p, bs, 4);
        return (uint32(x), sz);
    }

    function _decode_fixed64(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint64, uint256) {
        (uint256 x, uint256 sz) = _decode_uintf(p, bs, 8);
        return (uint64(x), sz);
    }

    function _decode_sfixed32(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int32, uint256) {
        (uint256 x, uint256 sz) = _decode_uintf(p, bs, 4);
        int256 r;
        assembly {
            r := x
        }
        return (int32(r), sz);
    }

    function _decode_sfixed64(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int64, uint256) {
        (uint256 x, uint256 sz) = _decode_uintf(p, bs, 8);
        int256 r;
        assembly {
            r := x
        }
        return (int64(r), sz);
    }

    /**
     * @dev Decode bytes array
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The decoded bytes array
     * @return The length of `bs` used to get decoded
     */
    function _decode_lendelim(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes memory, uint256) {
        /**
         * First read the size encoded in `varint`, then use the size to read bytes.
         */
        (uint256 len, uint256 sz) = _decode_varint(p, bs);
        bytes memory b = new bytes(len);
        uint256 length = bs.length + WORD_LENGTH;
        assert(p + sz + len <= length);
        uint256 sourcePtr;
        uint256 destPtr;
        assembly {
            destPtr := add(b, 32)
            sourcePtr := add(add(bs, p), sz)
        }
        copyBytes(sourcePtr, destPtr, len);
        return (b, sz + len);
    }

    /**
     * @dev Skip the decoding of a single field
     * @param wt The WireType of the field
     * @param p The memory offset of `bs`
     * @param bs The bytes array to be decoded
     * @return The length of `bs` to skipped
     */
    function _skip_field_decode(
        WireType wt,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        if (wt == ProtoBufRuntime.WireType.Fixed64) {
            return 8;
        } else if (wt == ProtoBufRuntime.WireType.Fixed32) {
            return 4;
        } else if (wt == ProtoBufRuntime.WireType.Varint) {
            (, uint256 size) = ProtoBufRuntime._decode_varint(p, bs);
            return size;
        } else {
            require(wt == ProtoBufRuntime.WireType.LengthDelim);
            (uint256 len, uint256 size) = ProtoBufRuntime._decode_varint(p, bs);
            return size + len;
        }
    }

    // Encoders
    /**
     * @dev Encode ProtoBuf key
     * @param x The field ID
     * @param wt The WireType specified in ProtoBuf
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The length of encoded bytes
     */
    function _encode_key(
        uint256 x,
        WireType wt,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 i;
        assembly {
            i := or(mul(x, 8), mod(wt, 8))
        }
        return _encode_varint(i, p, bs);
    }

    /**
     * @dev Encode ProtoBuf varint
     * @param x The unsigned integer to be encoded
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The length of encoded bytes
     */
    function _encode_varint(
        uint256 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        /**
         * Refer to https://developers.google.com/protocol-buffers/docs/encoding
         */
        uint256 sz = 0;
        assembly {
            let bsptr := add(bs, p)
            let byt := and(x, 0x7f)
            for {} gt(shr(7, x), 0) {} {
                mstore8(bsptr, or(0x80, byt))
                bsptr := add(bsptr, 1)
                sz := add(sz, 1)
                x := shr(7, x)
                byt := and(x, 0x7f)
            }
            mstore8(bsptr, byt)
            sz := add(sz, 1)
        }
        return sz;
    }

    /**
     * @dev Encode ProtoBuf zig-zag encoding
     * @param x The signed integer to be encoded
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The length of encoded bytes
     */
    function _encode_varints(
        int256 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        /**
         * Refer to https://developers.google.com/protocol-buffers/docs/encoding
         */
        uint256 encodedInt = _encode_zigzag(x);
        return _encode_varint(encodedInt, p, bs);
    }

    /**
     * @dev Encode ProtoBuf bytes
     * @param xs The bytes array to be encoded
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The length of encoded bytes
     */
    function _encode_bytes(
        bytes memory xs,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 xsLength = xs.length;
        uint256 sz = _encode_varint(xsLength, p, bs);
        uint256 count = 0;
        assembly {
            let bsptr := add(bs, add(p, sz))
            let xsptr := add(xs, 32)
            for {} lt(count, xsLength) {} {
                mstore8(bsptr, byte(0, mload(xsptr)))
                bsptr := add(bsptr, 1)
                xsptr := add(xsptr, 1)
                count := add(count, 1)
            }
        }
        return sz + count;
    }

    /**
     * @dev Encode ProtoBuf string
     * @param xs The string to be encoded
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The length of encoded bytes
     */
    function _encode_string(
        string memory xs,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_bytes(bytes(xs), p, bs);
    }

    /**
     * `_encode_(u)int(32|64)`, `_encode_enum` and `_encode_bool`
     * are concrete implementation of `_encode_varint`
     */
    function _encode_uint32(
        uint32 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_varint(x, p, bs);
    }

    function _encode_uint64(
        uint64 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_varint(x, p, bs);
    }

    function _encode_int32(
        int32 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint64 twosComplement;
        assembly {
            twosComplement := x
        }
        return _encode_varint(twosComplement, p, bs);
    }

    function _encode_int64(
        int64 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint64 twosComplement;
        assembly {
            twosComplement := x
        }
        return _encode_varint(twosComplement, p, bs);
    }

    function _encode_enum(
        int32 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_int32(x, p, bs);
    }

    function _encode_bool(
        bool x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        if (x) {
            return _encode_varint(1, p, bs);
        } else {
            return _encode_varint(0, p, bs);
        }
    }

    /**
     * `_encode_sint(32|64)`, `_encode_enum` and `_encode_bool`
     * are the concrete implementation of `_encode_varints`
     */
    function _encode_sint32(
        int32 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_varints(x, p, bs);
    }

    function _encode_sint64(
        int64 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_varints(x, p, bs);
    }

    /**
     * `_encode_(s)fixed(32|64)` is the concrete implementation of `_encode_uintf`
     */
    function _encode_fixed32(
        uint32 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_uintf(x, p, bs, 4);
    }

    function _encode_fixed64(
        uint64 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_uintf(x, p, bs, 8);
    }

    function _encode_sfixed32(
        int32 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint32 twosComplement;
        assembly {
            twosComplement := x
        }
        return _encode_uintf(twosComplement, p, bs, 4);
    }

    function _encode_sfixed64(
        int64 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint64 twosComplement;
        assembly {
            twosComplement := x
        }
        return _encode_uintf(twosComplement, p, bs, 8);
    }

    /**
     * @dev Encode ProtoBuf fixed-length integer
     * @param x The unsigned integer to be encoded
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The length of encoded bytes
     */
    function _encode_uintf(
        uint256 x,
        uint256 p,
        bytes memory bs,
        uint256 sz
    ) internal pure returns (uint256) {
        assembly {
            let bsptr := add(sz, add(bs, p))
            let count := sz
            for {} gt(count, 0) {} {
                bsptr := sub(bsptr, 1)
                mstore8(bsptr, byte(sub(32, count), x))
                count := sub(count, 1)
            }
        }
        return sz;
    }

    /**
     * @dev Encode ProtoBuf zig-zag signed integer
     * @param i The unsigned integer to be encoded
     * @return The encoded unsigned integer
     */
    function _encode_zigzag(int256 i) internal pure returns (uint256) {
        if (i >= 0) {
            return uint256(i) * 2;
        } else {
            return uint256(i * -2) - 1;
        }
    }

    // Estimators
    /**
     * @dev Estimate the length of encoded LengthDelim
     * @param i The length of LengthDelim
     * @return The estimated encoded length
     */
    function _sz_lendelim(uint256 i) internal pure returns (uint256) {
        return i + _sz_varint(i);
    }

    /**
     * @dev Estimate the length of encoded ProtoBuf field ID
     * @param i The field ID
     * @return The estimated encoded length
     */
    function _sz_key(uint256 i) internal pure returns (uint256) {
        if (i < 16) {
            return 1;
        } else if (i < 2048) {
            return 2;
        } else if (i < 262144) {
            return 3;
        } else {
            revert("not supported");
        }
    }

    /**
     * @dev Estimate the length of encoded ProtoBuf varint
     * @param i The unsigned integer
     * @return The estimated encoded length
     */
    function _sz_varint(uint256 i) internal pure returns (uint256) {
        uint256 count = 1;
        assembly {
            i := shr(7, i)
            for {} gt(i, 0) {} {
                i := shr(7, i)
                count := add(count, 1)
            }
        }
        return count;
    }

    /**
     * `_sz_(u)int(32|64)` and `_sz_enum` are the concrete implementation of `_sz_varint`
     */
    function _sz_uint32(uint32 i) internal pure returns (uint256) {
        return _sz_varint(i);
    }

    function _sz_uint64(uint64 i) internal pure returns (uint256) {
        return _sz_varint(i);
    }

    function _sz_int32(int32 i) internal pure returns (uint256) {
        if (i < 0) {
            return 10;
        } else {
            return _sz_varint(uint32(i));
        }
    }

    function _sz_int64(int64 i) internal pure returns (uint256) {
        if (i < 0) {
            return 10;
        } else {
            return _sz_varint(uint64(i));
        }
    }

    function _sz_enum(int64 i) internal pure returns (uint256) {
        if (i < 0) {
            return 10;
        } else {
            return _sz_varint(uint64(i));
        }
    }

    /**
     * `_sz_sint(32|64)` and `_sz_enum` are the concrete implementation of zig-zag encoding
     */
    function _sz_sint32(int32 i) internal pure returns (uint256) {
        return _sz_varint(_encode_zigzag(i));
    }

    function _sz_sint64(int64 i) internal pure returns (uint256) {
        return _sz_varint(_encode_zigzag(i));
    }

    /**
     * `_estimate_packed_repeated_(uint32|uint64|int32|int64|sint32|sint64)`
     */
    function _estimate_packed_repeated_uint32(uint32[] memory a)
        internal
        pure
        returns (uint256)
    {
        uint256 e = 0;
        for (uint256 i; i < a.length; i++) {
            e += _sz_uint32(a[i]);
        }
        return e;
    }

    function _estimate_packed_repeated_uint64(uint64[] memory a)
        internal
        pure
        returns (uint256)
    {
        uint256 e = 0;
        for (uint256 i; i < a.length; i++) {
            e += _sz_uint64(a[i]);
        }
        return e;
    }

    function _estimate_packed_repeated_int32(int32[] memory a)
        internal
        pure
        returns (uint256)
    {
        uint256 e = 0;
        for (uint256 i; i < a.length; i++) {
            e += _sz_int32(a[i]);
        }
        return e;
    }

    function _estimate_packed_repeated_int64(int64[] memory a)
        internal
        pure
        returns (uint256)
    {
        uint256 e = 0;
        for (uint256 i; i < a.length; i++) {
            e += _sz_int64(a[i]);
        }
        return e;
    }

    function _estimate_packed_repeated_sint32(int32[] memory a)
        internal
        pure
        returns (uint256)
    {
        uint256 e = 0;
        for (uint256 i; i < a.length; i++) {
            e += _sz_sint32(a[i]);
        }
        return e;
    }

    function _estimate_packed_repeated_sint64(int64[] memory a)
        internal
        pure
        returns (uint256)
    {
        uint256 e = 0;
        for (uint256 i; i < a.length; i++) {
            e += _sz_sint64(a[i]);
        }
        return e;
    }

    // Element counters for packed repeated fields
    function _count_packed_repeated_varint(
        uint256 p,
        uint256 len,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 count = 0;
        uint256 end = p + len;
        while (p < end) {
            uint256 sz;
            (, sz) = _decode_varint(p, bs);
            p += sz;
            count += 1;
        }
        return count;
    }

    // Soltype extensions
    /**
     * @dev Decode Solidity integer and/or fixed-size bytes array, filling from lowest bit.
     * @param n The maximum number of bytes to read
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The bytes32 representation
     * @return The number of bytes used to decode
     */
    function _decode_sol_bytesN_lower(
        uint8 n,
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes32, uint256) {
        uint256 r;
        (uint256 len, uint256 sz) = _decode_varint(p, bs);
        if (len + sz > n + 3) {
            revert(OVERFLOW_MESSAGE);
        }
        p += 3;
        assert(p < bs.length + WORD_LENGTH);
        assembly {
            r := mload(add(p, bs))
        }
        for (uint256 i = len - 2; i < WORD_LENGTH; i++) {
            r /= 256;
        }
        return (bytes32(r), len + sz);
    }

    /**
     * @dev Decode Solidity integer and/or fixed-size bytes array, filling from highest bit.
     * @param n The maximum number of bytes to read
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The bytes32 representation
     * @return The number of bytes used to decode
     */
    function _decode_sol_bytesN(
        uint8 n,
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes32, uint256) {
        (uint256 len, uint256 sz) = _decode_varint(p, bs);
        uint256 wordLength = WORD_LENGTH;
        uint256 byteSize = BYTE_SIZE;
        if (len + sz > n + 3) {
            revert(OVERFLOW_MESSAGE);
        }
        p += 3;
        bytes32 acc;
        assert(p < bs.length + WORD_LENGTH);
        assembly {
            acc := mload(add(p, bs))
            let difference := sub(wordLength, sub(len, 2))
            let bits := mul(byteSize, difference)
            acc := shl(bits, shr(bits, acc))
        }
        return (acc, len + sz);
    }

    /*
     * `_decode_sol*` are the concrete implementation of decoding Solidity types
     */
    function _decode_sol_address(
        uint256 p,
        bytes memory bs
    ) internal pure returns (address, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytesN(20, p, bs);
        return (address(bytes20(r)), sz);
    }

    function _decode_sol_bool(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bool, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(1, p, bs);
        if (r == 0) {
            return (false, sz);
        }
        return (true, sz);
    }

    function _decode_sol_uint(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256, uint256) {
        return _decode_sol_uint256(p, bs);
    }

    function _decode_sol_uintN(
        uint8 n,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256, uint256) {
        (bytes32 u, uint256 sz) = _decode_sol_bytesN_lower(n, p, bs);
        uint256 r;
        assembly {
            r := u
        }
        return (r, sz);
    }

    function _decode_sol_uint8(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint8, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(1, p, bs);
        return (uint8(r), sz);
    }

    function _decode_sol_uint16(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint16, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(2, p, bs);
        return (uint16(r), sz);
    }

    function _decode_sol_uint24(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint24, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(3, p, bs);
        return (uint24(r), sz);
    }

    function _decode_sol_uint32(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint32, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(4, p, bs);
        return (uint32(r), sz);
    }

    function _decode_sol_uint40(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint40, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(5, p, bs);
        return (uint40(r), sz);
    }

    function _decode_sol_uint48(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint48, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(6, p, bs);
        return (uint48(r), sz);
    }

    function _decode_sol_uint56(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint56, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(7, p, bs);
        return (uint56(r), sz);
    }

    function _decode_sol_uint64(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint64, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(8, p, bs);
        return (uint64(r), sz);
    }

    function _decode_sol_uint72(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint72, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(9, p, bs);
        return (uint72(r), sz);
    }

    function _decode_sol_uint80(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint80, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(10, p, bs);
        return (uint80(r), sz);
    }

    function _decode_sol_uint88(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint88, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(11, p, bs);
        return (uint88(r), sz);
    }

    function _decode_sol_uint96(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint96, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(12, p, bs);
        return (uint96(r), sz);
    }

    function _decode_sol_uint104(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint104, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(13, p, bs);
        return (uint104(r), sz);
    }

    function _decode_sol_uint112(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint112, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(14, p, bs);
        return (uint112(r), sz);
    }

    function _decode_sol_uint120(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint120, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(15, p, bs);
        return (uint120(r), sz);
    }

    function _decode_sol_uint128(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint128, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(16, p, bs);
        return (uint128(r), sz);
    }

    function _decode_sol_uint136(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint136, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(17, p, bs);
        return (uint136(r), sz);
    }

    function _decode_sol_uint144(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint144, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(18, p, bs);
        return (uint144(r), sz);
    }

    function _decode_sol_uint152(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint152, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(19, p, bs);
        return (uint152(r), sz);
    }

    function _decode_sol_uint160(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint160, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(20, p, bs);
        return (uint160(r), sz);
    }

    function _decode_sol_uint168(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint168, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(21, p, bs);
        return (uint168(r), sz);
    }

    function _decode_sol_uint176(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint176, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(22, p, bs);
        return (uint176(r), sz);
    }

    function _decode_sol_uint184(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint184, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(23, p, bs);
        return (uint184(r), sz);
    }

    function _decode_sol_uint192(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint192, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(24, p, bs);
        return (uint192(r), sz);
    }

    function _decode_sol_uint200(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint200, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(25, p, bs);
        return (uint200(r), sz);
    }

    function _decode_sol_uint208(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint208, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(26, p, bs);
        return (uint208(r), sz);
    }

    function _decode_sol_uint216(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint216, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(27, p, bs);
        return (uint216(r), sz);
    }

    function _decode_sol_uint224(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint224, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(28, p, bs);
        return (uint224(r), sz);
    }

    function _decode_sol_uint232(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint232, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(29, p, bs);
        return (uint232(r), sz);
    }

    function _decode_sol_uint240(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint240, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(30, p, bs);
        return (uint240(r), sz);
    }

    function _decode_sol_uint248(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint248, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(31, p, bs);
        return (uint248(r), sz);
    }

    function _decode_sol_uint256(
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256, uint256) {
        (uint256 r, uint256 sz) = _decode_sol_uintN(32, p, bs);
        return (uint256(r), sz);
    }

    function _decode_sol_int(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int256, uint256) {
        return _decode_sol_int256(p, bs);
    }

    function _decode_sol_intN(
        uint8 n,
        uint256 p,
        bytes memory bs
    ) internal pure returns (int256, uint256) {
        (bytes32 u, uint256 sz) = _decode_sol_bytesN_lower(n, p, bs);
        int256 r;
        assembly {
            r := u
            r := signextend(sub(sz, 4), r)
        }
        return (r, sz);
    }

    function _decode_sol_bytes(
        uint8 n,
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes32, uint256) {
        (bytes32 u, uint256 sz) = _decode_sol_bytesN(n, p, bs);
        return (u, sz);
    }

    function _decode_sol_int8(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int8, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(1, p, bs);
        return (int8(r), sz);
    }

    function _decode_sol_int16(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int16, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(2, p, bs);
        return (int16(r), sz);
    }

    function _decode_sol_int24(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int24, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(3, p, bs);
        return (int24(r), sz);
    }

    function _decode_sol_int32(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int32, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(4, p, bs);
        return (int32(r), sz);
    }

    function _decode_sol_int40(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int40, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(5, p, bs);
        return (int40(r), sz);
    }

    function _decode_sol_int48(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int48, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(6, p, bs);
        return (int48(r), sz);
    }

    function _decode_sol_int56(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int56, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(7, p, bs);
        return (int56(r), sz);
    }

    function _decode_sol_int64(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int64, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(8, p, bs);
        return (int64(r), sz);
    }

    function _decode_sol_int72(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int72, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(9, p, bs);
        return (int72(r), sz);
    }

    function _decode_sol_int80(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int80, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(10, p, bs);
        return (int80(r), sz);
    }

    function _decode_sol_int88(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int88, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(11, p, bs);
        return (int88(r), sz);
    }

    function _decode_sol_int96(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int96, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(12, p, bs);
        return (int96(r), sz);
    }

    function _decode_sol_int104(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int104, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(13, p, bs);
        return (int104(r), sz);
    }

    function _decode_sol_int112(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int112, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(14, p, bs);
        return (int112(r), sz);
    }

    function _decode_sol_int120(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int120, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(15, p, bs);
        return (int120(r), sz);
    }

    function _decode_sol_int128(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int128, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(16, p, bs);
        return (int128(r), sz);
    }

    function _decode_sol_int136(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int136, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(17, p, bs);
        return (int136(r), sz);
    }

    function _decode_sol_int144(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int144, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(18, p, bs);
        return (int144(r), sz);
    }

    function _decode_sol_int152(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int152, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(19, p, bs);
        return (int152(r), sz);
    }

    function _decode_sol_int160(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int160, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(20, p, bs);
        return (int160(r), sz);
    }

    function _decode_sol_int168(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int168, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(21, p, bs);
        return (int168(r), sz);
    }

    function _decode_sol_int176(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int176, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(22, p, bs);
        return (int176(r), sz);
    }

    function _decode_sol_int184(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int184, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(23, p, bs);
        return (int184(r), sz);
    }

    function _decode_sol_int192(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int192, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(24, p, bs);
        return (int192(r), sz);
    }

    function _decode_sol_int200(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int200, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(25, p, bs);
        return (int200(r), sz);
    }

    function _decode_sol_int208(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int208, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(26, p, bs);
        return (int208(r), sz);
    }

    function _decode_sol_int216(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int216, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(27, p, bs);
        return (int216(r), sz);
    }

    function _decode_sol_int224(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int224, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(28, p, bs);
        return (int224(r), sz);
    }

    function _decode_sol_int232(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int232, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(29, p, bs);
        return (int232(r), sz);
    }

    function _decode_sol_int240(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int240, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(30, p, bs);
        return (int240(r), sz);
    }

    function _decode_sol_int248(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int248, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(31, p, bs);
        return (int248(r), sz);
    }

    function _decode_sol_int256(
        uint256 p,
        bytes memory bs
    ) internal pure returns (int256, uint256) {
        (int256 r, uint256 sz) = _decode_sol_intN(32, p, bs);
        return (int256(r), sz);
    }

    function _decode_sol_bytes1(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes1, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(1, p, bs);
        return (bytes1(r), sz);
    }

    function _decode_sol_bytes2(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes2, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(2, p, bs);
        return (bytes2(r), sz);
    }

    function _decode_sol_bytes3(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes3, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(3, p, bs);
        return (bytes3(r), sz);
    }

    function _decode_sol_bytes4(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes4, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(4, p, bs);
        return (bytes4(r), sz);
    }

    function _decode_sol_bytes5(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes5, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(5, p, bs);
        return (bytes5(r), sz);
    }

    function _decode_sol_bytes6(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes6, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(6, p, bs);
        return (bytes6(r), sz);
    }

    function _decode_sol_bytes7(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes7, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(7, p, bs);
        return (bytes7(r), sz);
    }

    function _decode_sol_bytes8(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes8, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(8, p, bs);
        return (bytes8(r), sz);
    }

    function _decode_sol_bytes9(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes9, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(9, p, bs);
        return (bytes9(r), sz);
    }

    function _decode_sol_bytes10(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes10, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(10, p, bs);
        return (bytes10(r), sz);
    }

    function _decode_sol_bytes11(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes11, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(11, p, bs);
        return (bytes11(r), sz);
    }

    function _decode_sol_bytes12(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes12, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(12, p, bs);
        return (bytes12(r), sz);
    }

    function _decode_sol_bytes13(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes13, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(13, p, bs);
        return (bytes13(r), sz);
    }

    function _decode_sol_bytes14(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes14, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(14, p, bs);
        return (bytes14(r), sz);
    }

    function _decode_sol_bytes15(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes15, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(15, p, bs);
        return (bytes15(r), sz);
    }

    function _decode_sol_bytes16(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes16, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(16, p, bs);
        return (bytes16(r), sz);
    }

    function _decode_sol_bytes17(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes17, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(17, p, bs);
        return (bytes17(r), sz);
    }

    function _decode_sol_bytes18(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes18, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(18, p, bs);
        return (bytes18(r), sz);
    }

    function _decode_sol_bytes19(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes19, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(19, p, bs);
        return (bytes19(r), sz);
    }

    function _decode_sol_bytes20(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes20, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(20, p, bs);
        return (bytes20(r), sz);
    }

    function _decode_sol_bytes21(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes21, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(21, p, bs);
        return (bytes21(r), sz);
    }

    function _decode_sol_bytes22(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes22, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(22, p, bs);
        return (bytes22(r), sz);
    }

    function _decode_sol_bytes23(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes23, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(23, p, bs);
        return (bytes23(r), sz);
    }

    function _decode_sol_bytes24(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes24, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(24, p, bs);
        return (bytes24(r), sz);
    }

    function _decode_sol_bytes25(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes25, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(25, p, bs);
        return (bytes25(r), sz);
    }

    function _decode_sol_bytes26(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes26, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(26, p, bs);
        return (bytes26(r), sz);
    }

    function _decode_sol_bytes27(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes27, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(27, p, bs);
        return (bytes27(r), sz);
    }

    function _decode_sol_bytes28(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes28, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(28, p, bs);
        return (bytes28(r), sz);
    }

    function _decode_sol_bytes29(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes29, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(29, p, bs);
        return (bytes29(r), sz);
    }

    function _decode_sol_bytes30(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes30, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(30, p, bs);
        return (bytes30(r), sz);
    }

    function _decode_sol_bytes31(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes31, uint256) {
        (bytes32 r, uint256 sz) = _decode_sol_bytes(31, p, bs);
        return (bytes31(r), sz);
    }

    function _decode_sol_bytes32(
        uint256 p,
        bytes memory bs
    ) internal pure returns (bytes32, uint256) {
        return _decode_sol_bytes(32, p, bs);
    }

    /*
     * `_encode_sol*` are the concrete implementation of encoding Solidity types
     */
    function _encode_sol_address(
        address x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(uint160(x)), 20, p, bs);
    }

    function _encode_sol_uint(
        uint256 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 32, p, bs);
    }

    function _encode_sol_uint8(
        uint8 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 1, p, bs);
    }

    function _encode_sol_uint16(
        uint16 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 2, p, bs);
    }

    function _encode_sol_uint24(
        uint24 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 3, p, bs);
    }

    function _encode_sol_uint32(
        uint32 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 4, p, bs);
    }

    function _encode_sol_uint40(
        uint40 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 5, p, bs);
    }

    function _encode_sol_uint48(
        uint48 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 6, p, bs);
    }

    function _encode_sol_uint56(
        uint56 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 7, p, bs);
    }

    function _encode_sol_uint64(
        uint64 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 8, p, bs);
    }

    function _encode_sol_uint72(
        uint72 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 9, p, bs);
    }

    function _encode_sol_uint80(
        uint80 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 10, p, bs);
    }

    function _encode_sol_uint88(
        uint88 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 11, p, bs);
    }

    function _encode_sol_uint96(
        uint96 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 12, p, bs);
    }

    function _encode_sol_uint104(
        uint104 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 13, p, bs);
    }

    function _encode_sol_uint112(
        uint112 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 14, p, bs);
    }

    function _encode_sol_uint120(
        uint120 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 15, p, bs);
    }

    function _encode_sol_uint128(
        uint128 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 16, p, bs);
    }

    function _encode_sol_uint136(
        uint136 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 17, p, bs);
    }

    function _encode_sol_uint144(
        uint144 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 18, p, bs);
    }

    function _encode_sol_uint152(
        uint152 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 19, p, bs);
    }

    function _encode_sol_uint160(
        uint160 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 20, p, bs);
    }

    function _encode_sol_uint168(
        uint168 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 21, p, bs);
    }

    function _encode_sol_uint176(
        uint176 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 22, p, bs);
    }

    function _encode_sol_uint184(
        uint184 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 23, p, bs);
    }

    function _encode_sol_uint192(
        uint192 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 24, p, bs);
    }

    function _encode_sol_uint200(
        uint200 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 25, p, bs);
    }

    function _encode_sol_uint208(
        uint208 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 26, p, bs);
    }

    function _encode_sol_uint216(
        uint216 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 27, p, bs);
    }

    function _encode_sol_uint224(
        uint224 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 28, p, bs);
    }

    function _encode_sol_uint232(
        uint232 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 29, p, bs);
    }

    function _encode_sol_uint240(
        uint240 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 30, p, bs);
    }

    function _encode_sol_uint248(
        uint248 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 31, p, bs);
    }

    function _encode_sol_uint256(
        uint256 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(uint256(x), 32, p, bs);
    }

    function _encode_sol_int(
        int256 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(x, 32, p, bs);
    }

    function _encode_sol_int8(
        int8 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 1, p, bs);
    }

    function _encode_sol_int16(
        int16 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 2, p, bs);
    }

    function _encode_sol_int24(
        int24 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 3, p, bs);
    }

    function _encode_sol_int32(
        int32 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 4, p, bs);
    }

    function _encode_sol_int40(
        int40 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 5, p, bs);
    }

    function _encode_sol_int48(
        int48 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 6, p, bs);
    }

    function _encode_sol_int56(
        int56 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 7, p, bs);
    }

    function _encode_sol_int64(
        int64 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 8, p, bs);
    }

    function _encode_sol_int72(
        int72 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 9, p, bs);
    }

    function _encode_sol_int80(
        int80 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 10, p, bs);
    }

    function _encode_sol_int88(
        int88 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 11, p, bs);
    }

    function _encode_sol_int96(
        int96 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 12, p, bs);
    }

    function _encode_sol_int104(
        int104 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 13, p, bs);
    }

    function _encode_sol_int112(
        int112 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 14, p, bs);
    }

    function _encode_sol_int120(
        int120 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 15, p, bs);
    }

    function _encode_sol_int128(
        int128 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 16, p, bs);
    }

    function _encode_sol_int136(
        int136 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 17, p, bs);
    }

    function _encode_sol_int144(
        int144 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 18, p, bs);
    }

    function _encode_sol_int152(
        int152 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 19, p, bs);
    }

    function _encode_sol_int160(
        int160 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 20, p, bs);
    }

    function _encode_sol_int168(
        int168 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 21, p, bs);
    }

    function _encode_sol_int176(
        int176 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 22, p, bs);
    }

    function _encode_sol_int184(
        int184 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 23, p, bs);
    }

    function _encode_sol_int192(
        int192 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 24, p, bs);
    }

    function _encode_sol_int200(
        int200 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 25, p, bs);
    }

    function _encode_sol_int208(
        int208 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 26, p, bs);
    }

    function _encode_sol_int216(
        int216 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 27, p, bs);
    }

    function _encode_sol_int224(
        int224 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 28, p, bs);
    }

    function _encode_sol_int232(
        int232 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 29, p, bs);
    }

    function _encode_sol_int240(
        int240 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 30, p, bs);
    }

    function _encode_sol_int248(
        int248 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(int256(x), 31, p, bs);
    }

    function _encode_sol_int256(
        int256 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol(x, 32, p, bs);
    }

    function _encode_sol_bytes1(
        bytes1 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 1, p, bs);
    }

    function _encode_sol_bytes2(
        bytes2 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 2, p, bs);
    }

    function _encode_sol_bytes3(
        bytes3 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 3, p, bs);
    }

    function _encode_sol_bytes4(
        bytes4 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 4, p, bs);
    }

    function _encode_sol_bytes5(
        bytes5 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 5, p, bs);
    }

    function _encode_sol_bytes6(
        bytes6 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 6, p, bs);
    }

    function _encode_sol_bytes7(
        bytes7 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 7, p, bs);
    }

    function _encode_sol_bytes8(
        bytes8 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 8, p, bs);
    }

    function _encode_sol_bytes9(
        bytes9 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 9, p, bs);
    }

    function _encode_sol_bytes10(
        bytes10 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 10, p, bs);
    }

    function _encode_sol_bytes11(
        bytes11 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 11, p, bs);
    }

    function _encode_sol_bytes12(
        bytes12 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 12, p, bs);
    }

    function _encode_sol_bytes13(
        bytes13 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 13, p, bs);
    }

    function _encode_sol_bytes14(
        bytes14 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 14, p, bs);
    }

    function _encode_sol_bytes15(
        bytes15 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 15, p, bs);
    }

    function _encode_sol_bytes16(
        bytes16 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 16, p, bs);
    }

    function _encode_sol_bytes17(
        bytes17 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 17, p, bs);
    }

    function _encode_sol_bytes18(
        bytes18 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 18, p, bs);
    }

    function _encode_sol_bytes19(
        bytes19 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 19, p, bs);
    }

    function _encode_sol_bytes20(
        bytes20 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 20, p, bs);
    }

    function _encode_sol_bytes21(
        bytes21 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 21, p, bs);
    }

    function _encode_sol_bytes22(
        bytes22 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 22, p, bs);
    }

    function _encode_sol_bytes23(
        bytes23 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 23, p, bs);
    }

    function _encode_sol_bytes24(
        bytes24 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 24, p, bs);
    }

    function _encode_sol_bytes25(
        bytes25 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 25, p, bs);
    }

    function _encode_sol_bytes26(
        bytes26 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 26, p, bs);
    }

    function _encode_sol_bytes27(
        bytes27 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 27, p, bs);
    }

    function _encode_sol_bytes28(
        bytes28 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 28, p, bs);
    }

    function _encode_sol_bytes29(
        bytes29 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 29, p, bs);
    }

    function _encode_sol_bytes30(
        bytes30 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 30, p, bs);
    }

    function _encode_sol_bytes31(
        bytes31 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(bytes32(x), 31, p, bs);
    }

    function _encode_sol_bytes32(
        bytes32 x,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        return _encode_sol_bytes(x, 32, p, bs);
    }

    /**
     * @dev Encode the key of Solidity integer and/or fixed-size bytes array.
     * @param sz The number of bytes used to encode Solidity types
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The number of bytes used to encode
     */
    function _encode_sol_header(
        uint256 sz,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 offset = p;
        p += _encode_varint(sz + 2, p, bs);
        p += _encode_key(1, WireType.LengthDelim, p, bs);
        p += _encode_varint(sz, p, bs);
        return p - offset;
    }

    /**
     * @dev Encode Solidity type
     * @param x The unsinged integer to be encoded
     * @param sz The number of bytes used to encode Solidity types
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The number of bytes used to encode
     */
    function _encode_sol(
        uint256 x,
        uint256 sz,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 size;
        p += 3;
        size = _encode_sol_raw_other(x, p, bs, sz);
        p += size;
        _encode_sol_header(size, offset, bs);
        return p - offset;
    }

    /**
     * @dev Encode Solidity type
     * @param x The signed integer to be encoded
     * @param sz The number of bytes used to encode Solidity types
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The number of bytes used to encode
     */
    function _encode_sol(
        int256 x,
        uint256 sz,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 size;
        p += 3;
        size = _encode_sol_raw_other(x, p, bs, sz);
        p += size;
        _encode_sol_header(size, offset, bs);
        return p - offset;
    }

    /**
     * @dev Encode Solidity type
     * @param x The fixed-size byte array to be encoded
     * @param sz The number of bytes used to encode Solidity types
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The number of bytes used to encode
     */
    function _encode_sol_bytes(
        bytes32 x,
        uint256 sz,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 size;
        p += 3;
        size = _encode_sol_raw_bytes_array(x, p, bs, sz);
        p += size;
        _encode_sol_header(size, offset, bs);
        return p - offset;
    }

    /**
     * @dev Get the actual size needed to encoding an unsigned integer
     * @param x The unsigned integer to be encoded
     * @param sz The maximum number of bytes used to encode Solidity types
     * @return The number of bytes needed for encoding `x`
     */
    function _get_real_size(
        uint256 x,
        uint256 sz
    ) internal pure returns (uint256) {
        uint256 base = 0xff;
        uint256 realSize = sz;
        while (
            x & (base << (realSize * BYTE_SIZE - BYTE_SIZE)) == 0
                && realSize > 0
        ) {
            realSize -= 1;
        }
        if (realSize == 0) {
            realSize = 1;
        }
        return realSize;
    }

    /**
     * @dev Get the actual size needed to encoding an signed integer
     * @param x The signed integer to be encoded
     * @param sz The maximum number of bytes used to encode Solidity types
     * @return The number of bytes needed for encoding `x`
     */
    function _get_real_size(
        int256 x,
        uint256 sz
    ) internal pure returns (uint256) {
        int256 base = 0xff;
        if (x >= 0) {
            uint256 tmp = _get_real_size(uint256(x), sz);
            int256 remainder = (x & (base << (tmp * BYTE_SIZE - BYTE_SIZE)))
                >> (tmp * BYTE_SIZE - BYTE_SIZE);
            if (remainder >= 128) {
                tmp += 1;
            }
            return tmp;
        }

        uint256 realSize = sz;
        while (
            x & (base << (realSize * BYTE_SIZE - BYTE_SIZE))
                == (base << (realSize * BYTE_SIZE - BYTE_SIZE)) && realSize > 0
        ) {
            realSize -= 1;
        }
        {
            int256 remainder = (
                x & (base << (realSize * BYTE_SIZE - BYTE_SIZE))
            ) >> (realSize * BYTE_SIZE - BYTE_SIZE);
            if (remainder < 128) {
                realSize += 1;
            }
        }
        return realSize;
    }

    /**
     * @dev Encode the fixed-bytes array
     * @param x The fixed-size byte array to be encoded
     * @param sz The maximum number of bytes used to encode Solidity types
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The number of bytes needed for encoding `x`
     */
    function _encode_sol_raw_bytes_array(
        bytes32 x,
        uint256 p,
        bytes memory bs,
        uint256 sz
    ) internal pure returns (uint256) {
        /**
         * The idea is to not encode the leading bytes of zero.
         */
        uint256 actualSize = sz;
        for (uint256 i; i < sz; i++) {
            uint8 current = uint8(x[sz - 1 - i]);
            if (current == 0 && actualSize > 1) {
                actualSize--;
            } else {
                break;
            }
        }
        assembly {
            let bsptr := add(bs, p)
            let count := actualSize
            for {} gt(count, 0) {} {
                mstore8(bsptr, byte(sub(actualSize, count), x))
                bsptr := add(bsptr, 1)
                count := sub(count, 1)
            }
        }
        return actualSize;
    }

    /**
     * @dev Encode the signed integer
     * @param x The signed integer to be encoded
     * @param sz The maximum number of bytes used to encode Solidity types
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The number of bytes needed for encoding `x`
     */
    function _encode_sol_raw_other(
        int256 x,
        uint256 p,
        bytes memory bs,
        uint256 sz
    ) internal pure returns (uint256) {
        /**
         * The idea is to not encode the leading bytes of zero.or one,
         * depending on whether it is positive.
         */
        uint256 realSize = _get_real_size(x, sz);
        assembly {
            let bsptr := add(bs, p)
            let count := realSize
            for {} gt(count, 0) {} {
                mstore8(bsptr, byte(sub(32, count), x))
                bsptr := add(bsptr, 1)
                count := sub(count, 1)
            }
        }
        return realSize;
    }

    /**
     * @dev Encode the unsigned integer
     * @param x The unsigned integer to be encoded
     * @param sz The maximum number of bytes used to encode Solidity types
     * @param p The offset of bytes array `bs`
     * @param bs The bytes array to encode
     * @return The number of bytes needed for encoding `x`
     */
    function _encode_sol_raw_other(
        uint256 x,
        uint256 p,
        bytes memory bs,
        uint256 sz
    ) internal pure returns (uint256) {
        uint256 realSize = _get_real_size(x, sz);
        assembly {
            let bsptr := add(bs, p)
            let count := realSize
            for {} gt(count, 0) {} {
                mstore8(bsptr, byte(sub(32, count), x))
                bsptr := add(bsptr, 1)
                count := sub(count, 1)
            }
        }
        return realSize;
    }
}
