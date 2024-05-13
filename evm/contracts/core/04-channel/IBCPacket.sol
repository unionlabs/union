pragma solidity ^0.8.23;

import "solady/utils/LibString.sol";
import "@openzeppelin/utils/Strings.sol";

import "../../proto/ibc/core/channel/v1/channel.sol";
import "../25-handler/IBCMsgs.sol";
import "../02-client/IBCHeight.sol";
import "../24-host/IBCStore.sol";
import "../24-host/IBCCommitment.sol";
import "../04-channel/IIBCPacket.sol";
import "../05-port/ModuleManager.sol";
import "../05-port/IIBCModule.sol";

library IBCPacketLib {
    event SendPacket(
        uint64 sequence,
        string sourcePort,
        string sourceChannel,
        IbcCoreClientV1Height.Data timeoutHeight,
        uint64 timeoutTimestamp,
        bytes data
    );
    event RecvPacket(IbcCoreChannelV1Packet.Data packet);
    event WriteAcknowledgement(
        string destinationPort,
        string destinationChannel,
        uint64 sequence,
        bytes acknowledgement
    );
    event AcknowledgePacket(
        IbcCoreChannelV1Packet.Data packet, bytes acknowledgement
    );
    event TimeoutPacket(IbcCoreChannelV1Packet.Data packet);

    error ErrUnauthorized();
    error ErrInvalidChannelState();
    error ErrLatestHeightNotFound();
    error ErrLatestTimestampNotFound();
    error ErrInvalidTimeoutHeight();
    error ErrInvalidTimeoutTimestamp();
    error ErrSourceAndCounterpartyPortMismatch();
    error ErrSourceAndCounterpartyChannelMismatch();
    error ErrDestinationAndCounterpartyPortMismatch();
    error ErrDestinationAndCounterpartyChannelMismatch();
    error ErrInvalidConnectionState();
    error ErrHeightTimeout();
    error ErrTimestampTimeout();
    error ErrInvalidProof();
    error ErrPacketAlreadyReceived();
    error ErrPacketSequenceNextSequenceMismatch();
    error ErrUnknownChannelOrdering();
    error ErrAcknowledgementIsEmpty();
    error ErrAcknowledgementAlreadyExists();
    error ErrPacketCommitmentNotFound();
    error ErrInvalidPacketCommitment();
    error ErrPacketWithoutTimeout();
    error ErrTimeoutHeightNotReached();
    error ErrTimeoutTimestampNotReached();
    error ErrNextSequenceMustBeGreaterThanTimeoutSequence();
}

/**
 * @dev IBCPacket is a contract that implements [ICS-4](https://github.com/cosmos/ibc/tree/main/spec/core/ics-004-channel-and-packet-semantics).
 */
contract IBCPacket is IBCStore, IIBCPacket, ModuleManager {
    using IBCHeight for IbcCoreClientV1Height.Data;
    using LibString for *;

    /**
     * @dev sendPacket is called by a module in order to send an IBC packet on a channel.
     * The packet sequence generated for the packet to be sent is returned. An error
     * is returned if one occurs.
     */
    function sendPacket(
        string calldata sourceChannel,
        IbcCoreClientV1Height.Data calldata timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external override returns (uint64) {
        string memory sourcePort = msg.sender.toHexString();
        if (
            !authenticateCapability(
                channelCapabilityPath(sourcePort, sourceChannel)
            )
        ) {
            revert IBCPacketLib.ErrUnauthorized();
        }

        IbcCoreChannelV1Channel.Data storage channel =
            ensureChannelState(sourcePort, sourceChannel);

        string memory clientId =
            connections[channel.connection_hops[0]].client_id;
        ILightClient client = getClient(clientId);

        IbcCoreClientV1Height.Data memory latestHeight =
            client.getLatestHeight(clientId);
        if (latestHeight.revision_height == 0) {
            revert IBCPacketLib.ErrLatestHeightNotFound();
        }
        if (!timeoutHeight.isZero() && latestHeight.gte(timeoutHeight)) {
            revert IBCPacketLib.ErrInvalidTimeoutHeight();
        }

        uint64 latestTimestamp;
        latestTimestamp = client.getTimestampAtHeight(clientId, latestHeight);
        if (latestTimestamp == 0) {
            revert IBCPacketLib.ErrLatestTimestampNotFound();
        }
        if (timeoutTimestamp != 0 && latestTimestamp >= timeoutTimestamp) {
            revert IBCPacketLib.ErrInvalidTimeoutTimestamp();
        }

        uint64 packetSequence = uint64(
            uint256(
                commitments[IBCCommitment.nextSequenceSendCommitmentKey(
                    sourcePort, sourceChannel
                )]
            )
        );
        commitments[IBCCommitment.nextSequenceSendCommitmentKey(
            sourcePort, sourceChannel
        )] = bytes32(uint256(packetSequence + 1));
        commitments[IBCCommitment.packetCommitmentKey(
            sourcePort, sourceChannel, packetSequence
        )] = keccak256(
            abi.encodePacked(
                sha256(
                    abi.encodePacked(
                        timeoutTimestamp,
                        timeoutHeight.revision_number,
                        timeoutHeight.revision_height,
                        sha256(data)
                    )
                )
            )
        );

        emit IBCPacketLib.SendPacket(
            packetSequence,
            sourcePort,
            sourceChannel,
            timeoutHeight,
            timeoutTimestamp,
            data
        );

        return packetSequence;
    }

    /**
     * @dev recvPacket is called by a module in order to receive & process an IBC packet
     * sent on the corresponding channel end on the counterparty chain.
     */
    function recvPacket(IBCMsgs.MsgPacketRecv calldata msg_)
        external
        override
    {
        IbcCoreChannelV1Channel.Data storage channel = ensureChannelState(
            msg_.packet.destination_port, msg_.packet.destination_channel
        );

        if (
            hashString(msg_.packet.source_port)
                != hashString(channel.counterparty.port_id)
        ) {
            revert IBCPacketLib.ErrSourceAndCounterpartyPortMismatch();
        }
        if (
            hashString(msg_.packet.source_channel)
                != hashString(channel.counterparty.channel_id)
        ) {
            revert IBCPacketLib.ErrSourceAndCounterpartyChannelMismatch();
        }

        IbcCoreConnectionV1ConnectionEnd.Data storage connection =
            connections[channel.connection_hops[0]];
        if (connection.state != IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN)
        {
            revert IBCPacketLib.ErrInvalidConnectionState();
        }

        if (
            msg_.packet.timeout_height.revision_height != 0
                && (block.number >= msg_.packet.timeout_height.revision_height)
        ) {
            revert IBCPacketLib.ErrHeightTimeout();
        }

        // For some reason cosmos is using nanos, we try to follow their convention to avoid friction
        uint64 currentTimestamp = uint64(block.timestamp * 1e9);
        if (
            msg_.packet.timeout_timestamp != 0
                && (currentTimestamp >= msg_.packet.timeout_timestamp)
        ) {
            revert IBCPacketLib.ErrTimestampTimeout();
        }

        if (
            !verifyCommitment(
                connection,
                msg_.proofHeight,
                msg_.proof,
                IBCCommitment.packetCommitmentPath(
                    msg_.packet.source_port,
                    msg_.packet.source_channel,
                    msg_.packet.sequence
                ),
                abi.encodePacked(
                    sha256(
                        abi.encodePacked(
                            msg_.packet.timeout_timestamp,
                            msg_.packet.timeout_height.revision_number,
                            msg_.packet.timeout_height.revision_height,
                            sha256(msg_.packet.data)
                        )
                    )
                )
            )
        ) {
            revert IBCPacketLib.ErrInvalidProof();
        }

        if (
            channel.ordering
                == IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED
        ) {
            bytes32 receiptCommitmentKey = IBCCommitment
                .packetReceiptCommitmentKey(
                msg_.packet.destination_port,
                msg_.packet.destination_channel,
                msg_.packet.sequence
            );
            bytes32 receipt = commitments[receiptCommitmentKey];
            if (receipt != bytes32(0)) {
                revert IBCPacketLib.ErrPacketAlreadyReceived();
            }
            commitments[receiptCommitmentKey] = bytes32(uint256(1));
        } else if (
            channel.ordering == IbcCoreChannelV1GlobalEnums.Order.ORDER_ORDERED
        ) {
            uint64 expectedRecvSequence = uint64(
                uint256(
                    commitments[IBCCommitment.nextSequenceRecvCommitmentKey(
                        msg_.packet.destination_port,
                        msg_.packet.destination_channel
                    )]
                )
            );
            if (expectedRecvSequence != msg_.packet.sequence) {
                revert IBCPacketLib.ErrPacketSequenceNextSequenceMismatch();
            }
            commitments[IBCCommitment.nextSequenceRecvCommitmentKey(
                msg_.packet.destination_port, msg_.packet.destination_channel
            )] = bytes32(uint256(expectedRecvSequence + 1));
        } else {
            revert IBCPacketLib.ErrUnknownChannelOrdering();
        }

        IIBCModule module = lookupModuleByChannel(
            msg_.packet.destination_port, msg_.packet.destination_channel
        );
        bytes memory acknowledgement =
            module.onRecvPacket(msg_.packet, msg.sender);
        if (acknowledgement.length > 0) {
            _writeAcknowledgement(
                msg_.packet.destination_port,
                msg_.packet.destination_channel,
                msg_.packet.sequence,
                acknowledgement
            );
        }
        emit IBCPacketLib.RecvPacket(msg_.packet);
    }

    function _writeAcknowledgement(
        string memory destinationPort,
        string calldata destinationChannel,
        uint64 sequence,
        bytes memory acknowledgement
    ) internal {
        if (acknowledgement.length == 0) {
            revert IBCPacketLib.ErrAcknowledgementIsEmpty();
        }

        IbcCoreChannelV1Channel.Data storage channel =
            ensureChannelState(destinationPort, destinationChannel);

        bytes32 ackCommitmentKey = IBCCommitment
            .packetAcknowledgementCommitmentKey(
            destinationPort, destinationChannel, sequence
        );
        bytes32 ackCommitment = commitments[ackCommitmentKey];
        if (ackCommitment != bytes32(0)) {
            revert IBCPacketLib.ErrAcknowledgementAlreadyExists();
        }
        commitments[ackCommitmentKey] =
            keccak256(abi.encodePacked(sha256(acknowledgement)));

        emit IBCPacketLib.WriteAcknowledgement(
            destinationPort, destinationChannel, sequence, acknowledgement
        );
    }

    /**
     * @dev writeAcknowledgement writes the packet execution acknowledgement to the state,
     * which will be verified by the counterparty chain using AcknowledgePacket.
     */
    function writeAcknowledgement(
        string calldata destinationChannel,
        uint64 sequence,
        bytes calldata acknowledgement
    ) external override {
        string memory destinationPort = msg.sender.toHexString();
        if (
            !authenticateCapability(
                channelCapabilityPath(destinationPort, destinationChannel)
            )
        ) {
            revert IBCPacketLib.ErrUnauthorized();
        }
        _writeAcknowledgement(
            destinationPort, destinationChannel, sequence, acknowledgement
        );
    }

    /**
     * @dev AcknowledgePacket is called by a module to process the acknowledgement of a
     * packet previously sent by the calling module on a channel to a counterparty
     * module on the counterparty chain. Its intended usage is within the ante
     * handler. AcknowledgePacket will clean up the packet commitment,
     * which is no longer necessary since the packet has been received and acted upon.
     * It will also increment NextSequenceAck in case of ORDERED channels.
     */
    function acknowledgePacket(IBCMsgs.MsgPacketAcknowledgement calldata msg_)
        external
        override
    {
        IbcCoreChannelV1Channel.Data storage channel = ensureChannelState(
            msg_.packet.source_port, msg_.packet.source_channel
        );

        if (
            hashString(msg_.packet.destination_port)
                != hashString(channel.counterparty.port_id)
        ) {
            revert IBCPacketLib.ErrDestinationAndCounterpartyPortMismatch();
        }
        if (
            hashString(msg_.packet.destination_channel)
                != hashString(channel.counterparty.channel_id)
        ) {
            revert IBCPacketLib.ErrDestinationAndCounterpartyChannelMismatch();
        }

        IbcCoreConnectionV1ConnectionEnd.Data storage connection =
            connections[channel.connection_hops[0]];
        if (connection.state != IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN)
        {
            revert IBCPacketLib.ErrInvalidConnectionState();
        }

        bytes32 packetCommitmentKey = IBCCommitment.packetCommitmentKey(
            msg_.packet.source_port,
            msg_.packet.source_channel,
            msg_.packet.sequence
        );
        bytes32 expectedPacketCommitment = commitments[packetCommitmentKey];
        if (expectedPacketCommitment == bytes32(0)) {
            revert IBCPacketLib.ErrPacketCommitmentNotFound();
        }
        bytes32 packetCommitment = keccak256(
            abi.encodePacked(
                sha256(
                    abi.encodePacked(
                        msg_.packet.timeout_timestamp,
                        msg_.packet.timeout_height.revision_number,
                        msg_.packet.timeout_height.revision_height,
                        sha256(msg_.packet.data)
                    )
                )
            )
        );
        if (expectedPacketCommitment != packetCommitment) {
            revert IBCPacketLib.ErrInvalidPacketCommitment();
        }

        if (
            !verifyCommitment(
                connection,
                msg_.proofHeight,
                msg_.proof,
                IBCCommitment.packetAcknowledgementCommitmentPath(
                    msg_.packet.destination_port,
                    msg_.packet.destination_channel,
                    msg_.packet.sequence
                ),
                abi.encodePacked(sha256(msg_.acknowledgement))
            )
        ) {
            revert IBCPacketLib.ErrInvalidProof();
        }

        if (channel.ordering == IbcCoreChannelV1GlobalEnums.Order.ORDER_ORDERED)
        {
            uint64 expectedAckSequence = uint64(
                uint256(
                    commitments[IBCCommitment.nextSequenceAckCommitmentKey(
                        msg_.packet.source_port, msg_.packet.source_channel
                    )]
                )
            );
            if (expectedAckSequence != msg_.packet.sequence) {
                revert IBCPacketLib.ErrPacketSequenceNextSequenceMismatch();
            }
            commitments[IBCCommitment.nextSequenceAckCommitmentKey(
                msg_.packet.source_port, msg_.packet.source_channel
            )] = bytes32(uint256(expectedAckSequence + 1));
        }

        delete commitments[packetCommitmentKey];

        IIBCModule module = lookupModuleByChannel(
            msg_.packet.source_port, msg_.packet.source_channel
        );
        module.onAcknowledgementPacket(
            msg_.packet, msg_.acknowledgement, msg.sender
        );

        emit IBCPacketLib.AcknowledgePacket(msg_.packet, msg_.acknowledgement);
    }

    function hashString(string memory s) private pure returns (bytes32) {
        return keccak256(abi.encodePacked(s));
    }

    function timeoutPacket(IBCMsgs.MsgPacketTimeout calldata msg_)
        external
        override
    {
        IbcCoreChannelV1Channel.Data storage channel = ensureChannelState(
            msg_.packet.source_port, msg_.packet.source_channel
        );

        if (
            hashString(msg_.packet.destination_port)
                != hashString(channel.counterparty.port_id)
        ) {
            revert IBCPacketLib.ErrDestinationAndCounterpartyPortMismatch();
        }
        if (
            hashString(msg_.packet.destination_channel)
                != hashString(channel.counterparty.channel_id)
        ) {
            revert IBCPacketLib.ErrDestinationAndCounterpartyChannelMismatch();
        }

        IbcCoreConnectionV1ConnectionEnd.Data storage connection =
            connections[channel.connection_hops[0]];
        if (connection.state != IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN)
        {
            revert IBCPacketLib.ErrInvalidConnectionState();
        }

        bytes32 packetCommitmentKey = IBCCommitment.packetCommitmentKey(
            msg_.packet.source_port,
            msg_.packet.source_channel,
            msg_.packet.sequence
        );
        bytes32 expectedPacketCommitment = commitments[packetCommitmentKey];
        if (expectedPacketCommitment == bytes32(0)) {
            revert IBCPacketLib.ErrPacketCommitmentNotFound();
        }
        bytes32 packetCommitment = keccak256(
            abi.encodePacked(
                sha256(
                    abi.encodePacked(
                        msg_.packet.timeout_timestamp,
                        msg_.packet.timeout_height.revision_number,
                        msg_.packet.timeout_height.revision_height,
                        sha256(msg_.packet.data)
                    )
                )
            )
        );
        if (expectedPacketCommitment != packetCommitment) {
            revert IBCPacketLib.ErrInvalidPacketCommitment();
        }

        ILightClient client = getClient(connection.client_id);
        uint64 proofTimestamp =
            client.getTimestampAtHeight(connection.client_id, msg_.proofHeight);
        if (proofTimestamp == 0) {
            revert IBCPacketLib.ErrLatestTimestampNotFound();
        }

        if (
            msg_.packet.timeout_timestamp == 0
                && msg_.packet.timeout_height.isZero()
        ) {
            revert IBCPacketLib.ErrPacketWithoutTimeout();
        }
        if (
            msg_.packet.timeout_timestamp > 0
                && msg_.packet.timeout_timestamp >= proofTimestamp
        ) {
            revert IBCPacketLib.ErrTimeoutTimestampNotReached();
        }
        if (
            !msg_.packet.timeout_height.isZero()
                && msg_.packet.timeout_height.gte(msg_.proofHeight)
        ) {
            revert IBCPacketLib.ErrTimeoutHeightNotReached();
        }

        bool isOrdered =
            channel.ordering == IbcCoreChannelV1GlobalEnums.Order.ORDER_ORDERED;
        bool isUnordered = channel.ordering
            == IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED;
        if (isOrdered) {
            if (msg_.nextSequenceRecv <= msg_.packet.sequence) {
                revert
                    IBCPacketLib
                    .ErrNextSequenceMustBeGreaterThanTimeoutSequence();
            }
            if (
                !verifyCommitment(
                    connection,
                    msg_.proofHeight,
                    msg_.proof,
                    IBCCommitment.nextSequenceRecvCommitmentPath(
                        msg_.packet.destination_port,
                        msg_.packet.destination_channel
                    ),
                    abi.encodePacked(msg_.nextSequenceRecv)
                )
            ) {
                revert IBCPacketLib.ErrInvalidProof();
            }
            channel.state = IbcCoreChannelV1GlobalEnums.State.STATE_CLOSED;
        } else if (isUnordered) {
            if (
                !verifyAbsentCommitment(
                    connection,
                    msg_.proofHeight,
                    msg_.proof,
                    IBCCommitment.packetReceiptCommitmentPath(
                        msg_.packet.destination_port,
                        msg_.packet.destination_channel,
                        msg_.packet.sequence
                    )
                )
            ) {
                revert IBCPacketLib.ErrInvalidProof();
            }
        } else {
            revert IBCPacketLib.ErrUnknownChannelOrdering();
        }

        delete commitments[packetCommitmentKey];

        IIBCModule module = lookupModuleByChannel(
            msg_.packet.source_port, msg_.packet.source_channel
        );
        module.onTimeoutPacket(msg_.packet, msg.sender);

        emit IBCPacketLib.TimeoutPacket(msg_.packet);
    }

    function verifyCommitment(
        IbcCoreConnectionV1ConnectionEnd.Data storage connection,
        IbcCoreClientV1Height.Data calldata height,
        bytes calldata proof,
        bytes memory path,
        bytes memory commitment
    ) private returns (bool) {
        return getClient(connection.client_id).verifyMembership(
            connection.client_id,
            height,
            connection.delay_period,
            0,
            proof,
            connection.counterparty.prefix.key_prefix,
            path,
            commitment
        );
    }

    function verifyAbsentCommitment(
        IbcCoreConnectionV1ConnectionEnd.Data storage connection,
        IbcCoreClientV1Height.Data calldata height,
        bytes calldata proof,
        bytes memory path
    ) private returns (bool) {
        return getClient(connection.client_id).verifyNonMembership(
            connection.client_id,
            height,
            connection.delay_period,
            0,
            proof,
            connection.counterparty.prefix.key_prefix,
            path
        );
    }

    function ensureChannelState(
        string memory portId,
        string calldata channelId
    ) internal returns (IbcCoreChannelV1Channel.Data storage) {
        IbcCoreChannelV1Channel.Data storage channel =
            channels[portId][channelId];
        if (channel.state != IbcCoreChannelV1GlobalEnums.State.STATE_OPEN) {
            revert IBCPacketLib.ErrInvalidChannelState();
        }
        return channel;
    }
}
