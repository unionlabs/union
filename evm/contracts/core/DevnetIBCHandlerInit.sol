pragma solidity ^0.8.21;

import "./OwnableIBCHandler.sol";
import "./24-host/IBCHost.sol";

contract DevnetIBCHandlerInit is IBCHost {
    function setupInitialChannel(
        string calldata connectionId,
        IbcCoreConnectionV1ConnectionEnd.Data calldata connection,
        string calldata portId,
        string calldata channelId,
        IbcCoreChannelV1Channel.Data calldata channel,
        address moduleAddress
    ) public {
        nextSequenceSends[portId][channelId] = 1;
        nextSequenceRecvs[portId][channelId] = 1;
        nextSequenceAcks[portId][channelId] = 1;
        nextConnectionSequence = 1;
        nextChannelSequence++;

        connections[connectionId].client_id = connection.client_id;
        connections[connectionId].state = connection.state;
        connections[connectionId].delay_period = connection.delay_period;
        delete connections[connectionId].versions;
        for (uint8 i = 0; i < connection.versions.length; i++) {
            connections[connectionId].versions.push(connection.versions[i]);
        }
        connections[connectionId].counterparty = connection.counterparty;
        commitments[
            keccak256(IBCCommitment.connectionPath(connectionId))
        ] = keccak256(IbcCoreConnectionV1ConnectionEnd.encode(connection));

        channels[portId][channelId] = channel;
        commitments[
            keccak256(IBCCommitment.channelPath(portId, channelId))
        ] = keccak256(IbcCoreChannelV1Channel.encode(channel));

        bindPort(portId, moduleAddress);
        IIBCModule module = lookupModuleByPort(portId);
        module.onChanOpenInit(
            channel.ordering,
            channel.connection_hops,
            portId,
            channelId,
            channel.counterparty,
            channel.version
        );
        module.onChanOpenAck(portId, channelId, channel.version);
        claimCapability(
            channelCapabilityPath(portId, channelId),
            address(module)
        );
    }
}
