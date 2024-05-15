// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.23;

import "../ProtoBufRuntime.sol";
import "../GoogleProtobufAny.sol";

library Cosmos_protoInterfaceDescriptor {
    //struct definition
    struct Data {
        string name;
        string description;
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
                pointer += _read_name(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_description(pointer, bs, r);
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
    function _read_name(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.name = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_description(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.description = x;
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

        if (bytes(r.name).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.name, pointer, bs);
        }
        if (bytes(r.description).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_string(r.description, pointer, bs);
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.name).length);
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.description).length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.name).length != 0) {
            return false;
        }

        if (bytes(r.description).length != 0) {
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
        output.name = input.name;
        output.description = input.description;
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

//library Cosmos_protoInterfaceDescriptor

library Cosmos_protoScalarDescriptor {
    //struct definition
    struct Data {
        string name;
        string description;
        CosmosProtoCosmosProtoGlobalEnums.ScalarType[] field_type;
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
        uint256[4] memory counters;
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
                pointer += _read_name(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_description(pointer, bs, r);
            } else if (fieldId == 3) {
                if (wireType == ProtoBufRuntime.WireType.LengthDelim) {
                    pointer += _read_packed_repeated_field_type(pointer, bs, r);
                } else {
                    pointer += _read_unpacked_repeated_field_type(
                        pointer, bs, nil(), counters
                    );
                }
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[3] > 0) {
            require(r.field_type.length == 0);
            r.field_type =
                new CosmosProtoCosmosProtoGlobalEnums.ScalarType[](counters[3]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (
                fieldId == 3 && wireType != ProtoBufRuntime.WireType.LengthDelim
            ) {
                pointer +=
                    _read_unpacked_repeated_field_type(pointer, bs, r, counters);
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
    function _read_name(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.name = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_description(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.description = x;
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
    function _read_unpacked_repeated_field_type(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[4] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (int64 tmp, uint256 sz) = ProtoBufRuntime._decode_enum(p, bs);
        CosmosProtoCosmosProtoGlobalEnums.ScalarType x =
            CosmosProtoCosmosProtoGlobalEnums.decode_ScalarType(tmp);
        if (isNil(r)) {
            counters[3] += 1;
        } else {
            r.field_type[r.field_type.length - counters[3]] = x;
            counters[3] -= 1;
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
    function _read_packed_repeated_field_type(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (uint256 len, uint256 size) = ProtoBufRuntime._decode_varint(p, bs);
        p += size;
        uint256 count =
            ProtoBufRuntime._count_packed_repeated_varint(p, len, bs);
        r.field_type = new CosmosProtoCosmosProtoGlobalEnums.ScalarType[](count);
        for (uint256 i; i < count; i++) {
            (int64 tmp, uint256 sz) = ProtoBufRuntime._decode_enum(p, bs);
            CosmosProtoCosmosProtoGlobalEnums.ScalarType x =
                CosmosProtoCosmosProtoGlobalEnums.decode_ScalarType(tmp);
            p += sz;
            r.field_type[i] = x;
        }
        return size + len;
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
        if (bytes(r.name).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.name, pointer, bs);
        }
        if (bytes(r.description).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_string(r.description, pointer, bs);
        }
        if (r.field_type.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_varint(
                CosmosProtoCosmosProtoGlobalEnums
                    .estimate_packed_repeated_ScalarType(r.field_type),
                pointer,
                bs
            );
            for (i = 0; i < r.field_type.length; i++) {
                int32 _enum_field_type = CosmosProtoCosmosProtoGlobalEnums
                    .encode_ScalarType(r.field_type[i]);
                pointer +=
                    ProtoBufRuntime._encode_enum(_enum_field_type, pointer, bs);
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.name).length);
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.description).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosProtoCosmosProtoGlobalEnums
                    .estimate_packed_repeated_ScalarType(r.field_type)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.name).length != 0) {
            return false;
        }

        if (bytes(r.description).length != 0) {
            return false;
        }

        if (r.field_type.length != 0) {
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
        output.name = input.name;
        output.description = input.description;
        output.field_type = input.field_type;
    }

    //array helpers for FieldType
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addFieldType(
        Data memory self,
        CosmosProtoCosmosProtoGlobalEnums.ScalarType value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        CosmosProtoCosmosProtoGlobalEnums.ScalarType[] memory tmp = new CosmosProtoCosmosProtoGlobalEnums
            .ScalarType[](self.field_type.length + 1);
        for (uint256 i; i < self.field_type.length; i++) {
            tmp[i] = self.field_type[i];
        }
        tmp[self.field_type.length] = value;
        self.field_type = tmp;
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

//library Cosmos_protoScalarDescriptor

library CosmosProtoCosmosProtoGlobalEnums {
    //enum definition
    // Solidity enum definitions
    enum ScalarType {
        SCALAR_TYPE_UNSPECIFIED,
        SCALAR_TYPE_STRING,
        SCALAR_TYPE_BYTES
    }

    // Solidity enum encoder
    function encode_ScalarType(ScalarType x) internal pure returns (int32) {
        if (x == ScalarType.SCALAR_TYPE_UNSPECIFIED) {
            return 0;
        }

        if (x == ScalarType.SCALAR_TYPE_STRING) {
            return 1;
        }

        if (x == ScalarType.SCALAR_TYPE_BYTES) {
            return 2;
        }
        revert();
    }

    // Solidity enum decoder
    function decode_ScalarType(int64 x) internal pure returns (ScalarType) {
        if (x == 0) {
            return ScalarType.SCALAR_TYPE_UNSPECIFIED;
        }

        if (x == 1) {
            return ScalarType.SCALAR_TYPE_STRING;
        }

        if (x == 2) {
            return ScalarType.SCALAR_TYPE_BYTES;
        }
        revert();
    }

    /**
     * @dev The estimator for an packed enum array
     * @return The number of bytes encoded
     */
    function estimate_packed_repeated_ScalarType(ScalarType[] memory a)
        internal
        pure
        returns (uint256)
    {
        uint256 e = 0;
        for (uint256 i; i < a.length; i++) {
            e += ProtoBufRuntime._sz_enum(encode_ScalarType(a[i]));
        }
        return e;
    }
}
//library CosmosProtoCosmosProtoGlobalEnums
