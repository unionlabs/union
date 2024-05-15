pragma solidity ^0.8.23;

import "../../proto/ibc/core/client/v1/client.sol";
import "../02-client/ILightClient.sol";
import "../24-host/IBCStore.sol";
import "../05-port/ModuleManager.sol";
import "../24-host/IBCCommitment.sol";

abstract contract IBCQuerier is IBCStore {
    function getConnection(string calldata connectionId)
        external
        view
        returns (IbcCoreConnectionV1ConnectionEnd.Data memory)
    {
        return connections[connectionId];
    }

    function getChannel(
        string calldata portId,
        string calldata channelId
    ) external view returns (IbcCoreChannelV1Channel.Data memory) {
        return channels[portId][channelId];
    }
}
