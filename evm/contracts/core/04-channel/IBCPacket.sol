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
    bytes32 public constant COMMITMENT_MAGIC_ACK =
        0x0200000000000000000000000000000000000000000000000000000000000000;
    bytes32 public constant COMMITMENT_NULL = bytes32(uint256(0));

    event PacketSend(
        uint32 indexed channelId, bytes32 indexed packetHash, IBCPacket packet
    );
    event PacketRecv(
        uint32 indexed channelId,
        bytes32 indexed packetHash,
        address indexed maker,
        bytes makerMsg
    );
    event IntentPacketRecv(
        uint32 indexed channelId,
        bytes32 indexed packetHash,
        address indexed maker,
        bytes makerMsg
    );
    event WriteAck(
        uint32 indexed channelId,
        bytes32 indexed packetHash,
        bytes acknowledgement
    );
    event PacketAck(
        uint32 indexed channelId,
        bytes32 indexed packetHash,
        bytes acknowledgement,
        address indexed maker
    );
    event PacketTimeout(
        uint32 indexed channelId,
        bytes32 indexed packetHash,
        address indexed maker
    );
    event BatchedPreviouslySent(
        uint32 indexed channelId,
        bytes32 indexed batchHash,
        bytes32 indexed packetHash
    );
    event BatchedPreviouslyAcked(
        uint32 indexed channelId,
        bytes32 indexed batchHash,
        bytes32 indexed packetHash
    );

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
        bytes memory ack
    ) internal pure returns (bytes32) {
        bytes[] memory acks = new bytes[](1);
        acks[0] = ack;
        return commitAcksMemory(acks);
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

    function commitPacket(
        IBCPacket memory packet
    ) internal pure returns (bytes32) {
        IBCPacket[] memory packets = new IBCPacket[](1);
        packets[0] = packet;
        return commitPacketsMemory(packets);
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
    ) external override restricted {
        uint256 l = msg_.packets.length;
        // No reason to batch less than 2 packets as they are already individually committed.
        if (l < 2) {
            revert IBCErrors.ErrNotEnoughPackets();
        }
        uint32 channelId = msg_.packets[0].sourceChannelId;
        bytes32 batchHash = IBCPacketLib.commitPackets(msg_.packets);
        for (uint256 i = 0; i < l; i++) {
            IBCPacket calldata packet = msg_.packets[i];
            if (i > 0) {
                if (packet.sourceChannelId != channelId) {
                    revert IBCErrors.ErrBatchSameChannelOnly();
                }
            }
            // If the channel mismatch, the commitment will be zero
            bytes32 packetHash = IBCPacketLib.commitPacket(packet);
            bytes32 commitment =
                commitments[IBCCommitment.batchPacketsCommitmentKey(packetHash)];
            // Every packet must have been previously sent to be batched
            if (commitment != IBCPacketLib.COMMITMENT_MAGIC) {
                revert IBCErrors.ErrPacketCommitmentNotFound();
            }
            emit IBCPacketLib.BatchedPreviouslySent(
                channelId, batchHash, packetHash
            );
        }
        commitments[IBCCommitment.batchPacketsCommitmentKey(batchHash)] =
            IBCPacketLib.COMMITMENT_MAGIC;
    }

    function batchAcks(
        IBCMsgs.MsgBatchAcks calldata msg_
    ) external override restricted {
        uint256 l = msg_.packets.length;
        // No reason to batch less than 2 packets as they are already individually committed.
        if (l < 2) {
            revert IBCErrors.ErrNotEnoughPackets();
        }
        uint32 channelId = msg_.packets[0].destinationChannelId;
        bytes32 batchHash = IBCPacketLib.commitPackets(msg_.packets);
        for (uint256 i = 0; i < l; i++) {
            IBCPacket calldata packet = msg_.packets[i];
            if (i > 0) {
                if (packet.destinationChannelId != channelId) {
                    revert IBCErrors.ErrBatchSameChannelOnly();
                }
            }
            bytes calldata ack = msg_.acks[i];
            // If the channel mismatch, the commitment will be zero.
            bytes32 packetHash = IBCPacketLib.commitPacket(packet);
            bytes32 commitment = commitments[IBCCommitment
                .batchReceiptsCommitmentKey(packetHash)];
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
            emit IBCPacketLib.BatchedPreviouslyAcked(
                channelId, batchHash, packetHash
            );
        }
        commitments[IBCCommitment.batchReceiptsCommitmentKey(
            IBCPacketLib.commitPackets(msg_.packets)
        )] = IBCPacketLib.commitAcks(msg_.acks);
    }

    function sendPacket(
        uint32 sourceChannelId,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external override returns (IBCPacket memory) {
        // Deprecated timeout height
        if (timeoutHeight != 0) {
            revert IBCErrors.ErrTimeoutHeightUnsupported();
        }
        if (timeoutTimestamp == 0) {
            revert IBCErrors.ErrTimeoutMustBeSet();
        }
        if (!authenticateChannelOwner(sourceChannelId)) {
            revert IBCErrors.ErrUnauthorized();
        }
        IBCChannel storage channel = ensureChannelState(sourceChannelId);
        IBCPacket memory packet = IBCPacket({
            sourceChannelId: sourceChannelId,
            destinationChannelId: channel.counterpartyChannelId,
            data: data,
            timeoutHeight: timeoutHeight,
            timeoutTimestamp: timeoutTimestamp
        });
        bytes32 packetHash = IBCPacketLib.commitPacket(packet);
        bytes32 commitmentKey =
            IBCCommitment.batchPacketsCommitmentKey(packetHash);
        if (commitments[commitmentKey] != IBCPacketLib.COMMITMENT_NULL) {
            revert IBCErrors.ErrPacketAlreadyExist();
        }
        commitments[commitmentKey] = IBCPacketLib.COMMITMENT_MAGIC;
        emit IBCPacketLib.PacketSend(sourceChannelId, packetHash, packet);
        return packet;
    }

    function _markPacketAsReceived(
        bytes32 commitmentKey
    ) internal returns (bool) {
        bool alreadyReceived =
            commitments[commitmentKey] != IBCPacketLib.COMMITMENT_NULL;
        if (!alreadyReceived) {
            commitments[commitmentKey] = IBCPacketLib.COMMITMENT_MAGIC;
        }
        return alreadyReceived;
    }

    function _processReceive(
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
        uint32 destinationChannelId = packets[0].destinationChannelId;
        IBCChannel storage channel = ensureChannelState(destinationChannelId);
        uint32 clientId = ensureConnectionState(channel.connectionId);
        if (!intent) {
            bytes32 proofCommitmentKey = IBCCommitment.batchPacketsCommitmentKey(
                IBCPacketLib.commitPackets(packets)
            );
            if (
                !_verifyCommitment(
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
            if (packet.destinationChannelId != destinationChannelId) {
                revert IBCErrors.ErrBatchSameChannelOnly();
            }
            // Deprecated timeout height
            if (packet.timeoutHeight != 0) {
                revert IBCErrors.ErrTimeoutHeightUnsupported();
            }

            // Check packet timestamp timeout
            // For some reason cosmos is using nanos, we try to follow their convention to avoid friction
            uint64 currentTimestamp = uint64(block.timestamp * 1e9);
            if (currentTimestamp >= packet.timeoutTimestamp) {
                revert IBCErrors.ErrTimestampTimeout();
            }

            bytes32 packetHash = IBCPacketLib.commitPacket(packet);
            bytes32 commitmentKey =
                IBCCommitment.batchReceiptsCommitmentKey(packetHash);

            if (!_markPacketAsReceived(commitmentKey)) {
                bytes memory acknowledgement;
                bytes calldata makerMsg = makerMsgs[i];
                if (intent) {
                    acknowledgement = module.onRecvIntentPacket(
                        msg.sender, packet, maker, makerMsg
                    );
                    emit IBCPacketLib.IntentPacketRecv(
                        packet.destinationChannelId, packetHash, maker, makerMsg
                    );
                } else {
                    acknowledgement =
                        module.onRecvPacket(msg.sender, packet, maker, makerMsg);
                    emit IBCPacketLib.PacketRecv(
                        packet.destinationChannelId, packetHash, maker, makerMsg
                    );
                }
                if (acknowledgement.length > 0) {
                    _writeAcknowledgement(commitmentKey, acknowledgement);
                    emit IBCPacketLib.WriteAck(
                        packet.destinationChannelId, packetHash, acknowledgement
                    );
                }
            }
        }
    }

    function recvPacket(
        IBCMsgs.MsgPacketRecv calldata msg_
    ) external restricted {
        _processReceive(
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
    ) external override restricted {
        _processReceive(
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
        commitments[commitmentKey] = IBCPacketLib.commitAck(acknowledgement);
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
        bytes32 packetHash = IBCPacketLib.commitPacket(packet);
        bytes32 commitmentKey =
            IBCCommitment.batchReceiptsCommitmentKey(packetHash);
        _writeAcknowledgement(commitmentKey, acknowledgement);
        emit IBCPacketLib.WriteAck(
            packet.destinationChannelId, packetHash, acknowledgement
        );
    }

    function acknowledgePacket(
        IBCMsgs.MsgPacketAcknowledgement calldata msg_
    ) external override restricted {
        uint256 l = msg_.packets.length;
        if (l == 0) {
            revert IBCErrors.ErrNotEnoughPackets();
        }
        uint32 sourceChannelId = msg_.packets[0].sourceChannelId;
        IBCChannel storage channel = ensureChannelState(sourceChannelId);
        uint32 clientId = ensureConnectionState(channel.connectionId);
        bytes32 commitmentKey = IBCCommitment.batchReceiptsCommitmentKey(
            IBCPacketLib.commitPackets(msg_.packets)
        );
        bytes32 commitmentValue = IBCPacketLib.commitAcks(msg_.acknowledgements);
        if (
            !_verifyCommitment(
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
            if (packet.sourceChannelId != sourceChannelId) {
                revert IBCErrors.ErrBatchSameChannelOnly();
            }
            _markPacketAsAcknowledged(packet);
            bytes calldata acknowledgement = msg_.acknowledgements[i];
            module.onAcknowledgementPacket(
                msg.sender, packet, acknowledgement, msg_.relayer
            );
            emit IBCPacketLib.PacketAck(
                sourceChannelId,
                IBCPacketLib.commitPacket(packet),
                acknowledgement,
                msg_.relayer
            );
        }
    }

    function timeoutPacket(
        IBCMsgs.MsgPacketTimeout calldata msg_
    ) external override restricted {
        IBCPacket calldata packet = msg_.packet;
        uint32 sourceChannelId = packet.sourceChannelId;
        IBCChannel storage channel = ensureChannelState(sourceChannelId);
        uint32 clientId = ensureConnectionState(channel.connectionId);
        ILightClient client = getClientInternal(clientId);
        uint64 proofTimestamp =
            client.getTimestampAtHeight(clientId, msg_.proofHeight);
        if (proofTimestamp == 0) {
            revert IBCErrors.ErrLatestTimestampNotFound();
        }
        bytes32 packetHash = IBCPacketLib.commitPacket(packet);
        bytes32 commitmentKey =
            IBCCommitment.batchReceiptsCommitmentKey(packetHash);
        if (
            !_verifyAbsentCommitment(
                clientId, msg_.proofHeight, msg_.proof, commitmentKey
            )
        ) {
            revert IBCErrors.ErrInvalidProof();
        }
        IIBCModule module = lookupModuleByChannel(sourceChannelId);
        _markPacketAsAcknowledged(packet);
        if (packet.timeoutTimestamp == 0) {
            revert IBCErrors.ErrTimeoutMustBeSet();
        }
        if (packet.timeoutTimestamp > proofTimestamp) {
            revert IBCErrors.ErrTimeoutTimestampNotReached();
        }
        module.onTimeoutPacket(msg.sender, packet, msg_.relayer);
        emit IBCPacketLib.PacketTimeout(
            sourceChannelId, packetHash, msg_.relayer
        );
    }

    function _verifyCommitment(
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

    function _verifyAbsentCommitment(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes32 path
    ) internal virtual returns (bool) {
        return getClientInternal(clientId).verifyNonMembership(
            clientId, height, proof, abi.encodePacked(path)
        );
    }

    function _markPacketAsAcknowledged(
        IBCPacket calldata packet
    ) internal {
        bytes32 commitmentKey = IBCCommitment.batchPacketsCommitmentKey(
            IBCPacketLib.commitPacket(packet)
        );
        bytes32 commitment = commitments[commitmentKey];
        if (commitment == IBCPacketLib.COMMITMENT_MAGIC_ACK) {
            revert IBCErrors.ErrPacketAlreadyAcknowledged();
        }
        if (commitment != IBCPacketLib.COMMITMENT_MAGIC) {
            revert IBCErrors.ErrPacketCommitmentNotFound();
        }
        commitments[commitmentKey] = IBCPacketLib.COMMITMENT_MAGIC_ACK;
    }
}
