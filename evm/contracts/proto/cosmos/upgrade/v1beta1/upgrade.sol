// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.23;

import "../../../ProtoBufRuntime.sol";
import "../../../GoogleProtobufAny.sol";
import "../../../cosmos_proto/cosmos.sol";

library CosmosUpgradeV1beta1Plan {
    //struct definition
    struct Data {
        string name;
        GoogleProtobufTimestamp.Data time;
        int64 height;
        string info;
        GoogleProtobufAny.Data upgraded_client_state;
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
                pointer += _read_time(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_height(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_info(pointer, bs, r);
            } else if (fieldId == 5) {
                pointer += _read_upgraded_client_state(pointer, bs, r);
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
    function _read_time(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (GoogleProtobufTimestamp.Data memory x, uint256 sz) =
            _decode_GoogleProtobufTimestamp(p, bs);
        r.time = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_height(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int64 x, uint256 sz) = ProtoBufRuntime._decode_int64(p, bs);
        r.height = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_info(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.info = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_upgraded_client_state(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (GoogleProtobufAny.Data memory x, uint256 sz) =
            _decode_GoogleProtobufAny(p, bs);
        r.upgraded_client_state = x;
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
    function _decode_GoogleProtobufTimestamp(
        uint256 p,
        bytes memory bs
    ) internal pure returns (GoogleProtobufTimestamp.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (GoogleProtobufTimestamp.Data memory r,) =
            GoogleProtobufTimestamp._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_GoogleProtobufAny(
        uint256 p,
        bytes memory bs
    ) internal pure returns (GoogleProtobufAny.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (GoogleProtobufAny.Data memory r,) =
            GoogleProtobufAny._decode(pointer, bs, sz);
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

        if (bytes(r.name).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.name, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += GoogleProtobufTimestamp._encode_nested(r.time, pointer, bs);

        if (r.height != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_int64(r.height, pointer, bs);
        }
        if (bytes(r.info).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.info, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            5, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += GoogleProtobufAny._encode_nested(
            r.upgraded_client_state, pointer, bs
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.name).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                GoogleProtobufTimestamp._estimate(r.time)
            );
        e += 1 + ProtoBufRuntime._sz_int64(r.height);
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.info).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                GoogleProtobufAny._estimate(r.upgraded_client_state)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.name).length != 0) {
            return false;
        }

        if (r.height != 0) {
            return false;
        }

        if (bytes(r.info).length != 0) {
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
        GoogleProtobufTimestamp.store(input.time, output.time);
        output.height = input.height;
        output.info = input.info;
        GoogleProtobufAny.store(
            input.upgraded_client_state, output.upgraded_client_state
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

//library CosmosUpgradeV1beta1Plan

library CosmosUpgradeV1beta1SoftwareUpgradeProposal {
    //struct definition
    struct Data {
        string title;
        string description;
        CosmosUpgradeV1beta1Plan.Data plan;
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
                pointer += _read_title(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_description(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_plan(pointer, bs, r);
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
    function _read_title(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.title = x;
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
     * @return The number of bytes decoded
     */
    function _read_plan(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (CosmosUpgradeV1beta1Plan.Data memory x, uint256 sz) =
            _decode_CosmosUpgradeV1beta1Plan(p, bs);
        r.plan = x;
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
    function _decode_CosmosUpgradeV1beta1Plan(
        uint256 p,
        bytes memory bs
    ) internal pure returns (CosmosUpgradeV1beta1Plan.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosUpgradeV1beta1Plan.Data memory r,) =
            CosmosUpgradeV1beta1Plan._decode(pointer, bs, sz);
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

        if (bytes(r.title).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.title, pointer, bs);
        }
        if (bytes(r.description).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_string(r.description, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += CosmosUpgradeV1beta1Plan._encode_nested(r.plan, pointer, bs);

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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.title).length);
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.description).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                CosmosUpgradeV1beta1Plan._estimate(r.plan)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.title).length != 0) {
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
        output.title = input.title;
        output.description = input.description;
        CosmosUpgradeV1beta1Plan.store(input.plan, output.plan);
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

//library CosmosUpgradeV1beta1SoftwareUpgradeProposal

library CosmosUpgradeV1beta1CancelSoftwareUpgradeProposal {
    //struct definition
    struct Data {
        string title;
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
                pointer += _read_title(pointer, bs, r);
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
    function _read_title(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.title = x;
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

        if (bytes(r.title).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.title, pointer, bs);
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.title).length);
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.description).length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.title).length != 0) {
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
        output.title = input.title;
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

//library CosmosUpgradeV1beta1CancelSoftwareUpgradeProposal

library CosmosUpgradeV1beta1ModuleVersion {
    //struct definition
    struct Data {
        string name;
        uint64 version;
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
                pointer += _read_version(pointer, bs, r);
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
    function _read_version(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (uint64 x, uint256 sz) = ProtoBufRuntime._decode_uint64(p, bs);
        r.version = x;
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
        if (r.version != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_uint64(r.version, pointer, bs);
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
        e += 1 + ProtoBufRuntime._sz_uint64(r.version);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.name).length != 0) {
            return false;
        }

        if (r.version != 0) {
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
        output.version = input.version;
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
//library CosmosUpgradeV1beta1ModuleVersion
