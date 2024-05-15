pragma solidity ^0.8.23;

import "../../../../ProtoBufRuntime.sol";
import "../../../../GoogleProtobufAny.sol";

import "./connection.sol";

library IbcCoreConnectionV1GenesisState {
    //struct definition
    struct Data {
        IbcCoreConnectionV1IdentifiedConnection.Data[] connections;
        IbcCoreConnectionV1ConnectionPaths.Data[] client_connection_paths;
        uint64 next_connection_sequence;
        IbcCoreConnectionV1Params.Data params;
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
                pointer += _read_unpacked_repeated_connections(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_client_connection_paths(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 3) {
                pointer += _read_next_connection_sequence(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_params(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[1] > 0) {
            require(r.connections.length == 0);
            r.connections =
                new IbcCoreConnectionV1IdentifiedConnection.Data[](counters[1]);
        }
        if (counters[2] > 0) {
            require(r.client_connection_paths.length == 0);
            r.client_connection_paths =
                new IbcCoreConnectionV1ConnectionPaths.Data[](counters[2]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer += _read_unpacked_repeated_connections(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_client_connection_paths(
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
    function _read_unpacked_repeated_connections(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[5] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreConnectionV1IdentifiedConnection.Data memory x, uint256 sz) =
            _decode_IbcCoreConnectionV1IdentifiedConnection(p, bs);
        if (isNil(r)) {
            counters[1] += 1;
        } else {
            r.connections[r.connections.length - counters[1]] = x;
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
    function _read_unpacked_repeated_client_connection_paths(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[5] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreConnectionV1ConnectionPaths.Data memory x, uint256 sz) =
            _decode_IbcCoreConnectionV1ConnectionPaths(p, bs);
        if (isNil(r)) {
            counters[2] += 1;
        } else {
            r.client_connection_paths[r.client_connection_paths.length
                - counters[2]] = x;
            counters[2] -= 1;
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
    function _read_next_connection_sequence(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (uint64 x, uint256 sz) = ProtoBufRuntime._decode_uint64(p, bs);
        r.next_connection_sequence = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_params(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreConnectionV1Params.Data memory x, uint256 sz) =
            _decode_IbcCoreConnectionV1Params(p, bs);
        r.params = x;
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
    function _decode_IbcCoreConnectionV1IdentifiedConnection(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcCoreConnectionV1IdentifiedConnection.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreConnectionV1IdentifiedConnection.Data memory r,) =
            IbcCoreConnectionV1IdentifiedConnection._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreConnectionV1ConnectionPaths(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcCoreConnectionV1ConnectionPaths.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreConnectionV1ConnectionPaths.Data memory r,) =
            IbcCoreConnectionV1ConnectionPaths._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreConnectionV1Params(
        uint256 p,
        bytes memory bs
    ) internal pure returns (IbcCoreConnectionV1Params.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreConnectionV1Params.Data memory r,) =
            IbcCoreConnectionV1Params._decode(pointer, bs, sz);
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
        if (r.connections.length != 0) {
            for (i = 0; i < r.connections.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreConnectionV1IdentifiedConnection
                    ._encode_nested(r.connections[i], pointer, bs);
            }
        }
        if (r.client_connection_paths.length != 0) {
            for (i = 0; i < r.client_connection_paths.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreConnectionV1ConnectionPaths._encode_nested(
                    r.client_connection_paths[i], pointer, bs
                );
            }
        }
        if (r.next_connection_sequence != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_uint64(
                r.next_connection_sequence, pointer, bs
            );
        }

        pointer += ProtoBufRuntime._encode_key(
            4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            IbcCoreConnectionV1Params._encode_nested(r.params, pointer, bs);

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
        for (i = 0; i < r.connections.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreConnectionV1IdentifiedConnection._estimate(
                        r.connections[i]
                    )
                );
        }
        for (i = 0; i < r.client_connection_paths.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreConnectionV1ConnectionPaths._estimate(
                        r.client_connection_paths[i]
                    )
                );
        }
        e += 1 + ProtoBufRuntime._sz_uint64(r.next_connection_sequence);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreConnectionV1Params._estimate(r.params)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.connections.length != 0) {
            return false;
        }

        if (r.client_connection_paths.length != 0) {
            return false;
        }

        if (r.next_connection_sequence != 0) {
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
        for (uint256 i1 = 0; i1 < input.connections.length; i1++) {
            output.connections.push(input.connections[i1]);
        }

        for (uint256 i2 = 0; i2 < input.client_connection_paths.length; i2++) {
            output.client_connection_paths.push(
                input.client_connection_paths[i2]
            );
        }

        output.next_connection_sequence = input.next_connection_sequence;
        IbcCoreConnectionV1Params.store(input.params, output.params);
    }

    //array helpers for Connections
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addConnections(
        Data memory self,
        IbcCoreConnectionV1IdentifiedConnection.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreConnectionV1IdentifiedConnection.Data[] memory tmp = new IbcCoreConnectionV1IdentifiedConnection
            .Data[](self.connections.length + 1);
        for (uint256 i; i < self.connections.length; i++) {
            tmp[i] = self.connections[i];
        }
        tmp[self.connections.length] = value;
        self.connections = tmp;
    }

    //array helpers for ClientConnectionPaths
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addClientConnectionPaths(
        Data memory self,
        IbcCoreConnectionV1ConnectionPaths.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreConnectionV1ConnectionPaths.Data[] memory tmp = new IbcCoreConnectionV1ConnectionPaths
            .Data[](self.client_connection_paths.length + 1);
        for (uint256 i; i < self.client_connection_paths.length; i++) {
            tmp[i] = self.client_connection_paths[i];
        }
        tmp[self.client_connection_paths.length] = value;
        self.client_connection_paths = tmp;
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
//library IbcCoreConnectionV1GenesisState
