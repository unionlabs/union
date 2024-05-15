pragma solidity ^0.8.23;

import "../../../../ProtoBufRuntime.sol";
import "../../../../GoogleProtobufAny.sol";

import "./channel.sol";

library IbcCoreChannelV1GenesisState {
    //struct definition
    struct Data {
        IbcCoreChannelV1IdentifiedChannel.Data[] channels;
        IbcCoreChannelV1PacketState.Data[] acknowledgements;
        IbcCoreChannelV1PacketState.Data[] commitments;
        IbcCoreChannelV1PacketState.Data[] receipts;
        IbcCoreChannelV1PacketSequence.Data[] send_sequences;
        IbcCoreChannelV1PacketSequence.Data[] recv_sequences;
        IbcCoreChannelV1PacketSequence.Data[] ack_sequences;
        uint64 next_channel_sequence;
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
        uint256[9] memory counters;
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
                pointer += _read_unpacked_repeated_channels(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_acknowledgements(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 3) {
                pointer += _read_unpacked_repeated_commitments(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 4) {
                pointer += _read_unpacked_repeated_receipts(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 5) {
                pointer += _read_unpacked_repeated_send_sequences(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 6) {
                pointer += _read_unpacked_repeated_recv_sequences(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 7) {
                pointer += _read_unpacked_repeated_ack_sequences(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 8) {
                pointer += _read_next_channel_sequence(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[1] > 0) {
            require(r.channels.length == 0);
            r.channels =
                new IbcCoreChannelV1IdentifiedChannel.Data[](counters[1]);
        }
        if (counters[2] > 0) {
            require(r.acknowledgements.length == 0);
            r.acknowledgements =
                new IbcCoreChannelV1PacketState.Data[](counters[2]);
        }
        if (counters[3] > 0) {
            require(r.commitments.length == 0);
            r.commitments = new IbcCoreChannelV1PacketState.Data[](counters[3]);
        }
        if (counters[4] > 0) {
            require(r.receipts.length == 0);
            r.receipts = new IbcCoreChannelV1PacketState.Data[](counters[4]);
        }
        if (counters[5] > 0) {
            require(r.send_sequences.length == 0);
            r.send_sequences =
                new IbcCoreChannelV1PacketSequence.Data[](counters[5]);
        }
        if (counters[6] > 0) {
            require(r.recv_sequences.length == 0);
            r.recv_sequences =
                new IbcCoreChannelV1PacketSequence.Data[](counters[6]);
        }
        if (counters[7] > 0) {
            require(r.ack_sequences.length == 0);
            r.ack_sequences =
                new IbcCoreChannelV1PacketSequence.Data[](counters[7]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer +=
                    _read_unpacked_repeated_channels(pointer, bs, r, counters);
            } else if (fieldId == 2) {
                pointer += _read_unpacked_repeated_acknowledgements(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 3) {
                pointer += _read_unpacked_repeated_commitments(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 4) {
                pointer +=
                    _read_unpacked_repeated_receipts(pointer, bs, r, counters);
            } else if (fieldId == 5) {
                pointer += _read_unpacked_repeated_send_sequences(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 6) {
                pointer += _read_unpacked_repeated_recv_sequences(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 7) {
                pointer += _read_unpacked_repeated_ack_sequences(
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
    function _read_unpacked_repeated_channels(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[9] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreChannelV1IdentifiedChannel.Data memory x, uint256 sz) =
            _decode_IbcCoreChannelV1IdentifiedChannel(p, bs);
        if (isNil(r)) {
            counters[1] += 1;
        } else {
            r.channels[r.channels.length - counters[1]] = x;
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
    function _read_unpacked_repeated_acknowledgements(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[9] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreChannelV1PacketState.Data memory x, uint256 sz) =
            _decode_IbcCoreChannelV1PacketState(p, bs);
        if (isNil(r)) {
            counters[2] += 1;
        } else {
            r.acknowledgements[r.acknowledgements.length - counters[2]] = x;
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
    function _read_unpacked_repeated_commitments(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[9] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreChannelV1PacketState.Data memory x, uint256 sz) =
            _decode_IbcCoreChannelV1PacketState(p, bs);
        if (isNil(r)) {
            counters[3] += 1;
        } else {
            r.commitments[r.commitments.length - counters[3]] = x;
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
    function _read_unpacked_repeated_receipts(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[9] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreChannelV1PacketState.Data memory x, uint256 sz) =
            _decode_IbcCoreChannelV1PacketState(p, bs);
        if (isNil(r)) {
            counters[4] += 1;
        } else {
            r.receipts[r.receipts.length - counters[4]] = x;
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
    function _read_unpacked_repeated_send_sequences(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[9] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreChannelV1PacketSequence.Data memory x, uint256 sz) =
            _decode_IbcCoreChannelV1PacketSequence(p, bs);
        if (isNil(r)) {
            counters[5] += 1;
        } else {
            r.send_sequences[r.send_sequences.length - counters[5]] = x;
            counters[5] -= 1;
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
    function _read_unpacked_repeated_recv_sequences(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[9] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreChannelV1PacketSequence.Data memory x, uint256 sz) =
            _decode_IbcCoreChannelV1PacketSequence(p, bs);
        if (isNil(r)) {
            counters[6] += 1;
        } else {
            r.recv_sequences[r.recv_sequences.length - counters[6]] = x;
            counters[6] -= 1;
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
    function _read_unpacked_repeated_ack_sequences(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[9] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (IbcCoreChannelV1PacketSequence.Data memory x, uint256 sz) =
            _decode_IbcCoreChannelV1PacketSequence(p, bs);
        if (isNil(r)) {
            counters[7] += 1;
        } else {
            r.ack_sequences[r.ack_sequences.length - counters[7]] = x;
            counters[7] -= 1;
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
    function _read_next_channel_sequence(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (uint64 x, uint256 sz) = ProtoBufRuntime._decode_uint64(p, bs);
        r.next_channel_sequence = x;
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
    function _decode_IbcCoreChannelV1IdentifiedChannel(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcCoreChannelV1IdentifiedChannel.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreChannelV1IdentifiedChannel.Data memory r,) =
            IbcCoreChannelV1IdentifiedChannel._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreChannelV1PacketState(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcCoreChannelV1PacketState.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreChannelV1PacketState.Data memory r,) =
            IbcCoreChannelV1PacketState._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreChannelV1PacketSequence(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcCoreChannelV1PacketSequence.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreChannelV1PacketSequence.Data memory r,) =
            IbcCoreChannelV1PacketSequence._decode(pointer, bs, sz);
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
        if (r.channels.length != 0) {
            for (i = 0; i < r.channels.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreChannelV1IdentifiedChannel._encode_nested(
                    r.channels[i], pointer, bs
                );
            }
        }
        if (r.acknowledgements.length != 0) {
            for (i = 0; i < r.acknowledgements.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreChannelV1PacketState._encode_nested(
                    r.acknowledgements[i], pointer, bs
                );
            }
        }
        if (r.commitments.length != 0) {
            for (i = 0; i < r.commitments.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreChannelV1PacketState._encode_nested(
                    r.commitments[i], pointer, bs
                );
            }
        }
        if (r.receipts.length != 0) {
            for (i = 0; i < r.receipts.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreChannelV1PacketState._encode_nested(
                    r.receipts[i], pointer, bs
                );
            }
        }
        if (r.send_sequences.length != 0) {
            for (i = 0; i < r.send_sequences.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    5, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreChannelV1PacketSequence._encode_nested(
                    r.send_sequences[i], pointer, bs
                );
            }
        }
        if (r.recv_sequences.length != 0) {
            for (i = 0; i < r.recv_sequences.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    6, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreChannelV1PacketSequence._encode_nested(
                    r.recv_sequences[i], pointer, bs
                );
            }
        }
        if (r.ack_sequences.length != 0) {
            for (i = 0; i < r.ack_sequences.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    7, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += IbcCoreChannelV1PacketSequence._encode_nested(
                    r.ack_sequences[i], pointer, bs
                );
            }
        }
        if (r.next_channel_sequence != 0) {
            pointer += ProtoBufRuntime._encode_key(
                8, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_uint64(
                r.next_channel_sequence, pointer, bs
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
        for (i = 0; i < r.channels.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreChannelV1IdentifiedChannel._estimate(r.channels[i])
                );
        }
        for (i = 0; i < r.acknowledgements.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreChannelV1PacketState._estimate(r.acknowledgements[i])
                );
        }
        for (i = 0; i < r.commitments.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreChannelV1PacketState._estimate(r.commitments[i])
                );
        }
        for (i = 0; i < r.receipts.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreChannelV1PacketState._estimate(r.receipts[i])
                );
        }
        for (i = 0; i < r.send_sequences.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreChannelV1PacketSequence._estimate(r.send_sequences[i])
                );
        }
        for (i = 0; i < r.recv_sequences.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreChannelV1PacketSequence._estimate(r.recv_sequences[i])
                );
        }
        for (i = 0; i < r.ack_sequences.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    IbcCoreChannelV1PacketSequence._estimate(r.ack_sequences[i])
                );
        }
        e += 1 + ProtoBufRuntime._sz_uint64(r.next_channel_sequence);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.channels.length != 0) {
            return false;
        }

        if (r.acknowledgements.length != 0) {
            return false;
        }

        if (r.commitments.length != 0) {
            return false;
        }

        if (r.receipts.length != 0) {
            return false;
        }

        if (r.send_sequences.length != 0) {
            return false;
        }

        if (r.recv_sequences.length != 0) {
            return false;
        }

        if (r.ack_sequences.length != 0) {
            return false;
        }

        if (r.next_channel_sequence != 0) {
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
        for (uint256 i1 = 0; i1 < input.channels.length; i1++) {
            output.channels.push(input.channels[i1]);
        }

        for (uint256 i2 = 0; i2 < input.acknowledgements.length; i2++) {
            output.acknowledgements.push(input.acknowledgements[i2]);
        }

        for (uint256 i3 = 0; i3 < input.commitments.length; i3++) {
            output.commitments.push(input.commitments[i3]);
        }

        for (uint256 i4 = 0; i4 < input.receipts.length; i4++) {
            output.receipts.push(input.receipts[i4]);
        }

        for (uint256 i5 = 0; i5 < input.send_sequences.length; i5++) {
            output.send_sequences.push(input.send_sequences[i5]);
        }

        for (uint256 i6 = 0; i6 < input.recv_sequences.length; i6++) {
            output.recv_sequences.push(input.recv_sequences[i6]);
        }

        for (uint256 i7 = 0; i7 < input.ack_sequences.length; i7++) {
            output.ack_sequences.push(input.ack_sequences[i7]);
        }

        output.next_channel_sequence = input.next_channel_sequence;
    }

    //array helpers for Channels
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addChannels(
        Data memory self,
        IbcCoreChannelV1IdentifiedChannel.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreChannelV1IdentifiedChannel.Data[] memory tmp = new IbcCoreChannelV1IdentifiedChannel
            .Data[](self.channels.length + 1);
        for (uint256 i; i < self.channels.length; i++) {
            tmp[i] = self.channels[i];
        }
        tmp[self.channels.length] = value;
        self.channels = tmp;
    }

    //array helpers for Acknowledgements
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addAcknowledgements(
        Data memory self,
        IbcCoreChannelV1PacketState.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreChannelV1PacketState.Data[] memory tmp = new IbcCoreChannelV1PacketState
            .Data[](self.acknowledgements.length + 1);
        for (uint256 i; i < self.acknowledgements.length; i++) {
            tmp[i] = self.acknowledgements[i];
        }
        tmp[self.acknowledgements.length] = value;
        self.acknowledgements = tmp;
    }

    //array helpers for Commitments
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addCommitments(
        Data memory self,
        IbcCoreChannelV1PacketState.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreChannelV1PacketState.Data[] memory tmp =
            new IbcCoreChannelV1PacketState.Data[](self.commitments.length + 1);
        for (uint256 i; i < self.commitments.length; i++) {
            tmp[i] = self.commitments[i];
        }
        tmp[self.commitments.length] = value;
        self.commitments = tmp;
    }

    //array helpers for Receipts
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addReceipts(
        Data memory self,
        IbcCoreChannelV1PacketState.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreChannelV1PacketState.Data[] memory tmp =
            new IbcCoreChannelV1PacketState.Data[](self.receipts.length + 1);
        for (uint256 i; i < self.receipts.length; i++) {
            tmp[i] = self.receipts[i];
        }
        tmp[self.receipts.length] = value;
        self.receipts = tmp;
    }

    //array helpers for SendSequences
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addSendSequences(
        Data memory self,
        IbcCoreChannelV1PacketSequence.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreChannelV1PacketSequence.Data[] memory tmp = new IbcCoreChannelV1PacketSequence
            .Data[](self.send_sequences.length + 1);
        for (uint256 i; i < self.send_sequences.length; i++) {
            tmp[i] = self.send_sequences[i];
        }
        tmp[self.send_sequences.length] = value;
        self.send_sequences = tmp;
    }

    //array helpers for RecvSequences
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addRecvSequences(
        Data memory self,
        IbcCoreChannelV1PacketSequence.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreChannelV1PacketSequence.Data[] memory tmp = new IbcCoreChannelV1PacketSequence
            .Data[](self.recv_sequences.length + 1);
        for (uint256 i; i < self.recv_sequences.length; i++) {
            tmp[i] = self.recv_sequences[i];
        }
        tmp[self.recv_sequences.length] = value;
        self.recv_sequences = tmp;
    }

    //array helpers for AckSequences
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addAckSequences(
        Data memory self,
        IbcCoreChannelV1PacketSequence.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        IbcCoreChannelV1PacketSequence.Data[] memory tmp = new IbcCoreChannelV1PacketSequence
            .Data[](self.ack_sequences.length + 1);
        for (uint256 i; i < self.ack_sequences.length; i++) {
            tmp[i] = self.ack_sequences[i];
        }
        tmp[self.ack_sequences.length] = value;
        self.ack_sequences = tmp;
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

//library IbcCoreChannelV1GenesisState

library IbcCoreChannelV1PacketSequence {
    //struct definition
    struct Data {
        string port_id;
        string channel_id;
        uint64 sequence;
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
            } else if (fieldId == 3) {
                pointer += _read_sequence(pointer, bs, r);
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

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_sequence(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (uint64 x, uint256 sz) = ProtoBufRuntime._decode_uint64(p, bs);
        r.sequence = x;
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
        if (r.sequence != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_uint64(r.sequence, pointer, bs);
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
        e += 1 + ProtoBufRuntime._sz_uint64(r.sequence);
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

        if (r.sequence != 0) {
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
        output.sequence = input.sequence;
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
//library IbcCoreChannelV1PacketSequence
