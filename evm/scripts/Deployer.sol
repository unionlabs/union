pragma solidity ^0.8.27;

import "solady/utils/CREATE3.sol";
import "solady/utils/LibString.sol";

contract Deployer {
    using LibString for *;

    function deploy(
        string memory salt,
        bytes calldata creationCode,
        uint256 value
    ) public returns (address) {
        return CREATE3.deployDeterministic(
            value,
            creationCode,
            keccak256(abi.encodePacked(msg.sender.toHexString(), "/", salt))
        );
    }
}
