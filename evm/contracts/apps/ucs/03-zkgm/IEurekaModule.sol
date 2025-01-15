pragma solidity ^0.8.27;

interface IEurekaModule {
    function onZkgm(
        uint32 channelId,
        bytes calldata sender,
        bytes calldata message
    ) external;
}
