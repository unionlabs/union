// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.23;

import "../../../ProtoBufRuntime.sol";
import "../../../GoogleProtobufAny.sol";

library CosmosIcs23V1ExistenceProof {
    //struct definition
    struct Data {
        bytes key;
        bytes value;
        CosmosIcs23V1LeafOp.Data leaf;
        CosmosIcs23V1InnerOp.Data[] path;
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
        uint256[5] memory counters;
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
                pointer += _read_key(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_value(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_leaf(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer +=
                    _read_unpacked_repeated_path(pointer, bs, nil(), counters);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[4] > 0) {
            require(r.path.length == 0);
            r.path = new CosmosIcs23V1InnerOp.Data[](counters[4]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 4) {
                pointer +=
                    _read_unpacked_repeated_path(pointer, bs, r, counters);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
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
     * @return The number of bytes decoded
     */
    function _read_key(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.key = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_value(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.value = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_leaf(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1LeafOp.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1LeafOp(p, bs);
        r.leaf = x;
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
    function _read_unpacked_repeated_path(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[5] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (CosmosIcs23V1InnerOp.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1InnerOp(p, bs);
        if (isNil(r)) {
            counters[4] += 1;
        } else {
            r.path[r.path.length - counters[4]] = x;
            counters[4] -= 1;
        }
        return sz;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1LeafOp(
        uint256 p,
        bytes memory bs
    ) internal pure returns (CosmosIcs23V1LeafOp.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1LeafOp.Data memory r,) =
            CosmosIcs23V1LeafOp._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1InnerOp(
        uint256 p,
        bytes memory bs
    ) internal pure returns (CosmosIcs23V1InnerOp.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1InnerOp.Data memory r,) =
            CosmosIcs23V1InnerOp._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
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
        uint256 i;
        if (r.key.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.key, pointer, bs);
        }
        if (r.value.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.value, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += CosmosIcs23V1LeafOp._encode_nested(r.leaf, pointer, bs);

        if (r.path.length != 0) {
            for (i = 0; i < r.path.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer +=
                    CosmosIcs23V1InnerOp._encode_nested(r.path[i], pointer, bs);
            }
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
        uint256 i;
        e += 1 + ProtoBufRuntime._sz_lendelim(r.key.length);
        e += 1 + ProtoBufRuntime._sz_lendelim(r.value.length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(CosmosIcs23V1LeafOp._estimate(r.leaf));
        for (i = 0; i < r.path.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    CosmosIcs23V1InnerOp._estimate(r.path[i])
                );
        }
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.key.length != 0) {
            return false;
        }

        if (r.value.length != 0) {
            return false;
        }

        if (r.path.length != 0) {
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
        output.key = input.key;
        output.value = input.value;
        CosmosIcs23V1LeafOp.store(input.leaf, output.leaf);

        for (uint256 i4 = 0; i4 < input.path.length; i4++) {
            output.path.push(input.path[i4]);
        }
    }

    //array helpers for Path
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addPath(
        Data memory self,
        CosmosIcs23V1InnerOp.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        CosmosIcs23V1InnerOp.Data[] memory tmp =
            new CosmosIcs23V1InnerOp.Data[](self.path.length + 1);
        for (uint256 i; i < self.path.length; i++) {
            tmp[i] = self.path[i];
        }
        tmp[self.path.length] = value;
        self.path = tmp;
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

//library CosmosIcs23V1ExistenceProof

library CosmosIcs23V1NonExistenceProof {
    //struct definition
    struct Data {
        bytes key;
        CosmosIcs23V1ExistenceProof.Data left;
        CosmosIcs23V1ExistenceProof.Data right;
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
                pointer += _read_key(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_left(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_right(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
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
     * @return The number of bytes decoded
     */
    function _read_key(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.key = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_left(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1ExistenceProof.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1ExistenceProof(p, bs);
        r.left = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_right(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1ExistenceProof.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1ExistenceProof(p, bs);
        r.right = x;
        return sz;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1ExistenceProof(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (CosmosIcs23V1ExistenceProof.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1ExistenceProof.Data memory r,) =
            CosmosIcs23V1ExistenceProof._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
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

        if (r.key.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.key, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            CosmosIcs23V1ExistenceProof._encode_nested(r.left, pointer, bs);

        pointer += ProtoBufRuntime._encode_key(
            3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            CosmosIcs23V1ExistenceProof._encode_nested(r.right, pointer, bs);

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
        e += 1 + ProtoBufRuntime._sz_lendelim(r.key.length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1ExistenceProof._estimate(r.left)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1ExistenceProof._estimate(r.right)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.key.length != 0) {
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
        output.key = input.key;
        CosmosIcs23V1ExistenceProof.store(input.left, output.left);
        CosmosIcs23V1ExistenceProof.store(input.right, output.right);
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

//library CosmosIcs23V1NonExistenceProof

library CosmosIcs23V1CommitmentProof {
    //struct definition
    struct Data {
        CosmosIcs23V1ExistenceProof.Data exist;
        CosmosIcs23V1NonExistenceProof.Data nonexist;
        CosmosIcs23V1BatchProof.Data batch;
        CosmosIcs23V1CompressedBatchProof.Data compressed;
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
                pointer += _read_exist(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_nonexist(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_batch(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_compressed(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
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
     * @return The number of bytes decoded
     */
    function _read_exist(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1ExistenceProof.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1ExistenceProof(p, bs);
        r.exist = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_nonexist(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1NonExistenceProof.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1NonExistenceProof(p, bs);
        r.nonexist = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_batch(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1BatchProof.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1BatchProof(p, bs);
        r.batch = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_compressed(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1CompressedBatchProof.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1CompressedBatchProof(p, bs);
        r.compressed = x;
        return sz;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1ExistenceProof(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (CosmosIcs23V1ExistenceProof.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1ExistenceProof.Data memory r,) =
            CosmosIcs23V1ExistenceProof._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1NonExistenceProof(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (CosmosIcs23V1NonExistenceProof.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1NonExistenceProof.Data memory r,) =
            CosmosIcs23V1NonExistenceProof._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1BatchProof(
        uint256 p,
        bytes memory bs
    ) internal pure returns (CosmosIcs23V1BatchProof.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1BatchProof.Data memory r,) =
            CosmosIcs23V1BatchProof._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1CompressedBatchProof(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (CosmosIcs23V1CompressedBatchProof.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1CompressedBatchProof.Data memory r,) =
            CosmosIcs23V1CompressedBatchProof._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
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

        pointer += ProtoBufRuntime._encode_key(
            1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            CosmosIcs23V1ExistenceProof._encode_nested(r.exist, pointer, bs);

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += CosmosIcs23V1NonExistenceProof._encode_nested(
            r.nonexist, pointer, bs
        );

        pointer += ProtoBufRuntime._encode_key(
            3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += CosmosIcs23V1BatchProof._encode_nested(r.batch, pointer, bs);

        pointer += ProtoBufRuntime._encode_key(
            4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += CosmosIcs23V1CompressedBatchProof._encode_nested(
            r.compressed, pointer, bs
        );

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
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1ExistenceProof._estimate(r.exist)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1NonExistenceProof._estimate(r.nonexist)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1BatchProof._estimate(r.batch)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1CompressedBatchProof._estimate(r.compressed)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory) internal pure returns (bool) {
        return true;
    }

    //store function
    /**
     * @dev Store in-memory struct to storage
     * @param input The in-memory struct
     * @param output The in-storage struct
     */
    function store(Data memory input, Data storage output) internal {
        CosmosIcs23V1ExistenceProof.store(input.exist, output.exist);
        CosmosIcs23V1NonExistenceProof.store(input.nonexist, output.nonexist);
        CosmosIcs23V1BatchProof.store(input.batch, output.batch);
        CosmosIcs23V1CompressedBatchProof.store(
            input.compressed, output.compressed
        );
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

//library CosmosIcs23V1CommitmentProof

library CosmosIcs23V1LeafOp {
    //struct definition
    struct Data {
        CosmosIcs23V1GlobalEnums.HashOp hash;
        CosmosIcs23V1GlobalEnums.HashOp prehash_key;
        CosmosIcs23V1GlobalEnums.HashOp prehash_value;
        CosmosIcs23V1GlobalEnums.LengthOp length;
        bytes prefix;
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
                pointer += _read_hash(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_prehash_key(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_prehash_value(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_length(pointer, bs, r);
            } else if (fieldId == 5) {
                pointer += _read_prefix(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
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
     * @return The number of bytes decoded
     */
    function _read_hash(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int64 tmp, uint256 sz) = ProtoBufRuntime._decode_enum(p, bs);
        CosmosIcs23V1GlobalEnums.HashOp x =
            CosmosIcs23V1GlobalEnums.decode_HashOp(tmp);
        r.hash = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_prehash_key(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int64 tmp, uint256 sz) = ProtoBufRuntime._decode_enum(p, bs);
        CosmosIcs23V1GlobalEnums.HashOp x =
            CosmosIcs23V1GlobalEnums.decode_HashOp(tmp);
        r.prehash_key = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_prehash_value(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int64 tmp, uint256 sz) = ProtoBufRuntime._decode_enum(p, bs);
        CosmosIcs23V1GlobalEnums.HashOp x =
            CosmosIcs23V1GlobalEnums.decode_HashOp(tmp);
        r.prehash_value = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_length(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int64 tmp, uint256 sz) = ProtoBufRuntime._decode_enum(p, bs);
        CosmosIcs23V1GlobalEnums.LengthOp x =
            CosmosIcs23V1GlobalEnums.decode_LengthOp(tmp);
        r.length = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_prefix(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.prefix = x;
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

        if (uint256(r.hash) != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            int32 _enum_hash = CosmosIcs23V1GlobalEnums.encode_HashOp(r.hash);
            pointer += ProtoBufRuntime._encode_enum(_enum_hash, pointer, bs);
        }
        if (uint256(r.prehash_key) != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            int32 _enum_prehash_key =
                CosmosIcs23V1GlobalEnums.encode_HashOp(r.prehash_key);
            pointer +=
                ProtoBufRuntime._encode_enum(_enum_prehash_key, pointer, bs);
        }
        if (uint256(r.prehash_value) != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            int32 _enum_prehash_value =
                CosmosIcs23V1GlobalEnums.encode_HashOp(r.prehash_value);
            pointer +=
                ProtoBufRuntime._encode_enum(_enum_prehash_value, pointer, bs);
        }
        if (uint256(r.length) != 0) {
            pointer += ProtoBufRuntime._encode_key(
                4, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            int32 _enum_length =
                CosmosIcs23V1GlobalEnums.encode_LengthOp(r.length);
            pointer += ProtoBufRuntime._encode_enum(_enum_length, pointer, bs);
        }
        if (r.prefix.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                5, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.prefix, pointer, bs);
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
        e += 1
            + ProtoBufRuntime._sz_enum(
                CosmosIcs23V1GlobalEnums.encode_HashOp(r.hash)
            );
        e += 1
            + ProtoBufRuntime._sz_enum(
                CosmosIcs23V1GlobalEnums.encode_HashOp(r.prehash_key)
            );
        e += 1
            + ProtoBufRuntime._sz_enum(
                CosmosIcs23V1GlobalEnums.encode_HashOp(r.prehash_value)
            );
        e += 1
            + ProtoBufRuntime._sz_enum(
                CosmosIcs23V1GlobalEnums.encode_LengthOp(r.length)
            );
        e += 1 + ProtoBufRuntime._sz_lendelim(r.prefix.length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (uint256(r.hash) != 0) {
            return false;
        }

        if (uint256(r.prehash_key) != 0) {
            return false;
        }

        if (uint256(r.prehash_value) != 0) {
            return false;
        }

        if (uint256(r.length) != 0) {
            return false;
        }

        if (r.prefix.length != 0) {
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
        output.hash = input.hash;
        output.prehash_key = input.prehash_key;
        output.prehash_value = input.prehash_value;
        output.length = input.length;
        output.prefix = input.prefix;
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

//library CosmosIcs23V1LeafOp

library CosmosIcs23V1InnerOp {
    //struct definition
    struct Data {
        CosmosIcs23V1GlobalEnums.HashOp hash;
        bytes prefix;
        bytes suffix;
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
                pointer += _read_hash(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_prefix(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_suffix(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
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
     * @return The number of bytes decoded
     */
    function _read_hash(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int64 tmp, uint256 sz) = ProtoBufRuntime._decode_enum(p, bs);
        CosmosIcs23V1GlobalEnums.HashOp x =
            CosmosIcs23V1GlobalEnums.decode_HashOp(tmp);
        r.hash = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_prefix(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.prefix = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_suffix(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.suffix = x;
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

        if (uint256(r.hash) != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            int32 _enum_hash = CosmosIcs23V1GlobalEnums.encode_HashOp(r.hash);
            pointer += ProtoBufRuntime._encode_enum(_enum_hash, pointer, bs);
        }
        if (r.prefix.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.prefix, pointer, bs);
        }
        if (r.suffix.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.suffix, pointer, bs);
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
        e += 1
            + ProtoBufRuntime._sz_enum(
                CosmosIcs23V1GlobalEnums.encode_HashOp(r.hash)
            );
        e += 1 + ProtoBufRuntime._sz_lendelim(r.prefix.length);
        e += 1 + ProtoBufRuntime._sz_lendelim(r.suffix.length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (uint256(r.hash) != 0) {
            return false;
        }

        if (r.prefix.length != 0) {
            return false;
        }

        if (r.suffix.length != 0) {
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
        output.hash = input.hash;
        output.prefix = input.prefix;
        output.suffix = input.suffix;
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

//library CosmosIcs23V1InnerOp

library CosmosIcs23V1ProofSpec {
    //struct definition
    struct Data {
        CosmosIcs23V1LeafOp.Data leaf_spec;
        CosmosIcs23V1InnerSpec.Data inner_spec;
        int32 max_depth;
        int32 min_depth;
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
                pointer += _read_leaf_spec(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_inner_spec(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_max_depth(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_min_depth(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
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
     * @return The number of bytes decoded
     */
    function _read_leaf_spec(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1LeafOp.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1LeafOp(p, bs);
        r.leaf_spec = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_inner_spec(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1InnerSpec.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1InnerSpec(p, bs);
        r.inner_spec = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_max_depth(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int32 x, uint256 sz) = ProtoBufRuntime._decode_int32(p, bs);
        r.max_depth = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_min_depth(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int32 x, uint256 sz) = ProtoBufRuntime._decode_int32(p, bs);
        r.min_depth = x;
        return sz;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1LeafOp(
        uint256 p,
        bytes memory bs
    ) internal pure returns (CosmosIcs23V1LeafOp.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1LeafOp.Data memory r,) =
            CosmosIcs23V1LeafOp._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1InnerSpec(
        uint256 p,
        bytes memory bs
    ) internal pure returns (CosmosIcs23V1InnerSpec.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1InnerSpec.Data memory r,) =
            CosmosIcs23V1InnerSpec._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
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

        pointer += ProtoBufRuntime._encode_key(
            1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += CosmosIcs23V1LeafOp._encode_nested(r.leaf_spec, pointer, bs);

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            CosmosIcs23V1InnerSpec._encode_nested(r.inner_spec, pointer, bs);

        if (r.max_depth != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_int32(r.max_depth, pointer, bs);
        }
        if (r.min_depth != 0) {
            pointer += ProtoBufRuntime._encode_key(
                4, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_int32(r.min_depth, pointer, bs);
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
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1LeafOp._estimate(r.leaf_spec)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1InnerSpec._estimate(r.inner_spec)
            );
        e += 1 + ProtoBufRuntime._sz_int32(r.max_depth);
        e += 1 + ProtoBufRuntime._sz_int32(r.min_depth);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.max_depth != 0) {
            return false;
        }

        if (r.min_depth != 0) {
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
        CosmosIcs23V1LeafOp.store(input.leaf_spec, output.leaf_spec);
        CosmosIcs23V1InnerSpec.store(input.inner_spec, output.inner_spec);
        output.max_depth = input.max_depth;
        output.min_depth = input.min_depth;
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

//library CosmosIcs23V1ProofSpec

library CosmosIcs23V1InnerSpec {
    //struct definition
    struct Data {
        int32[] child_order;
        int32 child_size;
        int32 min_prefix_length;
        int32 max_prefix_length;
        bytes empty_child;
        CosmosIcs23V1GlobalEnums.HashOp hash;
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
        uint256[7] memory counters;
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
                if (wireType == ProtoBufRuntime.WireType.LengthDelim) {
                    pointer += _read_packed_repeated_child_order(pointer, bs, r);
                } else {
                    pointer += _read_unpacked_repeated_child_order(
                        pointer, bs, nil(), counters
                    );
                }
            } else if (fieldId == 2) {
                pointer += _read_child_size(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_min_prefix_length(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_max_prefix_length(pointer, bs, r);
            } else if (fieldId == 5) {
                pointer += _read_empty_child(pointer, bs, r);
            } else if (fieldId == 6) {
                pointer += _read_hash(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[1] > 0) {
            require(r.child_order.length == 0);
            r.child_order = new int32[](counters[1]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (
                fieldId == 1 && wireType != ProtoBufRuntime.WireType.LengthDelim
            ) {
                pointer += _read_unpacked_repeated_child_order(
                    pointer, bs, r, counters
                );
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
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
    function _read_unpacked_repeated_child_order(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[7] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (int32 x, uint256 sz) = ProtoBufRuntime._decode_int32(p, bs);
        if (isNil(r)) {
            counters[1] += 1;
        } else {
            r.child_order[r.child_order.length - counters[1]] = x;
            counters[1] -= 1;
        }
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_packed_repeated_child_order(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (uint256 len, uint256 size) = ProtoBufRuntime._decode_varint(p, bs);
        p += size;
        uint256 count =
            ProtoBufRuntime._count_packed_repeated_varint(p, len, bs);
        r.child_order = new int32[](count);
        for (uint256 i; i < count; i++) {
            (int32 x, uint256 sz) = ProtoBufRuntime._decode_int32(p, bs);
            p += sz;
            r.child_order[i] = x;
        }
        return size + len;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_child_size(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int32 x, uint256 sz) = ProtoBufRuntime._decode_int32(p, bs);
        r.child_size = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_min_prefix_length(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int32 x, uint256 sz) = ProtoBufRuntime._decode_int32(p, bs);
        r.min_prefix_length = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_max_prefix_length(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int32 x, uint256 sz) = ProtoBufRuntime._decode_int32(p, bs);
        r.max_prefix_length = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_empty_child(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.empty_child = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_hash(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int64 tmp, uint256 sz) = ProtoBufRuntime._decode_enum(p, bs);
        CosmosIcs23V1GlobalEnums.HashOp x =
            CosmosIcs23V1GlobalEnums.decode_HashOp(tmp);
        r.hash = x;
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
        uint256 i;
        if (r.child_order.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_varint(
                ProtoBufRuntime._estimate_packed_repeated_int32(r.child_order),
                pointer,
                bs
            );
            for (i = 0; i < r.child_order.length; i++) {
                pointer +=
                    ProtoBufRuntime._encode_int32(r.child_order[i], pointer, bs);
            }
        }
        if (r.child_size != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_int32(r.child_size, pointer, bs);
        }
        if (r.min_prefix_length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_int32(r.min_prefix_length, pointer, bs);
        }
        if (r.max_prefix_length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                4, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_int32(r.max_prefix_length, pointer, bs);
        }
        if (r.empty_child.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                5, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.empty_child, pointer, bs);
        }
        if (uint256(r.hash) != 0) {
            pointer += ProtoBufRuntime._encode_key(
                6, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            int32 _enum_hash = CosmosIcs23V1GlobalEnums.encode_HashOp(r.hash);
            pointer += ProtoBufRuntime._encode_enum(_enum_hash, pointer, bs);
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
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                ProtoBufRuntime._estimate_packed_repeated_int32(r.child_order)
            );
        e += 1 + ProtoBufRuntime._sz_int32(r.child_size);
        e += 1 + ProtoBufRuntime._sz_int32(r.min_prefix_length);
        e += 1 + ProtoBufRuntime._sz_int32(r.max_prefix_length);
        e += 1 + ProtoBufRuntime._sz_lendelim(r.empty_child.length);
        e += 1
            + ProtoBufRuntime._sz_enum(
                CosmosIcs23V1GlobalEnums.encode_HashOp(r.hash)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.child_order.length != 0) {
            return false;
        }

        if (r.child_size != 0) {
            return false;
        }

        if (r.min_prefix_length != 0) {
            return false;
        }

        if (r.max_prefix_length != 0) {
            return false;
        }

        if (r.empty_child.length != 0) {
            return false;
        }

        if (uint256(r.hash) != 0) {
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
        output.child_order = input.child_order;
        output.child_size = input.child_size;
        output.min_prefix_length = input.min_prefix_length;
        output.max_prefix_length = input.max_prefix_length;
        output.empty_child = input.empty_child;
        output.hash = input.hash;
    }

    //array helpers for ChildOrder
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addChildOrder(Data memory self, int32 value) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        int32[] memory tmp = new int32[](self.child_order.length + 1);
        for (uint256 i; i < self.child_order.length; i++) {
            tmp[i] = self.child_order[i];
        }
        tmp[self.child_order.length] = value;
        self.child_order = tmp;
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

//library CosmosIcs23V1InnerSpec

library CosmosIcs23V1BatchProof {
    //struct definition
    struct Data {
        CosmosIcs23V1BatchEntry.Data[] entries;
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
        uint256[2] memory counters;
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
                pointer += _read_unpacked_repeated_entries(
                    pointer, bs, nil(), counters
                );
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[1] > 0) {
            require(r.entries.length == 0);
            r.entries = new CosmosIcs23V1BatchEntry.Data[](counters[1]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer +=
                    _read_unpacked_repeated_entries(pointer, bs, r, counters);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
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
    function _read_unpacked_repeated_entries(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[2] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (CosmosIcs23V1BatchEntry.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1BatchEntry(p, bs);
        if (isNil(r)) {
            counters[1] += 1;
        } else {
            r.entries[r.entries.length - counters[1]] = x;
            counters[1] -= 1;
        }
        return sz;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1BatchEntry(
        uint256 p,
        bytes memory bs
    ) internal pure returns (CosmosIcs23V1BatchEntry.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1BatchEntry.Data memory r,) =
            CosmosIcs23V1BatchEntry._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
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
        uint256 i;
        if (r.entries.length != 0) {
            for (i = 0; i < r.entries.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += CosmosIcs23V1BatchEntry._encode_nested(
                    r.entries[i], pointer, bs
                );
            }
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
        uint256 i;
        for (i = 0; i < r.entries.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    CosmosIcs23V1BatchEntry._estimate(r.entries[i])
                );
        }
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.entries.length != 0) {
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
        for (uint256 i1 = 0; i1 < input.entries.length; i1++) {
            output.entries.push(input.entries[i1]);
        }
    }

    //array helpers for Entries
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addEntries(
        Data memory self,
        CosmosIcs23V1BatchEntry.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        CosmosIcs23V1BatchEntry.Data[] memory tmp =
            new CosmosIcs23V1BatchEntry.Data[](self.entries.length + 1);
        for (uint256 i; i < self.entries.length; i++) {
            tmp[i] = self.entries[i];
        }
        tmp[self.entries.length] = value;
        self.entries = tmp;
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

//library CosmosIcs23V1BatchProof

library CosmosIcs23V1BatchEntry {
    //struct definition
    struct Data {
        CosmosIcs23V1ExistenceProof.Data exist;
        CosmosIcs23V1NonExistenceProof.Data nonexist;
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
                pointer += _read_exist(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_nonexist(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
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
     * @return The number of bytes decoded
     */
    function _read_exist(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1ExistenceProof.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1ExistenceProof(p, bs);
        r.exist = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_nonexist(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1NonExistenceProof.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1NonExistenceProof(p, bs);
        r.nonexist = x;
        return sz;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1ExistenceProof(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (CosmosIcs23V1ExistenceProof.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1ExistenceProof.Data memory r,) =
            CosmosIcs23V1ExistenceProof._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1NonExistenceProof(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (CosmosIcs23V1NonExistenceProof.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1NonExistenceProof.Data memory r,) =
            CosmosIcs23V1NonExistenceProof._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
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

        pointer += ProtoBufRuntime._encode_key(
            1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            CosmosIcs23V1ExistenceProof._encode_nested(r.exist, pointer, bs);

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += CosmosIcs23V1NonExistenceProof._encode_nested(
            r.nonexist, pointer, bs
        );

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
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1ExistenceProof._estimate(r.exist)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1NonExistenceProof._estimate(r.nonexist)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory) internal pure returns (bool) {
        return true;
    }

    //store function
    /**
     * @dev Store in-memory struct to storage
     * @param input The in-memory struct
     * @param output The in-storage struct
     */
    function store(Data memory input, Data storage output) internal {
        CosmosIcs23V1ExistenceProof.store(input.exist, output.exist);
        CosmosIcs23V1NonExistenceProof.store(input.nonexist, output.nonexist);
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

//library CosmosIcs23V1BatchEntry

library CosmosIcs23V1CompressedBatchProof {
    //struct definition
    struct Data {
        CosmosIcs23V1CompressedBatchEntry.Data[] entries;
        CosmosIcs23V1InnerOp.Data[] lookup_inners;
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
                pointer += _read_unpacked_repeated_entries(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_lookup_inners(
                    pointer, bs, nil(), counters
                );
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[1] > 0) {
            require(r.entries.length == 0);
            r.entries =
                new CosmosIcs23V1CompressedBatchEntry.Data[](counters[1]);
        }
        if (counters[2] > 0) {
            require(r.lookup_inners.length == 0);
            r.lookup_inners = new CosmosIcs23V1InnerOp.Data[](counters[2]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer +=
                    _read_unpacked_repeated_entries(pointer, bs, r, counters);
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_lookup_inners(
                    pointer, bs, r, counters
                );
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
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
    function _read_unpacked_repeated_entries(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[3] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (CosmosIcs23V1CompressedBatchEntry.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1CompressedBatchEntry(p, bs);
        if (isNil(r)) {
            counters[1] += 1;
        } else {
            r.entries[r.entries.length - counters[1]] = x;
            counters[1] -= 1;
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
    function _read_unpacked_repeated_lookup_inners(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[3] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (CosmosIcs23V1InnerOp.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1InnerOp(p, bs);
        if (isNil(r)) {
            counters[2] += 1;
        } else {
            r.lookup_inners[r.lookup_inners.length - counters[2]] = x;
            counters[2] -= 1;
        }
        return sz;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1CompressedBatchEntry(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (CosmosIcs23V1CompressedBatchEntry.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1CompressedBatchEntry.Data memory r,) =
            CosmosIcs23V1CompressedBatchEntry._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1InnerOp(
        uint256 p,
        bytes memory bs
    ) internal pure returns (CosmosIcs23V1InnerOp.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1InnerOp.Data memory r,) =
            CosmosIcs23V1InnerOp._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
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
        uint256 i;
        if (r.entries.length != 0) {
            for (i = 0; i < r.entries.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += CosmosIcs23V1CompressedBatchEntry._encode_nested(
                    r.entries[i], pointer, bs
                );
            }
        }
        if (r.lookup_inners.length != 0) {
            for (i = 0; i < r.lookup_inners.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += CosmosIcs23V1InnerOp._encode_nested(
                    r.lookup_inners[i], pointer, bs
                );
            }
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
        uint256 i;
        for (i = 0; i < r.entries.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    CosmosIcs23V1CompressedBatchEntry._estimate(r.entries[i])
                );
        }
        for (i = 0; i < r.lookup_inners.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    CosmosIcs23V1InnerOp._estimate(r.lookup_inners[i])
                );
        }
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.entries.length != 0) {
            return false;
        }

        if (r.lookup_inners.length != 0) {
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
        for (uint256 i1 = 0; i1 < input.entries.length; i1++) {
            output.entries.push(input.entries[i1]);
        }

        for (uint256 i2 = 0; i2 < input.lookup_inners.length; i2++) {
            output.lookup_inners.push(input.lookup_inners[i2]);
        }
    }

    //array helpers for Entries
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addEntries(
        Data memory self,
        CosmosIcs23V1CompressedBatchEntry.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        CosmosIcs23V1CompressedBatchEntry.Data[] memory tmp = new CosmosIcs23V1CompressedBatchEntry
            .Data[](self.entries.length + 1);
        for (uint256 i; i < self.entries.length; i++) {
            tmp[i] = self.entries[i];
        }
        tmp[self.entries.length] = value;
        self.entries = tmp;
    }

    //array helpers for LookupInners
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addLookupInners(
        Data memory self,
        CosmosIcs23V1InnerOp.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        CosmosIcs23V1InnerOp.Data[] memory tmp =
            new CosmosIcs23V1InnerOp.Data[](self.lookup_inners.length + 1);
        for (uint256 i; i < self.lookup_inners.length; i++) {
            tmp[i] = self.lookup_inners[i];
        }
        tmp[self.lookup_inners.length] = value;
        self.lookup_inners = tmp;
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

//library CosmosIcs23V1CompressedBatchProof

library CosmosIcs23V1CompressedBatchEntry {
    //struct definition
    struct Data {
        CosmosIcs23V1CompressedExistenceProof.Data exist;
        CosmosIcs23V1CompressedNonExistenceProof.Data nonexist;
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
                pointer += _read_exist(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_nonexist(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
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
     * @return The number of bytes decoded
     */
    function _read_exist(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1CompressedExistenceProof.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1CompressedExistenceProof(p, bs);
        r.exist = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_nonexist(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1CompressedNonExistenceProof.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1CompressedNonExistenceProof(p, bs);
        r.nonexist = x;
        return sz;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1CompressedExistenceProof(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (CosmosIcs23V1CompressedExistenceProof.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1CompressedExistenceProof.Data memory r,) =
            CosmosIcs23V1CompressedExistenceProof._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1CompressedNonExistenceProof(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (CosmosIcs23V1CompressedNonExistenceProof.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1CompressedNonExistenceProof.Data memory r,) =
            CosmosIcs23V1CompressedNonExistenceProof._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
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

        pointer += ProtoBufRuntime._encode_key(
            1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += CosmosIcs23V1CompressedExistenceProof._encode_nested(
            r.exist, pointer, bs
        );

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += CosmosIcs23V1CompressedNonExistenceProof._encode_nested(
            r.nonexist, pointer, bs
        );

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
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1CompressedExistenceProof._estimate(r.exist)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1CompressedNonExistenceProof._estimate(r.nonexist)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory) internal pure returns (bool) {
        return true;
    }

    //store function
    /**
     * @dev Store in-memory struct to storage
     * @param input The in-memory struct
     * @param output The in-storage struct
     */
    function store(Data memory input, Data storage output) internal {
        CosmosIcs23V1CompressedExistenceProof.store(input.exist, output.exist);
        CosmosIcs23V1CompressedNonExistenceProof.store(
            input.nonexist, output.nonexist
        );
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

//library CosmosIcs23V1CompressedBatchEntry

library CosmosIcs23V1CompressedExistenceProof {
    //struct definition
    struct Data {
        bytes key;
        bytes value;
        CosmosIcs23V1LeafOp.Data leaf;
        int32[] path;
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
        uint256[5] memory counters;
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
                pointer += _read_key(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_value(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_leaf(pointer, bs, r);
            } else if (fieldId == 4) {
                if (wireType == ProtoBufRuntime.WireType.LengthDelim) {
                    pointer += _read_packed_repeated_path(pointer, bs, r);
                } else {
                    pointer += _read_unpacked_repeated_path(
                        pointer, bs, nil(), counters
                    );
                }
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[4] > 0) {
            require(r.path.length == 0);
            r.path = new int32[](counters[4]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (
                fieldId == 4 && wireType != ProtoBufRuntime.WireType.LengthDelim
            ) {
                pointer +=
                    _read_unpacked_repeated_path(pointer, bs, r, counters);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
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
     * @return The number of bytes decoded
     */
    function _read_key(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.key = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_value(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.value = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_leaf(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1LeafOp.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1LeafOp(p, bs);
        r.leaf = x;
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
    function _read_unpacked_repeated_path(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[5] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (int32 x, uint256 sz) = ProtoBufRuntime._decode_int32(p, bs);
        if (isNil(r)) {
            counters[4] += 1;
        } else {
            r.path[r.path.length - counters[4]] = x;
            counters[4] -= 1;
        }
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_packed_repeated_path(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (uint256 len, uint256 size) = ProtoBufRuntime._decode_varint(p, bs);
        p += size;
        uint256 count =
            ProtoBufRuntime._count_packed_repeated_varint(p, len, bs);
        r.path = new int32[](count);
        for (uint256 i; i < count; i++) {
            (int32 x, uint256 sz) = ProtoBufRuntime._decode_int32(p, bs);
            p += sz;
            r.path[i] = x;
        }
        return size + len;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1LeafOp(
        uint256 p,
        bytes memory bs
    ) internal pure returns (CosmosIcs23V1LeafOp.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1LeafOp.Data memory r,) =
            CosmosIcs23V1LeafOp._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
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
        uint256 i;
        if (r.key.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.key, pointer, bs);
        }
        if (r.value.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.value, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += CosmosIcs23V1LeafOp._encode_nested(r.leaf, pointer, bs);

        if (r.path.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_varint(
                ProtoBufRuntime._estimate_packed_repeated_int32(r.path),
                pointer,
                bs
            );
            for (i = 0; i < r.path.length; i++) {
                pointer += ProtoBufRuntime._encode_int32(r.path[i], pointer, bs);
            }
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
        e += 1 + ProtoBufRuntime._sz_lendelim(r.key.length);
        e += 1 + ProtoBufRuntime._sz_lendelim(r.value.length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(CosmosIcs23V1LeafOp._estimate(r.leaf));
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                ProtoBufRuntime._estimate_packed_repeated_int32(r.path)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.key.length != 0) {
            return false;
        }

        if (r.value.length != 0) {
            return false;
        }

        if (r.path.length != 0) {
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
        output.key = input.key;
        output.value = input.value;
        CosmosIcs23V1LeafOp.store(input.leaf, output.leaf);
        output.path = input.path;
    }

    //array helpers for Path
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addPath(Data memory self, int32 value) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        int32[] memory tmp = new int32[](self.path.length + 1);
        for (uint256 i; i < self.path.length; i++) {
            tmp[i] = self.path[i];
        }
        tmp[self.path.length] = value;
        self.path = tmp;
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

//library CosmosIcs23V1CompressedExistenceProof

library CosmosIcs23V1CompressedNonExistenceProof {
    //struct definition
    struct Data {
        bytes key;
        CosmosIcs23V1CompressedExistenceProof.Data left;
        CosmosIcs23V1CompressedExistenceProof.Data right;
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
                pointer += _read_key(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_left(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_right(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
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
     * @return The number of bytes decoded
     */
    function _read_key(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.key = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_left(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1CompressedExistenceProof.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1CompressedExistenceProof(p, bs);
        r.left = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_right(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosIcs23V1CompressedExistenceProof.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1CompressedExistenceProof(p, bs);
        r.right = x;
        return sz;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1CompressedExistenceProof(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (CosmosIcs23V1CompressedExistenceProof.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1CompressedExistenceProof.Data memory r,) =
            CosmosIcs23V1CompressedExistenceProof._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
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

        if (r.key.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.key, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += CosmosIcs23V1CompressedExistenceProof._encode_nested(
            r.left, pointer, bs
        );

        pointer += ProtoBufRuntime._encode_key(
            3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += CosmosIcs23V1CompressedExistenceProof._encode_nested(
            r.right, pointer, bs
        );

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
        e += 1 + ProtoBufRuntime._sz_lendelim(r.key.length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1CompressedExistenceProof._estimate(r.left)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosIcs23V1CompressedExistenceProof._estimate(r.right)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.key.length != 0) {
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
        output.key = input.key;
        CosmosIcs23V1CompressedExistenceProof.store(input.left, output.left);
        CosmosIcs23V1CompressedExistenceProof.store(input.right, output.right);
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

//library CosmosIcs23V1CompressedNonExistenceProof

library CosmosIcs23V1GlobalEnums {
    //enum definition
    // Solidity enum definitions
    enum HashOp {
        NO_HASH,
        SHA256,
        SHA512,
        KECCAK,
        RIPEMD160,
        BITCOIN,
        SHA512_256
    }

    // Solidity enum encoder
    function encode_HashOp(HashOp x) internal pure returns (int32) {
        if (x == HashOp.NO_HASH) {
            return 0;
        }

        if (x == HashOp.SHA256) {
            return 1;
        }

        if (x == HashOp.SHA512) {
            return 2;
        }

        if (x == HashOp.KECCAK) {
            return 3;
        }

        if (x == HashOp.RIPEMD160) {
            return 4;
        }

        if (x == HashOp.BITCOIN) {
            return 5;
        }

        if (x == HashOp.SHA512_256) {
            return 6;
        }
        revert();
    }

    // Solidity enum decoder
    function decode_HashOp(int64 x) internal pure returns (HashOp) {
        if (x == 0) {
            return HashOp.NO_HASH;
        }

        if (x == 1) {
            return HashOp.SHA256;
        }

        if (x == 2) {
            return HashOp.SHA512;
        }

        if (x == 3) {
            return HashOp.KECCAK;
        }

        if (x == 4) {
            return HashOp.RIPEMD160;
        }

        if (x == 5) {
            return HashOp.BITCOIN;
        }

        if (x == 6) {
            return HashOp.SHA512_256;
        }
        revert();
    }

    /**
     * @dev The estimator for an packed enum array
     * @return The number of bytes encoded
     */
    function estimate_packed_repeated_HashOp(HashOp[] memory a)
        internal
        pure
        returns (uint256)
    {
        uint256 e = 0;
        for (uint256 i; i < a.length; i++) {
            e += ProtoBufRuntime._sz_enum(encode_HashOp(a[i]));
        }
        return e;
    }

    // Solidity enum definitions
    enum LengthOp {
        NO_PREFIX,
        VAR_PROTO,
        VAR_RLP,
        FIXED32_BIG,
        FIXED32_LITTLE,
        FIXED64_BIG,
        FIXED64_LITTLE,
        REQUIRE_32_BYTES,
        REQUIRE_64_BYTES
    }

    // Solidity enum encoder
    function encode_LengthOp(LengthOp x) internal pure returns (int32) {
        if (x == LengthOp.NO_PREFIX) {
            return 0;
        }

        if (x == LengthOp.VAR_PROTO) {
            return 1;
        }

        if (x == LengthOp.VAR_RLP) {
            return 2;
        }

        if (x == LengthOp.FIXED32_BIG) {
            return 3;
        }

        if (x == LengthOp.FIXED32_LITTLE) {
            return 4;
        }

        if (x == LengthOp.FIXED64_BIG) {
            return 5;
        }

        if (x == LengthOp.FIXED64_LITTLE) {
            return 6;
        }

        if (x == LengthOp.REQUIRE_32_BYTES) {
            return 7;
        }

        if (x == LengthOp.REQUIRE_64_BYTES) {
            return 8;
        }
        revert();
    }

    // Solidity enum decoder
    function decode_LengthOp(int64 x) internal pure returns (LengthOp) {
        if (x == 0) {
            return LengthOp.NO_PREFIX;
        }

        if (x == 1) {
            return LengthOp.VAR_PROTO;
        }

        if (x == 2) {
            return LengthOp.VAR_RLP;
        }

        if (x == 3) {
            return LengthOp.FIXED32_BIG;
        }

        if (x == 4) {
            return LengthOp.FIXED32_LITTLE;
        }

        if (x == 5) {
            return LengthOp.FIXED64_BIG;
        }

        if (x == 6) {
            return LengthOp.FIXED64_LITTLE;
        }

        if (x == 7) {
            return LengthOp.REQUIRE_32_BYTES;
        }

        if (x == 8) {
            return LengthOp.REQUIRE_64_BYTES;
        }
        revert();
    }

    /**
     * @dev The estimator for an packed enum array
     * @return The number of bytes encoded
     */
    function estimate_packed_repeated_LengthOp(LengthOp[] memory a)
        internal
        pure
        returns (uint256)
    {
        uint256 e = 0;
        for (uint256 i; i < a.length; i++) {
            e += ProtoBufRuntime._sz_enum(encode_LengthOp(a[i]));
        }
        return e;
    }
}
//library CosmosIcs23V1GlobalEnums
