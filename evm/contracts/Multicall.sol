pragma solidity ^0.8.27;

struct Call3 {
    address target;
    bool allowFailure;
    bytes callData;
}

struct Result {
    bool success;
    bytes returnData;
}

event MulticallResult(Result[]);

contract Multicall {
    function multicall(
        Call3[] calldata calls
    ) public payable returns (Result[] memory returnData) {
        uint256 length = calls.length;
        returnData = new Result[](length);
        Call3 calldata calli;
        for (uint256 i = 0; i < length;) {
            Result memory result = returnData[i];
            calli = calls[i];
            (result.success, result.returnData) =
                calli.target.call(calli.callData);
            assembly {
                // Revert if the call fails and failure is not allowed
                // `allowFailure := calldataload(add(calli, 0x20))` and `success := mload(result)`
                if iszero(or(calldataload(add(calli, 0x20)), mload(result))) {
                    // set "Error(string)" signature: bytes32(bytes4(keccak256("Error(string)")))
                    mstore(
                        0x00,
                        0x08c379a000000000000000000000000000000000000000000000000000000000
                    )
                    // set data offset
                    mstore(
                        0x04,
                        0x0000000000000000000000000000000000000000000000000000000000000020
                    )
                    // set length of revert string
                    mstore(
                        0x24,
                        0x0000000000000000000000000000000000000000000000000000000000000017
                    )
                    // set revert string: bytes32(abi.encodePacked("Multicall3: call failed"))
                    mstore(
                        0x44,
                        0x4d756c746963616c6c333a2063616c6c206661696c6564000000000000000000
                    )
                    revert(0x00, 0x64)
                }
            }
            unchecked {
                ++i;
            }
        }
        emit MulticallResult(returnData);
    }
}
