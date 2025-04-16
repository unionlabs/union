pragma solidity ^0.8.27;

import "./Types.sol";

interface IZkgm {
    function send(
        uint32 channelId,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes32 salt,
        Instruction calldata instruction
    ) external;

    function predictWrappedToken(
        uint256 path,
        uint32 channel,
        bytes calldata token
    ) external view returns (address, bytes32);

    function tokenOrigin(
        address token
    ) external view returns (uint256);
}
