pragma solidity ^0.8.18;

import "../../../contracts/core/IMembershipVerifier.sol";

contract MembershipVerifier_Testable is IMembershipVerifier {
    function verifyMembership(
        bytes memory root,
        bytes calldata proof,
        bytes memory prefix,
        bytes calldata path,
        bytes calldata value
    ) external view override returns (bool) {
        return true;
    }

    function verifyNonMembership(
        bytes memory root,
        bytes calldata proof,
        bytes calldata prefix,
        bytes calldata path
    ) external view override returns (bool) {
        revert("not implemented yet");
    }
}
