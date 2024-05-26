pragma solidity ^0.8.23;

import "../../../../ProtoBufRuntime.sol";
import "../../../../GoogleProtobufAny.sol";

import "./fee.sol";
import "../../../core/channel/v1/channel.sol";

library IbcApplicationsFeeV1GenesisState {
    //struct definition
    struct Data {
        IbcApplicationsFeeV1IdentifiedPacketFees.Data[] identified_fees;
        IbcApplicationsFeeV1FeeEnabledChannel.Data[] fee_enabled_channels;
        IbcApplicationsFeeV1RegisteredPayee.Data[] registered_payees;
        IbcApplicationsFeeV1RegisteredCounterpartyPayee.Data[]
            registered_counterparty_payees;
        IbcApplicationsFeeV1ForwardRelayerAddress.Data[] forward_relayers;
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
        uint256[6] memory counters;
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
                pointer += _read_unpacked_repeated_identified_fees(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_fee_enabled_channels(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 3) {
                pointer += _read_unpacked_repeated_registered_payees(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 4) {
                pointer +=
                _read_unpacked_repeated_registered_counterparty_payees(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 5) {
                pointer += _read_unpacked_repeated_forward_relayers(
                    pointer, bs, nil(), counters
                );
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[1] > 0) {
            require(r.identified_fees.length == 0);
            r.identified_fees =
                new IbcApplicationsFeeV1IdentifiedPacketFees.Data[](counters[1]);
        }
        if (counters[2] > 0) {
            require(r.fee_enabled_channels.length == 0);
            r.fee_enabled_channels =
                new IbcApplicationsFeeV1FeeEnabledChannel.Data[](counters[2]);
        }
        if (counters[3] > 0) {
            require(r.registered_payees.length == 0);
            r.registered_payees =
                new IbcApplicationsFeeV1RegisteredPayee.Data[](counters[3]);
        }
        if (counters[4] > 0) {
            require(r.registered_counterparty_payees.length == 0);
            r.registered_counterparty_payees = new IbcApplicationsFeeV1RegisteredCounterpartyPayee
                .Data[](counters[4]);
        }
        if (counters[5] > 0) {
            require(r.forward_relayers.length == 0);
            r.forward_relayers = new IbcApplicationsFeeV1ForwardRelayerAddress
                .Data[](counters[5]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer += _read_unpacked_repeated_identified_fees(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_fee_enabled_channels(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 3) {
                pointer += _read_unpacked_repeated_registered_payees(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 4) {
                pointer +=
                _read_unpacked_repeated_registered_counterparty_payees(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 5) {
                pointer += _read_unpacked_repeated_forward_relayers(
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
    function _read_unpacked_repeated_identified_fees(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[6] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcApplicationsFeeV1IdentifiedPacketFees.Data memory x, uint256 sz) =
            _decode_IbcApplicationsFeeV1IdentifiedPacketFees(p, bs);
        if (isNil(r)) {
            counters[1] += 1;
        } else {
            r.identified_fees[r.identified_fees.length - counters[1]] = x;
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
    function _read_unpacked_repeated_fee_enabled_channels(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[6] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcApplicationsFeeV1FeeEnabledChannel.Data memory x, uint256 sz) =
            _decode_IbcApplicationsFeeV1FeeEnabledChannel(p, bs);
        if (isNil(r)) {
            counters[2] += 1;
        } else {
            r.fee_enabled_channels[r.fee_enabled_channels.length - counters[2]]
            = x;
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
    function _read_unpacked_repeated_registered_payees(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[6] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcApplicationsFeeV1RegisteredPayee.Data memory x, uint256 sz) =
            _decode_IbcApplicationsFeeV1RegisteredPayee(p, bs);
        if (isNil(r)) {
            counters[3] += 1;
        } else {
            r.registered_payees[r.registered_payees.length - counters[3]] = x;
            counters[3] -= 1;
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
    function _read_unpacked_repeated_registered_counterparty_payees(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[6] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (
            IbcApplicationsFeeV1RegisteredCounterpartyPayee.Data memory x,
            uint256 sz
        ) = _decode_IbcApplicationsFeeV1RegisteredCounterpartyPayee(p, bs);
        if (isNil(r)) {
            counters[4] += 1;
        } else {
            r.registered_counterparty_payees[r
                .registered_counterparty_payees
                .length - counters[4]] = x;
            counters[4] -= 1;
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
    function _read_unpacked_repeated_forward_relayers(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[6] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcApplicationsFeeV1ForwardRelayerAddress.Data memory x, uint256 sz) =
            _decode_IbcApplicationsFeeV1ForwardRelayerAddress(p, bs);
        if (isNil(r)) {
            counters[5] += 1;
        } else {
            r.forward_relayers[r.forward_relayers.length - counters[5]] = x;
            counters[5] -= 1;
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
    function _decode_IbcApplicationsFeeV1IdentifiedPacketFees(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcApplicationsFeeV1IdentifiedPacketFees.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcApplicationsFeeV1IdentifiedPacketFees.Data memory r,) =
            IbcApplicationsFeeV1IdentifiedPacketFees._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcApplicationsFeeV1FeeEnabledChannel(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcApplicationsFeeV1FeeEnabledChannel.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcApplicationsFeeV1FeeEnabledChannel.Data memory r,) =
            IbcApplicationsFeeV1FeeEnabledChannel._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcApplicationsFeeV1RegisteredPayee(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcApplicationsFeeV1RegisteredPayee.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcApplicationsFeeV1RegisteredPayee.Data memory r,) =
            IbcApplicationsFeeV1RegisteredPayee._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcApplicationsFeeV1RegisteredCounterpartyPayee(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (
            IbcApplicationsFeeV1RegisteredCounterpartyPayee.Data memory,
            uint256
        )
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcApplicationsFeeV1RegisteredCounterpartyPayee.Data memory r,) =
        IbcApplicationsFeeV1RegisteredCounterpartyPayee._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcApplicationsFeeV1ForwardRelayerAddress(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcApplicationsFeeV1ForwardRelayerAddress.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcApplicationsFeeV1ForwardRelayerAddress.Data memory r,) =
            IbcApplicationsFeeV1ForwardRelayerAddress._decode(pointer, bs, sz);
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
        if (r.identified_fees.length != 0) {
            for (i = 0; i < r.identified_fees.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcApplicationsFeeV1IdentifiedPacketFees
                    ._encode_nested(r.identified_fees[i], pointer, bs);
            }
        }
        if (r.fee_enabled_channels.length != 0) {
            for (i = 0; i < r.fee_enabled_channels.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcApplicationsFeeV1FeeEnabledChannel._encode_nested(
                    r.fee_enabled_channels[i], pointer, bs
                );
            }
        }
        if (r.registered_payees.length != 0) {
            for (i = 0; i < r.registered_payees.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcApplicationsFeeV1RegisteredPayee._encode_nested(
                    r.registered_payees[i], pointer, bs
                );
            }
        }
        if (r.registered_counterparty_payees.length != 0) {
            for (i = 0; i < r.registered_counterparty_payees.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcApplicationsFeeV1RegisteredCounterpartyPayee
                    ._encode_nested(
                    r.registered_counterparty_payees[i], pointer, bs
                );
            }
        }
        if (r.forward_relayers.length != 0) {
            for (i = 0; i < r.forward_relayers.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    5, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcApplicationsFeeV1ForwardRelayerAddress
                    ._encode_nested(r.forward_relayers[i], pointer, bs);
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
        for (i = 0; i < r.identified_fees.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcApplicationsFeeV1IdentifiedPacketFees._estimate(
                        r.identified_fees[i]
                    )
                );
        }
        for (i = 0; i < r.fee_enabled_channels.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcApplicationsFeeV1FeeEnabledChannel._estimate(
                        r.fee_enabled_channels[i]
                    )
                );
        }
        for (i = 0; i < r.registered_payees.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcApplicationsFeeV1RegisteredPayee._estimate(
                        r.registered_payees[i]
                    )
                );
        }
        for (i = 0; i < r.registered_counterparty_payees.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcApplicationsFeeV1RegisteredCounterpartyPayee._estimate(
                        r.registered_counterparty_payees[i]
                    )
                );
        }
        for (i = 0; i < r.forward_relayers.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcApplicationsFeeV1ForwardRelayerAddress._estimate(
                        r.forward_relayers[i]
                    )
                );
        }
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.identified_fees.length != 0) {
            return false;
        }

        if (r.fee_enabled_channels.length != 0) {
            return false;
        }

        if (r.registered_payees.length != 0) {
            return false;
        }

        if (r.registered_counterparty_payees.length != 0) {
            return false;
        }

        if (r.forward_relayers.length != 0) {
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
        for (uint256 i1 = 0; i1 < input.identified_fees.length; i1++) {
            output.identified_fees.push(input.identified_fees[i1]);
        }

        for (uint256 i2 = 0; i2 < input.fee_enabled_channels.length; i2++) {
            output.fee_enabled_channels.push(input.fee_enabled_channels[i2]);
        }

        for (uint256 i3 = 0; i3 < input.registered_payees.length; i3++) {
            output.registered_payees.push(input.registered_payees[i3]);
        }

        for (
            uint256 i4 = 0;
            i4 < input.registered_counterparty_payees.length;
            i4++
        ) {
            output.registered_counterparty_payees.push(
                input.registered_counterparty_payees[i4]
            );
        }

        for (uint256 i5 = 0; i5 < input.forward_relayers.length; i5++) {
            output.forward_relayers.push(input.forward_relayers[i5]);
        }
    }

    //array helpers for IdentifiedFees
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addIdentifiedFees(
        Data memory self,
        IbcApplicationsFeeV1IdentifiedPacketFees.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcApplicationsFeeV1IdentifiedPacketFees.Data[] memory tmp = new IbcApplicationsFeeV1IdentifiedPacketFees
            .Data[](self.identified_fees.length + 1);
        for (uint256 i; i < self.identified_fees.length; i++) {
            tmp[i] = self.identified_fees[i];
        }
        tmp[self.identified_fees.length] = value;
        self.identified_fees = tmp;
    }

    //array helpers for FeeEnabledChannels
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addFeeEnabledChannels(
        Data memory self,
        IbcApplicationsFeeV1FeeEnabledChannel.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcApplicationsFeeV1FeeEnabledChannel.Data[] memory tmp = new IbcApplicationsFeeV1FeeEnabledChannel
            .Data[](self.fee_enabled_channels.length + 1);
        for (uint256 i; i < self.fee_enabled_channels.length; i++) {
            tmp[i] = self.fee_enabled_channels[i];
        }
        tmp[self.fee_enabled_channels.length] = value;
        self.fee_enabled_channels = tmp;
    }

    //array helpers for RegisteredPayees
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addRegisteredPayees(
        Data memory self,
        IbcApplicationsFeeV1RegisteredPayee.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcApplicationsFeeV1RegisteredPayee.Data[] memory tmp = new IbcApplicationsFeeV1RegisteredPayee
            .Data[](self.registered_payees.length + 1);
        for (uint256 i; i < self.registered_payees.length; i++) {
            tmp[i] = self.registered_payees[i];
        }
        tmp[self.registered_payees.length] = value;
        self.registered_payees = tmp;
    }

    //array helpers for RegisteredCounterpartyPayees
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addRegisteredCounterpartyPayees(
        Data memory self,
        IbcApplicationsFeeV1RegisteredCounterpartyPayee.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcApplicationsFeeV1RegisteredCounterpartyPayee.Data[] memory tmp = new IbcApplicationsFeeV1RegisteredCounterpartyPayee
            .Data[](self.registered_counterparty_payees.length + 1);
        for (uint256 i; i < self.registered_counterparty_payees.length; i++) {
            tmp[i] = self.registered_counterparty_payees[i];
        }
        tmp[self.registered_counterparty_payees.length] = value;
        self.registered_counterparty_payees = tmp;
    }

    //array helpers for ForwardRelayers
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addForwardRelayers(
        Data memory self,
        IbcApplicationsFeeV1ForwardRelayerAddress.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcApplicationsFeeV1ForwardRelayerAddress.Data[] memory tmp = new IbcApplicationsFeeV1ForwardRelayerAddress
            .Data[](self.forward_relayers.length + 1);
        for (uint256 i; i < self.forward_relayers.length; i++) {
            tmp[i] = self.forward_relayers[i];
        }
        tmp[self.forward_relayers.length] = value;
        self.forward_relayers = tmp;
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

//library IbcApplicationsFeeV1GenesisState

library IbcApplicationsFeeV1FeeEnabledChannel {
    //struct definition
    struct Data {
        string port_id;
        string channel_id;
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
                pointer += _read_port_id(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_channel_id(pointer, bs, r);
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

        if (bytes(r.port_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.port_id, pointer, bs);
        }
        if (bytes(r.channel_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.channel_id, pointer, bs);
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.port_id).length);
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.channel_id).length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.port_id).length != 0) {
            return false;
        }

        if (bytes(r.channel_id).length != 0) {
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
        output.port_id = input.port_id;
        output.channel_id = input.channel_id;
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

//library IbcApplicationsFeeV1FeeEnabledChannel

library IbcApplicationsFeeV1RegisteredPayee {
    //struct definition
    struct Data {
        string channel_id;
        string relayer;
        string payee;
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
                pointer += _read_channel_id(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_relayer(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_payee(pointer, bs, r);
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
    function _read_relayer(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.relayer = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_payee(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.payee = x;
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

        if (bytes(r.channel_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.channel_id, pointer, bs);
        }
        if (bytes(r.relayer).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.relayer, pointer, bs);
        }
        if (bytes(r.payee).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.payee, pointer, bs);
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.channel_id).length);
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.relayer).length);
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.payee).length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.channel_id).length != 0) {
            return false;
        }

        if (bytes(r.relayer).length != 0) {
            return false;
        }

        if (bytes(r.payee).length != 0) {
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
        output.channel_id = input.channel_id;
        output.relayer = input.relayer;
        output.payee = input.payee;
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

//library IbcApplicationsFeeV1RegisteredPayee

library IbcApplicationsFeeV1RegisteredCounterpartyPayee {
    //struct definition
    struct Data {
        string channel_id;
        string relayer;
        string counterparty_payee;
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
                pointer += _read_channel_id(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_relayer(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_counterparty_payee(pointer, bs, r);
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
    function _read_relayer(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.relayer = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_counterparty_payee(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.counterparty_payee = x;
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

        if (bytes(r.channel_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.channel_id, pointer, bs);
        }
        if (bytes(r.relayer).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.relayer, pointer, bs);
        }
        if (bytes(r.counterparty_payee).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(
                r.counterparty_payee, pointer, bs
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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.channel_id).length);
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.relayer).length);
        e +=
            1 + ProtoBufRuntime._sz_lendelim(bytes(r.counterparty_payee).length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.channel_id).length != 0) {
            return false;
        }

        if (bytes(r.relayer).length != 0) {
            return false;
        }

        if (bytes(r.counterparty_payee).length != 0) {
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
        output.channel_id = input.channel_id;
        output.relayer = input.relayer;
        output.counterparty_payee = input.counterparty_payee;
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

//library IbcApplicationsFeeV1RegisteredCounterpartyPayee

library IbcApplicationsFeeV1ForwardRelayerAddress {
    //struct definition
    struct Data {
        string address_;
        IbcCoreChannelV1PacketId.Data packet_id;
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
                pointer += _read_address(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_packet_id(pointer, bs, r);
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
    function _read_address(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.address_ = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_packet_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreChannelV1PacketId.Data memory x, uint256 sz) =
            _decode_IbcCoreChannelV1PacketId(p, bs);
        r.packet_id = x;
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
    function _decode_IbcCoreChannelV1PacketId(
        uint256 p,
        bytes memory bs
    ) internal pure returns (IbcCoreChannelV1PacketId.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreChannelV1PacketId.Data memory r,) =
            IbcCoreChannelV1PacketId._decode(pointer, bs, sz);
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

        if (bytes(r.address_).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.address_, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            IbcCoreChannelV1PacketId._encode_nested(r.packet_id, pointer, bs);

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
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.address_).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreChannelV1PacketId._estimate(r.packet_id)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.address_).length != 0) {
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
        output.address_ = input.address_;
        IbcCoreChannelV1PacketId.store(input.packet_id, output.packet_id);
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
//library IbcApplicationsFeeV1ForwardRelayerAddress
