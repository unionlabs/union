pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "./utils/MsgMocks.sol";
import "./utils/Cometbls.sol";
import "./utils/MockApp.sol";
import "./utils/IBCHandler_Testable.sol";

abstract contract TestPlus is Test {
    function assertStrEq(bytes memory a, string memory b) internal pure {
        require(
            keccak256(abi.encodePacked(a)) == keccak256(abi.encodePacked(b)),
            "strings not equal"
        );
    }

    function assertStrEq(string memory a, string memory b) internal pure {
        require(
            keccak256(abi.encodePacked(a)) == keccak256(abi.encodePacked(b)),
            "strings not equal"
        );
    }

    function assertStrNotEq(string memory a, string memory b) internal pure {
        require(
            keccak256(abi.encodePacked(a)) != keccak256(abi.encodePacked(b)),
            "strings equal"
        );
    }
}
