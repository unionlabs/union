pragma solidity ^0.8.23;

import "./../proto/ProtoBufRuntime.sol";

library Encoder {
    uint64 private constant _MAX_UINT64 = 0xFFFFFFFFFFFFFFFF;

    function cdcEncode(string memory item)
        internal
        pure
        returns (bytes memory)
    {
        uint256 estimatedSize =
            1 + ProtoBufRuntime._sz_lendelim(bytes(item).length);
        bytes memory bs = new bytes(estimatedSize);

        uint256 offset = 32;
        uint256 pointer = 32;

        if (bytes(item).length > 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(item, pointer, bs);
        }

        uint256 sz = pointer - offset;
        assembly {
            mstore(bs, sz)
        }
        return bs;
    }

    function cdcEncode(bytes memory item)
        internal
        pure
        returns (bytes memory)
    {
        uint256 estimatedSize = 1 + ProtoBufRuntime._sz_lendelim(item.length);
        bytes memory bs = new bytes(estimatedSize);

        uint256 offset = 32;
        uint256 pointer = 32;

        if (item.length > 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(item, pointer, bs);
        }

        uint256 sz = pointer - offset;
        assembly {
            mstore(bs, sz)
        }
        return bs;
    }

    function cdcEncode(int64 item) internal pure returns (bytes memory) {
        uint256 estimatedSize = 1 + ProtoBufRuntime._sz_int64(item);
        bytes memory bs = new bytes(estimatedSize);

        uint256 offset = 32;
        uint256 pointer = 32;

        if (item != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_int64(item, pointer, bs);
        }

        uint256 sz = pointer - offset;
        assembly {
            mstore(bs, sz)
        }
        return bs;
    }

    // TODO: Can we make this cheaper?
    // https://docs.soliditylang.org/en/v0.6.5/types.html#allocating-memory-arrays
    function encodeDelim(bytes memory input)
        internal
        pure
        returns (bytes memory)
    {
        require(input.length < _MAX_UINT64, "Encoder: out of bounds (uint64)");

        uint64 length = uint64(input.length);
        uint256 additionalEstimated = ProtoBufRuntime._sz_uint64(length);

        bytes memory delimitedPrefix = new bytes(additionalEstimated);
        uint256 delimitedPrefixLen =
            ProtoBufRuntime._encode_uint64(length, 32, delimitedPrefix);

        assembly {
            mstore(delimitedPrefix, delimitedPrefixLen)
        }

        // concatenate buffers
        return abi.encodePacked(delimitedPrefix, input);
    }
}
