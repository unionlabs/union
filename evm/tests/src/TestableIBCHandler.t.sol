pragma solidity ^0.8.18;

import "../../contracts/core/OwnableIBCHandler.sol";

contract TestableIBCHandler is OwnableIBCHandler {
    constructor(address ibcClient, address ibcConnection, address ibcChannelHandshake, address ibcPacket)
        OwnableIBCHandler(ibcClient, ibcConnection, ibcChannelHandshake, ibcPacket)
    {}

    function setConnection(string memory connectionId, IbcCoreConnectionV1ConnectionEnd.Data memory connection) external {
        connections[connectionId].client_id = connection.client_id;
        connections[connectionId].state = connection.state;
        connections[connectionId].delay_period = connection.delay_period;
        delete connections[connectionId].versions;
        for (uint8 i = 0; i < connection.versions.length; i++) {
            connections[connectionId].versions.push(connection.versions[i]);
        }
        connections[connectionId].counterparty = connection.counterparty;
        commitments[keccak256(IBCCommitment.connectionPath(connectionId))] = keccak256(IbcCoreConnectionV1ConnectionEnd.encode(connection));
    }

    function setChannel(string memory portId, string memory channelId, IbcCoreChannelV1Channel.Data memory channel) external {
        channels[portId][channelId] = channel;
        commitments[keccak256(IBCCommitment.channelPath(portId, channelId))] = keccak256(IbcCoreChannelV1Channel.encode(channel));
    }

    function setNextSequenceSend(string calldata portId, string calldata channelId, uint64 sequence) external {
        nextSequenceSends[portId][channelId] = sequence;
    }

    function setNextSequenceRecv(string calldata portId, string calldata channelId, uint64 sequence) external {
        nextSequenceRecvs[portId][channelId] = sequence;
        commitments[keccak256(IBCCommitment.nextSequenceRecvCommitmentPath(portId, channelId))] =
            keccak256(abi.encodePacked(sequence));
    }

    function setNextSequenceAck(string calldata portId, string calldata channelId, uint64 sequence) external {
        nextSequenceAcks[portId][channelId] = sequence;
    }

    function setNextClientSequence(uint64 sequence) external {
        nextClientSequence = sequence;
    }

    function setNextConnectionSequence(uint64 sequence) external {
        nextConnectionSequence = sequence;
    }

    function setNextChannelSequence(uint64 sequence) external {
        nextChannelSequence = sequence;
    }

    function claimCapabilityDirectly(bytes calldata name, address addr) external {
        for (uint32 i = 0; i < capabilities[name].length; i++) {
            require(capabilities[name][i] != addr);
        }
        capabilities[name].push(addr);
    }
}
