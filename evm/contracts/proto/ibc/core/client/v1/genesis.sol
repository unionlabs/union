pragma solidity ^0.8.23;

import "../../../../ProtoBufRuntime.sol";
import "../../../../GoogleProtobufAny.sol";
import "./client.sol";

library IbcCoreClientV1GenesisState {
    //struct definition
    struct Data {
        IbcCoreClientV1IdentifiedClientState.Data[] clients;
        IbcCoreClientV1ClientConsensusStates.Data[] clients_consensus;
        IbcCoreClientV1IdentifiedGenesisMetadata.Data[] clients_metadata;
        IbcCoreClientV1Params.Data params;
        bool create_localhost;
        uint64 next_client_sequence;
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
                pointer += _read_unpacked_repeated_clients(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_clients_consensus(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 3) {
                pointer += _read_unpacked_repeated_clients_metadata(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 4) {
                pointer += _read_params(pointer, bs, r);
            } else if (fieldId == 5) {
                pointer += _read_create_localhost(pointer, bs, r);
            } else if (fieldId == 6) {
                pointer += _read_next_client_sequence(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[1] > 0) {
            require(r.clients.length == 0);
            r.clients =
                new IbcCoreClientV1IdentifiedClientState.Data[](counters[1]);
        }
        if (counters[2] > 0) {
            require(r.clients_consensus.length == 0);
            r.clients_consensus =
                new IbcCoreClientV1ClientConsensusStates.Data[](counters[2]);
        }
        if (counters[3] > 0) {
            require(r.clients_metadata.length == 0);
            r.clients_metadata =
                new IbcCoreClientV1IdentifiedGenesisMetadata.Data[](counters[3]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer +=
                    _read_unpacked_repeated_clients(pointer, bs, r, counters);
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_clients_consensus(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 3) {
                pointer += _read_unpacked_repeated_clients_metadata(
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
    function _read_unpacked_repeated_clients(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[7] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreClientV1IdentifiedClientState.Data memory x, uint256 sz) =
            _decode_IbcCoreClientV1IdentifiedClientState(p, bs);
        if (isNil(r)) {
            counters[1] += 1;
        } else {
            r.clients[r.clients.length - counters[1]] = x;
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
    function _read_unpacked_repeated_clients_consensus(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[7] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreClientV1ClientConsensusStates.Data memory x, uint256 sz) =
            _decode_IbcCoreClientV1ClientConsensusStates(p, bs);
        if (isNil(r)) {
            counters[2] += 1;
        } else {
            r.clients_consensus[r.clients_consensus.length - counters[2]] = x;
            counters[2] -= 1;
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
    function _read_unpacked_repeated_clients_metadata(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[7] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreClientV1IdentifiedGenesisMetadata.Data memory x, uint256 sz) =
            _decode_IbcCoreClientV1IdentifiedGenesisMetadata(p, bs);
        if (isNil(r)) {
            counters[3] += 1;
        } else {
            r.clients_metadata[r.clients_metadata.length - counters[3]] = x;
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
    function _read_params(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreClientV1Params.Data memory x, uint256 sz) =
            _decode_IbcCoreClientV1Params(p, bs);
        r.params = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_create_localhost(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bool x, uint256 sz) = ProtoBufRuntime._decode_bool(p, bs);
        r.create_localhost = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_next_client_sequence(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (uint64 x, uint256 sz) = ProtoBufRuntime._decode_uint64(p, bs);
        r.next_client_sequence = x;
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
    function _decode_IbcCoreClientV1IdentifiedClientState(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcCoreClientV1IdentifiedClientState.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreClientV1IdentifiedClientState.Data memory r,) =
            IbcCoreClientV1IdentifiedClientState._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreClientV1ClientConsensusStates(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcCoreClientV1ClientConsensusStates.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreClientV1ClientConsensusStates.Data memory r,) =
            IbcCoreClientV1ClientConsensusStates._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreClientV1IdentifiedGenesisMetadata(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcCoreClientV1IdentifiedGenesisMetadata.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreClientV1IdentifiedGenesisMetadata.Data memory r,) =
            IbcCoreClientV1IdentifiedGenesisMetadata._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreClientV1Params(
        uint256 p,
        bytes memory bs
    ) internal pure returns (IbcCoreClientV1Params.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreClientV1Params.Data memory r,) =
            IbcCoreClientV1Params._decode(pointer, bs, sz);
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
        if (r.clients.length != 0) {
            for (i = 0; i < r.clients.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreClientV1IdentifiedClientState._encode_nested(
                    r.clients[i], pointer, bs
                );
            }
        }
        if (r.clients_consensus.length != 0) {
            for (i = 0; i < r.clients_consensus.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreClientV1ClientConsensusStates._encode_nested(
                    r.clients_consensus[i], pointer, bs
                );
            }
        }
        if (r.clients_metadata.length != 0) {
            for (i = 0; i < r.clients_metadata.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreClientV1IdentifiedGenesisMetadata
                    ._encode_nested(r.clients_metadata[i], pointer, bs);
            }
        }

        pointer += ProtoBufRuntime._encode_key(
            4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += IbcCoreClientV1Params._encode_nested(r.params, pointer, bs);

        if (r.create_localhost != false) {
            pointer += ProtoBufRuntime._encode_key(
                5, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_bool(r.create_localhost, pointer, bs);
        }
        if (r.next_client_sequence != 0) {
            pointer += ProtoBufRuntime._encode_key(
                6, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_uint64(
                r.next_client_sequence, pointer, bs
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
        for (i = 0; i < r.clients.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreClientV1IdentifiedClientState._estimate(r.clients[i])
                );
        }
        for (i = 0; i < r.clients_consensus.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreClientV1ClientConsensusStates._estimate(
                        r.clients_consensus[i]
                    )
                );
        }
        for (i = 0; i < r.clients_metadata.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreClientV1IdentifiedGenesisMetadata._estimate(
                        r.clients_metadata[i]
                    )
                );
        }
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreClientV1Params._estimate(r.params)
            );
        e += 1 + 1;
        e += 1 + ProtoBufRuntime._sz_uint64(r.next_client_sequence);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.clients.length != 0) {
            return false;
        }

        if (r.clients_consensus.length != 0) {
            return false;
        }

        if (r.clients_metadata.length != 0) {
            return false;
        }

        if (r.create_localhost != false) {
            return false;
        }

        if (r.next_client_sequence != 0) {
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
        for (uint256 i1 = 0; i1 < input.clients.length; i1++) {
            output.clients.push(input.clients[i1]);
        }

        for (uint256 i2 = 0; i2 < input.clients_consensus.length; i2++) {
            output.clients_consensus.push(input.clients_consensus[i2]);
        }

        for (uint256 i3 = 0; i3 < input.clients_metadata.length; i3++) {
            output.clients_metadata.push(input.clients_metadata[i3]);
        }

        IbcCoreClientV1Params.store(input.params, output.params);
        output.create_localhost = input.create_localhost;
        output.next_client_sequence = input.next_client_sequence;
    }

    //array helpers for Clients
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addClients(
        Data memory self,
        IbcCoreClientV1IdentifiedClientState.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreClientV1IdentifiedClientState.Data[] memory tmp = new IbcCoreClientV1IdentifiedClientState
            .Data[](self.clients.length + 1);
        for (uint256 i; i < self.clients.length; i++) {
            tmp[i] = self.clients[i];
        }
        tmp[self.clients.length] = value;
        self.clients = tmp;
    }

    //array helpers for ClientsConsensus
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addClientsConsensus(
        Data memory self,
        IbcCoreClientV1ClientConsensusStates.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreClientV1ClientConsensusStates.Data[] memory tmp = new IbcCoreClientV1ClientConsensusStates
            .Data[](self.clients_consensus.length + 1);
        for (uint256 i; i < self.clients_consensus.length; i++) {
            tmp[i] = self.clients_consensus[i];
        }
        tmp[self.clients_consensus.length] = value;
        self.clients_consensus = tmp;
    }

    //array helpers for ClientsMetadata
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addClientsMetadata(
        Data memory self,
        IbcCoreClientV1IdentifiedGenesisMetadata.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreClientV1IdentifiedGenesisMetadata.Data[] memory tmp = new IbcCoreClientV1IdentifiedGenesisMetadata
            .Data[](self.clients_metadata.length + 1);
        for (uint256 i; i < self.clients_metadata.length; i++) {
            tmp[i] = self.clients_metadata[i];
        }
        tmp[self.clients_metadata.length] = value;
        self.clients_metadata = tmp;
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

//library IbcCoreClientV1GenesisState

library IbcCoreClientV1GenesisMetadata {
    //struct definition
    struct Data {
        bytes key;
        bytes value;
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
                pointer += _read_value(pointer, bs, r);
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
        if (r.value.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(r.value, pointer, bs);
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

//library IbcCoreClientV1GenesisMetadata

library IbcCoreClientV1IdentifiedGenesisMetadata {
    //struct definition
    struct Data {
        string client_id;
        IbcCoreClientV1GenesisMetadata.Data[] client_metadata;
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
                pointer += _read_client_id(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_client_metadata(
                    pointer, bs, nil(), counters
                );
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[2] > 0) {
            require(r.client_metadata.length == 0);
            r.client_metadata =
                new IbcCoreClientV1GenesisMetadata.Data[](counters[2]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 2) {
                pointer += _read_unpacked_repeated_client_metadata(
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
     * @param counters The counters for repeated fields
     * @return The number of bytes decoded
     */
    function _read_unpacked_repeated_client_metadata(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[3] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreClientV1GenesisMetadata.Data memory x, uint256 sz) =
            _decode_IbcCoreClientV1GenesisMetadata(p, bs);
        if (isNil(r)) {
            counters[2] += 1;
        } else {
            r.client_metadata[r.client_metadata.length - counters[2]] = x;
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
    function _decode_IbcCoreClientV1GenesisMetadata(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcCoreClientV1GenesisMetadata.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreClientV1GenesisMetadata.Data memory r,) =
            IbcCoreClientV1GenesisMetadata._decode(pointer, bs, sz);
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
        if (r.client_metadata.length != 0) {
            for (i = 0; i < r.client_metadata.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreClientV1GenesisMetadata._encode_nested(
                    r.client_metadata[i], pointer, bs
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.client_id).length);
        for (i = 0; i < r.client_metadata.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreClientV1GenesisMetadata._estimate(r.client_metadata[i])
                );
        }
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.client_id).length != 0) {
            return false;
        }

        if (r.client_metadata.length != 0) {
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

        for (uint256 i2 = 0; i2 < input.client_metadata.length; i2++) {
            output.client_metadata.push(input.client_metadata[i2]);
        }
    }

    //array helpers for ClientMetadata
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addClientMetadata(
        Data memory self,
        IbcCoreClientV1GenesisMetadata.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreClientV1GenesisMetadata.Data[] memory tmp = new IbcCoreClientV1GenesisMetadata
            .Data[](self.client_metadata.length + 1);
        for (uint256 i; i < self.client_metadata.length; i++) {
            tmp[i] = self.client_metadata[i];
        }
        tmp[self.client_metadata.length] = value;
        self.client_metadata = tmp;
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
//library IbcCoreClientV1IdentifiedGenesisMetadata
