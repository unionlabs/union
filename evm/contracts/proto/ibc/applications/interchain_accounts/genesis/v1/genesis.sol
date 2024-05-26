pragma solidity ^0.8.23;

import "../../../../../ProtoBufRuntime.sol";
import "../../../../../GoogleProtobufAny.sol";

import "../../controller/v1/controller.sol";
import "../../host/v1/host.sol";

library IbcApplicationsInterchain_accountsGenesisV1GenesisState {
    //struct definition
    struct Data {
        IbcApplicationsInterchain_accountsGenesisV1ControllerGenesisState.Data
            controller_genesis_state;
        IbcApplicationsInterchain_accountsGenesisV1HostGenesisState.Data
            host_genesis_state;
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
                pointer += _read_controller_genesis_state(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_host_genesis_state(pointer, bs, r);
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
    function _read_controller_genesis_state(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (
            IbcApplicationsInterchain_accountsGenesisV1ControllerGenesisState
                .Data memory x,
            uint256 sz
        ) =
        _decode_IbcApplicationsInterchain_accountsGenesisV1ControllerGenesisState(
            p, bs
        );
        r.controller_genesis_state = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_host_genesis_state(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (
            IbcApplicationsInterchain_accountsGenesisV1HostGenesisState.Data
                memory x,
            uint256 sz
        ) = _decode_IbcApplicationsInterchain_accountsGenesisV1HostGenesisState(
            p, bs
        );
        r.host_genesis_state = x;
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
    function _decode_IbcApplicationsInterchain_accountsGenesisV1ControllerGenesisState(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (
            IbcApplicationsInterchain_accountsGenesisV1ControllerGenesisState.Data
                memory,
            uint256
        )
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (
            IbcApplicationsInterchain_accountsGenesisV1ControllerGenesisState
                .Data memory r,
        ) = IbcApplicationsInterchain_accountsGenesisV1ControllerGenesisState
            ._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcApplicationsInterchain_accountsGenesisV1HostGenesisState(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (
            IbcApplicationsInterchain_accountsGenesisV1HostGenesisState.Data memory,
            uint256
        )
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (
            IbcApplicationsInterchain_accountsGenesisV1HostGenesisState.Data
                memory r,
        ) = IbcApplicationsInterchain_accountsGenesisV1HostGenesisState._decode(
            pointer, bs, sz
        );
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
        IbcApplicationsInterchain_accountsGenesisV1ControllerGenesisState
            ._encode_nested(r.controller_genesis_state, pointer, bs);

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += IbcApplicationsInterchain_accountsGenesisV1HostGenesisState
            ._encode_nested(r.host_genesis_state, pointer, bs);

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
                IbcApplicationsInterchain_accountsGenesisV1ControllerGenesisState
                    ._estimate(r.controller_genesis_state)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcApplicationsInterchain_accountsGenesisV1HostGenesisState
                    ._estimate(r.host_genesis_state)
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
        IbcApplicationsInterchain_accountsGenesisV1ControllerGenesisState.store(
            input.controller_genesis_state, output.controller_genesis_state
        );
        IbcApplicationsInterchain_accountsGenesisV1HostGenesisState.store(
            input.host_genesis_state, output.host_genesis_state
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

//library IbcApplicationsInterchain_accountsGenesisV1GenesisState

library IbcApplicationsInterchain_accountsGenesisV1ControllerGenesisState {
    //struct definition
    struct Data {
        IbcApplicationsInterchain_accountsGenesisV1ActiveChannel.Data[]
            active_channels;
        IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
            .Data[] interchain_accounts;
        string[] ports;
        IbcApplicationsInterchain_accountsControllerV1Params.Data params;
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
                pointer += _read_unpacked_repeated_active_channels(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_interchain_accounts(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 3) {
                pointer +=
                    _read_unpacked_repeated_ports(pointer, bs, nil(), counters);
            } else if (fieldId == 4) {
                pointer += _read_params(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[1] > 0) {
            require(r.active_channels.length == 0);
            r.active_channels = new IbcApplicationsInterchain_accountsGenesisV1ActiveChannel
                .Data[](counters[1]);
        }
        if (counters[2] > 0) {
            require(r.interchain_accounts.length == 0);
            r.interchain_accounts = new IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
                .Data[](counters[2]);
        }
        if (counters[3] > 0) {
            require(r.ports.length == 0);
            r.ports = new string[](counters[3]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer += _read_unpacked_repeated_active_channels(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_interchain_accounts(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 3) {
                pointer +=
                    _read_unpacked_repeated_ports(pointer, bs, r, counters);
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
    function _read_unpacked_repeated_active_channels(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[5] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (
            IbcApplicationsInterchain_accountsGenesisV1ActiveChannel.Data memory
                x,
            uint256 sz
        ) = _decode_IbcApplicationsInterchain_accountsGenesisV1ActiveChannel(
            p, bs
        );
        if (isNil(r)) {
            counters[1] += 1;
        } else {
            r.active_channels[r.active_channels.length - counters[1]] = x;
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
    function _read_unpacked_repeated_interchain_accounts(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[5] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (
            IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
                .Data memory x,
            uint256 sz
        ) =
        _decode_IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount(
            p, bs
        );
        if (isNil(r)) {
            counters[2] += 1;
        } else {
            r.interchain_accounts[r.interchain_accounts.length - counters[2]] =
                x;
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
    function _read_unpacked_repeated_ports(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[5] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        if (isNil(r)) {
            counters[3] += 1;
        } else {
            r.ports[r.ports.length - counters[3]] = x;
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
        (
            IbcApplicationsInterchain_accountsControllerV1Params.Data memory x,
            uint256 sz
        ) = _decode_IbcApplicationsInterchain_accountsControllerV1Params(p, bs);
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
    function _decode_IbcApplicationsInterchain_accountsGenesisV1ActiveChannel(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (
            IbcApplicationsInterchain_accountsGenesisV1ActiveChannel.Data memory,
            uint256
        )
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (
            IbcApplicationsInterchain_accountsGenesisV1ActiveChannel.Data memory
                r,
        ) = IbcApplicationsInterchain_accountsGenesisV1ActiveChannel._decode(
            pointer, bs, sz
        );
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (
            IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount.Data
                memory,
            uint256
        )
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (
            IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
                .Data memory r,
        ) =
        IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
            ._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcApplicationsInterchain_accountsControllerV1Params(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (
            IbcApplicationsInterchain_accountsControllerV1Params.Data memory,
            uint256
        )
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcApplicationsInterchain_accountsControllerV1Params.Data memory r,) =
        IbcApplicationsInterchain_accountsControllerV1Params._decode(
            pointer, bs, sz
        );
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
        if (r.active_channels.length != 0) {
            for (i = 0; i < r.active_channels.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer +=
                IbcApplicationsInterchain_accountsGenesisV1ActiveChannel
                    ._encode_nested(r.active_channels[i], pointer, bs);
            }
        }
        if (r.interchain_accounts.length != 0) {
            for (i = 0; i < r.interchain_accounts.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer +=
                IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
                    ._encode_nested(r.interchain_accounts[i], pointer, bs);
            }
        }
        if (r.ports.length != 0) {
            for (i = 0; i < r.ports.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer +=
                    ProtoBufRuntime._encode_string(r.ports[i], pointer, bs);
            }
        }

        pointer += ProtoBufRuntime._encode_key(
            4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += IbcApplicationsInterchain_accountsControllerV1Params
            ._encode_nested(r.params, pointer, bs);

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
        for (i = 0; i < r.active_channels.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcApplicationsInterchain_accountsGenesisV1ActiveChannel
                        ._estimate(r.active_channels[i])
                );
        }
        for (i = 0; i < r.interchain_accounts.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
                        ._estimate(r.interchain_accounts[i])
                );
        }
        for (i = 0; i < r.ports.length; i++) {
            e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.ports[i]).length);
        }
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcApplicationsInterchain_accountsControllerV1Params._estimate(
                    r.params
                )
            );
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.active_channels.length != 0) {
            return false;
        }

        if (r.interchain_accounts.length != 0) {
            return false;
        }

        if (r.ports.length != 0) {
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
        for (uint256 i1 = 0; i1 < input.active_channels.length; i1++) {
            output.active_channels.push(input.active_channels[i1]);
        }

        for (uint256 i2 = 0; i2 < input.interchain_accounts.length; i2++) {
            output.interchain_accounts.push(input.interchain_accounts[i2]);
        }

        output.ports = input.ports;
        IbcApplicationsInterchain_accountsControllerV1Params.store(
            input.params, output.params
        );
    }

    //array helpers for ActiveChannels
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addActiveChannels(
        Data memory self,
        IbcApplicationsInterchain_accountsGenesisV1ActiveChannel.Data memory
            value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcApplicationsInterchain_accountsGenesisV1ActiveChannel.Data[] memory
            tmp = new IbcApplicationsInterchain_accountsGenesisV1ActiveChannel
                .Data[](self.active_channels.length + 1);
        for (uint256 i; i < self.active_channels.length; i++) {
            tmp[i] = self.active_channels[i];
        }
        tmp[self.active_channels.length] = value;
        self.active_channels = tmp;
    }

    //array helpers for InterchainAccounts
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addInterchainAccounts(
        Data memory self,
        IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
            .Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
            .Data[] memory tmp = new IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
                .Data[](self.interchain_accounts.length + 1);
        for (uint256 i; i < self.interchain_accounts.length; i++) {
            tmp[i] = self.interchain_accounts[i];
        }
        tmp[self.interchain_accounts.length] = value;
        self.interchain_accounts = tmp;
    }

    //array helpers for Ports
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addPorts(Data memory self, string memory value) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        string[] memory tmp = new string[](self.ports.length + 1);
        for (uint256 i; i < self.ports.length; i++) {
            tmp[i] = self.ports[i];
        }
        tmp[self.ports.length] = value;
        self.ports = tmp;
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

//library IbcApplicationsInterchain_accountsGenesisV1ControllerGenesisState

library IbcApplicationsInterchain_accountsGenesisV1HostGenesisState {
    //struct definition
    struct Data {
        IbcApplicationsInterchain_accountsGenesisV1ActiveChannel.Data[]
            active_channels;
        IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
            .Data[] interchain_accounts;
        string port;
        IbcApplicationsInterchain_accountsHostV1Params.Data params;
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
                pointer += _read_unpacked_repeated_active_channels(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_interchain_accounts(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 3) {
                pointer += _read_port(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_params(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[1] > 0) {
            require(r.active_channels.length == 0);
            r.active_channels = new IbcApplicationsInterchain_accountsGenesisV1ActiveChannel
                .Data[](counters[1]);
        }
        if (counters[2] > 0) {
            require(r.interchain_accounts.length == 0);
            r.interchain_accounts = new IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
                .Data[](counters[2]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer += _read_unpacked_repeated_active_channels(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_interchain_accounts(
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
    function _read_unpacked_repeated_active_channels(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[5] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (
            IbcApplicationsInterchain_accountsGenesisV1ActiveChannel.Data memory
                x,
            uint256 sz
        ) = _decode_IbcApplicationsInterchain_accountsGenesisV1ActiveChannel(
            p, bs
        );
        if (isNil(r)) {
            counters[1] += 1;
        } else {
            r.active_channels[r.active_channels.length - counters[1]] = x;
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
    function _read_unpacked_repeated_interchain_accounts(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[5] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (
            IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
                .Data memory x,
            uint256 sz
        ) =
        _decode_IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount(
            p, bs
        );
        if (isNil(r)) {
            counters[2] += 1;
        } else {
            r.interchain_accounts[r.interchain_accounts.length - counters[2]] =
                x;
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
    function _read_port(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.port = x;
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
        (
            IbcApplicationsInterchain_accountsHostV1Params.Data memory x,
            uint256 sz
        ) = _decode_IbcApplicationsInterchain_accountsHostV1Params(p, bs);
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
    function _decode_IbcApplicationsInterchain_accountsGenesisV1ActiveChannel(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (
            IbcApplicationsInterchain_accountsGenesisV1ActiveChannel.Data memory,
            uint256
        )
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (
            IbcApplicationsInterchain_accountsGenesisV1ActiveChannel.Data memory
                r,
        ) = IbcApplicationsInterchain_accountsGenesisV1ActiveChannel._decode(
            pointer, bs, sz
        );
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (
            IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount.Data
                memory,
            uint256
        )
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (
            IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
                .Data memory r,
        ) =
        IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
            ._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcApplicationsInterchain_accountsHostV1Params(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (
            IbcApplicationsInterchain_accountsHostV1Params.Data memory,
            uint256
        )
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcApplicationsInterchain_accountsHostV1Params.Data memory r,) =
        IbcApplicationsInterchain_accountsHostV1Params._decode(pointer, bs, sz);
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
        if (r.active_channels.length != 0) {
            for (i = 0; i < r.active_channels.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer +=
                IbcApplicationsInterchain_accountsGenesisV1ActiveChannel
                    ._encode_nested(r.active_channels[i], pointer, bs);
            }
        }
        if (r.interchain_accounts.length != 0) {
            for (i = 0; i < r.interchain_accounts.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer +=
                IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
                    ._encode_nested(r.interchain_accounts[i], pointer, bs);
            }
        }
        if (bytes(r.port).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.port, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += IbcApplicationsInterchain_accountsHostV1Params._encode_nested(
            r.params, pointer, bs
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
        uint256 i;
        for (i = 0; i < r.active_channels.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcApplicationsInterchain_accountsGenesisV1ActiveChannel
                        ._estimate(r.active_channels[i])
                );
        }
        for (i = 0; i < r.interchain_accounts.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
                        ._estimate(r.interchain_accounts[i])
                );
        }
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.port).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcApplicationsInterchain_accountsHostV1Params._estimate(r.params)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.active_channels.length != 0) {
            return false;
        }

        if (r.interchain_accounts.length != 0) {
            return false;
        }

        if (bytes(r.port).length != 0) {
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
        for (uint256 i1 = 0; i1 < input.active_channels.length; i1++) {
            output.active_channels.push(input.active_channels[i1]);
        }

        for (uint256 i2 = 0; i2 < input.interchain_accounts.length; i2++) {
            output.interchain_accounts.push(input.interchain_accounts[i2]);
        }

        output.port = input.port;
        IbcApplicationsInterchain_accountsHostV1Params.store(
            input.params, output.params
        );
    }

    //array helpers for ActiveChannels
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addActiveChannels(
        Data memory self,
        IbcApplicationsInterchain_accountsGenesisV1ActiveChannel.Data memory
            value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcApplicationsInterchain_accountsGenesisV1ActiveChannel.Data[] memory
            tmp = new IbcApplicationsInterchain_accountsGenesisV1ActiveChannel
                .Data[](self.active_channels.length + 1);
        for (uint256 i; i < self.active_channels.length; i++) {
            tmp[i] = self.active_channels[i];
        }
        tmp[self.active_channels.length] = value;
        self.active_channels = tmp;
    }

    //array helpers for InterchainAccounts
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addInterchainAccounts(
        Data memory self,
        IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
            .Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
            .Data[] memory tmp = new IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
                .Data[](self.interchain_accounts.length + 1);
        for (uint256 i; i < self.interchain_accounts.length; i++) {
            tmp[i] = self.interchain_accounts[i];
        }
        tmp[self.interchain_accounts.length] = value;
        self.interchain_accounts = tmp;
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

//library IbcApplicationsInterchain_accountsGenesisV1HostGenesisState

library IbcApplicationsInterchain_accountsGenesisV1ActiveChannel {
    //struct definition
    struct Data {
        string connection_id;
        string port_id;
        string channel_id;
        bool is_middleware_enabled;
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
                pointer += _read_port_id(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_channel_id(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_is_middleware_enabled(pointer, bs, r);
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
    function _read_port_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.port_id = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_channel_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.channel_id = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_is_middleware_enabled(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bool x, uint256 sz) = ProtoBufRuntime._decode_bool(p, bs);
        r.is_middleware_enabled = x;
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

        if (bytes(r.connection_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_string(r.connection_id, pointer, bs);
        }
        if (bytes(r.port_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.port_id, pointer, bs);
        }
        if (bytes(r.channel_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.channel_id, pointer, bs);
        }
        if (r.is_middleware_enabled != false) {
            pointer += ProtoBufRuntime._encode_key(
                4, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bool(
                r.is_middleware_enabled, pointer, bs
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.port_id).length);
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.channel_id).length);
        e += 1 + 1;
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.connection_id).length != 0) {
            return false;
        }

        if (bytes(r.port_id).length != 0) {
            return false;
        }

        if (bytes(r.channel_id).length != 0) {
            return false;
        }

        if (r.is_middleware_enabled != false) {
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
        output.port_id = input.port_id;
        output.channel_id = input.channel_id;
        output.is_middleware_enabled = input.is_middleware_enabled;
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

//library IbcApplicationsInterchain_accountsGenesisV1ActiveChannel

library IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount {
    //struct definition
    struct Data {
        string connection_id;
        string port_id;
        string account_address;
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
                pointer += _read_port_id(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_account_address(pointer, bs, r);
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
    function _read_port_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.port_id = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_account_address(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.account_address = x;
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

        if (bytes(r.connection_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_string(r.connection_id, pointer, bs);
        }
        if (bytes(r.port_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.port_id, pointer, bs);
        }
        if (bytes(r.account_address).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_string(r.account_address, pointer, bs);
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.port_id).length);
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.account_address).length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.connection_id).length != 0) {
            return false;
        }

        if (bytes(r.port_id).length != 0) {
            return false;
        }

        if (bytes(r.account_address).length != 0) {
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
        output.port_id = input.port_id;
        output.account_address = input.account_address;
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
//library IbcApplicationsInterchain_accountsGenesisV1RegisteredInterchainAccount
