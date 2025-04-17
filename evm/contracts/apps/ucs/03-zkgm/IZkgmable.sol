pragma solidity ^0.8.27;

interface IZkgmable {
    function onZkgm(
        address caller,
        uint256 path,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        bytes calldata sender,
        bytes calldata message,
        address relayer,
        bytes calldata relayerMsg
    ) external;

    function onIntentZkgm(
        address caller,
        uint256 path,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        bytes calldata sender,
        bytes calldata message,
        address marketMaker,
        bytes calldata marketMakerMsg
    ) external;
}
