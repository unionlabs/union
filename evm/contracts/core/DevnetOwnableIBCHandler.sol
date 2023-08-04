pragma solidity ^0.8.18;

// import "@openzeppelin/contracts/access/Ownable.sol";
import "./OwnableIBCHandler.sol";

/**
 * @dev DevnetOwnableIBCHandler is a contract that implements [ICS-25](https://github.com/cosmos/ibc/tree/main/spec/core/ics-025-handler-interface).
 */
contract DevnetOwnableIBCHandler is OwnableIBCHandler {
    /**
     * @dev The arguments of constructor must satisfy the followings:
     * @param ibcClient is the address of a contract that implements `IIBCClient`.
     * @param ibcConnection is the address of a contract that implements `IIBCConnectionHandshake`.
     * @param ibcChannel is the address of a contract that implements `IIBCChannelHandshake`.
     * @param ibcPacket is the address of a contract that implements `IIBCPacket`.
     */
    constructor(
        address ibcClient,
        address ibcConnection,
        address ibcChannel,
        address ibcPacket
    ) OwnableIBCHandler(ibcClient, ibcConnection, ibcChannel, ibcPacket) {}

    function setupInitialChannel(
        string calldata connectionId,
        IbcCoreConnectionV1ConnectionEnd.Data calldata connection,
        string calldata portId,
        string calldata channelId,
        IbcCoreChannelV1Channel.Data calldata channel,
        address moduleAddress
    ) public onlyOwner {
        nextSequenceSends[portId][channelId] = 1;
        nextSequenceRecvs[portId][channelId] = 1;
        nextSequenceAcks[portId][channelId] = 1;

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
