pragma solidity ^0.8.23;

import "../../../../ProtoBufRuntime.sol";
import "../../../../GoogleProtobufAny.sol";
import "../../../../cosmos/base/v1beta1/coin.sol";

import "../../../core/channel/v1/channel.sol";

library IbcApplicationsFeeV1Fee {
    //struct definition
    struct Data {
        CosmosBaseV1beta1Coin.Data[] recv_fee;
        CosmosBaseV1beta1Coin.Data[] ack_fee;
        CosmosBaseV1beta1Coin.Data[] timeout_fee;
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
                pointer += _read_unpacked_repeated_recv_fee(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_ack_fee(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 3) {
                pointer += _read_unpacked_repeated_timeout_fee(
                    pointer, bs, nil(), counters
                );
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[1] > 0) {
            require(r.recv_fee.length == 0);
            r.recv_fee = new CosmosBaseV1beta1Coin.Data[](counters[1]);
        }
        if (counters[2] > 0) {
            require(r.ack_fee.length == 0);
            r.ack_fee = new CosmosBaseV1beta1Coin.Data[](counters[2]);
        }
        if (counters[3] > 0) {
            require(r.timeout_fee.length == 0);
            r.timeout_fee = new CosmosBaseV1beta1Coin.Data[](counters[3]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer +=
                    _read_unpacked_repeated_recv_fee(pointer, bs, r, counters);
            } else if (fieldId == 2) {
                pointer +=
                    _read_unpacked_repeated_ack_fee(pointer, bs, r, counters);
            } else if (fieldId == 3) {
                pointer += _read_unpacked_repeated_timeout_fee(
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
    function _read_unpacked_repeated_recv_fee(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[4] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (CosmosBaseV1beta1Coin.Data memory x, uint256 sz) =
            _decode_CosmosBaseV1beta1Coin(p, bs);
        if (isNil(r)) {
            counters[1] += 1;
        } else {
            r.recv_fee[r.recv_fee.length - counters[1]] = x;
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
    function _read_unpacked_repeated_ack_fee(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[4] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (CosmosBaseV1beta1Coin.Data memory x, uint256 sz) =
            _decode_CosmosBaseV1beta1Coin(p, bs);
        if (isNil(r)) {
            counters[2] += 1;
        } else {
            r.ack_fee[r.ack_fee.length - counters[2]] = x;
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
    function _read_unpacked_repeated_timeout_fee(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[4] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (CosmosBaseV1beta1Coin.Data memory x, uint256 sz) =
            _decode_CosmosBaseV1beta1Coin(p, bs);
        if (isNil(r)) {
            counters[3] += 1;
        } else {
            r.timeout_fee[r.timeout_fee.length - counters[3]] = x;
            counters[3] -= 1;
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
    function _decode_CosmosBaseV1beta1Coin(
        uint256 p,
        bytes memory bs
    ) internal pure returns (CosmosBaseV1beta1Coin.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosBaseV1beta1Coin.Data memory r,) =
            CosmosBaseV1beta1Coin._decode(pointer, bs, sz);
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
        if (r.recv_fee.length != 0) {
            for (i = 0; i < r.recv_fee.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += CosmosBaseV1beta1Coin._encode_nested(
                    r.recv_fee[i], pointer, bs
                );
            }
        }
        if (r.ack_fee.length != 0) {
            for (i = 0; i < r.ack_fee.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += CosmosBaseV1beta1Coin._encode_nested(
                    r.ack_fee[i], pointer, bs
                );
            }
        }
        if (r.timeout_fee.length != 0) {
            for (i = 0; i < r.timeout_fee.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += CosmosBaseV1beta1Coin._encode_nested(
                    r.timeout_fee[i], pointer, bs
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
        for (i = 0; i < r.recv_fee.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    CosmosBaseV1beta1Coin._estimate(r.recv_fee[i])
                );
        }
        for (i = 0; i < r.ack_fee.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    CosmosBaseV1beta1Coin._estimate(r.ack_fee[i])
                );
        }
        for (i = 0; i < r.timeout_fee.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    CosmosBaseV1beta1Coin._estimate(r.timeout_fee[i])
                );
        }
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.recv_fee.length != 0) {
            return false;
        }

        if (r.ack_fee.length != 0) {
            return false;
        }

        if (r.timeout_fee.length != 0) {
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
        for (uint256 i1 = 0; i1 < input.recv_fee.length; i1++) {
            output.recv_fee.push(input.recv_fee[i1]);
        }

        for (uint256 i2 = 0; i2 < input.ack_fee.length; i2++) {
            output.ack_fee.push(input.ack_fee[i2]);
        }

        for (uint256 i3 = 0; i3 < input.timeout_fee.length; i3++) {
            output.timeout_fee.push(input.timeout_fee[i3]);
        }
    }

    //array helpers for RecvFee
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addRecvFee(
        Data memory self,
        CosmosBaseV1beta1Coin.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        CosmosBaseV1beta1Coin.Data[] memory tmp =
            new CosmosBaseV1beta1Coin.Data[](self.recv_fee.length + 1);
        for (uint256 i; i < self.recv_fee.length; i++) {
            tmp[i] = self.recv_fee[i];
        }
        tmp[self.recv_fee.length] = value;
        self.recv_fee = tmp;
    }

    //array helpers for AckFee
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addAckFee(
        Data memory self,
        CosmosBaseV1beta1Coin.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        CosmosBaseV1beta1Coin.Data[] memory tmp =
            new CosmosBaseV1beta1Coin.Data[](self.ack_fee.length + 1);
        for (uint256 i; i < self.ack_fee.length; i++) {
            tmp[i] = self.ack_fee[i];
        }
        tmp[self.ack_fee.length] = value;
        self.ack_fee = tmp;
    }

    //array helpers for TimeoutFee
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addTimeoutFee(
        Data memory self,
        CosmosBaseV1beta1Coin.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        CosmosBaseV1beta1Coin.Data[] memory tmp =
            new CosmosBaseV1beta1Coin.Data[](self.timeout_fee.length + 1);
        for (uint256 i; i < self.timeout_fee.length; i++) {
            tmp[i] = self.timeout_fee[i];
        }
        tmp[self.timeout_fee.length] = value;
        self.timeout_fee = tmp;
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

//library IbcApplicationsFeeV1Fee

library IbcApplicationsFeeV1PacketFee {
    //struct definition
    struct Data {
        IbcApplicationsFeeV1Fee.Data fee;
        string refund_address;
        string[] relayers;
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
                pointer += _read_fee(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_refund_address(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_unpacked_repeated_relayers(
                    pointer, bs, nil(), counters
                );
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[3] > 0) {
            require(r.relayers.length == 0);
            r.relayers = new string[](counters[3]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 3) {
                pointer +=
                    _read_unpacked_repeated_relayers(pointer, bs, r, counters);
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
    function _read_fee(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcApplicationsFeeV1Fee.Data memory x, uint256 sz) =
            _decode_IbcApplicationsFeeV1Fee(p, bs);
        r.fee = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_refund_address(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.refund_address = x;
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
    function _read_unpacked_repeated_relayers(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[4] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        if (isNil(r)) {
            counters[3] += 1;
        } else {
            r.relayers[r.relayers.length - counters[3]] = x;
            counters[3] -= 1;
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
    function _decode_IbcApplicationsFeeV1Fee(
        uint256 p,
        bytes memory bs
    ) internal pure returns (IbcApplicationsFeeV1Fee.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcApplicationsFeeV1Fee.Data memory r,) =
            IbcApplicationsFeeV1Fee._decode(pointer, bs, sz);
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

        pointer += ProtoBufRuntime._encode_key(
            1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += IbcApplicationsFeeV1Fee._encode_nested(r.fee, pointer, bs);

        if (bytes(r.refund_address).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_string(r.refund_address, pointer, bs);
        }
        if (r.relayers.length != 0) {
            for (i = 0; i < r.relayers.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer +=
                    ProtoBufRuntime._encode_string(r.relayers[i], pointer, bs);
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
        e += 1
            + ProtoBufRuntime._sz_lendelim(IbcApplicationsFeeV1Fee._estimate(r.fee));
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.refund_address).length);
        for (i = 0; i < r.relayers.length; i++) {
            e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.relayers[i]).length);
        }
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.refund_address).length != 0) {
            return false;
        }

        if (r.relayers.length != 0) {
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
        IbcApplicationsFeeV1Fee.store(input.fee, output.fee);
        output.refund_address = input.refund_address;
        output.relayers = input.relayers;
    }

    //array helpers for Relayers
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addRelayers(Data memory self, string memory value) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        string[] memory tmp = new string[](self.relayers.length + 1);
        for (uint256 i; i < self.relayers.length; i++) {
            tmp[i] = self.relayers[i];
        }
        tmp[self.relayers.length] = value;
        self.relayers = tmp;
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

//library IbcApplicationsFeeV1PacketFee

library IbcApplicationsFeeV1PacketFees {
    //struct definition
    struct Data {
        IbcApplicationsFeeV1PacketFee.Data[] packet_fees;
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
                pointer += _read_unpacked_repeated_packet_fees(
                    pointer, bs, nil(), counters
                );
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[1] > 0) {
            require(r.packet_fees.length == 0);
            r.packet_fees =
                new IbcApplicationsFeeV1PacketFee.Data[](counters[1]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer += _read_unpacked_repeated_packet_fees(
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
    function _read_unpacked_repeated_packet_fees(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[2] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcApplicationsFeeV1PacketFee.Data memory x, uint256 sz) =
            _decode_IbcApplicationsFeeV1PacketFee(p, bs);
        if (isNil(r)) {
            counters[1] += 1;
        } else {
            r.packet_fees[r.packet_fees.length - counters[1]] = x;
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
    function _decode_IbcApplicationsFeeV1PacketFee(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcApplicationsFeeV1PacketFee.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcApplicationsFeeV1PacketFee.Data memory r,) =
            IbcApplicationsFeeV1PacketFee._decode(pointer, bs, sz);
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
        if (r.packet_fees.length != 0) {
            for (i = 0; i < r.packet_fees.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcApplicationsFeeV1PacketFee._encode_nested(
                    r.packet_fees[i], pointer, bs
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
        for (i = 0; i < r.packet_fees.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcApplicationsFeeV1PacketFee._estimate(r.packet_fees[i])
                );
        }
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.packet_fees.length != 0) {
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
        for (uint256 i1 = 0; i1 < input.packet_fees.length; i1++) {
            output.packet_fees.push(input.packet_fees[i1]);
        }
    }

    //array helpers for PacketFees
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addPacketFees(
        Data memory self,
        IbcApplicationsFeeV1PacketFee.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcApplicationsFeeV1PacketFee.Data[] memory tmp = new IbcApplicationsFeeV1PacketFee
            .Data[](self.packet_fees.length + 1);
        for (uint256 i; i < self.packet_fees.length; i++) {
            tmp[i] = self.packet_fees[i];
        }
        tmp[self.packet_fees.length] = value;
        self.packet_fees = tmp;
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

//library IbcApplicationsFeeV1PacketFees

library IbcApplicationsFeeV1IdentifiedPacketFees {
    //struct definition
    struct Data {
        IbcCoreChannelV1PacketId.Data packet_id;
        IbcApplicationsFeeV1PacketFee.Data[] packet_fees;
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
                pointer += _read_packet_id(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_packet_fees(
                    pointer, bs, nil(), counters
                );
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[2] > 0) {
            require(r.packet_fees.length == 0);
            r.packet_fees =
                new IbcApplicationsFeeV1PacketFee.Data[](counters[2]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 2) {
                pointer += _read_unpacked_repeated_packet_fees(
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

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @param counters The counters for repeated fields
     * @return The number of bytes decoded
     */
    function _read_unpacked_repeated_packet_fees(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[3] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcApplicationsFeeV1PacketFee.Data memory x, uint256 sz) =
            _decode_IbcApplicationsFeeV1PacketFee(p, bs);
        if (isNil(r)) {
            counters[2] += 1;
        } else {
            r.packet_fees[r.packet_fees.length - counters[2]] = x;
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

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcApplicationsFeeV1PacketFee(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcApplicationsFeeV1PacketFee.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcApplicationsFeeV1PacketFee.Data memory r,) =
            IbcApplicationsFeeV1PacketFee._decode(pointer, bs, sz);
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

        pointer += ProtoBufRuntime._encode_key(
            1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            IbcCoreChannelV1PacketId._encode_nested(r.packet_id, pointer, bs);

        if (r.packet_fees.length != 0) {
            for (i = 0; i < r.packet_fees.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcApplicationsFeeV1PacketFee._encode_nested(
                    r.packet_fees[i], pointer, bs
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
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreChannelV1PacketId._estimate(r.packet_id)
            );
        for (i = 0; i < r.packet_fees.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcApplicationsFeeV1PacketFee._estimate(r.packet_fees[i])
                );
        }
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.packet_fees.length != 0) {
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
        IbcCoreChannelV1PacketId.store(input.packet_id, output.packet_id);

        for (uint256 i2 = 0; i2 < input.packet_fees.length; i2++) {
            output.packet_fees.push(input.packet_fees[i2]);
        }
    }

    //array helpers for PacketFees
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addPacketFees(
        Data memory self,
        IbcApplicationsFeeV1PacketFee.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcApplicationsFeeV1PacketFee.Data[] memory tmp = new IbcApplicationsFeeV1PacketFee
            .Data[](self.packet_fees.length + 1);
        for (uint256 i; i < self.packet_fees.length; i++) {
            tmp[i] = self.packet_fees[i];
        }
        tmp[self.packet_fees.length] = value;
        self.packet_fees = tmp;
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
//library IbcApplicationsFeeV1IdentifiedPacketFees
