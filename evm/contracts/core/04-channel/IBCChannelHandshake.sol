pragma solidity ^0.8.23;

import "@openzeppelin/utils/Strings.sol";
import "../../proto/ibc/core/channel/v1/channel.sol";
import "../25-handler/IBCMsgs.sol";
import "../02-client/IBCHeight.sol";
import "../24-host/IBCCommitment.sol";
import "../04-channel/IIBCChannel.sol";
import "../05-port/ModuleManager.sol";
import "../05-port/IIBCModule.sol";

library IBCChannelLib {
    event ChannelOpenInit(
        string portId,
        string channelId,
        string counterpartyPortId,
        string connectionId,
        string version
    );
    event ChannelOpenTry(
        string portId,
        string channelId,
        string counterpartyPortId,
        string counterpartyChannelId,
        string connectionId,
        string version
    );
    event ChannelOpenAck(
        string portId,
        string channelId,
        string counterpartyPortId,
        string counterpartyChannelId,
        string connectionId
    );
    event ChannelOpenConfirm(
        string portId,
        string channelId,
        string counterpartyPortId,
        string counterpartyChannelId,
        string connectionId
    );
    event ChannelCloseInit(string channelId, string portId);
    event ChannelCloseConfirm(string channelId, string portId);

    error ErrConnNotSingleHop();
    error ErrConnNotSingleVersion();
    error ErrInvalidConnectionState();
    error ErrUnsupportedFeature();
    error ErrInvalidChannelState();
    error ErrCounterpartyChannelNotEmpty();
    error ErrInvalidProof();

    string public constant ORDER_ORDERED = "ORDER_ORDERED";
    string public constant ORDER_UNORDERED = "ORDER_UNORDERED";
    string public constant ORDER_INVALID = "_ORDER_INVALID_";

    function verifySupportedFeature(
        IbcCoreConnectionV1Version.Data memory version,
        string memory feature
    ) internal pure returns (bool) {
        bytes32 h = keccak256(bytes(feature));
        for (uint256 i; i < version.features.length; i++) {
            if (keccak256(bytes(version.features[i])) == h) {
                return true;
            }
        }
        return false;
    }

    function toString(IbcCoreChannelV1GlobalEnums.Order ordering)
        internal
        pure
        returns (string memory)
    {
        if (ordering == IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED) {
            return ORDER_UNORDERED;
        } else if (ordering == IbcCoreChannelV1GlobalEnums.Order.ORDER_ORDERED)
        {
            return ORDER_ORDERED;
        } else {
            return ORDER_INVALID;
        }
    }
}

/**
 * @dev IBCChannelHandshake is a contract that implements [ICS-4](https://github.com/cosmos/ibc/tree/main/spec/core/ics-004-channel-and-packet-semantics).
 */
contract IBCChannelHandshake is ModuleManager, IIBCChannelHandshake {
    using IBCHeight for IbcCoreClientV1Height.Data;

    /* Handshake functions */

    /**
     * @dev channelOpenInit is called by a module to initiate a channel opening handshake with a module on another chain.
     */
    function channelOpenInit(IBCMsgs.MsgChannelOpenInit calldata msg_)
        external
        override
        returns (string memory)
    {
        (string memory connectionId,) = ensureConnectionFeature(
            msg_.channel.connection_hops, msg_.channel.ordering
        );
        if (msg_.channel.state != IbcCoreChannelV1GlobalEnums.State.STATE_INIT)
        {
            revert IBCChannelLib.ErrInvalidChannelState();
        }
        if (bytes(msg_.channel.counterparty.channel_id).length != 0) {
            revert IBCChannelLib.ErrCounterpartyChannelNotEmpty();
        }

        string memory channelId = generateChannelIdentifier();
        channels[msg_.portId][channelId] = msg_.channel;

        commitments[IBCCommitment.nextSequenceSendCommitmentKey(
            msg_.portId, channelId
        )] = bytes32(uint256(1));
        commitments[IBCCommitment.nextSequenceRecvCommitmentKey(
            msg_.portId, channelId
        )] = bytes32(uint256(1));

        commitments[IBCCommitment.nextSequenceAckCommitmentKey(
            msg_.portId, channelId
        )] = bytes32(uint256(1));

        updateChannelCommitment(msg_.portId, channelId);

        IIBCModule module = lookupModuleByPort(msg_.portId);

        claimCapability(
            channelCapabilityPath(msg_.portId, channelId), address(module)
        );

        module.onChanOpenInit(
            msg_.channel.ordering,
            msg_.channel.connection_hops,
            msg_.portId,
            channelId,
            msg_.channel.counterparty,
            msg_.channel.version,
            msg_.relayer
        );

        emit IBCChannelLib.ChannelOpenInit(
            msg_.portId,
            channelId,
            msg_.channel.counterparty.port_id,
            connectionId,
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
        returns (string memory)
    {
        (
            string memory connectionId,
            IbcCoreConnectionV1ConnectionEnd.Data memory connection
        ) = ensureConnectionFeature(
            msg_.channel.connection_hops, msg_.channel.ordering
        );
        if (
            msg_.channel.state
                != IbcCoreChannelV1GlobalEnums.State.STATE_TRYOPEN
        ) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }

        IbcCoreChannelV1Counterparty.Data memory expectedCounterparty =
        IbcCoreChannelV1Counterparty.Data({port_id: msg_.portId, channel_id: ""});
        IbcCoreChannelV1Channel.Data memory expectedChannel =
        IbcCoreChannelV1Channel.Data({
            state: IbcCoreChannelV1GlobalEnums.State.STATE_INIT,
            ordering: msg_.channel.ordering,
            counterparty: expectedCounterparty,
            connection_hops: getCounterpartyHops(msg_.channel.connection_hops[0]),
            version: msg_.counterpartyVersion
        });

        if (
            !verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofInit,
                msg_.channel.counterparty.port_id,
                msg_.channel.counterparty.channel_id,
                IbcCoreChannelV1Channel.encode(expectedChannel)
            )
        ) {
            revert IBCChannelLib.ErrInvalidProof();
        }

        string memory channelId = generateChannelIdentifier();

        emit IBCChannelLib.ChannelOpenTry(
            msg_.portId,
            channelId,
            msg_.channel.counterparty.port_id,
            msg_.channel.counterparty.channel_id,
            connectionId,
            msg_.counterpartyVersion
        );

        channels[msg_.portId][channelId] = msg_.channel;
        commitments[IBCCommitment.nextSequenceSendCommitmentKey(
            msg_.portId, channelId
        )] = bytes32(uint256(1));
        commitments[IBCCommitment.nextSequenceRecvCommitmentKey(
            msg_.portId, channelId
        )] = bytes32(uint256(1));
        commitments[IBCCommitment.nextSequenceAckCommitmentKey(
            msg_.portId, channelId
        )] = bytes32(uint256(1));
        updateChannelCommitment(msg_.portId, channelId);

        IIBCModule module = lookupModuleByPort(msg_.portId);

        claimCapability(
            channelCapabilityPath(msg_.portId, channelId), address(module)
        );

        module.onChanOpenTry(
            msg_.channel.ordering,
            msg_.channel.connection_hops,
            msg_.portId,
            channelId,
            msg_.channel.counterparty,
            msg_.channel.version,
            msg_.counterpartyVersion,
            msg_.relayer
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
        IbcCoreChannelV1Channel.Data storage channel =
            channels[msg_.portId][msg_.channelId];
        if (channel.state != IbcCoreChannelV1GlobalEnums.State.STATE_INIT) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }

        emit IBCChannelLib.ChannelOpenAck(
            msg_.portId,
            msg_.channelId,
            channel.counterparty.port_id,
            // haven't been saved yet, but we have to yield the even early to avoid overflowing the stack
            msg_.counterpartyChannelId,
            channel.connection_hops[0]
        );

        IbcCoreConnectionV1ConnectionEnd.Data memory connection =
            ensureConnectionState(channel.connection_hops[0]);

        IbcCoreChannelV1Counterparty.Data memory expectedCounterparty =
        IbcCoreChannelV1Counterparty.Data({
            port_id: msg_.portId,
            channel_id: msg_.channelId
        });
        IbcCoreChannelV1Channel.Data memory expectedChannel =
        IbcCoreChannelV1Channel.Data({
            state: IbcCoreChannelV1GlobalEnums.State.STATE_TRYOPEN,
            ordering: channel.ordering,
            counterparty: expectedCounterparty,
            connection_hops: getCounterpartyHops(channel.connection_hops[0]),
            version: msg_.counterpartyVersion
        });

        if (
            !verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofTry,
                channel.counterparty.port_id,
                msg_.counterpartyChannelId,
                IbcCoreChannelV1Channel.encode(expectedChannel)
            )
        ) {
            revert IBCChannelLib.ErrInvalidProof();
        }

        channel.state = IbcCoreChannelV1GlobalEnums.State.STATE_OPEN;
        channel.version = msg_.counterpartyVersion;
        channel.counterparty.channel_id = msg_.counterpartyChannelId;
        updateChannelCommitment(msg_.portId, msg_.channelId);

        lookupModuleByPort(msg_.portId).onChanOpenAck(
            msg_.portId,
            msg_.channelId,
            msg_.counterpartyChannelId,
            msg_.counterpartyVersion,
            msg_.relayer
        );
    }

    /**
     * @dev channelOpenConfirm is called by the counterparty module to close their end of the channel, since the other end has been closed.
     */
    function channelOpenConfirm(IBCMsgs.MsgChannelOpenConfirm calldata msg_)
        external
        override
    {
        IbcCoreChannelV1Channel.Data storage channel =
            channels[msg_.portId][msg_.channelId];
        if (channel.state != IbcCoreChannelV1GlobalEnums.State.STATE_TRYOPEN) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }

        emit IBCChannelLib.ChannelOpenConfirm(
            msg_.portId,
            msg_.channelId,
            channel.counterparty.port_id,
            channel.counterparty.channel_id,
            channel.connection_hops[0]
        );

        IbcCoreConnectionV1ConnectionEnd.Data memory connection =
            ensureConnectionState(channel.connection_hops[0]);

        IbcCoreChannelV1Counterparty.Data memory expectedCounterparty =
        IbcCoreChannelV1Counterparty.Data({
            port_id: msg_.portId,
            channel_id: msg_.channelId
        });
        IbcCoreChannelV1Channel.Data memory expectedChannel =
        IbcCoreChannelV1Channel.Data({
            state: IbcCoreChannelV1GlobalEnums.State.STATE_OPEN,
            ordering: channel.ordering,
            counterparty: expectedCounterparty,
            connection_hops: getCounterpartyHops(channel.connection_hops[0]),
            version: channel.version
        });

        if (
            !verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofAck,
                channel.counterparty.port_id,
                channel.counterparty.channel_id,
                IbcCoreChannelV1Channel.encode(expectedChannel)
            )
        ) {
            revert IBCChannelLib.ErrInvalidProof();
        }

        channel.state = IbcCoreChannelV1GlobalEnums.State.STATE_OPEN;
        updateChannelCommitment(msg_.portId, msg_.channelId);

        lookupModuleByPort(msg_.portId).onChanOpenConfirm(
            msg_.portId, msg_.channelId, msg_.relayer
        );
    }

    /**
     * @dev channelCloseInit is called by either module to close their end of the channel. Once closed, channels cannot be reopened.
     */
    function channelCloseInit(IBCMsgs.MsgChannelCloseInit calldata msg_)
        external
        override
    {
        IbcCoreChannelV1Channel.Data storage channel =
            channels[msg_.portId][msg_.channelId];
        if (channel.state != IbcCoreChannelV1GlobalEnums.State.STATE_OPEN) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }

        ensureConnectionState(channel.connection_hops[0]);

        channel.state = IbcCoreChannelV1GlobalEnums.State.STATE_CLOSED;
        updateChannelCommitment(msg_.portId, msg_.channelId);

        lookupModuleByPort(msg_.portId).onChanCloseInit(
            msg_.portId, msg_.channelId, msg_.relayer
        );

        emit IBCChannelLib.ChannelCloseInit(msg_.channelId, msg_.portId);
    }

    /**
     * @dev channelCloseConfirm is called by the counterparty module to close their end of the
     * channel, since the other end has been closed.
     */
    function channelCloseConfirm(IBCMsgs.MsgChannelCloseConfirm calldata msg_)
        external
        override
    {
        IbcCoreChannelV1Channel.Data storage channel =
            channels[msg_.portId][msg_.channelId];
        if (channel.state != IbcCoreChannelV1GlobalEnums.State.STATE_OPEN) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }

        IbcCoreConnectionV1ConnectionEnd.Data memory connection =
            ensureConnectionState(channel.connection_hops[0]);

        IbcCoreChannelV1Counterparty.Data memory expectedCounterparty =
        IbcCoreChannelV1Counterparty.Data({
            port_id: msg_.portId,
            channel_id: msg_.channelId
        });
        IbcCoreChannelV1Channel.Data memory expectedChannel =
        IbcCoreChannelV1Channel.Data({
            state: IbcCoreChannelV1GlobalEnums.State.STATE_CLOSED,
            ordering: channel.ordering,
            counterparty: expectedCounterparty,
            connection_hops: getCounterpartyHops(channel.connection_hops[0]),
            version: channel.version
        });

        if (
            !verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofInit,
                channel.counterparty.port_id,
                channel.counterparty.channel_id,
                IbcCoreChannelV1Channel.encode(expectedChannel)
            )
        ) {
            revert IBCChannelLib.ErrInvalidProof();
        }

        channel.state = IbcCoreChannelV1GlobalEnums.State.STATE_CLOSED;
        updateChannelCommitment(msg_.portId, msg_.channelId);

        lookupModuleByPort(msg_.portId).onChanCloseConfirm(
            msg_.portId, msg_.channelId, msg_.relayer
        );

        emit IBCChannelLib.ChannelCloseConfirm(msg_.channelId, msg_.portId);
    }

    function updateChannelCommitment(
        string memory portId,
        string memory channelId
    ) private {
        commitments[IBCCommitment.channelCommitmentKey(portId, channelId)] =
        keccak256(IbcCoreChannelV1Channel.encode(channels[portId][channelId]));
    }

    /* Verification functions */

    function verifyChannelState(
        IbcCoreConnectionV1ConnectionEnd.Data memory connection,
        IbcCoreClientV1Height.Data calldata height,
        bytes calldata proof,
        string memory portId,
        string memory channelId,
        bytes memory channelBytes
    ) private returns (bool) {
        return getClient(connection.client_id).verifyMembership(
            connection.client_id,
            height,
            0,
            0,
            proof,
            connection.counterparty.prefix.key_prefix,
            IBCCommitment.channelPath(portId, channelId),
            channelBytes
        );
    }

    /* Internal functions */

    function getCounterpartyHops(string memory connectionId)
        internal
        view
        returns (string[] memory hops)
    {
        hops = new string[](1);
        hops[0] = connections[connectionId].counterparty.connection_id;
        return hops;
    }

    function generateChannelIdentifier() internal returns (string memory) {
        uint256 nextChannelSequence =
            uint256(commitments[nextChannelSequencePath]);

        string memory identifier = string(
            abi.encodePacked("channel-", Strings.toString(nextChannelSequence))
        );
        commitments[nextChannelSequencePath] = bytes32(nextChannelSequence + 1);
        return identifier;
    }

    function ensureConnectionState(string memory connectionId)
        internal
        view
        returns (IbcCoreConnectionV1ConnectionEnd.Data memory)
    {
        IbcCoreConnectionV1ConnectionEnd.Data memory connection =
            connections[connectionId];
        if (connection.state != IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN)
        {
            revert IBCChannelLib.ErrInvalidConnectionState();
        }
        return connection;
    }

    function ensureConnectionFeature(
        string[] calldata connectionHops,
        IbcCoreChannelV1GlobalEnums.Order ordering
    )
        internal
        view
        returns (string memory, IbcCoreConnectionV1ConnectionEnd.Data memory)
    {
        if (connectionHops.length != 1) {
            revert IBCChannelLib.ErrConnNotSingleHop();
        }
        string memory connectionId = connectionHops[0];
        IbcCoreConnectionV1ConnectionEnd.Data memory connection =
            ensureConnectionState(connectionId);
        if (connection.versions.length != 1) {
            revert IBCChannelLib.ErrConnNotSingleVersion();
        }
        if (
            !IBCChannelLib.verifySupportedFeature(
                connection.versions[0], IBCChannelLib.toString(ordering)
            )
        ) {
            revert IBCChannelLib.ErrUnsupportedFeature();
        }
        return (connectionId, connection);
    }
}
