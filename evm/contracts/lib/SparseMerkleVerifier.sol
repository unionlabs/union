// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

library SparseMerkleVerifier {
    struct SparseMerkleLeafNode {
        bytes32 key;
        bytes32 valueHash;
        bool exists;
    }

    struct SparseMerkleProof {
        SparseMerkleLeafNode leaf;
        bytes32[] siblings;
    }

    function hashSparseMerkleLeafNode(
        bytes32 key,
        bytes32 valueHash
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encodePacked(
                keccak256(bytes("APTOS::SparseMerkleLeafNode")), key, valueHash
            )
        );
    }

    function hashSparseMerkleInternal(
        bytes32 leftChild,
        bytes32 rightChild
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encodePacked(
                keccak256(bytes("APTOS::SparseMerkleInternal")),
                leftChild,
                rightChild
            )
        );
    }

    function getBit(
        bytes32 data,
        uint256 bitIndex
    ) internal pure returns (bool) {
        require(bitIndex < 256, "bitIndex out of range");

        uint256 bytePos = bitIndex / 8;
        uint256 bitPosInByte = 7 - (bitIndex % 8);

        bytes1 b = data[bytePos];

        return (uint8(b) >> bitPosInByte) & 0x01 == 1;
    }

    error MaxSiblingsExceeded(uint256 limit, uint256 given);
    error ExpectedMembershipVerification();
    error LeafKeyMismatch(bytes32 expected, bytes32 actual);
    error LeafValueMismatch(bytes32 expected, bytes32 actual);
    error RootHashMismatch(bytes32 expected, bytes32 given);

    function verifyExistenceProof(
        SparseMerkleProof memory proof,
        bytes32 expectedRootHash,
        bytes32 elementKey,
        bytes32 elementHash
    ) public pure returns (bool) {
        if (proof.siblings.length > 256) {
            revert MaxSiblingsExceeded(256, proof.siblings.length);
        }

        if (!proof.leaf.exists) {
            revert ExpectedMembershipVerification();
        }

        if (proof.leaf.key != elementKey) {
            revert LeafKeyMismatch(elementKey, proof.leaf.key);
        }

        if (proof.leaf.valueHash != elementHash) {
            revert LeafValueMismatch(elementHash, proof.leaf.valueHash);
        }

        bytes32 currentHash =
            hashSparseMerkleLeafNode(proof.leaf.key, proof.leaf.valueHash);

        uint256 totalSiblings = proof.siblings.length;
        for (uint256 i = 0; i < totalSiblings; i++) {
            uint256 bitIndex = (256 - totalSiblings) + i;

            bool bit = getBit(elementKey, bitIndex);

            bytes32 siblingHash = proof.siblings[totalSiblings - 1 - i];

            if (bit) {
                currentHash = hashSparseMerkleInternal(siblingHash, currentHash);
            } else {
                currentHash = hashSparseMerkleInternal(currentHash, siblingHash);
            }
        }

        if (currentHash != expectedRootHash) {
            revert RootHashMismatch(expectedRootHash, currentHash);
        }

        return true;
    }
}
