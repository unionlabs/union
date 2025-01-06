pragma solidity ^0.8.27;

interface IEurekaModule {
    function onZkgm(bytes calldata sender, bytes calldata message) external;
}
