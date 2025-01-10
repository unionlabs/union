// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "../../../contracts/lib/SparseMerkleVerifier.sol";

contract SparseMerkleVerifierTests is Test {
    using SparseMerkleVerifier for SparseMerkleVerifier.SparseMerkleProof;

    /// @notice Demonstrates how to verify a single-sibling proof.
    function test_verify_ok() public {
        // Pause gas metering to mimic the style in MPTVerifierTests
        vm.pauseGasMetering();

        // 1. Setup test data: an element key/value, plus one sibling
        bytes32 elementKey = keccak256(abi.encodePacked("some-key"));
        bytes32 elementValue = keccak256(abi.encodePacked("some-value"));
        bytes32 siblingHash = keccak256(abi.encodePacked("some-sibling"));

        // 2. Manually compute the final expected root.
        //    i) Start with the leaf hash: hashSparseMerkleLeafNode(key, value)
        bytes32 leafTag = keccak256(bytes("APTOS::SparseMerkleLeafNode"));
        bytes32 leafHash =
            keccak256(abi.encodePacked(leafTag, elementKey, elementValue));

        //    ii) Suppose in our fake scenario the bit for this sibling is `false`,
        //        meaning we do hashSparseMerkleInternal(leafHash, siblingHash).
        bytes32 internalTag = keccak256(bytes("APTOS::SparseMerkleInternal"));
        bytes32 finalRootHash =
            keccak256(abi.encodePacked(internalTag, leafHash, siblingHash));

        // 3. Build the SparseMerkleProof struct
        //    In a real scenario, the sibling order and "bits" come from a real SMT proof.
        SparseMerkleVerifier.SparseMerkleLeafNode memory leaf =
        SparseMerkleVerifier.SparseMerkleLeafNode({
            key: elementKey,
            valueHash: elementValue,
            exists: true
        });

        bytes32[] memory siblings = new bytes32[](1);
        siblings[0] = siblingHash;

        SparseMerkleVerifier.SparseMerkleProof memory proof =
        SparseMerkleVerifier.SparseMerkleProof({leaf: leaf, siblings: siblings});

        // 4. Call our wrapper check to see if it verifies
        this.checkExistence(proof, finalRootHash, elementKey, elementValue);
    }

    /// @notice Helper that resumes gas metering, then calls verifyExistenceProof,
    ///         and checks it returns true (membership verified).
    function checkExistence(
        SparseMerkleVerifier.SparseMerkleProof memory proof,
        bytes32 expectedRootHash,
        bytes32 elementKey,
        bytes32 elementHash
    ) public {
        // Resume gas metering before the library call
        vm.resumeGasMetering();

        bool exists = proof.verifyExistenceProof(
            expectedRootHash, elementKey, elementHash
        );
        assertEq(
            exists,
            true,
            "SparseMerkleVerifier: expected membership proof to succeed"
        );
    }
}
