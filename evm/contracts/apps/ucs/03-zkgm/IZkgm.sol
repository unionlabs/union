pragma solidity ^0.8.27;

import "./Types.sol";
import "./IZkgmERC20.sol";
import "./ZkgmERC721.sol";

interface IZkgmStore {
    function tokenOrigin(
        address token
    ) external view returns (uint256);

    function predictStakeManagerAddress() external view returns (ZkgmERC721);
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
    ) external returns (address, bytes32);

    function predictWrappedTokenV2(
        uint256 path,
        uint32 channel,
        bytes calldata token,
        TokenMetadata calldata metadata
    ) external returns (address, bytes32);

    function predictWrappedTokenFromMetadataImageV2(
        uint256 path,
        uint32 channel,
        bytes calldata token,
        bytes32 metadataHash
    ) external returns (address, bytes32);

    function registerGovernanceToken(
        uint32 channelId,
        address governanceToken
    ) external;

    function overwriteGovernanceToken(
        uint32 channelId,
        address governanceToken
    ) external;
}
