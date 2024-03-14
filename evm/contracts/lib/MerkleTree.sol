pragma solidity ^0.8.23;

library MerkleTree {
    uint8 constant LEAF_PREFIX = 0x00;
    uint8 constant INNER_PREFIX = 0x01;

    function hashFromByteSlices(bytes[] memory data)
        internal
        pure
        returns (bytes32)
    {
        return merkleRootHash(data, 0, data.length);
    }

    /**
     * @dev returns empty hash
     */
    function emptyHash() internal pure returns (bytes32) {
        return sha256(abi.encode());
    }

    /**
     * @dev returns tmhash(0x00 || leaf)
     *
     */
    function leafHash(bytes memory leaf) internal pure returns (bytes32) {
        return sha256(abi.encodePacked(LEAF_PREFIX, leaf));
    }

    /**
     * @dev returns tmhash(0x01 || left || right)
     */
    function innerHash(
        bytes32 leaf,
        bytes32 right
    ) internal pure returns (bytes32) {
        return sha256(abi.encodePacked(INNER_PREFIX, leaf, right));
    }

    /**
     * @dev returns the largest power of 2 less than length
     *
     * TODO: This function can be optimized with bit shifting approach:
     * https://www.baeldung.com/java-largest-power-of-2-less-than-number
     */
    function getSplitPoint(uint256 input) internal pure returns (uint256) {
        require(input > 1, "MerkleTree: invalid input");

        uint256 result = 1;
        for (uint256 i = input - 1; i > 1; i--) {
            if ((i & (i - 1)) == 0) {
                result = i;
                break;
            }
        }
        return result;
    }

    /**
     * @dev computes a Merkle tree where the leaves are the byte slice in the provided order
     * Follows RFC-696
     */
    function merkleRootHash(
        bytes[] memory data,
        uint256 start,
        uint256 total
    ) internal pure returns (bytes32) {
        if (total == 0) {
            return emptyHash();
        } else if (total == 1) {
            return leafHash(data[start]);
        } else {
            uint256 k = getSplitPoint(total);
            bytes32 left = merkleRootHash(data, start, k); // validators[:k]
            bytes32 right = merkleRootHash(data, start + k, total - k); // validators[k:]
            return innerHash(left, right);
        }
    }

    function optimizedBlockRoot(bytes32[14] memory data)
        internal
        pure
        returns (bytes32)
    {
        bytes32 x0 = innerHash(data[0], data[1]);
        bytes32 x1 = innerHash(data[2], data[3]);
        bytes32 x2 = innerHash(data[4], data[5]);
        bytes32 x3 = innerHash(data[6], data[7]);
        bytes32 x4 = innerHash(data[8], data[9]);
        bytes32 x5 = innerHash(data[10], data[11]);
        bytes32 x6 = innerHash(data[12], data[13]);

        x0 = innerHash(x0, x1);
        x1 = innerHash(x2, x3);
        x2 = innerHash(x4, x5);
        x3 = x6;

        x0 = innerHash(x0, x1);
        x1 = innerHash(x2, x3);

        return innerHash(x0, x1);
    }
}
