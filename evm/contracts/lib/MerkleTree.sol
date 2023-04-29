// SPDX-License-Identifier: TBD
pragma solidity ^0.8.18;

library MerkleTree {
   function hashFromByteSlices(bytes[] memory data)
        internal
        pure
        returns (bytes32)
    {
        uint256 n = data.length;
        if (n == 0) return sha256(new bytes(0));
        if (n == 1) return leafHash(data[0]);
        uint256 k = getSplitPoint(data.length);
        bytes32 left = hashFromByteSlices(subArray(data, 0, k));
        bytes32 right = hashFromByteSlices(subArray(data, k, n));
        return innerHash(left, right);
    }

    function leafHash(bytes memory data) internal pure returns (bytes32) {
        bytes memory rs = new bytes(data.length + 1);
        rs[0] = 0x00;
        for (uint256 i = 0; i < data.length; i++) {
            rs[i + 1] = data[i];
        }
        return sha256(rs);
    }

    function innerHash(bytes32 left, bytes32 right)
        internal
        pure
        returns (bytes32)
    {
        bytes memory rs = new bytes(left.length + right.length + 1);
        rs[0] = 0x01;

        uint256 offset = 1;
        for (uint256 i = 0; i < left.length; i++) {
            rs[offset] = left[i];
            offset++;
        }

        for (uint256 i = 0; i < right.length; i++) {
            rs[offset] = right[i];
            offset++;
        }
        return sha256(rs);
    }

    function getSplitPoint(uint256 n) internal pure returns (uint256) {
        require(n >= 1, "Trying to split a tree with size < 1");
        if (n == 1) {
            return 0;
        }

        uint256 splitPoint = 1;
        while (splitPoint * 2 < n) {
            splitPoint *= 2;
        }
        return splitPoint;
    }

    function subArray(
        bytes[] memory data,
        uint256 begin,
        uint256 end
    ) internal pure returns (bytes[] memory) {
        bytes[] memory ret = new bytes[](end - begin);
        for (uint256 i = 0; i < ret.length; i++) {
            ret[i] = data[begin + i];
        }
        return ret;
    }}
