pragma solidity ^0.8.23;

import "../24-host/IBCStore.sol";
import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCCommitment.sol";
import "../04-channel/IIBCChannel.sol";
import "../05-port/IIBCModule.sol";

library IBCChannelLib {
    event ChannelOpenInit(
        address portId,
        bytes32 normalizedPortId,
        uint32 channelId,
        bytes32 counterpartyPortId,
        uint32 connectionId,
        bytes32 version
    );
    event ChannelOpenTry(
        address portId,
        bytes32 normalizedPortId,
        uint32 channelId,
        bytes32 counterpartyPortId,
        uint32 counterpartyChannelId,
        uint32 connectionId,
        bytes32 version
    );
    event ChannelOpenAck(
        address portId,
        bytes32 normalizedPortId,
        uint32 channelId,
        bytes32 counterpartyPortId,
        uint32 counterpartyChannelId,
        uint32 connectionId
    );
    event ChannelOpenConfirm(
        address portId,
        bytes32 normalizedPortId,
        uint32 channelId,
        bytes32 counterpartyPortId,
        uint32 counterpartyChannelId,
        uint32 connectionId
    );
    event ChannelCloseInit(
        address portId, bytes32 normalizedPortId, uint32 channelId
    );
    event ChannelCloseConfirm(
        address portId, bytes32 normalizedPortId, uint32 channelId
    );

    error ErrPortIdMustBeLowercase();
    error ErrConnNotSingleHop();
    error ErrConnNotSingleVersion();
    error ErrInvalidConnectionState();
    error ErrUnsupportedFeature();
    error ErrInvalidChannelState();
    error ErrCounterpartyChannelNotEmpty();
    error ErrInvalidProof();
    error ErrInvalidChannelOrdering();
}

/**
 * @dev IBCChannelHandshake is a contract that implements [ICS-4](https://github.com/cosmos/ibc/tree/main/spec/core/ics-004-channel-and-packet-semantics).
 */
abstract contract IBCChannelImpl is IBCStore, IIBCChannel {
    function normalizePortId(address portId) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(portId));
    }

    /**
     * @dev channelOpenInit is called by a module to initiate a channel opening handshake with a module on another chain.
     */
    function channelOpenInit(IBCMsgs.MsgChannelOpenInit calldata msg_)
        external
        override
        returns (uint32)
    {
        if (msg_.channel.state != IBCChannelState.Init) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }
        if (
            msg_.channel.ordering != IBCChannelOrder.Unordered
                && msg_.channel.ordering != IBCChannelOrder.Ordered
        ) {
            revert IBCChannelLib.ErrInvalidChannelOrdering();
        }
        uint32 channelId = generateChannelIdentifier();
        channels[channelId] = msg_.channel;
        initializeChannelSequences(channelId);
        commitChannelCalldata(channelId, msg_.channel);
        claimChannel(msg_.portId, channelId);
        IIBCModule(msg_.portId).onChanOpenInit(
            msg_.channel.ordering,
            msg_.channel.connectionId,
            channelId,
            msg_.channel.counterparty,
            msg_.channel.version,
            msg_.relayer
        );
        emit IBCChannelLib.ChannelOpenInit(
            msg_.portId,
            normalizePortId(msg_.portId),
            channelId,
            msg_.channel.counterparty.portId,
            msg_.channel.connectionId,
            msg_.channel.version
        );
        return channelId;
    }

    /**
     * @dev channelOpenTry is called by a module to accept the first step of a channel opening handshake initiated by a module on another chain.
     */
    function channelOpenTry(IBCMsgs.MsgChannelOpenTry calldata msg_)
        external
        override
        returns (uint32)
    {
        if (
            msg_.channel.ordering != IBCChannelOrder.Unordered
                && msg_.channel.ordering != IBCChannelOrder.Ordered
        ) {
            revert IBCChannelLib.ErrInvalidChannelOrdering();
        }
        if (msg_.channel.state != IBCChannelState.TryOpen) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }
        IBCConnection storage connection =
            ensureConnectionState(msg_.channel.connectionId);
        bytes32 normalizedPortId = normalizePortId(msg_.portId);
        IBCChannelCounterparty memory expectedCounterparty =
            IBCChannelCounterparty({portId: normalizedPortId, channelId: 0});
        IBCChannel memory expectedChannel = IBCChannel({
            state: IBCChannelState.Init,
            ordering: msg_.channel.ordering,
            counterparty: expectedCounterparty,
            connectionId: getCounterpartyConnection(msg_.channel.connectionId),
            version: msg_.counterpartyVersion
        });
        if (
            !verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofInit,
                msg_.channel.counterparty.channelId,
                expectedChannel
            )
        ) {
            revert IBCChannelLib.ErrInvalidProof();
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
            msg_.channel.counterparty,
            msg_.channel.version,
            msg_.counterpartyVersion,
            msg_.relayer
        );
        emit IBCChannelLib.ChannelOpenTry(
            msg_.portId,
            normalizePortId(msg_.portId),
            channelId,
            msg_.channel.counterparty.portId,
            msg_.channel.counterparty.channelId,
            msg_.channel.connectionId,
            msg_.counterpartyVersion
        );
        return channelId;
    }

    /**
     * @dev channelOpenAck is called by the handshake-originating module to acknowledge the acceptance of the initial request by the counterparty module on the other chain.
     */
    function channelOpenAck(IBCMsgs.MsgChannelOpenAck calldata msg_)
        external
        override
    {
        IBCChannel storage channel = channels[msg_.channelId];
        if (channel.state != IBCChannelState.Init) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }
        IBCConnection storage connection =
            ensureConnectionState(channel.connectionId);
        bytes32 normalizedPortId = normalizePortId(msg_.portId);
        IBCChannelCounterparty memory expectedCounterparty =
        IBCChannelCounterparty({
            portId: normalizedPortId,
            channelId: msg_.channelId
        });
        IBCChannel memory expectedChannel = IBCChannel({
            state: IBCChannelState.TryOpen,
            ordering: channel.ordering,
            counterparty: expectedCounterparty,
            connectionId: getCounterpartyConnection(channel.connectionId),
            version: msg_.counterpartyVersion
        });
        if (
            !verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofTry,
                msg_.counterpartyChannelId,
                expectedChannel
            )
        ) {
            revert IBCChannelLib.ErrInvalidProof();
        }
        channel.state = IBCChannelState.Open;
        channel.version = msg_.counterpartyVersion;
        channel.counterparty.channelId = msg_.counterpartyChannelId;
        commitChannel(msg_.channelId, channel);
        IIBCModule(msg_.portId).onChanOpenAck(
            msg_.channelId,
            msg_.counterpartyChannelId,
            msg_.counterpartyVersion,
            msg_.relayer
        );
        emit IBCChannelLib.ChannelOpenAck(
            msg_.portId,
            normalizedPortId,
            msg_.channelId,
            channel.counterparty.portId,
            msg_.counterpartyChannelId,
            channel.connectionId
        );
    }

    /**
     * @dev channelOpenConfirm is called by the counterparty module to close their end of the channel, since the other end has been closed.
     */
    function channelOpenConfirm(IBCMsgs.MsgChannelOpenConfirm calldata msg_)
        external
        override
    {
        IBCChannel storage channel = channels[msg_.channelId];
        if (channel.state != IBCChannelState.TryOpen) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }
        IBCConnection storage connection =
            ensureConnectionState(channel.connectionId);
        bytes32 normalizedPortId = normalizePortId(msg_.portId);
        IBCChannelCounterparty memory expectedCounterparty =
        IBCChannelCounterparty({
            portId: normalizedPortId,
            channelId: msg_.channelId
        });
        IBCChannel memory expectedChannel = IBCChannel({
            state: IBCChannelState.Open,
            ordering: channel.ordering,
            counterparty: expectedCounterparty,
            connectionId: getCounterpartyConnection(channel.connectionId),
            version: channel.version
        });
        if (
            !verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofAck,
                channel.counterparty.channelId,
                expectedChannel
            )
        ) {
            revert IBCChannelLib.ErrInvalidProof();
        }
        channel.state = IBCChannelState.Open;
        commitChannel(msg_.channelId, channel);
        IIBCModule(msg_.portId).onChanOpenConfirm(msg_.channelId, msg_.relayer);
        emit IBCChannelLib.ChannelOpenConfirm(
            msg_.portId,
            normalizedPortId,
            msg_.channelId,
            channel.counterparty.portId,
            channel.counterparty.channelId,
            channel.connectionId
        );
    }

    /**
     * @dev channelCloseInit is called by either module to close their end of the channel. Once closed, channels cannot be reopened.
     */
    function channelCloseInit(IBCMsgs.MsgChannelCloseInit calldata msg_)
        external
        override
    {
        IBCChannel storage channel = channels[msg_.channelId];
        if (channel.state != IBCChannelState.Open) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }
        ensureConnectionState(channel.connectionId);
        channel.state = IBCChannelState.Closed;
        commitChannel(msg_.channelId, channel);
        IIBCModule(msg_.portId).onChanCloseInit(msg_.channelId, msg_.relayer);
        emit IBCChannelLib.ChannelCloseInit(
            msg_.portId, normalizePortId(msg_.portId), msg_.channelId
        );
    }

    /**
     * @dev channelCloseConfirm is called by the counterparty module to close their end of the
     * channel, since the other end has been closed.
     */
    function channelCloseConfirm(IBCMsgs.MsgChannelCloseConfirm calldata msg_)
        external
        override
    {
        IBCChannel storage channel = channels[msg_.channelId];
        if (channel.state != IBCChannelState.Open) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }
        IBCConnection storage connection =
            ensureConnectionState(channel.connectionId);
        bytes32 normalizedPortId = normalizePortId(msg_.portId);
        IBCChannelCounterparty memory expectedCounterparty =
        IBCChannelCounterparty({
            portId: normalizedPortId,
            channelId: msg_.channelId
        });
        IBCChannel memory expectedChannel = IBCChannel({
            state: IBCChannelState.Closed,
            ordering: channel.ordering,
            counterparty: expectedCounterparty,
            connectionId: getCounterpartyConnection(channel.connectionId),
            version: channel.version
        });
        if (
            !verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofInit,
                channel.counterparty.channelId,
                expectedChannel
            )
        ) {
            revert IBCChannelLib.ErrInvalidProof();
        }
        channel.state = IBCChannelState.Closed;
        commitChannel(msg_.channelId, channel);
        IIBCModule(msg_.portId).onChanCloseConfirm(msg_.channelId, msg_.relayer);
        emit IBCChannelLib.ChannelCloseConfirm(
            msg_.portId, normalizedPortId, msg_.channelId
        );
    }

    function encodeChannel(IBCChannel memory channel)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(abi.encode(channel));
    }

    function commitChannel(
        uint32 channelId,
        IBCChannel storage channel
    ) internal {
        commitments[IBCCommitment.channelCommitmentKey(channelId)] =
            encodeChannel(channel);
    }

    function encodeChannelCalldata(IBCChannel calldata channel)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(abi.encode(channel));
    }

    function commitChannelCalldata(
        uint32 channelId,
        IBCChannel calldata channel
    ) internal {
        commitments[IBCCommitment.channelCommitmentKey(channelId)] =
            encodeChannelCalldata(channel);
    }

    function verifyChannelState(
        IBCConnection storage connection,
        uint64 height,
        bytes calldata proof,
        uint32 channelId,
        IBCChannel memory channel
    ) internal returns (bool) {
        return getClientInternal(connection.clientId).verifyMembership(
            connection.clientId,
            height,
            0,
            0,
            proof,
            abi.encodePacked(connection.counterparty.merklePrefix),
            abi.encodePacked(IBCCommitment.channelCommitmentKey(channelId)),
            abi.encodePacked(encodeChannel(channel))
        );
    }

    function getCounterpartyConnection(uint32 connectionId)
        internal
        view
        returns (uint32)
    {
        return connections[connectionId].counterparty.connectionId;
    }

    function generateChannelIdentifier() internal returns (uint32) {
        uint32 nextChannelSequence =
            uint32(uint256(commitments[nextChannelSequencePath]));
        commitments[nextChannelSequencePath] =
            bytes32(uint256(nextChannelSequence + 1));
        return nextChannelSequence;
    }

    function initializeChannelSequences(uint32 channelId) internal {
        commitments[IBCCommitment.nextSequenceSendCommitmentKey(channelId)] =
            bytes32(uint256(1));
        commitments[IBCCommitment.nextSequenceRecvCommitmentKey(channelId)] =
            bytes32(uint256(1));
        commitments[IBCCommitment.nextSequenceAckCommitmentKey(channelId)] =
            bytes32(uint256(1));
    }
}
