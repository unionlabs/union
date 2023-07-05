pragma solidity ^0.8.18;

import "../proto/ibc/core/client/v1/client.sol";

interface IMembershipVerifier {
    function verifyMembership(
        bytes memory root,
        bytes calldata proof,
        bytes memory prefix,
        bytes calldata path,
        bytes calldata value
    ) external view returns (bool);

    function verifyNonMembership(
        bytes memory root,
        bytes calldata proof,
        bytes calldata prefix,
        bytes calldata path
    ) external view returns (bool);
}
