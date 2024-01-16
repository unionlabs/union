pragma solidity ^0.8.23;

import "../proto/ibc/core/client/v1/client.sol";

interface IMembershipVerifier {
    function verifyMembership(
        bytes memory root,
        bytes calldata proof,
        bytes memory prefix,
        bytes calldata path,
        bytes calldata value
    ) external returns (bool);

    function verifyNonMembership(
        bytes memory root,
        bytes calldata proof,
        bytes calldata prefix,
        bytes calldata path
    ) external returns (bool);
}
