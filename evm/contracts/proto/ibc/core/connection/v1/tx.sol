pragma solidity ^0.8.23;

import "../../../../ProtoBufRuntime.sol";
import "../../../../GoogleProtobufAny.sol";

import "../../client/v1/client.sol";
import "./connection.sol";

library IbcCoreConnectionV1MsgConnectionOpenInit {
    //struct definition
    struct Data {
        string client_id;
        IbcCoreConnectionV1Counterparty.Data counterparty;
        IbcCoreConnectionV1Version.Data version;
        uint64 delay_period;
        string signer;
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
                pointer += _read_client_id(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_counterparty(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_version(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_delay_period(pointer, bs, r);
            } else if (fieldId == 5) {
                pointer += _read_signer(pointer, bs, r);
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
    function _read_client_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.client_id = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_counterparty(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreConnectionV1Counterparty.Data memory x, uint256 sz) =
            _decode_IbcCoreConnectionV1Counterparty(p, bs);
        r.counterparty = x;
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
        (IbcCoreConnectionV1Version.Data memory x, uint256 sz) =
            _decode_IbcCoreConnectionV1Version(p, bs);
        r.version = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_delay_period(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (uint64 x, uint256 sz) = ProtoBufRuntime._decode_uint64(p, bs);
        r.delay_period = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_signer(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.signer = x;
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
    function _decode_IbcCoreConnectionV1Counterparty(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcCoreConnectionV1Counterparty.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreConnectionV1Counterparty.Data memory r,) =
            IbcCoreConnectionV1Counterparty._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreConnectionV1Version(
        uint256 p,
        bytes memory bs
    ) internal pure returns (IbcCoreConnectionV1Version.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreConnectionV1Version.Data memory r,) =
            IbcCoreConnectionV1Version._decode(pointer, bs, sz);
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

        if (bytes(r.client_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.client_id, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += IbcCoreConnectionV1Counterparty._encode_nested(
            r.counterparty, pointer, bs
        );

        pointer += ProtoBufRuntime._encode_key(
            3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            IbcCoreConnectionV1Version._encode_nested(r.version, pointer, bs);

        if (r.delay_period != 0) {
            pointer += ProtoBufRuntime._encode_key(
                4, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_uint64(r.delay_period, pointer, bs);
        }
        if (bytes(r.signer).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                5, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.signer, pointer, bs);
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.client_id).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreConnectionV1Counterparty._estimate(r.counterparty)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreConnectionV1Version._estimate(r.version)
            );
        e += 1 + ProtoBufRuntime._sz_uint64(r.delay_period);
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.signer).length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.client_id).length != 0) {
            return false;
        }

        if (r.delay_period != 0) {
            return false;
        }

        if (bytes(r.signer).length != 0) {
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
        output.client_id = input.client_id;
        IbcCoreConnectionV1Counterparty.store(
            input.counterparty, output.counterparty
        );
        IbcCoreConnectionV1Version.store(input.version, output.version);
        output.delay_period = input.delay_period;
        output.signer = input.signer;
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

//library IbcCoreConnectionV1MsgConnectionOpenInit

library IbcCoreConnectionV1MsgConnectionOpenInitResponse {
    //struct definition
    struct Data {
        bool x;
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
        }
        return (r, sz);
    }

    // field readers

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
     * @param p The offset of bytes array to start decode
     * @return The number of bytes encoded
     */
    function _encode(
        Data memory,
        uint256 p,
        bytes memory
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 pointer = p;

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
     * @return The number of bytes encoded in estimation
     */
    function _estimate(Data memory /* r */ ) internal pure returns (uint256) {
        uint256 e;
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
    function store(Data memory input, Data storage output) internal {}

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

//library IbcCoreConnectionV1MsgConnectionOpenInitResponse

library IbcCoreConnectionV1MsgConnectionOpenTry {
    //struct definition
    struct Data {
        string client_id;
        string previous_connection_id;
        GoogleProtobufAny.Data client_state;
        IbcCoreConnectionV1Counterparty.Data counterparty;
        uint64 delay_period;
        IbcCoreConnectionV1Version.Data[] counterparty_versions;
        IbcCoreClientV1Height.Data proof_height;
        bytes proof_init;
        bytes proof_client;
        bytes proof_consensus;
        IbcCoreClientV1Height.Data consensus_height;
        string signer;
        bytes host_consensus_state_proof;
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
        uint256[14] memory counters;
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
                pointer += _read_client_id(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_previous_connection_id(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_client_state(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_counterparty(pointer, bs, r);
            } else if (fieldId == 5) {
                pointer += _read_delay_period(pointer, bs, r);
            } else if (fieldId == 6) {
                pointer += _read_unpacked_repeated_counterparty_versions(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 7) {
                pointer += _read_proof_height(pointer, bs, r);
            } else if (fieldId == 8) {
                pointer += _read_proof_init(pointer, bs, r);
            } else if (fieldId == 9) {
                pointer += _read_proof_client(pointer, bs, r);
            } else if (fieldId == 10) {
                pointer += _read_proof_consensus(pointer, bs, r);
            } else if (fieldId == 11) {
                pointer += _read_consensus_height(pointer, bs, r);
            } else if (fieldId == 12) {
                pointer += _read_signer(pointer, bs, r);
            } else if (fieldId == 13) {
                pointer += _read_host_consensus_state_proof(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[6] > 0) {
            require(r.counterparty_versions.length == 0);
            r.counterparty_versions =
                new IbcCoreConnectionV1Version.Data[](counters[6]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 6) {
                pointer += _read_unpacked_repeated_counterparty_versions(
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
     * @return The number of bytes decoded
     */
    function _read_client_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.client_id = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_previous_connection_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.previous_connection_id = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_client_state(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (GoogleProtobufAny.Data memory x, uint256 sz) =
            _decode_GoogleProtobufAny(p, bs);
        r.client_state = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_counterparty(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreConnectionV1Counterparty.Data memory x, uint256 sz) =
            _decode_IbcCoreConnectionV1Counterparty(p, bs);
        r.counterparty = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_delay_period(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (uint64 x, uint256 sz) = ProtoBufRuntime._decode_uint64(p, bs);
        r.delay_period = x;
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
    function _read_unpacked_repeated_counterparty_versions(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[14] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreConnectionV1Version.Data memory x, uint256 sz) =
            _decode_IbcCoreConnectionV1Version(p, bs);
        if (isNil(r)) {
            counters[6] += 1;
        } else {
            r.counterparty_versions[r.counterparty_versions.length - counters[6]]
            = x;
            counters[6] -= 1;
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
    function _read_proof_height(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreClientV1Height.Data memory x, uint256 sz) =
            _decode_IbcCoreClientV1Height(p, bs);
        r.proof_height = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_proof_init(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.proof_init = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_proof_client(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.proof_client = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_proof_consensus(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.proof_consensus = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_consensus_height(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreClientV1Height.Data memory x, uint256 sz) =
            _decode_IbcCoreClientV1Height(p, bs);
        r.consensus_height = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_signer(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.signer = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_host_consensus_state_proof(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.host_consensus_state_proof = x;
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

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreConnectionV1Counterparty(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcCoreConnectionV1Counterparty.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreConnectionV1Counterparty.Data memory r,) =
            IbcCoreConnectionV1Counterparty._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreConnectionV1Version(
        uint256 p,
        bytes memory bs
    ) internal pure returns (IbcCoreConnectionV1Version.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreConnectionV1Version.Data memory r,) =
            IbcCoreConnectionV1Version._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreClientV1Height(
        uint256 p,
        bytes memory bs
    ) internal pure returns (IbcCoreClientV1Height.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreClientV1Height.Data memory r,) =
            IbcCoreClientV1Height._decode(pointer, bs, sz);
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
        if (bytes(r.client_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.client_id, pointer, bs);
        }
        if (bytes(r.previous_connection_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(
                r.previous_connection_id, pointer, bs
            );
        }

        pointer += ProtoBufRuntime._encode_key(
            3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += GoogleProtobufAny._encode_nested(r.client_state, pointer, bs);

        pointer += ProtoBufRuntime._encode_key(
            4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += IbcCoreConnectionV1Counterparty._encode_nested(
            r.counterparty, pointer, bs
        );

        if (r.delay_period != 0) {
            pointer += ProtoBufRuntime._encode_key(
                5, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_uint64(r.delay_period, pointer, bs);
        }
        if (r.counterparty_versions.length != 0) {
            for (i = 0; i < r.counterparty_versions.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    6, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreConnectionV1Version._encode_nested(
                    r.counterparty_versions[i], pointer, bs
                );
            }
        }

        pointer += ProtoBufRuntime._encode_key(
            7, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            IbcCoreClientV1Height._encode_nested(r.proof_height, pointer, bs);

        if (r.proof_init.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                8, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.proof_init, pointer, bs);
        }
        if (r.proof_client.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                9, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_bytes(r.proof_client, pointer, bs);
        }
        if (r.proof_consensus.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                10, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_bytes(r.proof_consensus, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            11, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += IbcCoreClientV1Height._encode_nested(
            r.consensus_height, pointer, bs
        );

        if (bytes(r.signer).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                12, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.signer, pointer, bs);
        }
        if (r.host_consensus_state_proof.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                13, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(
                r.host_consensus_state_proof, pointer, bs
            );
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.client_id).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(bytes(r.previous_connection_id).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                GoogleProtobufAny._estimate(r.client_state)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreConnectionV1Counterparty._estimate(r.counterparty)
            );
        e += 1 + ProtoBufRuntime._sz_uint64(r.delay_period);
        for (i = 0; i < r.counterparty_versions.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreConnectionV1Version._estimate(r.counterparty_versions[i])
                );
        }
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreClientV1Height._estimate(r.proof_height)
            );
        e += 1 + ProtoBufRuntime._sz_lendelim(r.proof_init.length);
        e += 1 + ProtoBufRuntime._sz_lendelim(r.proof_client.length);
        e += 1 + ProtoBufRuntime._sz_lendelim(r.proof_consensus.length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreClientV1Height._estimate(r.consensus_height)
            );
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.signer).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(r.host_consensus_state_proof.length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.client_id).length != 0) {
            return false;
        }

        if (bytes(r.previous_connection_id).length != 0) {
            return false;
        }

        if (r.delay_period != 0) {
            return false;
        }

        if (r.counterparty_versions.length != 0) {
            return false;
        }

        if (r.proof_init.length != 0) {
            return false;
        }

        if (r.proof_client.length != 0) {
            return false;
        }

        if (r.proof_consensus.length != 0) {
            return false;
        }

        if (bytes(r.signer).length != 0) {
            return false;
        }

        if (r.host_consensus_state_proof.length != 0) {
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
        output.client_id = input.client_id;
        output.previous_connection_id = input.previous_connection_id;
        GoogleProtobufAny.store(input.client_state, output.client_state);
        IbcCoreConnectionV1Counterparty.store(
            input.counterparty, output.counterparty
        );
        output.delay_period = input.delay_period;

        for (uint256 i6 = 0; i6 < input.counterparty_versions.length; i6++) {
            output.counterparty_versions.push(input.counterparty_versions[i6]);
        }

        IbcCoreClientV1Height.store(input.proof_height, output.proof_height);
        output.proof_init = input.proof_init;
        output.proof_client = input.proof_client;
        output.proof_consensus = input.proof_consensus;
        IbcCoreClientV1Height.store(
            input.consensus_height, output.consensus_height
        );
        output.signer = input.signer;
        output.host_consensus_state_proof = input.host_consensus_state_proof;
    }

    //array helpers for CounterpartyVersions
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addCounterpartyVersions(
        Data memory self,
        IbcCoreConnectionV1Version.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreConnectionV1Version.Data[] memory tmp = new IbcCoreConnectionV1Version
            .Data[](self.counterparty_versions.length + 1);
        for (uint256 i; i < self.counterparty_versions.length; i++) {
            tmp[i] = self.counterparty_versions[i];
        }
        tmp[self.counterparty_versions.length] = value;
        self.counterparty_versions = tmp;
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

//library IbcCoreConnectionV1MsgConnectionOpenTry

library IbcCoreConnectionV1MsgConnectionOpenTryResponse {
    //struct definition
    struct Data {
        bool x;
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
        }
        return (r, sz);
    }

    // field readers

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
     * @param p The offset of bytes array to start decode
     * @return The number of bytes encoded
     */
    function _encode(
        Data memory,
        uint256 p,
        bytes memory
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 pointer = p;

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
     * @return The number of bytes encoded in estimation
     */
    function _estimate(Data memory /* r */ ) internal pure returns (uint256) {
        uint256 e;
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
    function store(Data memory input, Data storage output) internal {}

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

//library IbcCoreConnectionV1MsgConnectionOpenTryResponse

library IbcCoreConnectionV1MsgConnectionOpenAck {
    //struct definition
    struct Data {
        string connection_id;
        string counterparty_connection_id;
        IbcCoreConnectionV1Version.Data version;
        GoogleProtobufAny.Data client_state;
        IbcCoreClientV1Height.Data proof_height;
        bytes proof_try;
        bytes proof_client;
        bytes proof_consensus;
        IbcCoreClientV1Height.Data consensus_height;
        string signer;
        bytes host_consensus_state_proof;
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
                pointer += _read_connection_id(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_counterparty_connection_id(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_version(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_client_state(pointer, bs, r);
            } else if (fieldId == 5) {
                pointer += _read_proof_height(pointer, bs, r);
            } else if (fieldId == 6) {
                pointer += _read_proof_try(pointer, bs, r);
            } else if (fieldId == 7) {
                pointer += _read_proof_client(pointer, bs, r);
            } else if (fieldId == 8) {
                pointer += _read_proof_consensus(pointer, bs, r);
            } else if (fieldId == 9) {
                pointer += _read_consensus_height(pointer, bs, r);
            } else if (fieldId == 10) {
                pointer += _read_signer(pointer, bs, r);
            } else if (fieldId == 11) {
                pointer += _read_host_consensus_state_proof(pointer, bs, r);
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
    function _read_connection_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.connection_id = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_counterparty_connection_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.counterparty_connection_id = x;
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
        (IbcCoreConnectionV1Version.Data memory x, uint256 sz) =
            _decode_IbcCoreConnectionV1Version(p, bs);
        r.version = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_client_state(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (GoogleProtobufAny.Data memory x, uint256 sz) =
            _decode_GoogleProtobufAny(p, bs);
        r.client_state = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_proof_height(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreClientV1Height.Data memory x, uint256 sz) =
            _decode_IbcCoreClientV1Height(p, bs);
        r.proof_height = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_proof_try(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.proof_try = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_proof_client(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.proof_client = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_proof_consensus(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.proof_consensus = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_consensus_height(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreClientV1Height.Data memory x, uint256 sz) =
            _decode_IbcCoreClientV1Height(p, bs);
        r.consensus_height = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_signer(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.signer = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_host_consensus_state_proof(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.host_consensus_state_proof = x;
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
    function _decode_IbcCoreConnectionV1Version(
        uint256 p,
        bytes memory bs
    ) internal pure returns (IbcCoreConnectionV1Version.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreConnectionV1Version.Data memory r,) =
            IbcCoreConnectionV1Version._decode(pointer, bs, sz);
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

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreClientV1Height(
        uint256 p,
        bytes memory bs
    ) internal pure returns (IbcCoreClientV1Height.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreClientV1Height.Data memory r,) =
            IbcCoreClientV1Height._decode(pointer, bs, sz);
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

        if (bytes(r.connection_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_string(r.connection_id, pointer, bs);
        }
        if (bytes(r.counterparty_connection_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(
                r.counterparty_connection_id, pointer, bs
            );
        }

        pointer += ProtoBufRuntime._encode_key(
            3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            IbcCoreConnectionV1Version._encode_nested(r.version, pointer, bs);

        pointer += ProtoBufRuntime._encode_key(
            4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += GoogleProtobufAny._encode_nested(r.client_state, pointer, bs);

        pointer += ProtoBufRuntime._encode_key(
            5, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            IbcCoreClientV1Height._encode_nested(r.proof_height, pointer, bs);

        if (r.proof_try.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                6, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.proof_try, pointer, bs);
        }
        if (r.proof_client.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                7, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_bytes(r.proof_client, pointer, bs);
        }
        if (r.proof_consensus.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                8, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_bytes(r.proof_consensus, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            9, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += IbcCoreClientV1Height._encode_nested(
            r.consensus_height, pointer, bs
        );

        if (bytes(r.signer).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                10, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.signer, pointer, bs);
        }
        if (r.host_consensus_state_proof.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                11, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(
                r.host_consensus_state_proof, pointer, bs
            );
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.connection_id).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                bytes(r.counterparty_connection_id).length
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreConnectionV1Version._estimate(r.version)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                GoogleProtobufAny._estimate(r.client_state)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreClientV1Height._estimate(r.proof_height)
            );
        e += 1 + ProtoBufRuntime._sz_lendelim(r.proof_try.length);
        e += 1 + ProtoBufRuntime._sz_lendelim(r.proof_client.length);
        e += 1 + ProtoBufRuntime._sz_lendelim(r.proof_consensus.length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreClientV1Height._estimate(r.consensus_height)
            );
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.signer).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(r.host_consensus_state_proof.length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.connection_id).length != 0) {
            return false;
        }

        if (bytes(r.counterparty_connection_id).length != 0) {
            return false;
        }

        if (r.proof_try.length != 0) {
            return false;
        }

        if (r.proof_client.length != 0) {
            return false;
        }

        if (r.proof_consensus.length != 0) {
            return false;
        }

        if (bytes(r.signer).length != 0) {
            return false;
        }

        if (r.host_consensus_state_proof.length != 0) {
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
        output.connection_id = input.connection_id;
        output.counterparty_connection_id = input.counterparty_connection_id;
        IbcCoreConnectionV1Version.store(input.version, output.version);
        GoogleProtobufAny.store(input.client_state, output.client_state);
        IbcCoreClientV1Height.store(input.proof_height, output.proof_height);
        output.proof_try = input.proof_try;
        output.proof_client = input.proof_client;
        output.proof_consensus = input.proof_consensus;
        IbcCoreClientV1Height.store(
            input.consensus_height, output.consensus_height
        );
        output.signer = input.signer;
        output.host_consensus_state_proof = input.host_consensus_state_proof;
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

//library IbcCoreConnectionV1MsgConnectionOpenAck

library IbcCoreConnectionV1MsgConnectionOpenAckResponse {
    //struct definition
    struct Data {
        bool x;
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
        }
        return (r, sz);
    }

    // field readers

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
     * @param p The offset of bytes array to start decode
     * @return The number of bytes encoded
     */
    function _encode(
        Data memory,
        uint256 p,
        bytes memory
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 pointer = p;

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
     * @return The number of bytes encoded in estimation
     */
    function _estimate(Data memory /* r */ ) internal pure returns (uint256) {
        uint256 e;
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
    function store(Data memory input, Data storage output) internal {}

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

//library IbcCoreConnectionV1MsgConnectionOpenAckResponse

library IbcCoreConnectionV1MsgConnectionOpenConfirm {
    //struct definition
    struct Data {
        string connection_id;
        bytes proof_ack;
        IbcCoreClientV1Height.Data proof_height;
        string signer;
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
                pointer += _read_connection_id(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_proof_ack(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_proof_height(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_signer(pointer, bs, r);
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
    function _read_connection_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.connection_id = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_proof_ack(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.proof_ack = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_proof_height(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreClientV1Height.Data memory x, uint256 sz) =
            _decode_IbcCoreClientV1Height(p, bs);
        r.proof_height = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_signer(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.signer = x;
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
    function _decode_IbcCoreClientV1Height(
        uint256 p,
        bytes memory bs
    ) internal pure returns (IbcCoreClientV1Height.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreClientV1Height.Data memory r,) =
            IbcCoreClientV1Height._decode(pointer, bs, sz);
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

        if (bytes(r.connection_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_string(r.connection_id, pointer, bs);
        }
        if (r.proof_ack.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.proof_ack, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            IbcCoreClientV1Height._encode_nested(r.proof_height, pointer, bs);

        if (bytes(r.signer).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.signer, pointer, bs);
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.connection_id).length);
        e += 1 + ProtoBufRuntime._sz_lendelim(r.proof_ack.length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreClientV1Height._estimate(r.proof_height)
            );
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.signer).length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.connection_id).length != 0) {
            return false;
        }

        if (r.proof_ack.length != 0) {
            return false;
        }

        if (bytes(r.signer).length != 0) {
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
        output.connection_id = input.connection_id;
        output.proof_ack = input.proof_ack;
        IbcCoreClientV1Height.store(input.proof_height, output.proof_height);
        output.signer = input.signer;
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

//library IbcCoreConnectionV1MsgConnectionOpenConfirm

library IbcCoreConnectionV1MsgConnectionOpenConfirmResponse {
    //struct definition
    struct Data {
        bool x;
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
        }
        return (r, sz);
    }

    // field readers

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
     * @param p The offset of bytes array to start decode
     * @return The number of bytes encoded
     */
    function _encode(
        Data memory,
        uint256 p,
        bytes memory
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 pointer = p;

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
     * @return The number of bytes encoded in estimation
     */
    function _estimate(Data memory /* r */ ) internal pure returns (uint256) {
        uint256 e;
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
    function store(Data memory input, Data storage output) internal {}

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
//library IbcCoreConnectionV1MsgConnectionOpenConfirmResponse
