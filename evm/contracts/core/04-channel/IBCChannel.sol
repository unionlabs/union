pragma solidity ^0.8.27;

import "../24-host/IBCStore.sol";
import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCCommitment.sol";
import "../04-channel/IIBCChannel.sol";
import "../05-port/IIBCModule.sol";

library IBCChannelLib {
    event ChannelOpenInit(
        address portId, uint32 channelId, uint32 connectionId, bytes32 version
    );
    event ChannelOpenTry(
        address portId,
        uint32 channelId,
        uint32 counterpartyChannelId,
        uint32 connectionId,
        bytes32 version
    );
    event ChannelOpenAck(
        address portId,
        uint32 channelId,
        uint32 counterpartyChannelId,
        uint32 connectionId
    );
    event ChannelOpenConfirm(
        address portId,
        uint32 channelId,
        uint32 counterpartyChannelId,
        uint32 connectionId
    );
    event ChannelCloseInit(address portId, uint32 channelId);
    event ChannelCloseConfirm(address portId, uint32 channelId);
}

/**
 * @dev IBCChannelHandshake is a contract that implements [ICS-4](https://github.com/cosmos/ibc/tree/main/spec/core/ics-004-channel-and-packet-semantics).
 */
abstract contract IBCChannelImpl is IBCStore, IIBCChannel {
    /**
     * @dev channelOpenInit is called by a module to initiate a channel opening handshake with a module on another chain.
     */
    function channelOpenInit(
        IBCMsgs.MsgChannelOpenInit calldata msg_
    ) external override returns (uint32) {
        if (
            msg_.ordering != IBCChannelOrder.Unordered
                && msg_.ordering != IBCChannelOrder.Ordered
        ) {
            revert IBCErrors.ErrInvalidChannelOrdering();
        }
        ensureConnectionState(msg_.connectionId);
        uint32 channelId = generateChannelIdentifier();
        IBCChannel storage channel = channels[channelId];
        channel.state = IBCChannelState.Init;
        channel.connectionId = msg_.connectionId;
        channel.version = msg_.version;
        channel.ordering = msg_.ordering;
        initializeChannelSequences(channelId);
        commitChannel(channelId, channel);
        claimChannel(msg_.portId, channelId);
        IIBCModule(msg_.portId).onChanOpenInit(
            msg_.ordering,
            msg_.connectionId,
            channelId,
            msg_.version,
            msg_.relayer
        );
        emit IBCChannelLib.ChannelOpenInit(
            msg_.portId, channelId, msg_.connectionId, msg_.version
        );
        return channelId;
    }

    /**
     * @dev channelOpenTry is called by a module to accept the first step of a channel opening handshake initiated by a module on another chain.
     */
    function channelOpenTry(
        IBCMsgs.MsgChannelOpenTry calldata msg_
    ) external override returns (uint32) {
        if (
            msg_.channel.ordering != IBCChannelOrder.Unordered
                && msg_.channel.ordering != IBCChannelOrder.Ordered
        ) {
            revert IBCErrors.ErrInvalidChannelOrdering();
        }
        if (msg_.channel.state != IBCChannelState.TryOpen) {
            revert IBCErrors.ErrInvalidChannelState();
        }
        uint32 clientId = ensureConnectionState(msg_.channel.connectionId);
        IBCChannel memory expectedChannel = IBCChannel({
            state: IBCChannelState.Init,
            ordering: msg_.channel.ordering,
            counterpartyChannelId: 0,
            connectionId: getCounterpartyConnection(msg_.channel.connectionId),
            version: msg_.counterpartyVersion
        });
        if (
            !verifyChannelState(
                clientId,
                msg_.proofHeight,
                msg_.proofInit,
                msg_.channel.counterpartyChannelId,
                expectedChannel
            )
        ) {
            revert IBCErrors.ErrInvalidProof();
        }
        uint32 channelId = generateChannelIdentifier();
        channels[channelId] = msg_.channel;
        initializeChannelSequences(channelId);
        commitChannelCalldata(channelId, msg_.channel);
        claimChannel(msg_.portId, channelId);
        IIBCModule(msg_.portId).onChanOpenTry(
            msg_.channel.ordering,
            msg_.channel.connectionId,
            channelId,
            msg_.channel.counterpartyChannelId,
            msg_.channel.version,
            msg_.counterpartyVersion,
            msg_.relayer
        );
        emit IBCChannelLib.ChannelOpenTry(
            msg_.portId,
            channelId,
            msg_.channel.counterpartyChannelId,
            msg_.channel.connectionId,
            msg_.counterpartyVersion
        );
        return channelId;
    }

    /**
     * @dev channelOpenAck is called by the handshake-originating module to acknowledge the acceptance of the initial request by the counterparty module on the other chain.
     */
    function channelOpenAck(
        IBCMsgs.MsgChannelOpenAck calldata msg_
    ) external override {
        IBCChannel storage channel = channels[msg_.channelId];
        if (channel.state != IBCChannelState.Init) {
            revert IBCErrors.ErrInvalidChannelState();
        }
        uint32 clientId = ensureConnectionState(channel.connectionId);
        IBCChannel memory expectedChannel = IBCChannel({
            state: IBCChannelState.TryOpen,
            ordering: channel.ordering,
            counterpartyChannelId: msg_.channelId,
            connectionId: getCounterpartyConnection(channel.connectionId),
            version: msg_.counterpartyVersion
        });
        if (
            !verifyChannelState(
                clientId,
                msg_.proofHeight,
                msg_.proofTry,
                msg_.counterpartyChannelId,
                expectedChannel
            )
        ) {
            revert IBCErrors.ErrInvalidProof();
        }
        channel.state = IBCChannelState.Open;
        channel.version = msg_.counterpartyVersion;
        channel.counterpartyChannelId = msg_.counterpartyChannelId;
        commitChannel(msg_.channelId, channel);
        IIBCModule(msg_.portId).onChanOpenAck(
            msg_.channelId,
            msg_.counterpartyChannelId,
            msg_.counterpartyVersion,
            msg_.relayer
        );
        emit IBCChannelLib.ChannelOpenAck(
            msg_.portId,
            msg_.channelId,
            msg_.counterpartyChannelId,
            channel.connectionId
        );
    }

    /**
     * @dev channelOpenConfirm is called by the counterparty module to close their end of the channel, since the other end has been closed.
     */
    function channelOpenConfirm(
        IBCMsgs.MsgChannelOpenConfirm calldata msg_
    ) external override {
        IBCChannel storage channel = channels[msg_.channelId];
        if (channel.state != IBCChannelState.TryOpen) {
            revert IBCErrors.ErrInvalidChannelState();
        }
        uint32 clientId = ensureConnectionState(channel.connectionId);
        IBCChannel memory expectedChannel = IBCChannel({
            state: IBCChannelState.Open,
            ordering: channel.ordering,
            counterpartyChannelId: msg_.channelId,
            connectionId: getCounterpartyConnection(channel.connectionId),
            version: channel.version
        });
        if (
            !verifyChannelState(
                clientId,
                msg_.proofHeight,
                msg_.proofAck,
                channel.counterpartyChannelId,
                expectedChannel
            )
        ) {
            revert IBCErrors.ErrInvalidProof();
        }
        channel.state = IBCChannelState.Open;
        commitChannel(msg_.channelId, channel);
        IIBCModule(msg_.portId).onChanOpenConfirm(msg_.channelId, msg_.relayer);
        emit IBCChannelLib.ChannelOpenConfirm(
            msg_.portId,
            msg_.channelId,
            channel.counterpartyChannelId,
            channel.connectionId
        );
    }

    /**
     * @dev channelCloseInit is called by either module to close their end of the channel. Once closed, channels cannot be reopened.
     */
    function channelCloseInit(
        IBCMsgs.MsgChannelCloseInit calldata msg_
    ) external override {
        IBCChannel storage channel = channels[msg_.channelId];
        if (channel.state != IBCChannelState.Open) {
            revert IBCErrors.ErrInvalidChannelState();
        }
        ensureConnectionState(channel.connectionId);
        channel.state = IBCChannelState.Closed;
        commitChannel(msg_.channelId, channel);
        IIBCModule(msg_.portId).onChanCloseInit(msg_.channelId, msg_.relayer);
        emit IBCChannelLib.ChannelCloseInit(msg_.portId, msg_.channelId);
    }

    /**
     * @dev channelCloseConfirm is called by the counterparty module to close their end of the
     * channel, since the other end has been closed.
     */
    function channelCloseConfirm(
        IBCMsgs.MsgChannelCloseConfirm calldata msg_
    ) external override {
        IBCChannel storage channel = channels[msg_.channelId];
        if (channel.state != IBCChannelState.Open) {
            revert IBCErrors.ErrInvalidChannelState();
        }
        uint32 clientId = ensureConnectionState(channel.connectionId);
        IBCChannel memory expectedChannel = IBCChannel({
            state: IBCChannelState.Closed,
            ordering: channel.ordering,
            counterpartyChannelId: msg_.channelId,
            connectionId: getCounterpartyConnection(channel.connectionId),
            version: channel.version
        });
        if (
            !verifyChannelState(
                clientId,
                msg_.proofHeight,
                msg_.proofInit,
                channel.counterpartyChannelId,
                expectedChannel
            )
        ) {
            revert IBCErrors.ErrInvalidProof();
        }
        channel.state = IBCChannelState.Closed;
        commitChannel(msg_.channelId, channel);
        IIBCModule(msg_.portId).onChanCloseConfirm(msg_.channelId, msg_.relayer);
        emit IBCChannelLib.ChannelCloseConfirm(msg_.portId, msg_.channelId);
    }

    function encodeChannel(
        IBCChannel memory channel
    ) internal pure returns (bytes32) {
        return keccak256(abi.encode(channel));
    }

    function commitChannel(
        uint32 channelId,
        IBCChannel storage channel
    ) internal {
        commitments[IBCCommitment.channelCommitmentKey(channelId)] =
            encodeChannel(channel);
    }

    function commitChannelCalldata(
        uint32 channelId,
        IBCChannel calldata channel
    ) internal {
        commitments[IBCCommitment.channelCommitmentKey(channelId)] =
            encodeChannelCalldata(channel);
    }

    function encodeChannelCalldata(
        IBCChannel calldata channel
    ) internal pure returns (bytes32) {
        return keccak256(abi.encode(channel));
    }

    function verifyChannelState(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        uint32 channelId,
        IBCChannel memory channel
    ) internal returns (bool) {
        return getClientInternal(clientId).verifyMembership(
            clientId,
            height,
            proof,
            abi.encodePacked(IBCCommitment.channelCommitmentKey(channelId)),
            abi.encodePacked(encodeChannel(channel))
        );
    }

    function getCounterpartyConnection(
        uint32 connectionId
    ) internal view returns (uint32) {
        return connections[connectionId].counterpartyConnectionId;
    }

    function generateChannelIdentifier() internal returns (uint32) {
        uint32 nextChannelSequence =
            uint32(uint256(commitments[nextChannelSequencePath]));
        commitments[nextChannelSequencePath] =
            bytes32(uint256(nextChannelSequence + 1));
        return nextChannelSequence;
    }

    function initializeChannelSequences(
        uint32 channelId
    ) internal {
        commitments[IBCCommitment.nextSequenceSendCommitmentKey(channelId)] =
            bytes32(uint256(1));
        commitments[IBCCommitment.nextSequenceRecvCommitmentKey(channelId)] =
            bytes32(uint256(1));
        commitments[IBCCommitment.nextSequenceAckCommitmentKey(channelId)] =
            bytes32(uint256(1));
    }
}
