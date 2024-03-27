pragma solidity ^0.8.23;

import {BytesLib} from "solidity-bytes-utils/BytesLib.sol";
import {SafeCast} from "@openzeppelin/utils/math/SafeCast.sol";
import {ProtoBufRuntime} from "../proto/ProtoBufRuntime.sol";
import {Math} from "@openzeppelin/utils/math/Math.sol";
import "../proto/ibc/core/commitment/v1/commitment.sol";
import "../proto/cosmos/ics23/v1/proofs.sol";
import "./UnionICS23.sol";

library Ics23 {
    enum VerifyChainedNonMembershipError {
        None,
        NonExistenceProofIsNil,
        ExistenceProofIsNil,
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

    function verifyChainedNonMembership(
        UnionIcs23.NonExistenceProof memory nonExistProof,
        UnionIcs23.ExistenceProof memory existProof,
        bytes memory root,
        bytes[] memory path
    ) internal pure returns (VerifyChainedNonMembershipError) {
        (bytes memory subroot, Proof.CalculateRootError rCode) =
            Proof.calculateRoot(nonExistProof);
        if (rCode != Proof.CalculateRootError.None) {
            return VerifyChainedNonMembershipError.InvalidProofRoot;
        }

        bytes memory key = path[1];
        Proof.VerifyNonExistenceError vCode = Proof.verify(
            nonExistProof, UnionIcs23.getIavlProofSpec(), subroot, key
        );

        // Map non existence error to non membership error
        if (vCode != Proof.VerifyNonExistenceError.None) {
            if (vCode == Proof.VerifyNonExistenceError.VerifyLeft) {
                return VerifyChainedNonMembershipError.VerifyLeft;
            } else if (
                vCode == Proof.VerifyNonExistenceError.LeftAndRightKeyEmpty
            ) {
                return VerifyChainedNonMembershipError.LeftAndRightKeyEmpty;
            } else if (vCode == Proof.VerifyNonExistenceError.RightKeyRange) {
                return VerifyChainedNonMembershipError.RightKeyRange;
            } else if (vCode == Proof.VerifyNonExistenceError.LeftKeyRange) {
                return VerifyChainedNonMembershipError.LeftKeyRange;
            } else if (
                vCode == Proof.VerifyNonExistenceError.RightProofLeftMost
            ) {
                return VerifyChainedNonMembershipError.RightProofLeftMost;
            } else if (
                vCode == Proof.VerifyNonExistenceError.LeftProofRightMost
            ) {
                return VerifyChainedNonMembershipError.LeftProofRightMost;
            } else if (vCode == Proof.VerifyNonExistenceError.IsLeftNeighbor) {
                return VerifyChainedNonMembershipError.IsLeftNeighbor;
            }

            revert(
                "verifyChainedNonMembership: non exhaustive pattern matching on VerifyNonExistenceError"
            );
        }

        bytes memory subroot2;
        (subroot2, rCode) = Proof.calculateRoot(existProof);
        if (rCode != Proof.CalculateRootError.None) {
            return VerifyChainedNonMembershipError.InvalidProofRoot;
        }

        // We don't want the above root calculation to be done again. Since we calculated it, we also don't
        // need to check it against anything.
        Proof.VerifyExistenceError mCode = Proof.verifyNoRootCheck(
            existProof, UnionIcs23.getTendermintProofSpec(), path[0], subroot
        );

        if (mCode != Proof.VerifyExistenceError.None) {
            if (mCode == Proof.VerifyExistenceError.KeyNotMatching) {
                return VerifyChainedNonMembershipError.KeyMismatch;
            } else if (mCode == Proof.VerifyExistenceError.ValueNotMatching) {
                return VerifyChainedNonMembershipError.ValueMismatch;
            } else if (mCode == Proof.VerifyExistenceError.CheckSpec) {
                return VerifyChainedNonMembershipError.InvalidSpec;
            } else if (mCode == Proof.VerifyExistenceError.CalculateRoot) {
                return
                    VerifyChainedNonMembershipError.InvalidIntermediateProofRoot;
            } else if (mCode == Proof.VerifyExistenceError.RootNotMatching) {
                return
                    VerifyChainedNonMembershipError.IntermateProofRootMismatch;
            }
            revert(
                "verifyChainedNonMembership: non exhaustive pattern matching on VerifyNonExistenceError"
            );
        }

        if (keccak256(root) != keccak256(subroot2)) {
            return VerifyChainedNonMembershipError.RootMismatch;
        } else {
            return VerifyChainedNonMembershipError.None;
        }

        return VerifyChainedNonMembershipError.None;
    }

    enum VerifyChainedMembershipError {
        None,
        ExistenceProofIsNil,
        InvalidProofRoot,
        KeyMismatch,
        ValueMismatch,
        InvalidSpec,
        InvalidIntermediateProofRoot,
        IntermateProofRootMismatch,
        RootMismatch
    }

    function verifyChainedMembership(
        UnionIcs23.ExistenceProof[2] memory proofs,
        bytes memory root,
        bytes[] memory path,
        bytes memory value
    ) internal pure returns (VerifyChainedMembershipError) {
        (bytes memory subroot, Proof.CalculateRootError rCode) =
            Proof.calculateRoot(proofs[0]);
        if (rCode != Proof.CalculateRootError.None) {
            return VerifyChainedMembershipError.InvalidProofRoot;
        }

        // We don't want the above root calculation to be done again. Since we calculated it, we also don't
        // need to check it against anything.
        Proof.VerifyExistenceError vCode = Proof.verifyNoRootCheck(
            proofs[0], UnionIcs23.getIavlProofSpec(), path[1], value
        );
        if (vCode != Proof.VerifyExistenceError.None) {
            return convertExistenceError(vCode);
        }

        bytes memory subroot2;
        (subroot2, rCode) = Proof.calculateRoot(proofs[1]);
        if (rCode != Proof.CalculateRootError.None) {
            return VerifyChainedMembershipError.InvalidProofRoot;
        }

        vCode = Proof.verifyNoRootCheck(
            proofs[1], UnionIcs23.getTendermintProofSpec(), path[0], subroot
        );

        if (vCode != Proof.VerifyExistenceError.None) {
            return convertExistenceError(vCode);
        }

        if (keccak256(root) != keccak256(subroot2)) {
            return VerifyChainedMembershipError.RootMismatch;
        } else {
            return VerifyChainedMembershipError.None;
        }
    }

    function convertExistenceError(Proof.VerifyExistenceError vCode)
        internal
        pure
        returns (VerifyChainedMembershipError)
    {
        if (vCode == Proof.VerifyExistenceError.KeyNotMatching) {
            return VerifyChainedMembershipError.KeyMismatch;
        } else if (vCode == Proof.VerifyExistenceError.ValueNotMatching) {
            return VerifyChainedMembershipError.ValueMismatch;
        } else if (vCode == Proof.VerifyExistenceError.CheckSpec) {
            return VerifyChainedMembershipError.InvalidSpec;
        } else if (vCode == Proof.VerifyExistenceError.CalculateRoot) {
            return VerifyChainedMembershipError.InvalidIntermediateProofRoot;
        } else if (vCode == Proof.VerifyExistenceError.RootNotMatching) {
            return VerifyChainedMembershipError.IntermateProofRootMismatch;
        }

        revert(
            "verifyChainedMembership: non exhaustive pattern matching on VerifyExistenceError"
        );
    }

    function isLeft(
        UnionIcs23.ExistenceProof memory left,
        bytes memory key
    ) private pure returns (bool) {
        // CosmosIcs23V1ExistenceProof.isNil does not work
        return UnionIcs23.empty(left) || Ops.compare(left.key, key) < 0;
    }

    function isRight(
        UnionIcs23.ExistenceProof memory right,
        bytes memory key
    ) private pure returns (bool) {
        // CosmosIcs23V1ExistenceProof.isNil does not work
        return UnionIcs23.empty(right) || Ops.compare(right.key, key) > 0;
    }
}

library Ops {
    bytes constant empty = new bytes(0);

    enum ApplyLeafOpError {
        None,
        KeyLength,
        ValueLength
    }

    // LeafOp operations
    function applyLeafOp(
        bytes memory prefix,
        bytes memory key,
        bytes memory value
    ) internal pure returns (bytes memory, ApplyLeafOpError) {
        //require(key.length > 0); // dev: Leaf op needs key
        if (key.length == 0) return (empty, ApplyLeafOpError.KeyLength);
        //require(value.length > 0); // dev: Leaf op needs value
        if (value.length == 0) return (empty, ApplyLeafOpError.ValueLength);

        // tm/iavl specs set hashOp for prehash_key to NOOP and lengthOp to VAR_PROTO
        bytes memory encodedKey =
            new bytes(ProtoBufRuntime._sz_varint(key.length));
        ProtoBufRuntime._encode_varint(key.length, 32, encodedKey);
        bytes memory pKey = abi.encodePacked(encodedKey, key);

        // tm/iavl specs set hashOp for prehash_value to SHA256 and lengthOp to VAR_PROTO
        bytes memory hashedValue = abi.encodePacked(sha256(value));
        bytes memory encodedValue =
            new bytes(ProtoBufRuntime._sz_varint(hashedValue.length));
        ProtoBufRuntime._encode_varint(hashedValue.length, 32, encodedValue);
        bytes memory pValue = abi.encodePacked(encodedValue, hashedValue);

        bytes memory data = abi.encodePacked(prefix, pKey, pValue);
        bytes memory hashed = abi.encodePacked(sha256(data));
        return (hashed, ApplyLeafOpError.None);
    }

    enum CheckAgainstSpecError {
        None,
        MinPrefixLength,
        HasPrefix,
        MaxPrefixLength
    }

    enum ApplyInnerOpError {
        None,
        ChildLength,
        DoHash
    }

    // InnerOp operations
    function applyOp(
        UnionIcs23.InnerOp memory innerOp,
        bytes memory child
    ) internal pure returns (bytes memory, ApplyInnerOpError) {
        //require(child.length > 0); // dev: Inner op needs child value
        if (child.length == 0) return (empty, ApplyInnerOpError.ChildLength);
        bytes memory preImage =
            abi.encodePacked(innerOp.prefix, child, innerOp.suffix);

        // inner_spec.hash is always SHA256 in the tm/iavl specs
        return (abi.encodePacked(sha256(preImage)), ApplyInnerOpError.None);
    }
    
    function compare(
        bytes memory a,
        bytes memory b
    ) internal pure returns (int256) {
        uint256 minLen = Math.min(a.length, b.length);
        for (uint256 i = 0; i < minLen; i++) {
            if (uint8(a[i]) < uint8(b[i])) {
                return -1;
            } else if (uint8(a[i]) > uint8(b[i])) {
                return 1;
            }
        }
        if (a.length > minLen) {
            return 1;
        }
        if (b.length > minLen) {
            return -1;
        }
        return 0;
    }
}

library Proof {
    bytes constant empty = new bytes(0);

    enum VerifyExistenceError {
        None,
        KeyNotMatching,
        ValueNotMatching,
        CheckSpec,
        CalculateRoot,
        RootNotMatching
    }

    function verifyNoRootCheck(
        UnionIcs23.ExistenceProof memory proof,
        UnionIcs23.ProofSpec memory spec,
        bytes memory key,
        bytes memory value
    ) internal pure returns (VerifyExistenceError) {
        //require(BytesLib.equal(proof.key, key)); // dev: Provided key doesn't match proof
        bool keyMatch = BytesLib.equal(proof.key, key);
        if (keyMatch == false) return VerifyExistenceError.KeyNotMatching;
        //require(BytesLib.equal(proof.value, value)); // dev: Provided value doesn't match proof
        bool valueMatch = BytesLib.equal(proof.value, value);
        if (valueMatch == false) return VerifyExistenceError.ValueNotMatching;
        CheckAgainstSpecError cCode = checkAgainstSpec(proof, spec);
        if (cCode != CheckAgainstSpecError.None) {
            return VerifyExistenceError.CheckSpec;
        }

        return VerifyExistenceError.None;
    }

    // ExistenceProof
    function verify(
        UnionIcs23.ExistenceProof memory proof,
        UnionIcs23.ProofSpec memory spec,
        bytes memory commitmentRoot,
        bytes memory key,
        bytes memory value
    ) internal pure returns (VerifyExistenceError) {
        //require(BytesLib.equal(proof.key, key)); // dev: Provided key doesn't match proof
        bool keyMatch = BytesLib.equal(proof.key, key);
        if (keyMatch == false) return VerifyExistenceError.KeyNotMatching;
        //require(BytesLib.equal(proof.value, value)); // dev: Provided value doesn't match proof
        bool valueMatch = BytesLib.equal(proof.value, value);
        if (valueMatch == false) return VerifyExistenceError.ValueNotMatching;
        CheckAgainstSpecError cCode = checkAgainstSpec(proof, spec);
        if (cCode != CheckAgainstSpecError.None) {
            return VerifyExistenceError.CheckSpec;
        }
        (bytes memory root, CalculateRootError rCode) = calculateRoot(proof);
        if (rCode != CalculateRootError.None) {
            return VerifyExistenceError.CalculateRoot;
        }
        //require(BytesLib.equal(root, commitmentRoot)); // dev: Calculcated root doesn't match provided root
        if (BytesLib.equal(root, commitmentRoot) == false) {
            return VerifyExistenceError.RootNotMatching;
        }

        return VerifyExistenceError.None;
    }

    enum CalculateRootError {
        None,
        LeafNil,
        LeafOp,
        PathOp,
        EmptyProof
    }

    function calculateRoot(UnionIcs23.ExistenceProof memory proof)
        internal
        pure
        returns (bytes memory, CalculateRootError)
    {
        //require(LeafOp.isNil(proof.leaf) == false); // dev: Existence Proof needs defined LeafOp
        if (proof.leafPrefix.length == 0) {
            return (empty, CalculateRootError.LeafNil);
        }
        (bytes memory root, Ops.ApplyLeafOpError lCode) =
            Ops.applyLeafOp(proof.leafPrefix, proof.key, proof.value);
        if (lCode != Ops.ApplyLeafOpError.None) {
            return (empty, CalculateRootError.LeafOp);
        }
        for (uint256 i = 0; i < proof.path.length; i++) {
            Ops.ApplyInnerOpError iCode;
            (root, iCode) = Ops.applyOp(proof.path[i], root);
            if (iCode != Ops.ApplyInnerOpError.None) {
                return (empty, CalculateRootError.PathOp);
            }
        }

        return (root, CalculateRootError.None);
    }

    enum CheckAgainstSpecError {
        None,
        EmptyLeaf,
        OpsCheckAgainstSpec,
        InnerOpsDepthTooShort,
        InnerOpsDepthTooLong
    }

    function checkAgainstSpec(
        UnionIcs23.ExistenceProof memory proof,
        UnionIcs23.ProofSpec memory spec
    ) internal pure returns (CheckAgainstSpecError) {
        // LeafOp.isNil does not work
        //require(LeafOp._empty(proof.leaf) == false); // dev: Existence Proof needs defined LeafOp
        // TODO(aeryz): check if there is isempty function in solidity
        if (proof.leafPrefix.length == 0) {
            return CheckAgainstSpecError.EmptyLeaf;
        }
        // LeafOp's checkAgainstSpec is inlined here since we only need to check the prefix here
        //require(hasprefix); // dev: checkAgainstSpec for LeafOp - Leaf Prefix doesn't start with
        // Both specs have the prefix 0x00
        if (proof.leafPrefix[0] != 0) {
            return CheckAgainstSpecError.OpsCheckAgainstSpec;
        }
        // we don't do any checks regarding min_depth, max_depth since they both are 0 in both specs

        for (uint256 i = 0; i < proof.path.length; i++) {
            UnionIcs23.InnerOp memory innerOp = proof.path[i];

            if (
                innerOp.prefix.length < spec.minPrefixLength
                    || innerOp.prefix[0] == 0
                    || innerOp.prefix.length > spec.maxPrefixLength + spec.childSize
            ) {
                return CheckAgainstSpecError.OpsCheckAgainstSpec;
            }
        }
        return CheckAgainstSpecError.None;
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

    // CosmosIcs23V1NonExistenceProof
    function verify(
        UnionIcs23.NonExistenceProof memory proof,
        UnionIcs23.ProofSpec memory spec,
        bytes memory commitmentRoot,
        bytes memory key
    ) internal pure returns (VerifyNonExistenceError) {
        bytes memory leftKey;
        bytes memory rightKey;
        // CosmosIcs23V1ExistenceProof.isNil does not work
        if (UnionIcs23.empty(proof.left) == false) {
            VerifyExistenceError eCode = verify(
                proof.left,
                spec,
                commitmentRoot,
                proof.left.key,
                proof.left.value
            );
            if (eCode != VerifyExistenceError.None) {
                return VerifyNonExistenceError.VerifyLeft;
            }

            leftKey = proof.left.key;
        }
        if (UnionIcs23.empty(proof.right) == false) {
            VerifyExistenceError eCode = verify(
                proof.right,
                spec,
                commitmentRoot,
                proof.right.key,
                proof.right.value
            );
            if (eCode != VerifyExistenceError.None) {
                return VerifyNonExistenceError.VerifyRight;
            }

            rightKey = proof.right.key;
        }
        // If both proofs are missing, this is not a valid proof
        //require(leftKey.length > 0 || rightKey.length > 0); // dev: both left and right proofs missing
        if (leftKey.length == 0 && rightKey.length == 0) {
            return VerifyNonExistenceError.LeftAndRightKeyEmpty;
        }
        // Ensure in valid range
        if (rightKey.length > 0 && Ops.compare(key, rightKey) >= 0) {
            //require(Ops.compare(key, rightKey) < 0); // dev: key is not left of right proof
            return VerifyNonExistenceError.RightKeyRange;
        }
        if (leftKey.length > 0 && Ops.compare(key, leftKey) <= 0) {
            //require(Ops.compare(key, leftKey) > 0); // dev: key is not right of left proof
            return VerifyNonExistenceError.LeftKeyRange;
        }
        if (leftKey.length == 0) {
            //require(isLeftMost(spec, proof.right.path)); // dev: left proof missing, right proof must be left-most
            if (isLeftMost(spec, proof.right.path) == false) {
                return VerifyNonExistenceError.RightProofLeftMost;
            }
        } else if (rightKey.length == 0) {
            //require(isRightMost(spec, proof.left.path)); // dev: isRightMost: right proof missing, left proof must be right-most
            if (isRightMost(spec, proof.left.path) == false) {
                return VerifyNonExistenceError.LeftProofRightMost;
            }
        } else {
            //require(isLeftNeighbor(spec, proof.left.path, proof.right.path)); // dev: isLeftNeighbor: right proof missing, left proof must be right-most
            bool isLeftNeigh =
                isLeftNeighbor(spec, proof.left.path, proof.right.path);
            if (isLeftNeigh == false) {
                return VerifyNonExistenceError.IsLeftNeighbor;
            }
        }

        return VerifyNonExistenceError.None;
    }

    function calculateRoot(UnionIcs23.NonExistenceProof memory proof)
        internal
        pure
        returns (bytes memory, CalculateRootError)
    {
        if (!UnionIcs23.empty(proof.left)) {
            return calculateRoot(proof.left);
        }
        if (!UnionIcs23.empty(proof.right)) {
            return calculateRoot(proof.right);
        }
        //revert(); // dev: Nonexistence proof has empty Left and Right proof
        return (empty, CalculateRootError.EmptyProof);
    }

    // private
    function isLeftMost(
        UnionIcs23.ProofSpec memory spec,
        UnionIcs23.InnerOp[] memory path
    ) private pure returns (bool) {
        (uint256 minPrefix, uint256 maxPrefix, uint256 suffix) =
            getPadding(spec, 0);
        for (uint256 i = 0; i < path.length; i++) {
            if (hasPadding(path[i], minPrefix, maxPrefix, suffix) == false) {
                return false;
            }
        }
        return true;
    }

    function isRightMost(
        UnionIcs23.ProofSpec memory spec,
        UnionIcs23.InnerOp[] memory path
    ) private pure returns (bool) {
        (uint256 minPrefix, uint256 maxPrefix, uint256 suffix) =
            getPadding(spec, 1);
        for (uint256 i = 0; i < path.length; i++) {
            if (hasPadding(path[i], minPrefix, maxPrefix, suffix) == false) {
                return false;
            }
        }

        return true;
    }

    function isLeftStep(
        UnionIcs23.ProofSpec memory spec,
        UnionIcs23.InnerOp memory left,
        UnionIcs23.InnerOp memory right
    ) private pure returns (bool) {
        (uint256 leftIdx, OrderFromPaddingError lCode) =
            orderFromPadding(spec, left);
        if (lCode != OrderFromPaddingError.None) return false;
        (uint256 rightIdx, OrderFromPaddingError rCode) =
            orderFromPadding(spec, right);
        if (lCode != OrderFromPaddingError.None) return false;
        if (rCode != OrderFromPaddingError.None) return false;

        return rightIdx == leftIdx + 1;
    }

    function isLeftNeighbor(
        UnionIcs23.ProofSpec memory spec,
        UnionIcs23.InnerOp[] memory left,
        UnionIcs23.InnerOp[] memory right
    ) private pure returns (bool) {
        uint256 leftIdx = left.length - 1;
        uint256 rightIdx = right.length - 1;
        while (leftIdx >= 0 && rightIdx >= 0) {
            if (
                BytesLib.equal(left[leftIdx].prefix, right[rightIdx].prefix)
                    && BytesLib.equal(left[leftIdx].suffix, right[rightIdx].suffix)
            ) {
                leftIdx -= 1;
                rightIdx -= 1;
                continue;
            }
            break;
        }
        if (isLeftStep(spec, left[leftIdx], right[rightIdx]) == false) {
            return false;
        }
        // slicing does not work for ``memory`` types
        if (isRightMost(spec, sliceInnerOps(left, 0, leftIdx)) == false) {
            return false;
        }
        if (isLeftMost(spec, sliceInnerOps(right, 0, rightIdx)) == false) {
            return false;
        }
        return true;
    }

    enum OrderFromPaddingError {
        None,
        NotFound
    }

    function orderFromPadding(
        UnionIcs23.ProofSpec memory spec,
        UnionIcs23.InnerOp memory op
    ) private pure returns (uint256, OrderFromPaddingError) {
        for (uint256 branch = 0; branch < 2; branch++) {
            (uint256 minp, uint256 maxp, uint256 suffix) =
                getPadding(spec, branch);
            if (hasPadding(op, minp, maxp, suffix) == true) {
                return (branch, OrderFromPaddingError.None);
            }
        }
        //revert(); // dev: Cannot find any valid spacing for this node
        return (0, OrderFromPaddingError.NotFound);
    }

    function getPadding(
        UnionIcs23.ProofSpec memory spec,
        uint256 branch
    )
        private
        pure
        returns (uint256 minPrefix, uint256 maxPrefix, uint256 suffix)
    {
        uint256 prefix = branch * spec.childSize;
        minPrefix = prefix + spec.minPrefixLength;
        maxPrefix = prefix + spec.maxPrefixLength;
        suffix = (1 - branch) * spec.childSize;

        return (minPrefix, maxPrefix, suffix);
    }

    function hasPadding(
        UnionIcs23.InnerOp memory op,
        uint256 minPrefix,
        uint256 maxPrefix,
        uint256 suffix
    ) private pure returns (bool) {
        if (op.prefix.length < minPrefix) return false;
        if (op.prefix.length > maxPrefix) return false;
        return op.suffix.length == suffix;
    }

    function sliceInnerOps(
        UnionIcs23.InnerOp[] memory array,
        uint256 start,
        uint256 end
    ) private pure returns (UnionIcs23.InnerOp[] memory) {
        UnionIcs23.InnerOp[] memory slice =
            new UnionIcs23.InnerOp[](end - start);
        for (uint256 i = start; i < end; i++) {
            slice[i] = array[i];
        }
        return slice;
    }
}
