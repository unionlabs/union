pragma solidity ^0.8.27;

interface IEurekaModule {
    function onZkgm(
        uint256 path,
        uint32 sourceChannelId,
        uint32 destinationChannelId,
        bytes calldata sender,
        bytes calldata message
    ) external;
}
