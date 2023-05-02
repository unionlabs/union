// SPDX-License-Identifier: TBD
pragma solidity ^0.8.18;

library MerkleTree {
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
        uint8 leafPrefix = 0x00;
        return sha256(abi.encodePacked(leafPrefix, leaf));
    }

    /**
     * @dev returns tmhash(0x01 || left || right)
     */
    function innerHash(bytes32 leaf, bytes32 right) internal pure returns (bytes32) {
        uint8 innerPrefix = 0x01;
        return sha256(abi.encodePacked(innerPrefix, leaf, right));
    }

    /**
     * @dev returns the largest power of 2 less than length
     *
     * TODO: This function can be optimized with bit shifting approach:
     * https://www.baeldung.com/java-largest-power-of-2-less-than-number
     */
    function getSplitPoint(uint256 input) internal pure returns (uint) {
        require(input > 1, "MerkleTree: invalid input");

        uint result = 1;
        for (uint i = input - 1; i > 1; i--) {
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
    function merkleRootHash(bytes[] memory data, uint start, uint total) internal pure returns (bytes32) {
        if (total == 0) {
            return emptyHash();
        } else if (total == 1) {
            return leafHash(data[start]);
        }  else {
            uint k = getSplitPoint(total);
            bytes32 left = merkleRootHash(data, start, k); // validators[:k]
            bytes32 right = merkleRootHash(data, start+k, total-k); // validators[k:]
            return innerHash(left, right);
        }
    }
}
