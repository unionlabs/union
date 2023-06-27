pragma solidity ^0.8.18;

import "../core/IMembershipVerifier.sol";
import "../lib/ICS23.sol";
import "../proto/cosmos/ics23/v1/proofs.sol";
import "../proto/ibc/core/commitment/v1/commitment.sol";
import "solady/utils/LibString.sol";

contract ICS23MembershipVerifier is IMembershipVerifier {
    using LibString for string;

    function verifyMembership(
        bytes memory root,
        bytes calldata proof,
        bytes memory prefix,
        bytes[] calldata path,
        bytes calldata value
    ) external view override returns (bool) {
        bytes[] memory fullPath = new bytes[](1 + path.length);
        fullPath[0] = prefix;
        for (uint256 i = 0; i < path.length; i++) {
            fullPath[i + 1] = path[i];
        }
        // This call reverts if any verification issue happen
        Ics23.verifyChainedMembership(
            IbcCoreCommitmentV1MerkleProof.decode(proof),
            root,
            fullPath,
            value
        );
        return true;
    }

    function verifyNonMembership(
        bytes memory root,
        bytes calldata proof,
        bytes calldata prefix,
        bytes[] calldata path
    ) external view override returns (bool) {
        revert("not implemented yet");
    }
}
