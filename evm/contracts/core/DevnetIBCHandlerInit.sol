pragma solidity ^0.8.23;

import "./OwnableIBCHandler.sol";
import "./24-host/IBCHost.sol";

contract DevnetIBCHandlerInit is IBCHost {
    function setupInitialChannel(
        string calldata connectionId,
        IbcCoreConnectionV1ConnectionEnd.Data calldata connection,
        string calldata portId,
        ChannelId channelId,
        IBCChannelTypes.Channel calldata channel,
        address moduleAddress
    ) public {
        // BROKEN
        // LEAVE ME ALONE
        //
        // nextSequenceSends[portId][channelId] = 1;
        // nextSequenceRecvs[portId][channelId] = 1;
        // nextSequenceAcks[portId][channelId] = 1;
        // nextConnectionSequence = 1;
        // nextChannelSequence++;

        // connections[connectionId].client_id = connection.client_id;
        // connections[connectionId].state = connection.state;
        // connections[connectionId].delay_period = connection.delay_period;
        // delete connections[connectionId].versions;
        // for (uint8 i = 0; i < connection.versions.length; i++) {
        //     connections[connectionId].versions.push(connection.versions[i]);
        // }
        // connections[connectionId].counterparty = connection.counterparty;
        // commitments[keccak256(IBCCommitment.connectionPath(connectionId))] =
        //     keccak256(IbcCoreConnectionV1ConnectionEnd.encode(connection));

        // channels[portId][channelId] = channel;
        // commitments[keccak256(IBCCommitment.channelPath(portId, channelId))] =
        //     keccak256(IbcCoreChannelV1Channel.encode(channel));

        // IIBCModule module = lookupModuleByPort(portId);
        // module.onChanOpenInit(
        //     channel.ordering,
        //     channel.connectionHops,
        //     portId,
        //     channelId,
        //     channel.counterparty,
        //     channel.version
        // );
        // module.onChanOpenAck(
        //     portId, channelId, channel.counterparty.channelId, channel.version
        // );
        // claimCapability(channelId, address(module));
    }
}
