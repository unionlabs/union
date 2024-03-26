pragma solidity ^0.8.23;

import {BytesLib} from "solidity-bytes-utils/BytesLib.sol";
import {SafeCast} from "@openzeppelin/utils/math/SafeCast.sol";
import {ProtoBufRuntime} from "../proto/ProtoBufRuntime.sol";
import {Math} from "@openzeppelin/utils/math/Math.sol";
import "../proto/ibc/core/commitment/v1/commitment.sol";
import "../proto/cosmos/ics23/v1/proofs.sol";

library Ics23 {
    function getIavlProofSpec()
        internal
        pure
        returns (CosmosIcs23V1ProofSpec.Data memory iavlProofSpec)
    {
        int32[] memory childOrder = new int32[](2);
        childOrder[0] = 0;
        childOrder[1] = 1;
        iavlProofSpec = CosmosIcs23V1ProofSpec.Data({
            leaf_spec: CosmosIcs23V1LeafOp.Data({
                prefix: hex"00",
                prehash_key: CosmosIcs23V1GlobalEnums.HashOp.NO_HASH,
                hash: CosmosIcs23V1GlobalEnums.HashOp.SHA256,
                prehash_value: CosmosIcs23V1GlobalEnums.HashOp.SHA256,
                length: CosmosIcs23V1GlobalEnums.LengthOp.VAR_PROTO
            }),
            inner_spec: CosmosIcs23V1InnerSpec.Data({
                child_order: childOrder,
                child_size: 33,
                min_prefix_length: 4,
                max_prefix_length: 12,
                empty_child: abi.encodePacked(),
                hash: CosmosIcs23V1GlobalEnums.HashOp.SHA256
            }),
            min_depth: 0,
            max_depth: 0
        });
    }

    function getTendermintProofSpec()
        internal
        pure
        returns (CosmosIcs23V1ProofSpec.Data memory tendermintProofSpec)
    {
        int32[] memory childOrder = new int32[](2);
        childOrder[0] = 0;
        childOrder[1] = 1;
        tendermintProofSpec = CosmosIcs23V1ProofSpec.Data({
            leaf_spec: CosmosIcs23V1LeafOp.Data({
                prefix: hex"00",
                prehash_key: CosmosIcs23V1GlobalEnums.HashOp.NO_HASH,
                hash: CosmosIcs23V1GlobalEnums.HashOp.SHA256,
                prehash_value: CosmosIcs23V1GlobalEnums.HashOp.SHA256,
                length: CosmosIcs23V1GlobalEnums.LengthOp.VAR_PROTO
            }),
            inner_spec: CosmosIcs23V1InnerSpec.Data({
                child_order: childOrder,
                child_size: 32,
                min_prefix_length: 1,
                max_prefix_length: 1,
                empty_child: abi.encodePacked(),
                hash: CosmosIcs23V1GlobalEnums.HashOp.SHA256
            }),
            min_depth: 0,
            max_depth: 0
        });
    }

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
        CosmosIcs23V1NonExistenceProof.Data memory nonExistProof,
        CosmosIcs23V1ExistenceProof.Data memory existProof,
        bytes memory root,
        bytes[] memory path
    ) internal pure returns (VerifyChainedNonMembershipError) {
        CosmosIcs23V1ProofSpec.Data memory iavlSpec = getIavlProofSpec();

        (bytes memory subroot, Proof.CalculateRootError rCode) =
            Proof.calculateRoot(nonExistProof);
        if (rCode != Proof.CalculateRootError.None) {
            return VerifyChainedNonMembershipError.InvalidProofRoot;
        }

        bytes memory key = path[1];
        Proof.VerifyNonExistenceError vCode =
            Proof.verify(nonExistProof, iavlSpec, subroot, key);

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

        Proof.VerifyExistenceError mCode = Proof.verify(
            existProof,
            getTendermintProofSpec(),
            subroot2,
            path[0],
            subroot
        );

        if (mCode != Proof.VerifyExistenceError.None) {
            if (mCode == Proof.VerifyExistenceError.KeyNotMatching) {
                return VerifyChainedNonMembershipError.KeyMismatch;
            } else if (mCode == Proof.VerifyExistenceError.ValueNotMatching)
            {
                return VerifyChainedNonMembershipError.ValueMismatch;
            } else if (mCode == Proof.VerifyExistenceError.CheckSpec) {
                return VerifyChainedNonMembershipError.InvalidSpec;
            } else if (mCode == Proof.VerifyExistenceError.CalculateRoot) {
                return VerifyChainedNonMembershipError
                    .InvalidIntermediateProofRoot;
            } else if (mCode == Proof.VerifyExistenceError.RootNotMatching)
            {
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
        CosmosIcs23V1ExistenceProof.Data[2] memory proofs,
        bytes memory root,
        bytes[] memory path,
        bytes memory value
    ) internal pure returns (VerifyChainedMembershipError) {
        (bytes memory subroot, Proof.CalculateRootError rCode) = Proof.calculateRoot(proofs[0]);
        if (rCode != Proof.CalculateRootError.None) {
            return VerifyChainedMembershipError.InvalidProofRoot;
        }

        Proof.VerifyExistenceError vCode = Proof.verify(
            proofs[0],
            getIavlProofSpec(),
            subroot,
            path[1],
            value
        );
        if (vCode != Proof.VerifyExistenceError.None) {
            return convertExistenceError(vCode);
        }

        bytes memory subroot2;
        (subroot2, rCode) = Proof.calculateRoot(proofs[1]);
        if (rCode != Proof.CalculateRootError.None) {
            return VerifyChainedMembershipError.InvalidProofRoot;
        }

        vCode = Proof.verify(
            proofs[1],
            getTendermintProofSpec(),
            subroot2,
            path[0],
            subroot
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

    function convertExistenceError(
        Proof.VerifyExistenceError vCode
    ) internal pure returns (VerifyChainedMembershipError) {
        if (vCode == Proof.VerifyExistenceError.KeyNotMatching) {
            return VerifyChainedMembershipError.KeyMismatch;
        } else if (vCode == Proof.VerifyExistenceError.ValueNotMatching)
        {
            return VerifyChainedMembershipError.ValueMismatch;
        } else if (vCode == Proof.VerifyExistenceError.CheckSpec) {
            return VerifyChainedMembershipError.InvalidSpec;
        } else if (vCode == Proof.VerifyExistenceError.CalculateRoot) {
            return VerifyChainedMembershipError
                .InvalidIntermediateProofRoot;
        } else if (vCode == Proof.VerifyExistenceError.RootNotMatching)
        {
            return
                VerifyChainedMembershipError.IntermateProofRootMismatch;
        }

        revert(
            "verifyChainedMembership: non exhaustive pattern matching on VerifyExistenceError"
        );
    }

    enum VerifyMembershipError {
        None,
        ExistenceProofIsNil,
        ProofVerify
    }

    // verifyMembership, throws an exception in case anything goes wrong
    // NOTE: We are expecting `proof` to be `ExistentProof` only to avoid handling batch proofs
    // and doing decompressing.
    function verifyMembership(
        CosmosIcs23V1ProofSpec.Data memory spec,
        bytes memory commitmentRoot,
        CosmosIcs23V1CommitmentProof.Data memory proof,
        bytes memory key,
        bytes memory value
    ) internal pure returns (VerifyMembershipError) {
        CosmosIcs23V1ExistenceProof.Data memory exiProof = proof.exist;
        //require(CosmosIcs23V1ExistenceProof.isNil(exiProof) == false); // dev: getExistProofForKey not available
        if (CosmosIcs23V1ExistenceProof.isNil(exiProof)) {
            return VerifyMembershipError.ExistenceProofIsNil;
        }
        Proof.VerifyExistenceError vCode =
            Proof.verify(exiProof, spec, commitmentRoot, key, value);
        if (vCode != Proof.VerifyExistenceError.None) {
            return VerifyMembershipError.ProofVerify;
        }

        return VerifyMembershipError.None;
    }

    /* enum VerifyNonMembershipError { */
    enum VerifyNonMembershipError {
        None,
        NonExistenceProofIsNil,
        ProofVerify
    }

    // NOTE: We are expecting `proof` to be `NonExistentProof` to avoid handling batch proofs and decompressing
    function verifyNonMembership(
        CosmosIcs23V1ProofSpec.Data memory spec,
        bytes memory commitmentRoot,
        CosmosIcs23V1CommitmentProof.Data memory proof,
        bytes memory key
    ) internal pure returns (VerifyNonMembershipError) {
        CosmosIcs23V1NonExistenceProof.Data memory nonProof = proof.nonexist;
        //require(CosmosIcs23V1ExistenceProof.isNil(nonProof) == false); // dev: getNonExistProofForKey not available
        if (CosmosIcs23V1NonExistenceProof.isNil(nonProof)) {
            return VerifyNonMembershipError.NonExistenceProofIsNil;
        }
        Proof.VerifyNonExistenceError vCode =
            Proof.verify(nonProof, spec, commitmentRoot, key);
        if (vCode != Proof.VerifyNonExistenceError.None) {
            return VerifyNonMembershipError.ProofVerify;
        }

        return VerifyNonMembershipError.None;
    }

    function isLeft(
        CosmosIcs23V1ExistenceProof.Data memory left,
        bytes memory key
    ) private pure returns (bool) {
        // CosmosIcs23V1ExistenceProof.isNil does not work
        return CosmosIcs23V1ExistenceProof._empty(left)
            || Ops.compare(left.key, key) < 0;
    }

    function isRight(
        CosmosIcs23V1ExistenceProof.Data memory right,
        bytes memory key
    ) private pure returns (bool) {
        // CosmosIcs23V1ExistenceProof.isNil does not work
        return CosmosIcs23V1ExistenceProof._empty(right)
            || Ops.compare(right.key, key) > 0;
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
    function applyOp(
        CosmosIcs23V1LeafOp.Data memory leafOp,
        bytes memory key,
        bytes memory value
    ) internal pure returns (bytes memory, ApplyLeafOpError) {
        //require(key.length > 0); // dev: Leaf op needs key
        if (key.length == 0) return (empty, ApplyLeafOpError.KeyLength);
        //require(value.length > 0); // dev: Leaf op needs value
        if (value.length == 0) return (empty, ApplyLeafOpError.ValueLength);

        // tm/iavl specs set hashOp for prehash_key to NOOP and lengthOp to VAR_PROTO
        bytes memory encodedKey = new bytes(ProtoBufRuntime._sz_varint(key.length));
        ProtoBufRuntime._encode_varint(key.length, 32, encodedKey);
        bytes memory pKey = abi.encodePacked(encodedKey, key);

        // tm/iavl specs set hashOp for prehash_value to SHA256 and lengthOp to VAR_PROTO
        bytes memory hashedValue = abi.encodePacked(sha256(value));
        bytes memory encodedValue = new bytes(ProtoBufRuntime._sz_varint(hashedValue.length));
        ProtoBufRuntime._encode_varint(hashedValue.length, 32, encodedValue);
        bytes memory pValue = abi.encodePacked(encodedValue, hashedValue);

        bytes memory data = abi.encodePacked(leafOp.prefix, pKey, pValue);
        bytes memory hashed = abi.encodePacked(sha256(data));
        return (hashed, ApplyLeafOpError.None);
    }

    enum CheckAgainstSpecError {
        None,
        MinPrefixLength,
        HasPrefix,
        MaxPrefixLength
    }

    function checkAgainstSpec(
        CosmosIcs23V1LeafOp.Data memory leafOp,
        CosmosIcs23V1ProofSpec.Data memory spec
    ) internal pure returns (CheckAgainstSpecError) {
        // We don't check whether spec is compatible with the proof since only allow SHA256 and VAR_PROTO
        bool hasprefix = hasPrefix(leafOp.prefix, spec.leaf_spec.prefix);
        //require(hasprefix); // dev: checkAgainstSpec for LeafOp - Leaf Prefix doesn't start with
        if (hasprefix == false) return CheckAgainstSpecError.HasPrefix;

        return CheckAgainstSpecError.None;
    }

    enum ApplyInnerOpError {
        None,
        ChildLength,
        DoHash
    }

    // InnerOp operations
    function applyOp(
        CosmosIcs23V1InnerOp.Data memory innerOp,
        bytes memory child
    ) internal pure returns (bytes memory, ApplyInnerOpError) {
        //require(child.length > 0); // dev: Inner op needs child value
        if (child.length == 0) return (empty, ApplyInnerOpError.ChildLength);
        bytes memory preImage =
            abi.encodePacked(innerOp.prefix, child, innerOp.suffix);

        // inner_spec.hash is always SHA256 in the tm/iavl specs
        bytes memory hashed = abi.encodePacked(sha256(preImage));

        return (hashed, ApplyInnerOpError.None);
    }

    function checkAgainstSpec(
        CosmosIcs23V1InnerOp.Data memory innerOp,
        CosmosIcs23V1ProofSpec.Data memory spec
    ) internal pure returns (CheckAgainstSpecError) {
        // we don't check whether `hash` matches since we use `SHA256` anyways
        //require(innerOp.hash == spec.inner_spec.hash); // dev: checkAgainstSpec for InnerOp - Unexpected HashOp
        uint256 minPrefixLength =
            SafeCast.toUint256(spec.inner_spec.min_prefix_length);
        //require(innerOp.prefix.length >= minPrefixLength); // dev: InnerOp prefix too short;
        if (innerOp.prefix.length < minPrefixLength) {
            return CheckAgainstSpecError.MinPrefixLength;
        }
        bytes memory leafPrefix = spec.leaf_spec.prefix;
        bool hasprefix = hasPrefix(innerOp.prefix, leafPrefix);
        //require(hasprefix == false); // dev: Inner Prefix starts with wrong value
        if (hasprefix) return CheckAgainstSpecError.HasPrefix;
        uint256 childSize = SafeCast.toUint256(spec.inner_spec.child_size);
        uint256 maxLeftChildBytes =
            (spec.inner_spec.child_order.length - 1) * childSize;
        uint256 maxPrefixLength =
            SafeCast.toUint256(spec.inner_spec.max_prefix_length);
        //require(innerOp.prefix.length <= maxPrefixLength + maxLeftChildBytes); // dev: InnerOp prefix too long
        if (innerOp.prefix.length > maxPrefixLength + maxLeftChildBytes) {
            return CheckAgainstSpecError.MaxPrefixLength;
        }

        return CheckAgainstSpecError.None;
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

    // private
    enum DoLengthOpError {
        None,
        Require32DataLength,
        Require64DataLength,
        Unsupported
    }

    function hasPrefix(
        bytes memory element,
        bytes memory prefix
    ) internal pure returns (bool) {
        if (prefix.length == 0) {
            return true;
        }
        if (prefix.length > element.length) {
            return false;
        }
        bytes memory slice = BytesLib.slice(element, 0, prefix.length);
        return BytesLib.equal(prefix, slice);
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

    // ExistenceProof
    function verify(
        CosmosIcs23V1ExistenceProof.Data memory proof,
        CosmosIcs23V1ProofSpec.Data memory spec,
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
        bool rootMatch = BytesLib.equal(root, commitmentRoot);
        if (rootMatch == false) return VerifyExistenceError.RootNotMatching;

        return VerifyExistenceError.None;
    }

    enum CalculateRootError {
        None,
        LeafNil,
        LeafOp,
        PathOp,
        EmptyProof
    }

    function calculateRoot(CosmosIcs23V1ExistenceProof.Data memory proof)
        internal
        pure
        returns (bytes memory, CalculateRootError)
    {
        //require(LeafOp.isNil(proof.leaf) == false); // dev: Existence Proof needs defined LeafOp
        if (CosmosIcs23V1LeafOp.isNil(proof.leaf)) {
            return (empty, CalculateRootError.LeafNil);
        }
        (bytes memory root, Ops.ApplyLeafOpError lCode) =
            Ops.applyOp(proof.leaf, proof.key, proof.value);
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
        CosmosIcs23V1ExistenceProof.Data memory proof,
        CosmosIcs23V1ProofSpec.Data memory spec
    ) internal pure returns (CheckAgainstSpecError) {
        // LeafOp.isNil does not work
        //require(LeafOp._empty(proof.leaf) == false); // dev: Existence Proof needs defined LeafOp
        if (CosmosIcs23V1LeafOp._empty(proof.leaf)) {
            return CheckAgainstSpecError.EmptyLeaf;
        }
        // LeafOp's checkAgainstSpec is inlined here since we only need to check the prefix here
        //require(hasprefix); // dev: checkAgainstSpec for LeafOp - Leaf Prefix doesn't start with
        if (Ops.hasPrefix(proof.leaf.prefix, spec.leaf_spec.prefix) == false) {
            return CheckAgainstSpecError.OpsCheckAgainstSpec;
        }
        if (spec.min_depth > 0) {
            bool innerOpsDepthTooShort =
                proof.path.length < SafeCast.toUint256(int256(spec.min_depth));
            //require(innerOpsDepthTooShort == false); // dev: InnerOps depth too short
            if (innerOpsDepthTooShort) {
                return CheckAgainstSpecError.InnerOpsDepthTooShort;
            }
        }
        if (spec.max_depth > 0) {
            bool innerOpsDepthTooLong =
                proof.path.length > SafeCast.toUint256(int256(spec.max_depth));
            //require(innerOpsDepthTooLong == false); // dev: InnerOps depth too long
            if (innerOpsDepthTooLong) {
                return CheckAgainstSpecError.InnerOpsDepthTooLong;
            }
        }

        Ops.CheckAgainstSpecError cCode = Ops.CheckAgainstSpecError.None;
        for (uint256 i = 0; i < proof.path.length; i++) {
            cCode = Ops.checkAgainstSpec(proof.path[i], spec);
            if (cCode != Ops.CheckAgainstSpecError.None) {
                return CheckAgainstSpecError.OpsCheckAgainstSpec;
            }
        }
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
        CosmosIcs23V1NonExistenceProof.Data memory proof,
        CosmosIcs23V1ProofSpec.Data memory spec,
        bytes memory commitmentRoot,
        bytes memory key
    ) internal pure returns (VerifyNonExistenceError) {
        bytes memory leftKey;
        bytes memory rightKey;
        // CosmosIcs23V1ExistenceProof.isNil does not work
        if (CosmosIcs23V1ExistenceProof._empty(proof.left) == false) {
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
        if (CosmosIcs23V1ExistenceProof._empty(proof.right) == false) {
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
            //require(isLeftMost(spec.inner_spec, proof.right.path)); // dev: left proof missing, right proof must be left-most
            if (isLeftMost(spec.inner_spec, proof.right.path) == false) {
                return VerifyNonExistenceError.RightProofLeftMost;
            }
        } else if (rightKey.length == 0) {
            //require(isRightMost(spec.inner_spec, proof.left.path)); // dev: isRightMost: right proof missing, left proof must be right-most
            if (isRightMost(spec.inner_spec, proof.left.path) == false) {
                return VerifyNonExistenceError.LeftProofRightMost;
            }
        } else {
            //require(isLeftNeighbor(spec.inner_spec, proof.left.path, proof.right.path)); // dev: isLeftNeighbor: right proof missing, left proof must be right-most
            bool isLeftNeigh = isLeftNeighbor(
                spec.inner_spec, proof.left.path, proof.right.path
            );
            if (isLeftNeigh == false) {
                return VerifyNonExistenceError.IsLeftNeighbor;
            }
        }

        return VerifyNonExistenceError.None;
    }

    function calculateRoot(CosmosIcs23V1NonExistenceProof.Data memory proof)
        internal
        pure
        returns (bytes memory, CalculateRootError)
    {
        if (CosmosIcs23V1ExistenceProof._empty(proof.left) == false) {
            return calculateRoot(proof.left);
        }
        if (CosmosIcs23V1ExistenceProof._empty(proof.right) == false) {
            return calculateRoot(proof.right);
        }
        //revert(); // dev: Nonexistence proof has empty Left and Right proof
        return (empty, CalculateRootError.EmptyProof);
    }

    // commitment proof
    function calculateRoot(CosmosIcs23V1CommitmentProof.Data memory proof)
        internal
        pure
        returns (bytes memory, CalculateRootError)
    {
        // We ignore batch and compressed batch as an optimization
        if (CosmosIcs23V1ExistenceProof._empty(proof.exist) == false) {
            return calculateRoot(proof.exist);
        }
        if (CosmosIcs23V1NonExistenceProof._empty(proof.nonexist) == false) {
            return calculateRoot(proof.nonexist);
        }
        //revert(); // dev: calculateRoot(CommitmentProof) empty proof
        return (empty, CalculateRootError.EmptyProof);
    }

    // private
    function isLeftMost(
        CosmosIcs23V1InnerSpec.Data memory spec,
        CosmosIcs23V1InnerOp.Data[] memory path
    ) private pure returns (bool) {
        (
            uint256 minPrefix,
            uint256 maxPrefix,
            uint256 suffix,
            GetPaddingError gCode
        ) = getPadding(spec, 0);
        if (gCode != GetPaddingError.None) return false;
        for (uint256 i = 0; i < path.length; i++) {
            if (hasPadding(path[i], minPrefix, maxPrefix, suffix) == false) {
                return false;
            }
        }
        return true;
    }

    function isRightMost(
        CosmosIcs23V1InnerSpec.Data memory spec,
        CosmosIcs23V1InnerOp.Data[] memory path
    ) private pure returns (bool) {
        uint256 last = spec.child_order.length - 1;
        (
            uint256 minPrefix,
            uint256 maxPrefix,
            uint256 suffix,
            GetPaddingError gCode
        ) = getPadding(spec, last);
        if (gCode != GetPaddingError.None) return false;
        for (uint256 i = 0; i < path.length; i++) {
            if (hasPadding(path[i], minPrefix, maxPrefix, suffix) == false) {
                return false;
            }
        }

        return true;
    }

    function isLeftStep(
        CosmosIcs23V1InnerSpec.Data memory spec,
        CosmosIcs23V1InnerOp.Data memory left,
        CosmosIcs23V1InnerOp.Data memory right
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
        CosmosIcs23V1InnerSpec.Data memory spec,
        CosmosIcs23V1InnerOp.Data[] memory left,
        CosmosIcs23V1InnerOp.Data[] memory right
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
        NotFound,
        GetPadding
    }

    function orderFromPadding(
        CosmosIcs23V1InnerSpec.Data memory spec,
        CosmosIcs23V1InnerOp.Data memory op
    ) private pure returns (uint256, OrderFromPaddingError) {
        uint256 maxBranch = spec.child_order.length;
        for (uint256 branch = 0; branch < maxBranch; branch++) {
            (uint256 minp, uint256 maxp, uint256 suffix, GetPaddingError gCode)
            = getPadding(spec, branch);
            if (gCode != GetPaddingError.None) {
                return (0, OrderFromPaddingError.GetPadding);
            }
            if (hasPadding(op, minp, maxp, suffix) == true) {
                return (branch, OrderFromPaddingError.None);
            }
        }
        //revert(); // dev: Cannot find any valid spacing for this node
        return (0, OrderFromPaddingError.NotFound);
    }

    enum GetPaddingError {
        None,
        GetPosition
    }

    function getPadding(
        CosmosIcs23V1InnerSpec.Data memory spec,
        uint256 branch
    )
        private
        pure
        returns (
            uint256 minPrefix,
            uint256 maxPrefix,
            uint256 suffix,
            GetPaddingError
        )
    {
        uint256 uChildSize = SafeCast.toUint256(spec.child_size);
        (uint256 idx, GetPositionError gCode) =
            getPosition(spec.child_order, branch);
        if (gCode != GetPositionError.None) {
            return (0, 0, 0, GetPaddingError.GetPosition);
        }
        uint256 prefix = idx * uChildSize;
        minPrefix = prefix + SafeCast.toUint256(spec.min_prefix_length);
        maxPrefix = prefix + SafeCast.toUint256(spec.max_prefix_length);
        suffix = (spec.child_order.length - 1 - idx) * uChildSize;

        return (minPrefix, maxPrefix, suffix, GetPaddingError.None);
    }

    enum GetPositionError {
        None,
        BranchLength,
        NoFound
    }

    function getPosition(
        int32[] memory order,
        uint256 branch
    ) private pure returns (uint256, GetPositionError) {
        //require(branch < order.length); // dev: invalid branch
        if (branch >= order.length) return (0, GetPositionError.BranchLength);
        for (uint256 i = 0; i < order.length; i++) {
            if (SafeCast.toUint256(order[i]) == branch) {
                return (i, GetPositionError.None);
            }
        }
        //revert(); // dev: branch not found in order
        return (0, GetPositionError.NoFound);
    }

    function hasPadding(
        CosmosIcs23V1InnerOp.Data memory op,
        uint256 minPrefix,
        uint256 maxPrefix,
        uint256 suffix
    ) private pure returns (bool) {
        if (op.prefix.length < minPrefix) return false;
        if (op.prefix.length > maxPrefix) return false;
        return op.suffix.length == suffix;
    }

    function sliceInnerOps(
        CosmosIcs23V1InnerOp.Data[] memory array,
        uint256 start,
        uint256 end
    ) private pure returns (CosmosIcs23V1InnerOp.Data[] memory) {
        CosmosIcs23V1InnerOp.Data[] memory slice =
            new CosmosIcs23V1InnerOp.Data[](end - start);
        for (uint256 i = start; i < end; i++) {
            slice[i] = array[i];
        }
        return slice;
    }
}
