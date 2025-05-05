pragma solidity ^0.8.27;

import "./Types.sol";
import "./IZkgmERC20.sol";

interface IZkgmStore {
    function tokenOrigin(
        address token
    ) external view returns (uint256);
}

interface IZkgm is IZkgmStore {
    function send(
        uint32 channelId,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes32 salt,
        Instruction calldata instruction
    ) external payable;

    function predictWrappedToken(
        uint256 path,
        uint32 channel,
        bytes calldata token
    ) external view returns (address, bytes32);

    function registerGovernanceToken(
        uint32 channelId,
        bytes calldata unwrappedGovernanceToken
    ) external;

    function stake(
        uint32 channelId,
        address beneficiary,
        uint256 amount,
        uint64 timeout
    ) external;

    function unstake(uint256 tokenId, uint64 timeout, bytes32 salt) external;

    function withdrawStake(
        uint256 tokenId,
        address beneficiary,
        uint64 timeout,
        bytes32 salt
    ) external;
}
