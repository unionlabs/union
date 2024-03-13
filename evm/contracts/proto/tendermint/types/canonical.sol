pragma solidity ^0.8.23;

import "../../ProtoBufRuntime.sol";
import "../../GoogleProtobufAny.sol";
import "./types.sol";

library TendermintTypesCanonicalBlockID {
    //struct definition
    struct Data {
        bytes hash;
        TendermintTypesCanonicalPartSetHeader.Data part_set_header;
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
                pointer += _read_part_set_header(pointer, bs, r);
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
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
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
    function _read_part_set_header(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (TendermintTypesCanonicalPartSetHeader.Data memory x, uint256 sz) =
            _decode_TendermintTypesCanonicalPartSetHeader(p, bs);
        r.part_set_header = x;
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
    function _decode_TendermintTypesCanonicalPartSetHeader(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (TendermintTypesCanonicalPartSetHeader.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (TendermintTypesCanonicalPartSetHeader.Data memory r,) =
            TendermintTypesCanonicalPartSetHeader._decode(pointer, bs, sz);
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

        if (r.hash.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.hash, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += TendermintTypesCanonicalPartSetHeader._encode_nested(
            r.part_set_header, pointer, bs
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
        e += 1 + ProtoBufRuntime._sz_lendelim(r.hash.length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                TendermintTypesCanonicalPartSetHeader._estimate(r.part_set_header)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.hash.length != 0) {
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
        TendermintTypesCanonicalPartSetHeader.store(
            input.part_set_header, output.part_set_header
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

//library TendermintTypesCanonicalBlockID

library TendermintTypesCanonicalPartSetHeader {
    //struct definition
    struct Data {
        uint32 total;
        bytes hash;
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
                pointer += _read_total(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_hash(pointer, bs, r);
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
    function _read_total(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (uint32 x, uint256 sz) = ProtoBufRuntime._decode_uint32(p, bs);
        r.total = x;
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
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
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

        if (r.total != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_uint32(r.total, pointer, bs);
        }
        if (r.hash.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.hash, pointer, bs);
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
        e += 1 + ProtoBufRuntime._sz_uint32(r.total);
        e += 1 + ProtoBufRuntime._sz_lendelim(r.hash.length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.total != 0) {
            return false;
        }

        if (r.hash.length != 0) {
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
        output.total = input.total;
        output.hash = input.hash;
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

//library TendermintTypesCanonicalPartSetHeader

library TendermintTypesCanonicalProposal {
    //struct definition
    struct Data {
        TendermintTypesTypesGlobalEnums.SignedMsgType type_;
        int64 height;
        int64 round;
        int64 pol_round;
        TendermintTypesCanonicalBlockID.Data block_id;
        GoogleProtobufTimestamp.Data timestamp;
        string chain_id;
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
                pointer += _read_type(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_height(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_round(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_pol_round(pointer, bs, r);
            } else if (fieldId == 5) {
                pointer += _read_block_id(pointer, bs, r);
            } else if (fieldId == 6) {
                pointer += _read_timestamp(pointer, bs, r);
            } else if (fieldId == 7) {
                pointer += _read_chain_id(pointer, bs, r);
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
    function _read_type(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int64 tmp, uint256 sz) = ProtoBufRuntime._decode_enum(p, bs);
        TendermintTypesTypesGlobalEnums.SignedMsgType x =
            TendermintTypesTypesGlobalEnums.decode_SignedMsgType(tmp);
        r.type_ = x;
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
        (int64 x, uint256 sz) = ProtoBufRuntime._decode_sfixed64(p, bs);
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
    function _read_round(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int64 x, uint256 sz) = ProtoBufRuntime._decode_sfixed64(p, bs);
        r.round = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_pol_round(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int64 x, uint256 sz) = ProtoBufRuntime._decode_int64(p, bs);
        r.pol_round = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_block_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (TendermintTypesCanonicalBlockID.Data memory x, uint256 sz) =
            _decode_TendermintTypesCanonicalBlockID(p, bs);
        r.block_id = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_timestamp(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (GoogleProtobufTimestamp.Data memory x, uint256 sz) =
            _decode_GoogleProtobufTimestamp(p, bs);
        r.timestamp = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_chain_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.chain_id = x;
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
    function _decode_TendermintTypesCanonicalBlockID(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (TendermintTypesCanonicalBlockID.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (TendermintTypesCanonicalBlockID.Data memory r,) =
            TendermintTypesCanonicalBlockID._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

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

        if (uint256(r.type_) != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            int32 _enum_type =
                TendermintTypesTypesGlobalEnums.encode_SignedMsgType(r.type_);
            pointer += ProtoBufRuntime._encode_enum(_enum_type, pointer, bs);
        }
        if (r.height != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.Fixed64, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_sfixed64(r.height, pointer, bs);
        }
        if (r.round != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.Fixed64, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_sfixed64(r.round, pointer, bs);
        }
        if (r.pol_round != 0) {
            pointer += ProtoBufRuntime._encode_key(
                4, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_int64(r.pol_round, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            5, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += TendermintTypesCanonicalBlockID._encode_nested(
            r.block_id, pointer, bs
        );

        pointer += ProtoBufRuntime._encode_key(
            6, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            GoogleProtobufTimestamp._encode_nested(r.timestamp, pointer, bs);

        if (bytes(r.chain_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                7, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.chain_id, pointer, bs);
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
                TendermintTypesTypesGlobalEnums.encode_SignedMsgType(r.type_)
            );
        e += 1 + 8;
        e += 1 + 8;
        e += 1 + ProtoBufRuntime._sz_int64(r.pol_round);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                TendermintTypesCanonicalBlockID._estimate(r.block_id)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                GoogleProtobufTimestamp._estimate(r.timestamp)
            );
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.chain_id).length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (uint256(r.type_) != 0) {
            return false;
        }

        if (r.height != 0) {
            return false;
        }

        if (r.round != 0) {
            return false;
        }

        if (r.pol_round != 0) {
            return false;
        }

        if (bytes(r.chain_id).length != 0) {
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
        output.type_ = input.type_;
        output.height = input.height;
        output.round = input.round;
        output.pol_round = input.pol_round;
        TendermintTypesCanonicalBlockID.store(input.block_id, output.block_id);
        GoogleProtobufTimestamp.store(input.timestamp, output.timestamp);
        output.chain_id = input.chain_id;
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

//library TendermintTypesCanonicalProposal

library TendermintTypesCanonicalVote {
    //struct definition
    struct Data {
        TendermintTypesTypesGlobalEnums.SignedMsgType type_;
        int64 height;
        int64 round;
        TendermintTypesCanonicalBlockID.Data block_id;
        string chain_id;
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
                pointer += _read_type(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_height(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_round(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_block_id(pointer, bs, r);
            } else if (fieldId == 6) {
                pointer += _read_chain_id(pointer, bs, r);
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
    function _read_type(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int64 tmp, uint256 sz) = ProtoBufRuntime._decode_enum(p, bs);
        TendermintTypesTypesGlobalEnums.SignedMsgType x =
            TendermintTypesTypesGlobalEnums.decode_SignedMsgType(tmp);
        r.type_ = x;
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
        (int64 x, uint256 sz) = ProtoBufRuntime._decode_sfixed64(p, bs);
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
    function _read_round(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (int64 x, uint256 sz) = ProtoBufRuntime._decode_sfixed64(p, bs);
        r.round = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_block_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (TendermintTypesCanonicalBlockID.Data memory x, uint256 sz) =
            _decode_TendermintTypesCanonicalBlockID(p, bs);
        r.block_id = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_chain_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.chain_id = x;
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
    function _decode_TendermintTypesCanonicalBlockID(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (TendermintTypesCanonicalBlockID.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (TendermintTypesCanonicalBlockID.Data memory r,) =
            TendermintTypesCanonicalBlockID._decode(pointer, bs, sz);
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

        if (uint256(r.type_) != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            int32 _enum_type =
                TendermintTypesTypesGlobalEnums.encode_SignedMsgType(r.type_);
            pointer += ProtoBufRuntime._encode_enum(_enum_type, pointer, bs);
        }
        if (r.height != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.Fixed64, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_sfixed64(r.height, pointer, bs);
        }
        if (r.round != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.Fixed64, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_sfixed64(r.round, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += TendermintTypesCanonicalBlockID._encode_nested(
            r.block_id, pointer, bs
        );

        if (bytes(r.chain_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                6, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.chain_id, pointer, bs);
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
                TendermintTypesTypesGlobalEnums.encode_SignedMsgType(r.type_)
            );
        e += 1 + 8;
        e += 1 + 8;
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                TendermintTypesCanonicalBlockID._estimate(r.block_id)
            );
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.chain_id).length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (uint256(r.type_) != 0) {
            return false;
        }

        if (r.height != 0) {
            return false;
        }

        if (r.round != 0) {
            return false;
        }

        if (bytes(r.chain_id).length != 0) {
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
        output.type_ = input.type_;
        output.height = input.height;
        output.round = input.round;
        TendermintTypesCanonicalBlockID.store(input.block_id, output.block_id);
        output.chain_id = input.chain_id;
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
//library TendermintTypesCanonicalVote
