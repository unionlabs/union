pragma solidity ^0.8.27;

import "solady/utils/LibString.sol";

import "../24-host/IBCStore.sol";
import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCCommitment.sol";
import "../04-channel/IIBCChannel.sol";
import "../05-port/IIBCModule.sol";
import "../../lib/Hex.sol";

library IBCChannelLib {
    event ChannelOpenInit(
        address portId,
        uint32 channelId,
        bytes counterpartyPortId,
        uint32 connectionId,
        string version
    );
    event ChannelOpenTry(
        address portId,
        uint32 channelId,
        bytes counterpartyPortId,
        uint32 counterpartyChannelId,
        uint32 connectionId,
        string counterpartyVersion
    );
    event ChannelOpenAck(
        address portId,
        uint32 channelId,
        bytes counterpartyPortId,
        uint32 counterpartyChannelId,
        uint32 connectionId
    );
    event ChannelOpenConfirm(
        address portId,
        uint32 channelId,
        bytes counterpartyPortId,
        uint32 counterpartyChannelId,
        uint32 connectionId
    );
    event ChannelCloseInit(
        address portId,
        uint32 channelId,
        bytes counterpartyPortId,
        uint32 counterpartyChannelId
    );
    event ChannelCloseConfirm(
        address portId,
        uint32 channelId,
        bytes counterpartyPortId,
        uint32 counterpartyChannelId
    );
}

/**
 * @dev IBCChannelHandshake is a contract that implements [ICS-4](https://github.com/cosmos/ibc/tree/main/spec/core/ics-004-channel-and-packet-semantics).
 */
abstract contract IBCChannelImpl is IBCStore, IIBCChannel {
    using LibString for *;

    /**
     * @dev channelOpenInit is called by a module to initiate a channel opening handshake with a module on another chain.
     */
    function channelOpenInit(
        IBCMsgs.MsgChannelOpenInit calldata msg_
    ) external override returns (uint32) {
        ensureConnectionState(msg_.connectionId);
        uint32 channelId = generateChannelIdentifier();
        IBCChannel storage channel = channels[channelId];
        channel.state = IBCChannelState.Init;
        channel.connectionId = msg_.connectionId;
        channel.version = msg_.version;
        channel.counterpartyPortId = msg_.counterpartyPortId;
        commitChannel(channelId, channel);
        claimChannel(msg_.portId, channelId);
        IIBCModule(msg_.portId).onChanOpenInit(
            msg_.connectionId, channelId, msg_.version, msg_.relayer
        );
        emit IBCChannelLib.ChannelOpenInit(
            msg_.portId,
            channelId,
            channel.counterpartyPortId,
            msg_.connectionId,
            msg_.version
        );
        return channelId;
    }

    /**
     * @dev channelOpenTry is called by a module to accept the first step of a channel opening handshake initiated by a module on another chain.
     */
    function channelOpenTry(
        IBCMsgs.MsgChannelOpenTry calldata msg_
    ) external override returns (uint32) {
        if (msg_.channel.state != IBCChannelState.TryOpen) {
            revert IBCErrors.ErrInvalidChannelState();
        }
        uint32 clientId = ensureConnectionState(msg_.channel.connectionId);
        IBCChannel memory expectedChannel = IBCChannel({
            state: IBCChannelState.Init,
            counterpartyChannelId: 0,
            connectionId: getCounterpartyConnection(msg_.channel.connectionId),
            counterpartyPortId: abi.encodePacked(msg_.portId),
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
        commitChannelCalldata(channelId, msg_.channel);
        claimChannel(msg_.portId, channelId);
        IIBCModule(msg_.portId).onChanOpenTry(
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
            msg_.channel.counterpartyPortId,
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
        address portId = channelOwner[msg_.channelId];
        IBCChannel memory expectedChannel = IBCChannel({
            state: IBCChannelState.TryOpen,
            counterpartyChannelId: msg_.channelId,
            connectionId: getCounterpartyConnection(channel.connectionId),
            counterpartyPortId: abi.encodePacked(portId),
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
        IIBCModule(portId).onChanOpenAck(
            msg_.channelId,
            msg_.counterpartyChannelId,
            msg_.counterpartyVersion,
            msg_.relayer
        );
        emit IBCChannelLib.ChannelOpenAck(
            portId,
            msg_.channelId,
            channel.counterpartyPortId,
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
        address portId = channelOwner[msg_.channelId];
        IBCChannel memory expectedChannel = IBCChannel({
            state: IBCChannelState.Open,
            counterpartyChannelId: msg_.channelId,
            connectionId: getCounterpartyConnection(channel.connectionId),
            counterpartyPortId: abi.encodePacked(portId),
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
        IIBCModule(portId).onChanOpenConfirm(msg_.channelId, msg_.relayer);
        emit IBCChannelLib.ChannelOpenConfirm(
            portId,
            msg_.channelId,
            channel.counterpartyPortId,
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
        address portId = channelOwner[msg_.channelId];
        IIBCModule(portId).onChanCloseInit(msg_.channelId, msg_.relayer);
        emit IBCChannelLib.ChannelCloseInit(
            portId,
            msg_.channelId,
            channel.counterpartyPortId,
            channel.counterpartyChannelId
        );
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
        address portId = channelOwner[msg_.channelId];
        IBCChannel memory expectedChannel = IBCChannel({
            state: IBCChannelState.Closed,
            counterpartyChannelId: msg_.channelId,
            connectionId: getCounterpartyConnection(channel.connectionId),
            counterpartyPortId: abi.encodePacked(portId),
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
        IIBCModule(portId).onChanCloseConfirm(msg_.channelId, msg_.relayer);
        emit IBCChannelLib.ChannelCloseConfirm(
            portId,
            msg_.channelId,
            channel.counterpartyPortId,
            channel.counterpartyChannelId
        );
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
}
