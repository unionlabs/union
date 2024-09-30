pragma solidity ^0.8.23;

import "../24-host/IBCStore.sol";
import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCStore.sol";
import "../24-host/IBCCommitment.sol";
import "../04-channel/IIBCPacket.sol";
import "../05-port/IIBCModule.sol";
import "../Types.sol";

library IBCPacketLib {
    bytes32 public constant COMMITMENT_MAGIC = keccak256("zkgm");
    bytes32 public constant COMMITMENT_NULL = bytes32(uint256(0));

    event SendPacket(IBCPacket packet);
    event RecvPacket(IBCPacket packets, address relayer, bytes relayerMsg);
    event RecvIntentPacket(
        IBCPacket packet, address marketMaker, bytes marketMakerMsg
    );
    event WriteAcknowledgement(IBCPacket packet, bytes acknowledgement);
    event AcknowledgePacket(
        IBCPacket packet, bytes acknowledgement, address relayer
    );
    event TimeoutPacket(IBCPacket packet, address relayer);

    error ErrUnauthorized();
    error ErrInvalidChannelState();
    error ErrLatestTimestampNotFound();
    error ErrTimeoutMustBeSet();
    error ErrInvalidConnectionState();
    error ErrHeightTimeout();
    error ErrTimestampTimeout();
    error ErrInvalidProof();
    error ErrPacketSequenceNextSequenceMismatch();
    error ErrPacketSequenceAckSequenceMismatch();
    error ErrAcknowledgementIsEmpty();
    error ErrAcknowledgementAlreadyExists();
    error ErrPacketCommitmentNotFound();
    error ErrTimeoutHeightNotReached();
    error ErrTimeoutTimestampNotReached();
    error ErrNextSequenceMustBeGreaterThanTimeoutSequence();
    error ErrNotEnoughPackets();
    error ErrCommittedPacketNotPresent();
    error ErrCommittedAckNotPresent();
    error ErrCannotIntentOrderedPacket();
}

/**
 * @dev IBCPacket is a contract that implements [ICS-4](https://github.com/cosmos/ibc/tree/main/spec/core/ics-004-channel-and-packet-semantics).
 */
abstract contract IBCPacketImpl is IBCStore, IIBCPacket {
    function batchSingleAck(bytes calldata ack)
        internal
        pure
        returns (bytes[] memory)
    {
        bytes[] memory acks = new bytes[](1);
        acks[0] = ack;
        return acks;
    }

    function batchSingleAckMemory(bytes memory ack)
        internal
        pure
        returns (bytes[] memory)
    {
        bytes[] memory acks = new bytes[](1);
        acks[0] = ack;
        return acks;
    }

    function batchSingle(IBCPacket calldata packet)
        internal
        pure
        returns (IBCPacket[] memory)
    {
        IBCPacket[] memory packets = new IBCPacket[](1);
        packets[0] = packet;
        return packets;
    }

    function batchSingleMemory(IBCPacket memory packet)
        internal
        pure
        returns (IBCPacket[] memory)
    {
        IBCPacket[] memory packets = new IBCPacket[](1);
        packets[0] = packet;
        return packets;
    }

    /**
     * @dev batchSend is called by a module in order to commit multiple IBC packets.
     * An error occur if any of the packets wasn't sent.
     * If successful, a new commitment is registered for the batch.
     */
    function batchSend(IBCMsgs.MsgBatchSend calldata msg_) external override {
        uint256 l = msg_.packets.length;
        // No reason to batch less than 2 packets as they are already individually committed.
        if (l < 2) {
            revert IBCPacketLib.ErrNotEnoughPackets();
        }
        for (uint256 i; i < l; ) {
            IBCPacket calldata packet = msg_.packets[i];
            // If the channel mismatch, the commitment will be zero
            bytes32 commitment = commitments[IBCCommitment
                .batchPacketsCommitmentKey(
                msg_.sourceChannel, commitPacketsMemory(batchSingle(packet))
            )];
            // Every packet must have been previously sent to be batched
            if (commitment != IBCPacketLib.COMMITMENT_MAGIC) {
                revert IBCPacketLib.ErrCommittedPacketNotPresent();
            }

            unchecked {
                ++i;
            }
        }
        commitments[IBCCommitment.batchPacketsCommitmentKey(
            msg_.sourceChannel, commitPackets(msg_.packets)
        )] = IBCPacketLib.COMMITMENT_MAGIC;
    }

    /**
     * @dev batchAcks is called by a module in order to commit multiple IBC packets acknowledgements.
     * An error occur if any of the packets wasn't received.
     * If successful, a new commitment is registered for the batch.
     */
    function batchAcks(IBCMsgs.MsgBatchAcks calldata msg_) external override {
        uint256 l = msg_.packets.length;
        // No reason to batch less than 2 packets as they are already individually committed.
        if (l < 2) {
            revert IBCPacketLib.ErrNotEnoughPackets();
        }
        for (uint256 i; i < l;) {
            IBCPacket calldata packet = msg_.packets[i];
            bytes calldata ack = msg_.acks[i];
            // If the channel mismatch, the commitment will be zero.
            bytes32 commitment = commitments[IBCCommitment
                .batchAcksCommitmentKey(
                msg_.sourceChannel, commitPacketsMemory(batchSingle(packet))
            )];
            // Can't batch an empty ack.
            if (commitment == 0) {
                revert IBCPacketLib.ErrAcknowledgementIsEmpty();
            }
            // Every packet must have been received to be batched.
            if (commitment != commitAcksMemory(batchSingleAck(ack))) {
                revert IBCPacketLib.ErrCommittedAckNotPresent();
            }

            unchecked {
                ++i;
            }
        }
        commitments[IBCCommitment.batchAcksCommitmentKey(
            msg_.sourceChannel, commitPackets(msg_.packets)
        )] = commitAcks(msg_.acks);
    }

    /**
     * @dev sendPacket is called by a module in order to send an IBC packet on a channel.
     * The packet sequence generated for the packet to be sent is returned. An error
     * is returned if one occurs.
     */
    function sendPacket(
        uint32 sourceChannel,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external override returns (uint64) {
        if (timeoutTimestamp == 0 && timeoutHeight == 0) {
            revert IBCPacketLib.ErrTimeoutMustBeSet();
        }
        if (!authenticateChannelOwner(sourceChannel)) {
            revert IBCPacketLib.ErrUnauthorized();
        }
        IBCChannel storage channel = ensureChannelState(sourceChannel);
        uint64 sequence = generatePacketSequence(sourceChannel);
        address sourcePort = msg.sender;
        bytes32 normalizedSourcePort = keccak256(abi.encodePacked(sourcePort));
        IBCPacket memory packet = IBCPacket({
            sequence: sequence,
            sourcePort: normalizedSourcePort,
            sourceChannel: sourceChannel,
            destinationPort: channel.counterparty.portId,
            destinationChannel: channel.counterparty.channelId,
            data: data,
            timeoutHeight: timeoutHeight,
            timeoutTimestamp: timeoutTimestamp
        });
        commitments[IBCCommitment.batchPacketsCommitmentKey(
            sourceChannel, commitPacketsMemory(batchSingleMemory(packet))
        )] = IBCPacketLib.COMMITMENT_MAGIC;
        emit IBCPacketLib.SendPacket(packet);
        return sequence;
    }

    function setPacketReceive(IBCPacket calldata packet)
        internal
        returns (bool)
    {
        bytes32 receiptCommitmentKey = IBCCommitment.batchReceiptsCommitmentKey(
            packet.destinationChannel, commitPacketsMemory(batchSingle(packet))
        );
        bool alreadyReceived =
            commitments[receiptCommitmentKey] == IBCPacketLib.COMMITMENT_MAGIC;
        if (!alreadyReceived) {
            commitments[receiptCommitmentKey] = IBCPacketLib.COMMITMENT_MAGIC;
        }
        return alreadyReceived;
    }

    function setNextSequenceRecv(
        uint32 destinationChannel,
        uint64 receivedSequence
    ) internal {
        uint64 expectedRecvSequence = uint64(
            uint256(
                commitments[IBCCommitment.nextSequenceRecvCommitmentKey(
                    destinationChannel
                )]
            )
        );
        if (expectedRecvSequence != receivedSequence) {
            revert IBCPacketLib.ErrPacketSequenceNextSequenceMismatch();
        }
        commitments[IBCCommitment.nextSequenceRecvCommitmentKey(
            destinationChannel
        )] = bytes32(uint256(expectedRecvSequence + 1));
    }

    function processReceive(
        IBCPacket[] calldata packets,
        address maker,
        bytes[] calldata makerMsgs,
        uint64 proofHeight,
        bytes calldata proof,
        bool intent
    ) internal {
        uint32 destinationChannel = packets[0].destinationChannel;
        IBCChannel storage channel = ensureChannelState(destinationChannel);
        IBCConnection storage connection =
            ensureConnectionState(channel.connectionId);
        if (!intent) {
            if (
                !verifyCommitment(
                    connection,
                    proofHeight,
                    proof,
                    IBCCommitment.batchPacketsCommitmentKey(
                        destinationChannel, commitPackets(packets)
                    ),
                    IBCPacketLib.COMMITMENT_MAGIC
                )
            ) {
                revert IBCPacketLib.ErrInvalidProof();
            }
        }
        IIBCModule module = lookupModuleByChannel(destinationChannel);
        uint256 l = packets.length;
        for (uint256 i; i < l; ) {
            IBCPacket calldata packet = packets[i];
            // Check packet height timeout
            if (
                packet.timeoutHeight > 0
                    && (block.number >= packet.timeoutHeight)
            ) {
                revert IBCPacketLib.ErrHeightTimeout();
            }
            // Check packet timestamp timeout
            // For some reason cosmos is using nanos, we try to follow their convention to avoid friction
            uint64 currentTimestamp = uint64(block.timestamp * 1e9);
            if (
                packet.timeoutTimestamp != 0
                    && (currentTimestamp >= packet.timeoutTimestamp)
            ) {
                revert IBCPacketLib.ErrTimestampTimeout();
            }

            // Allow unordered channels to have packets already received.
            bool alreadyReceived;
            if (channel.ordering == IBCChannelOrder.Unordered) {
                alreadyReceived = setPacketReceive(packet);
            } else if (channel.ordering == IBCChannelOrder.Ordered) {
                // We increase the sequence, hence can't avoid proofs
                if (intent) {
                    revert IBCPacketLib.ErrCannotIntentOrderedPacket();
                }
                setNextSequenceRecv(destinationChannel, packet.sequence);
            }

            if (!alreadyReceived) {
                bytes memory acknowledgement;
                bytes calldata makerMsg = makerMsgs[i];
                if (intent) {
                    acknowledgement =
                        module.onRecvIntentPacket(packet, maker, makerMsg);
                    emit IBCPacketLib.RecvIntentPacket(packet, maker, makerMsg);
                } else {
                    acknowledgement =
                        module.onRecvPacket(packet, maker, makerMsg);
                    emit IBCPacketLib.RecvPacket(packet, maker, makerMsg);
                }
                if (acknowledgement.length > 0) {
                    _writeAcknowledgement(packet, acknowledgement);
                }
            }

            unchecked {
                ++i;
            }
        }
    }

    function recvPacket(IBCMsgs.MsgPacketRecv calldata msg_) external {
        processReceive(
            msg_.packets,
            msg_.relayer,
            msg_.relayerMsgs,
            msg_.proofHeight,
            msg_.proof,
            false
        );
    }

    function recvIntentPacket(IBCMsgs.MsgIntentPacketRecv calldata msg_)
        external
        override
    {
        processReceive(
            msg_.packets,
            msg_.marketMaker,
            msg_.marketMakerMsgs,
            0,
            msg_.emptyProof,
            true
        );
    }

    function _writeAcknowledgement(
        IBCPacket calldata packet,
        bytes memory acknowledgement
    ) internal {
        bytes32 ackCommitmentKey = IBCCommitment.batchAcksCommitmentKey(
            packet.destinationChannel, commitPacketsMemory(batchSingle(packet))
        );
        bytes32 ackCommitment = commitments[ackCommitmentKey];
        if (ackCommitment != IBCPacketLib.COMMITMENT_NULL) {
            revert IBCPacketLib.ErrAcknowledgementAlreadyExists();
        }
        commitments[ackCommitmentKey] =
            commitAcksMemory(batchSingleAckMemory(acknowledgement));
        emit IBCPacketLib.WriteAcknowledgement(packet, acknowledgement);
    }

    /**
     * @dev writeAcknowledgement writes the packet execution acknowledgement to the state,
     * which will be verified by the counterparty chain using AcknowledgePacket.
     */
    function writeAcknowledgement(
        IBCPacket calldata packet,
        bytes memory acknowledgement
    ) external override {
        if (acknowledgement.length == 0) {
            revert IBCPacketLib.ErrAcknowledgementIsEmpty();
        }
        if (!authenticateChannelOwner(packet.destinationChannel)) {
            revert IBCPacketLib.ErrUnauthorized();
        }
        ensureChannelState(packet.destinationChannel);
        _writeAcknowledgement(packet, acknowledgement);
    }

    function setNextSequenceAck(
        uint32 sourceChannel,
        uint64 ackSequence
    ) internal {
        uint64 expectedAckSequence = uint64(
            uint256(
                commitments[IBCCommitment.nextSequenceAckCommitmentKey(
                    sourceChannel
                )]
            )
        );
        if (expectedAckSequence != ackSequence) {
            revert IBCPacketLib.ErrPacketSequenceAckSequenceMismatch();
        }
        commitments[IBCCommitment.nextSequenceAckCommitmentKey(sourceChannel)] =
            bytes32(uint256(expectedAckSequence + 1));
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
        uint256 l = msg_.packets.length;
        if (l == 0) {
            revert IBCPacketLib.ErrNotEnoughPackets();
        }
        uint32 sourceChannel = msg_.packets[0].sourceChannel;
        uint32 destinationChannel = msg_.packets[0].destinationChannel;
        IBCChannel storage channel = ensureChannelState(sourceChannel);
        IBCConnection storage connection =
            ensureConnectionState(channel.connectionId);
        deletePacketsCommitment(sourceChannel, msg_.packets);
        if (
            !verifyCommitment(
                connection,
                msg_.proofHeight,
                msg_.proof,
                IBCCommitment.batchAcksCommitmentKey(
                    destinationChannel, commitPackets(msg_.packets)
                ),
                commitAcks(msg_.acknowledgements)
            )
        ) {
            revert IBCPacketLib.ErrInvalidProof();
        }
        for (uint256 i; i < l; ) {
            IBCPacket calldata packet = msg_.packets[i];
            bytes calldata acknowledgement = msg_.acknowledgements[i];
            if (channel.ordering == IBCChannelOrder.Ordered) {
                setNextSequenceAck(sourceChannel, packet.sequence);
            }
            lookupModuleByChannel(sourceChannel).onAcknowledgementPacket(
                packet, acknowledgement, msg_.relayer
            );
            emit IBCPacketLib.AcknowledgePacket(
                packet, acknowledgement, msg_.relayer
            );

            unchecked {
                ++i;
            }
        }
    }

    function timeoutPacket(IBCMsgs.MsgPacketTimeout calldata msg_)
        external
        override
    {
        uint256 l = msg_.packets.length;
        if (l == 0) {
            revert IBCPacketLib.ErrNotEnoughPackets();
        }
        uint32 sourceChannel = msg_.packets[0].sourceChannel;
        uint32 destinationChannel = msg_.packets[0].destinationChannel;
        IBCChannel storage channel = ensureChannelState(sourceChannel);
        IBCConnection storage connection =
            ensureConnectionState(channel.connectionId);
        deletePacketsCommitment(sourceChannel, msg_.packets);
        ILightClient client = getClientInternal(connection.clientId);
        uint64 proofTimestamp =
            client.getTimestampAtHeight(connection.clientId, msg_.proofHeight);
        if (proofTimestamp == 0) {
            revert IBCPacketLib.ErrLatestTimestampNotFound();
        }
        if (channel.ordering == IBCChannelOrder.Ordered) {
            if (
                !verifyCommitment(
                    connection,
                    msg_.proofHeight,
                    msg_.proof,
                    IBCCommitment.nextSequenceRecvCommitmentKey(
                        destinationChannel
                    ),
                    commitRecvSeq(msg_.nextSequenceRecv)
                )
            ) {
                revert IBCPacketLib.ErrInvalidProof();
            }
        } else if (channel.ordering == IBCChannelOrder.Unordered) {
            if (
                !verifyAbsentCommitment(
                    connection,
                    msg_.proofHeight,
                    msg_.proof,
                    IBCCommitment.batchReceiptsCommitmentKey(
                        destinationChannel, commitPackets(msg_.packets)
                    )
                )
            ) {
                revert IBCPacketLib.ErrInvalidProof();
            }
        }
        for (uint256 i; i < l; ) {
            IBCPacket calldata packet = msg_.packets[i];
            if (
                packet.timeoutTimestamp > 0
                    && packet.timeoutTimestamp >= proofTimestamp
            ) {
                revert IBCPacketLib.ErrTimeoutTimestampNotReached();
            }
            if (
                packet.timeoutHeight > 0
                    && packet.timeoutHeight > msg_.proofHeight
            ) {
                revert IBCPacketLib.ErrTimeoutHeightNotReached();
            }
            if (channel.ordering == IBCChannelOrder.Ordered) {
                if (msg_.nextSequenceRecv > packet.sequence) {
                    revert
                        IBCPacketLib
                        .ErrNextSequenceMustBeGreaterThanTimeoutSequence();
                }
            }
            lookupModuleByChannel(sourceChannel).onTimeoutPacket(
                packet, msg_.relayer
            );
            emit IBCPacketLib.TimeoutPacket(packet, msg_.relayer);
            unchecked {
                ++i;
            }
        }
        
    }

    function verifyCommitment(
        IBCConnection storage connection,
        uint64 height,
        bytes calldata proof,
        bytes32 path,
        bytes32 commitment
    ) internal virtual returns (bool) {
        uint32 clientId = connection.clientId;
        return getClientInternal(clientId).verifyMembership(
            clientId,
            height,
            proof,
            abi.encodePacked(connection.counterparty.merklePrefix),
            abi.encodePacked(path),
            abi.encodePacked(commitment)
        );
    }

    function verifyAbsentCommitment(
        IBCConnection storage connection,
        uint64 height,
        bytes calldata proof,
        bytes32 path
    ) internal virtual returns (bool) {
        uint32 clientId = connection.clientId;
        return getClientInternal(clientId).verifyNonMembership(
            clientId,
            height,
            proof,
            abi.encodePacked(connection.counterparty.merklePrefix),
            abi.encodePacked(path)
        );
    }

    function ensureChannelState(uint32 channelId)
        internal
        view
        returns (IBCChannel storage)
    {
        IBCChannel storage channel = channels[channelId];
        if (channel.state != IBCChannelState.Open) {
            revert IBCPacketLib.ErrInvalidChannelState();
        }
        return channel;
    }

    function generatePacketSequence(uint32 channelId)
        internal
        returns (uint64)
    {
        uint64 seq = uint64(
            uint256(
                commitments[IBCCommitment.nextSequenceSendCommitmentKey(
                    channelId
                )]
            )
        );
        commitments[IBCCommitment.nextSequenceSendCommitmentKey(channelId)] =
            bytes32(uint256(seq + 1));
        return seq;
    }

    function deletePacketsCommitment(
        uint32 sourceChannel,
        IBCPacket[] calldata packets
    ) internal {
        bytes32 packetCommitmentKey = IBCCommitment.batchPacketsCommitmentKey(
            sourceChannel, commitPackets(packets)
        );
        bytes32 packetCommitment = commitments[packetCommitmentKey];
        if (packetCommitment != IBCPacketLib.COMMITMENT_MAGIC) {
            revert IBCPacketLib.ErrPacketCommitmentNotFound();
        }
        delete commitments[packetCommitmentKey];
    }

    function commitAcks(bytes[] calldata acknowledgements)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(abi.encode(acknowledgements));
    }

    function commitAcksMemory(bytes[] memory acknowledgements)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(abi.encode(acknowledgements));
    }

    function commitPackets(IBCPacket[] calldata packets)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(abi.encode(packets));
    }

    function commitPacketsMemory(IBCPacket[] memory packets)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(abi.encode(packets));
    }

    function commitRecvSeq(uint64 sequence) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(sequence));
    }
}
