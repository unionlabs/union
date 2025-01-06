pragma solidity ^0.8.27;

import "../24-host/IBCStore.sol";
import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCStore.sol";
import "../24-host/IBCCommitment.sol";
import "../04-channel/IIBCPacket.sol";
import "../05-port/IIBCModule.sol";
import "../Types.sol";

library IBCPacketLib {
    bytes32 public constant COMMITMENT_MAGIC =
        0x0100000000000000000000000000000000000000000000000000000000000000;
    bytes32 public constant COMMITMENT_NULL = bytes32(uint256(0));

    event PacketSend(IBCPacket packet);
    event PacketRecv(IBCPacket packet, address maker, bytes makerMsg);
    event IntentPacketRecv(IBCPacket packet, address maker, bytes makerMsg);
    event WriteAck(IBCPacket packet, bytes acknowledgement);
    event PacketAck(IBCPacket packet, bytes acknowledgement, address maker);
    event PacketTimeout(IBCPacket packet, address maker);

    function commitAcksMemory(
        bytes[] memory acks
    ) internal pure returns (bytes32) {
        return mergeAck(keccak256(abi.encode(acks)));
    }

    function commitAcks(
        bytes[] calldata acks
    ) internal pure returns (bytes32) {
        return mergeAck(keccak256(abi.encode(acks)));
    }

    function commitAck(
        bytes calldata ack
    ) internal pure returns (bytes32) {
        return mergeAck(keccak256(abi.encodePacked(ack)));
    }

    function commitAckMemory(
        bytes memory ack
    ) internal pure returns (bytes32) {
        return mergeAck(keccak256(abi.encodePacked(ack)));
    }

    function commitPacketsMemory(
        IBCPacket[] memory packets
    ) internal pure returns (bytes32) {
        return keccak256(abi.encode(packets));
    }

    function commitPackets(
        IBCPacket[] calldata packets
    ) internal pure returns (bytes32) {
        return keccak256(abi.encode(packets));
    }

    function commitPacketMemory(
        IBCPacket memory packet
    ) internal pure returns (bytes32) {
        return keccak256(abi.encode(packet));
    }

    function commitPacket(
        IBCPacket calldata packet
    ) internal pure returns (bytes32) {
        return keccak256(abi.encode(packet));
    }

    function mergeAck(
        bytes32 ack
    ) internal pure returns (bytes32) {
        return COMMITMENT_MAGIC
            | (
                ack
                    & 0x00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
            );
    }
}

/**
 * @dev IBCPacket is a contract that implements [ICS-4](https://github.com/cosmos/ibc/tree/main/spec/core/ics-004-channel-and-packet-semantics).
 */
abstract contract IBCPacketImpl is IBCStore, IIBCPacket {
    function batchSend(
        IBCMsgs.MsgBatchSend calldata msg_
    ) external override {
        uint256 l = msg_.packets.length;
        // No reason to batch less than 2 packets as they are already individually committed.
        if (l < 2) {
            revert IBCErrors.ErrNotEnoughPackets();
        }
        for (uint256 i = 0; i < l; i++) {
            IBCPacket calldata packet = msg_.packets[i];
            // If the channel mismatch, the commitment will be zero
            bytes32 commitment = commitments[IBCCommitment
                .batchPacketsCommitmentKey(
                msg_.sourceChannelId, IBCPacketLib.commitPacket(packet)
            )];
            // Every packet must have been previously sent to be batched
            if (commitment != IBCPacketLib.COMMITMENT_MAGIC) {
                revert IBCErrors.ErrPacketCommitmentNotFound();
            }
        }
        commitments[IBCCommitment.batchPacketsCommitmentKey(
            msg_.sourceChannelId, IBCPacketLib.commitPackets(msg_.packets)
        )] = IBCPacketLib.COMMITMENT_MAGIC;
    }

    function batchAcks(
        IBCMsgs.MsgBatchAcks calldata msg_
    ) external override {
        uint256 l = msg_.packets.length;
        // No reason to batch less than 2 packets as they are already individually committed.
        if (l < 2) {
            revert IBCErrors.ErrNotEnoughPackets();
        }
        for (uint256 i = 0; i < l; i++) {
            IBCPacket calldata packet = msg_.packets[i];
            bytes calldata ack = msg_.acks[i];
            // If the channel mismatch, the commitment will be zero.
            bytes32 commitment = commitments[IBCCommitment
                .batchReceiptsCommitmentKey(
                msg_.sourceChannelId, IBCPacketLib.commitPacket(packet)
            )];
            // Can't batch an empty ack.
            if (
                commitment == IBCPacketLib.COMMITMENT_NULL
                    || commitment == IBCPacketLib.COMMITMENT_MAGIC
            ) {
                revert IBCErrors.ErrAcknowledgementIsEmpty();
            }
            // Of course the ack must match.
            if (commitment != IBCPacketLib.commitAck(ack)) {
                revert IBCErrors.ErrCommittedAckNotPresent();
            }
        }
        commitments[IBCCommitment.batchReceiptsCommitmentKey(
            msg_.sourceChannelId, IBCPacketLib.commitPackets(msg_.packets)
        )] = IBCPacketLib.commitAcks(msg_.acks);
    }

    function sendPacket(
        uint32 sourceChannel,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external override returns (IBCPacket memory) {
        if (timeoutTimestamp == 0 && timeoutHeight == 0) {
            revert IBCErrors.ErrTimeoutMustBeSet();
        }
        if (!authenticateChannelOwner(sourceChannel)) {
            revert IBCErrors.ErrUnauthorized();
        }
        IBCChannel storage channel = ensureChannelState(sourceChannel);
        IBCPacket memory packet = IBCPacket({
            sourceChannelId: sourceChannel,
            destinationChannelId: channel.counterpartyChannelId,
            data: data,
            timeoutHeight: timeoutHeight,
            timeoutTimestamp: timeoutTimestamp
        });
        bytes32 commitmentKey = IBCCommitment.batchPacketsCommitmentKey(
            sourceChannel, IBCPacketLib.commitPacketMemory(packet)
        );
        if (commitments[commitmentKey] != IBCPacketLib.COMMITMENT_NULL) {
            revert IBCErrors.ErrPacketAlreadyExist();
        }
        commitments[commitmentKey] = IBCPacketLib.COMMITMENT_MAGIC;

        emit IBCPacketLib.PacketSend(packet);

        return packet;
    }

    function setPacketReceive(
        bytes32 commitmentKey
    ) internal returns (bool) {
        bool alreadyReceived =
            commitments[commitmentKey] != IBCPacketLib.COMMITMENT_NULL;
        if (!alreadyReceived) {
            commitments[commitmentKey] = IBCPacketLib.COMMITMENT_MAGIC;
        }
        return alreadyReceived;
    }

    function processReceive(
        IBCPacket[] calldata packets,
        address maker,
        bytes[] calldata makerMsgs,
        uint64 proofHeight,
        bytes calldata proof,
        bool intent
    ) internal {
        uint256 l = packets.length;
        if (l == 0) {
            revert IBCErrors.ErrNotEnoughPackets();
        }
        uint32 sourceChannelId = packets[0].sourceChannelId;
        uint32 destinationChannelId = packets[0].destinationChannelId;
        IBCChannel storage channel = ensureChannelState(destinationChannelId);
        uint32 clientId = ensureConnectionState(channel.connectionId);
        if (!intent) {
            bytes32 proofCommitmentKey;
            if (l == 1) {
                proofCommitmentKey = IBCCommitment.batchPacketsCommitmentKey(
                    sourceChannelId, IBCPacketLib.commitPacket(packets[0])
                );
            } else {
                proofCommitmentKey = IBCCommitment.batchPacketsCommitmentKey(
                    sourceChannelId, IBCPacketLib.commitPackets(packets)
                );
            }
            if (
                !verifyCommitment(
                    clientId,
                    proofHeight,
                    proof,
                    proofCommitmentKey,
                    IBCPacketLib.COMMITMENT_MAGIC
                )
            ) {
                revert IBCErrors.ErrInvalidProof();
            }
        }
        IIBCModule module = lookupModuleByChannel(destinationChannelId);
        for (uint256 i = 0; i < l; i++) {
            IBCPacket calldata packet = packets[i];
            // Check packet height timeout
            if (
                packet.timeoutHeight > 0
                    && (block.number >= packet.timeoutHeight)
            ) {
                revert IBCErrors.ErrHeightTimeout();
            }
            // Check packet timestamp timeout
            // For some reason cosmos is using nanos, we try to follow their convention to avoid friction
            uint64 currentTimestamp = uint64(block.timestamp * 1e9);
            if (
                packet.timeoutTimestamp != 0
                    && (currentTimestamp >= packet.timeoutTimestamp)
            ) {
                revert IBCErrors.ErrTimestampTimeout();
            }

            bytes32 commitmentKey = IBCCommitment.batchReceiptsCommitmentKey(
                destinationChannelId, IBCPacketLib.commitPacket(packet)
            );

            if (!setPacketReceive(commitmentKey)) {
                bytes memory acknowledgement;
                bytes calldata makerMsg = makerMsgs[i];
                if (intent) {
                    acknowledgement =
                        module.onRecvIntentPacket(packet, maker, makerMsg);
                    emit IBCPacketLib.IntentPacketRecv(packet, maker, makerMsg);
                } else {
                    acknowledgement =
                        module.onRecvPacket(packet, maker, makerMsg);
                    emit IBCPacketLib.PacketRecv(packet, maker, makerMsg);
                }
                if (acknowledgement.length > 0) {
                    _writeAcknowledgement(commitmentKey, acknowledgement);
                    emit IBCPacketLib.WriteAck(packet, acknowledgement);
                }
            }
        }
    }

    function recvPacket(
        IBCMsgs.MsgPacketRecv calldata msg_
    ) external {
        processReceive(
            msg_.packets,
            msg_.relayer,
            msg_.relayerMsgs,
            msg_.proofHeight,
            msg_.proof,
            false
        );
    }

    function recvIntentPacket(
        IBCMsgs.MsgIntentPacketRecv calldata msg_
    ) external override {
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
        bytes32 commitmentKey,
        bytes memory acknowledgement
    ) internal {
        bytes32 commitment = commitments[commitmentKey];
        if (commitment == IBCPacketLib.COMMITMENT_NULL) {
            revert IBCErrors.ErrPacketNotReceived();
        }
        if (commitment != IBCPacketLib.COMMITMENT_MAGIC) {
            revert IBCErrors.ErrAcknowledgementAlreadyExists();
        }
        commitments[commitmentKey] =
            IBCPacketLib.commitAckMemory(acknowledgement);
    }

    function writeAcknowledgement(
        IBCPacket calldata packet,
        bytes memory acknowledgement
    ) external override {
        if (acknowledgement.length == 0) {
            revert IBCErrors.ErrAcknowledgementIsEmpty();
        }
        if (!authenticateChannelOwner(packet.destinationChannelId)) {
            revert IBCErrors.ErrUnauthorized();
        }
        ensureChannelState(packet.destinationChannelId);
        bytes32 commitmentKey = IBCCommitment.batchReceiptsCommitmentKey(
            packet.destinationChannelId, IBCPacketLib.commitPacket(packet)
        );
        _writeAcknowledgement(commitmentKey, acknowledgement);
        emit IBCPacketLib.WriteAck(packet, acknowledgement);
    }

    function acknowledgePacket(
        IBCMsgs.MsgPacketAcknowledgement calldata msg_
    ) external override {
        uint256 l = msg_.packets.length;
        if (l == 0) {
            revert IBCErrors.ErrNotEnoughPackets();
        }
        uint32 sourceChannelId = msg_.packets[0].sourceChannelId;
        uint32 destinationChannelId = msg_.packets[0].destinationChannelId;
        IBCChannel storage channel = ensureChannelState(sourceChannelId);
        uint32 clientId = ensureConnectionState(channel.connectionId);
        bytes32 commitmentKey;
        bytes32 commitmentValue;
        if (l == 1) {
            commitmentKey = IBCCommitment.batchReceiptsCommitmentKey(
                destinationChannelId, IBCPacketLib.commitPacket(msg_.packets[0])
            );
            commitmentValue = IBCPacketLib.commitAck(msg_.acknowledgements[0]);
        } else {
            commitmentKey = IBCCommitment.batchReceiptsCommitmentKey(
                destinationChannelId, IBCPacketLib.commitPackets(msg_.packets)
            );
            commitmentValue = IBCPacketLib.commitAcks(msg_.acknowledgements);
        }
        if (
            !verifyCommitment(
                clientId,
                msg_.proofHeight,
                msg_.proof,
                commitmentKey,
                commitmentValue
            )
        ) {
            revert IBCErrors.ErrInvalidProof();
        }
        IIBCModule module = lookupModuleByChannel(sourceChannelId);
        for (uint256 i = 0; i < l; i++) {
            IBCPacket calldata packet = msg_.packets[i];
            deletePacketCommitment(sourceChannelId, packet);
            bytes calldata acknowledgement = msg_.acknowledgements[i];
            module.onAcknowledgementPacket(
                packet, acknowledgement, msg_.relayer
            );
            emit IBCPacketLib.PacketAck(packet, acknowledgement, msg_.relayer);
        }
    }

    function timeoutPacket(
        IBCMsgs.MsgPacketTimeout calldata msg_
    ) external override {
        IBCPacket calldata packet = msg_.packet;
        uint32 sourceChannelId = packet.sourceChannelId;
        uint32 destinationChannelId = packet.destinationChannelId;
        IBCChannel storage channel = ensureChannelState(sourceChannelId);
        uint32 clientId = ensureConnectionState(channel.connectionId);
        ILightClient client = getClientInternal(clientId);
        uint64 proofTimestamp =
            client.getTimestampAtHeight(clientId, msg_.proofHeight);
        if (proofTimestamp == 0) {
            revert IBCErrors.ErrLatestTimestampNotFound();
        }
        bytes32 commitmentKey = IBCCommitment.batchReceiptsCommitmentKey(
            destinationChannelId, IBCPacketLib.commitPacket(packet)
        );
        if (
            !verifyAbsentCommitment(
                clientId, msg_.proofHeight, msg_.proof, commitmentKey
            )
        ) {
            revert IBCErrors.ErrInvalidProof();
        }
        IIBCModule module = lookupModuleByChannel(sourceChannelId);
        deletePacketCommitment(sourceChannelId, packet);
        if (packet.timeoutTimestamp == 0 && packet.timeoutHeight == 0) {
            revert IBCErrors.ErrTimeoutMustBeSet();
        }
        if (
            packet.timeoutTimestamp > 0
                && packet.timeoutTimestamp > proofTimestamp
        ) {
            revert IBCErrors.ErrTimeoutTimestampNotReached();
        }
        if (packet.timeoutHeight > 0 && packet.timeoutHeight > msg_.proofHeight)
        {
            revert IBCErrors.ErrTimeoutHeightNotReached();
        }
        module.onTimeoutPacket(packet, msg_.relayer);
        emit IBCPacketLib.PacketTimeout(packet, msg_.relayer);
    }

    function verifyCommitment(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes32 path,
        bytes32 commitment
    ) internal virtual returns (bool) {
        return getClientInternal(clientId).verifyMembership(
            clientId,
            height,
            proof,
            abi.encodePacked(path),
            abi.encodePacked(commitment)
        );
    }

    function verifyAbsentCommitment(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes32 path
    ) internal virtual returns (bool) {
        return getClientInternal(clientId).verifyNonMembership(
            clientId, height, proof, abi.encodePacked(path)
        );
    }

    function deletePacketCommitment(
        uint32 sourceChannelId,
        IBCPacket calldata packet
    ) internal {
        bytes32 commitmentKey = IBCCommitment.batchPacketsCommitmentKey(
            sourceChannelId, IBCPacketLib.commitPacket(packet)
        );
        bytes32 commitment = commitments[commitmentKey];
        if (commitment != IBCPacketLib.COMMITMENT_MAGIC) {
            revert IBCErrors.ErrPacketCommitmentNotFound();
        }
        delete commitments[commitmentKey];
    }
}
