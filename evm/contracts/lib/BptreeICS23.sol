pragma solidity ^0.8.27;

import "./UnionICS23.sol";

// Standalone ics23 verifier for gno.land's bptree `main` store, chained
// through a Tendermint-spec L1 layer. Kept separate from ICS23.sol/
// ICS23Verifier.sol rather than generalizing them, so IAVL-backed consumers
// (StateLensIcs23Ics23Client, CometblsClient) are unaffected.
// https://github.com/gnolang/gno/blob/master/tm2/pkg/bptree/proof_spec.go
library BptreeIcs23 {
    // bptree and tendermint share the same shape (child_size=32, prefix
    // length in [1,1]); only emptyChild/depth are bptree-specific and only
    // apply to the L2 layer.
    uint256 private constant CHILD_SIZE = 32;
    uint256 private constant MIN_PREFIX_LENGTH = 1;
    uint256 private constant MAX_PREFIX_LENGTH = 1;
    bytes32 private constant EMPTY_CHILD =
        0xdbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d986; // SHA256(0x02)
    uint256 private constant MIN_DEPTH = 5;
    uint256 private constant MAX_DEPTH = 60;

    enum VerifyChainedMembershipError {
        None,
        InvalidProofRoot,
        KeyMismatch,
        ValueMismatch,
        InvalidSpec,
        InvalidIntermediateProofRoot,
        IntermateProofRootMismatch,
        RootMismatch
    }

    enum VerifyChainedNonMembershipError {
        None,
        InvalidProofRoot,
        KeyMismatch,
        ValueMismatch,
        InvalidSpec,
        InvalidIntermediateProofRoot,
        IntermateProofRootMismatch,
        RootMismatch,
        VerifyLeft,
        VerifyRight,
        LeftAndRightKeyEmpty,
        RightKeyRange,
        LeftKeyRange,
        RightProofLeftMost,
        LeftProofRightMost,
        IsLeftNeighbor
    }

    function verifyChainedMembership(
        UnionIcs23.ExistenceProof[2] calldata proofs,
        bytes32 root,
        bytes memory prefix,
        bytes memory key,
        bytes calldata value
    ) internal pure returns (VerifyChainedMembershipError) {
        (bytes32 subroot, CalculateRootError rCode) =
            calculateRoot(proofs[0]);
        if (rCode != CalculateRootError.None) {
            return VerifyChainedMembershipError.InvalidProofRoot;
        }

        VerifyExistenceError vCode =
            verifyNoRootCheck(proofs[0], true, key, value);
        if (vCode != VerifyExistenceError.None) {
            return convertExistenceError(vCode);
        }

        vCode =
            verify(proofs[1], false, root, prefix, abi.encodePacked(subroot));
        if (vCode != VerifyExistenceError.None) {
            return convertExistenceError(vCode);
        }

        return VerifyChainedMembershipError.None;
    }

    function verifyChainedNonMembership(
        UnionIcs23.NonExistenceProof calldata l2Proof,
        UnionIcs23.ExistenceProof calldata l1Proof,
        bytes32 root,
        bytes memory prefix,
        bytes memory key
    ) internal pure returns (VerifyChainedNonMembershipError) {
        (bytes32 subroot, CalculateRootError rCode) =
            calculateRootNonExistence(l2Proof);
        if (rCode != CalculateRootError.None) {
            return VerifyChainedNonMembershipError.InvalidProofRoot;
        }

        VerifyNonExistenceError nCode = verifyNonExistence(l2Proof, subroot, key);
        if (nCode != VerifyNonExistenceError.None) {
            return convertNonExistenceError(nCode);
        }

        VerifyExistenceError vCode = verifyNoRootCheck(
            l1Proof, false, prefix, abi.encodePacked(subroot)
        );
        if (vCode != VerifyExistenceError.None) {
            return convertExistenceErrorForNonMembership(vCode);
        }

        (bytes32 l1Root, CalculateRootError l1RCode) = calculateRoot(l1Proof);
        if (l1RCode != CalculateRootError.None) {
            return VerifyChainedNonMembershipError.InvalidIntermediateProofRoot;
        }
        if (l1Root != root) {
            return VerifyChainedNonMembershipError.RootMismatch;
        }

        return VerifyChainedNonMembershipError.None;
    }

    function convertExistenceError(
        VerifyExistenceError vCode
    ) private pure returns (VerifyChainedMembershipError) {
        if (vCode == VerifyExistenceError.KeyNotMatching) {
            return VerifyChainedMembershipError.KeyMismatch;
        } else if (vCode == VerifyExistenceError.ValueNotMatching) {
            return VerifyChainedMembershipError.ValueMismatch;
        } else if (vCode == VerifyExistenceError.CheckSpec) {
            return VerifyChainedMembershipError.InvalidSpec;
        } else if (vCode == VerifyExistenceError.CalculateRoot) {
            return VerifyChainedMembershipError.InvalidIntermediateProofRoot;
        } else if (vCode == VerifyExistenceError.RootNotMatching) {
            return VerifyChainedMembershipError.IntermateProofRootMismatch;
        }
        revert("BptreeIcs23: non exhaustive pattern matching");
    }

    function convertExistenceErrorForNonMembership(
        VerifyExistenceError vCode
    ) private pure returns (VerifyChainedNonMembershipError) {
        if (vCode == VerifyExistenceError.KeyNotMatching) {
            return VerifyChainedNonMembershipError.KeyMismatch;
        } else if (vCode == VerifyExistenceError.ValueNotMatching) {
            return VerifyChainedNonMembershipError.ValueMismatch;
        } else if (vCode == VerifyExistenceError.CheckSpec) {
            return VerifyChainedNonMembershipError.InvalidSpec;
        } else if (vCode == VerifyExistenceError.CalculateRoot) {
            return VerifyChainedNonMembershipError.InvalidIntermediateProofRoot;
        } else if (vCode == VerifyExistenceError.RootNotMatching) {
            return VerifyChainedNonMembershipError.IntermateProofRootMismatch;
        }
        revert("BptreeIcs23: non exhaustive pattern matching");
    }

    function convertNonExistenceError(
        VerifyNonExistenceError vCode
    ) private pure returns (VerifyChainedNonMembershipError) {
        if (vCode == VerifyNonExistenceError.VerifyLeft) {
            return VerifyChainedNonMembershipError.VerifyLeft;
        } else if (vCode == VerifyNonExistenceError.VerifyRight) {
            return VerifyChainedNonMembershipError.VerifyRight;
        } else if (vCode == VerifyNonExistenceError.LeftAndRightKeyEmpty) {
            return VerifyChainedNonMembershipError.LeftAndRightKeyEmpty;
        } else if (vCode == VerifyNonExistenceError.RightKeyRange) {
            return VerifyChainedNonMembershipError.RightKeyRange;
        } else if (vCode == VerifyNonExistenceError.LeftKeyRange) {
            return VerifyChainedNonMembershipError.LeftKeyRange;
        } else if (vCode == VerifyNonExistenceError.RightProofLeftMost) {
            return VerifyChainedNonMembershipError.RightProofLeftMost;
        } else if (vCode == VerifyNonExistenceError.LeftProofRightMost) {
            return VerifyChainedNonMembershipError.LeftProofRightMost;
        } else if (vCode == VerifyNonExistenceError.IsLeftNeighbor) {
            return VerifyChainedNonMembershipError.IsLeftNeighbor;
        }
        revert("BptreeIcs23: non exhaustive pattern matching");
    }

    enum ApplyLeafOpError {
        None,
        KeyLength,
        ValueLength
    }

    function applyLeafOp(
        bytes calldata prefix,
        bytes calldata key,
        bytes calldata value
    ) private pure returns (bytes32, ApplyLeafOpError) {
        if (key.length == 0) return ("", ApplyLeafOpError.KeyLength);
        if (value.length == 0) return ("", ApplyLeafOpError.ValueLength);

        bytes memory encodedKey = new bytes(_sz_varint(key.length));
        _encode_varint(key.length, encodedKey);

        bytes32 hashedValue = sha256(value);
        bytes memory encodedValue = new bytes(_sz_varint(32));
        _encode_varint(32, encodedValue);

        // prefix passed through as-is: checkAgainstSpec only requires it
        // start with 0, real proofs can have longer prefixes.
        bytes32 data = sha256(
            abi.encodePacked(prefix, encodedKey, key, encodedValue, hashedValue)
        );
        return (data, ApplyLeafOpError.None);
    }

    function applyOp(
        UnionIcs23.InnerOp calldata innerOp,
        bytes32 child
    ) private pure returns (bytes32) {
        return sha256(abi.encodePacked(innerOp.prefix, child, innerOp.suffix));
    }

    enum CalculateRootError {
        None,
        LeafNil,
        LeafOp,
        EmptyProof
    }

    function calculateRoot(
        UnionIcs23.ExistenceProof calldata proof
    ) private pure returns (bytes32, CalculateRootError) {
        if (proof.leafPrefix.length == 0) {
            return ("", CalculateRootError.LeafNil);
        }
        (bytes32 root, ApplyLeafOpError lCode) =
            applyLeafOp(proof.leafPrefix, proof.key, proof.value);
        if (lCode != ApplyLeafOpError.None) {
            return ("", CalculateRootError.LeafOp);
        }
        uint256 proofPathLength = proof.path.length;
        for (uint256 i; i < proofPathLength; i++) {
            root = applyOp(proof.path[i], root);
        }
        return (root, CalculateRootError.None);
    }

    enum VerifyExistenceError {
        None,
        KeyNotMatching,
        ValueNotMatching,
        CheckSpec,
        CalculateRoot,
        RootNotMatching
    }

    function verifyNoRootCheck(
        UnionIcs23.ExistenceProof calldata proof,
        bool checkDepth,
        bytes memory key,
        bytes memory value
    ) private pure returns (VerifyExistenceError) {
        if (keccak256(proof.key) != keccak256(key)) {
            return VerifyExistenceError.KeyNotMatching;
        }
        if (keccak256(proof.value) != keccak256(value)) {
            return VerifyExistenceError.ValueNotMatching;
        }
        if (!checkAgainstSpec(proof, checkDepth)) {
            return VerifyExistenceError.CheckSpec;
        }
        return VerifyExistenceError.None;
    }

    function verify(
        UnionIcs23.ExistenceProof calldata proof,
        bool checkDepth,
        bytes32 commitmentRoot,
        bytes memory key,
        bytes memory value
    ) private pure returns (VerifyExistenceError) {
        VerifyExistenceError vCode =
            verifyNoRootCheck(proof, checkDepth, key, value);
        if (vCode != VerifyExistenceError.None) {
            return vCode;
        }
        (bytes32 root, CalculateRootError rCode) = calculateRoot(proof);
        if (rCode != CalculateRootError.None) {
            return VerifyExistenceError.CalculateRoot;
        }
        if (root != commitmentRoot) {
            return VerifyExistenceError.RootNotMatching;
        }
        return VerifyExistenceError.None;
    }

    function checkAgainstSpec(
        UnionIcs23.ExistenceProof calldata proof,
        bool checkDepth
    ) private pure returns (bool) {
        if (proof.leafPrefix.length == 0 || proof.leafPrefix[0] != 0) {
            return false;
        }

        uint256 proofPathLength = proof.path.length;
        if (
            checkDepth
                && (proofPathLength < MIN_DEPTH || proofPathLength > MAX_DEPTH)
        ) {
            return false;
        }

        uint256 max = MAX_PREFIX_LENGTH + CHILD_SIZE;
        for (uint256 i; i < proofPathLength; i++) {
            UnionIcs23.InnerOp calldata innerOp = proof.path[i];
            if (
                innerOp.prefix.length < MIN_PREFIX_LENGTH
                    || innerOp.prefix[0] == 0 || innerOp.prefix.length > max
            ) {
                return false;
            }
        }
        return true;
    }

    enum VerifyNonExistenceError {
        None,
        VerifyLeft,
        VerifyRight,
        LeftAndRightKeyEmpty,
        RightKeyRange,
        LeftKeyRange,
        RightProofLeftMost,
        LeftProofRightMost,
        IsLeftNeighbor
    }

    function calculateRootNonExistence(
        UnionIcs23.NonExistenceProof calldata proof
    ) private pure returns (bytes32, CalculateRootError) {
        if (!UnionIcs23.empty(proof.left)) {
            return calculateRoot(proof.left);
        }
        if (!UnionIcs23.empty(proof.right)) {
            return calculateRoot(proof.right);
        }
        return ("", CalculateRootError.EmptyProof);
    }

    function verifyNonExistence(
        UnionIcs23.NonExistenceProof calldata proof,
        bytes32 commitmentRoot,
        bytes memory key
    ) private pure returns (VerifyNonExistenceError) {
        bytes calldata leftKey = proof.left.key;
        bytes calldata rightKey = proof.right.key;

        if (!UnionIcs23.empty(proof.left)) {
            if (
                verify(
                    proof.left, true, commitmentRoot, proof.left.key, proof.left.value
                ) != VerifyExistenceError.None
            ) {
                return VerifyNonExistenceError.VerifyLeft;
            }
        }
        if (!UnionIcs23.empty(proof.right)) {
            if (
                verify(
                    proof.right,
                    true,
                    commitmentRoot,
                    proof.right.key,
                    proof.right.value
                ) != VerifyExistenceError.None
            ) {
                return VerifyNonExistenceError.VerifyRight;
            }
        }

        if (leftKey.length == 0 && rightKey.length == 0) {
            return VerifyNonExistenceError.LeftAndRightKeyEmpty;
        }
        if (rightKey.length > 0 && compare(key, rightKey) >= 0) {
            return VerifyNonExistenceError.RightKeyRange;
        }
        if (leftKey.length > 0 && compare(key, leftKey) <= 0) {
            return VerifyNonExistenceError.LeftKeyRange;
        }

        if (leftKey.length == 0) {
            if (!isLeftMost(proof.right.path, proof.right.path.length)) {
                return VerifyNonExistenceError.RightProofLeftMost;
            }
        } else if (rightKey.length == 0) {
            if (!isRightMost(proof.left.path, proof.left.path.length)) {
                return VerifyNonExistenceError.LeftProofRightMost;
            }
        } else if (!isLeftNeighbor(proof.left.path, proof.right.path)) {
            return VerifyNonExistenceError.IsLeftNeighbor;
        }

        return VerifyNonExistenceError.None;
    }

    function compare(
        bytes memory a,
        bytes calldata b
    ) private pure returns (int256) {
        uint256 minLen = a.length < b.length ? a.length : b.length;
        for (uint256 i; i < minLen; i++) {
            bytes1 ai = a[i];
            bytes1 bi = b[i];
            if (ai < bi) return -1;
            if (ai > bi) return 1;
        }
        if (a.length > minLen) return 1;
        if (b.length > minLen) return -1;
        return 0;
    }

    function isLeftMost(
        UnionIcs23.InnerOp[] calldata path,
        uint256 length
    ) private pure returns (bool) {
        for (uint256 i; i < length; i++) {
            if (!hasPadding(path[i], 0) && !leftBranchesAreEmpty(path[i])) {
                return false;
            }
        }
        return true;
    }

    function isRightMost(
        UnionIcs23.InnerOp[] calldata path,
        uint256 length
    ) private pure returns (bool) {
        for (uint256 i; i < length; i++) {
            if (!hasPadding(path[i], 1) && !rightBranchesAreEmpty(path[i])) {
                return false;
            }
        }
        return true;
    }

    function isLeftStep(
        UnionIcs23.InnerOp calldata left,
        UnionIcs23.InnerOp calldata right
    ) private pure returns (bool) {
        (uint256 leftIdx, bool leftOk) = orderFromPadding(left);
        if (!leftOk) return false;
        (uint256 rightIdx, bool rightOk) = orderFromPadding(right);
        if (!rightOk) return false;
        return rightIdx == leftIdx + 1;
    }

    function isLeftNeighbor(
        UnionIcs23.InnerOp[] calldata left,
        UnionIcs23.InnerOp[] calldata right
    ) private pure returns (bool) {
        if (left.length == 0 || right.length == 0) {
            return false;
        }
        uint256 leftIdx = left.length - 1;
        uint256 rightIdx = right.length - 1;
        while (
            keccak256(left[leftIdx].prefix) == keccak256(right[rightIdx].prefix)
                && keccak256(left[leftIdx].suffix)
                    == keccak256(right[rightIdx].suffix)
        ) {
            // one path is a strict prefix of the other — not a valid
            // neighbor pair.
            if (leftIdx == 0 || rightIdx == 0) {
                return false;
            }
            leftIdx -= 1;
            rightIdx -= 1;
        }
        if (!isLeftStep(left[leftIdx], right[rightIdx])) {
            return false;
        }
        if (!isRightMost(left, leftIdx)) {
            return false;
        }
        if (!isLeftMost(right, rightIdx)) {
            return false;
        }
        return true;
    }

    function getPadding(
        uint256 branch
    )
        private
        pure
        returns (uint256 minPrefix, uint256 maxPrefix, uint256 suffix)
    {
        uint256 prefix = branch * CHILD_SIZE;
        minPrefix = prefix + MIN_PREFIX_LENGTH;
        maxPrefix = prefix + MAX_PREFIX_LENGTH;
        suffix = (1 - branch) * CHILD_SIZE;
    }

    function hasPadding(
        UnionIcs23.InnerOp calldata op,
        uint256 branch
    ) private pure returns (bool) {
        (uint256 minPrefix, uint256 maxPrefix, uint256 suffix) =
            getPadding(branch);
        if (op.prefix.length < minPrefix || op.prefix.length > maxPrefix) {
            return false;
        }
        return op.suffix.length == suffix;
    }

    function orderFromPadding(
        UnionIcs23.InnerOp calldata op
    ) private pure returns (uint256, bool) {
        for (uint256 branch; branch < 2; branch++) {
            if (hasPadding(op, branch)) {
                return (branch, true);
            }
        }
        return (0, false);
    }

    function rightBranchesAreEmpty(
        UnionIcs23.InnerOp calldata op
    ) private pure returns (bool) {
        (uint256 idx, bool ok) = orderFromPadding(op);
        if (!ok || idx == 1) return false;
        if (op.suffix.length != CHILD_SIZE) return false;
        return bytes32(op.suffix) == EMPTY_CHILD;
    }

    function leftBranchesAreEmpty(
        UnionIcs23.InnerOp calldata op
    ) private pure returns (bool) {
        (uint256 idx, bool ok) = orderFromPadding(op);
        if (!ok || idx == 0) return false;
        if (op.prefix.length < CHILD_SIZE) return false;
        uint256 markerLength = op.prefix.length - CHILD_SIZE;
        return bytes32(op.prefix[markerLength:markerLength + CHILD_SIZE])
            == EMPTY_CHILD;
    }

    function _sz_varint(
        uint256 x
    ) private pure returns (uint256 sz) {
        sz = 1;
        while (x >= 0x80) {
            x >>= 7;
            sz++;
        }
    }

    function _encode_varint(uint256 x, bytes memory dst) private pure {
        uint256 dstPtr;
        assembly {
            dstPtr := add(dst, 0x20)
        }
        while (x >= 0x80) {
            assembly {
                mstore8(dstPtr, or(and(x, 0x7f), 0x80))
                dstPtr := add(dstPtr, 1)
                x := shr(7, x)
            }
        }
        assembly {
            mstore8(dstPtr, x)
        }
    }
}
