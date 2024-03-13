pragma solidity ^0.8.23;

import "../core/IMembershipVerifier.sol";
import "../lib/ICS23.sol";
import "../proto/cosmos/ics23/v1/proofs.sol";
import "../proto/ibc/core/commitment/v1/commitment.sol";

contract ICS23MembershipVerifier is IMembershipVerifier {
    function verifyMembership(
        bytes memory root,
        bytes calldata proof,
        bytes memory prefix,
        bytes calldata path,
        bytes calldata value
    ) external override returns (bool) {
        bytes[] memory fullPath = new bytes[](2);
        fullPath[0] = prefix;
        fullPath[1] = path;
        return Ics23.verifyChainedMembership(
            abi.decode(proof, (IbcCoreCommitmentV1MerkleProof.Data)),
            root,
            fullPath,
            value
        ) == Ics23.VerifyChainedMembershipError.None;
    }

    function verifyNonMembership(
        bytes memory root,
        bytes calldata proof,
        bytes calldata prefix,
        bytes calldata path
    ) public override returns (bool) {
        bytes[] memory fullPath = new bytes[](2);
        fullPath[0] = prefix;
        fullPath[1] = path;
        return Ics23.verifyChainedNonMembership(
            abi.decode(proof, (IbcCoreCommitmentV1MerkleProof.Data)),
            root,
            fullPath
        ) == Ics23.VerifyChainedNonMembershipError.None;
    }
}
