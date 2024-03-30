pragma solidity ^0.8.23;

import "@openzeppelin/utils/Strings.sol";
import "../../proto/ibc/core/channel/v1/channel.sol";
import "../../proto/ibc/core/connection/v1/connection.sol";
import "../25-handler/IBCMsgs.sol";
import "../02-client/IBCHeight.sol";
import "../24-host/IBCCommitment.sol";
import "./IIBCChannel.sol";
import "../05-port/ModuleManager.sol";
import "../05-port/IIBCModule.sol";

library IBCChannelLib {
    event ChannelOpenInit(
        ChannelId channelId,
        string connectionId,
        string portId,
        string counterpartyPortId
    );
    event ChannelOpenTry(
        ChannelId channelId,
        string connectionId,
        string portId,
        string counterpartyPortId,
        string version
    );
    event ChannelOpenAck(ChannelId channelId, string portId);
    event ChannelOpenConfirm(ChannelId channelId, string portId);
    event ChannelCloseInit(ChannelId channelId, string portId);
    event ChannelCloseConfirm(ChannelId channelId, string portId);

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
        for (uint256 i = 0; i < version.features.length; i++) {
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
    using {parseChannelIdCalldata, parseChannelIdMemory} for string;

    /* Handshake functions */

    /**
     * @dev channelOpenInit is called by a module to initiate a channel opening handshake with a module on another chain.
     */
    function channelOpenInit(IBCMsgs.MsgChannelOpenInit calldata msg_)
        external
        override
        returns (ChannelId)
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

        ChannelId channelId = generateChannelIdentifier();
        channels[msg_.portId][channelId] =
            IBCChannelTypes.optimizedChannel(msg_.channel);
        nextSequenceSends[msg_.portId][channelId] = 1;
        nextSequenceRecvs[msg_.portId][channelId] = 1;
        nextSequenceAcks[msg_.portId][channelId] = 1;
        updateChannelCommitment(msg_.portId, channelId);

        IIBCModule module = lookupModuleByPort(msg_.portId);

        claimCapability(channelId, address(module));

        module.onChanOpenInit(
            msg_.channel.ordering,
            msg_.channel.connection_hops,
            msg_.portId,
            channelId,
            IBCChannelTypes.optimizedCounterparty(msg_.channel.counterparty),
            msg_.channel.version
        );

        emit IBCChannelLib.ChannelOpenInit(
            channelId,
            connectionId,
            msg_.portId,
            msg_.channel.counterparty.port_id
        );

        return channelId;
    }

    /**
     * @dev channelOpenTry is called by a module to accept the first step of a channel opening handshake initiated by a module on another chain.
     */
    function channelOpenTry(IBCMsgs.MsgChannelOpenTry calldata msg_)
        external
        override
        returns (ChannelId)
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

        // NOTE: We use the proto types here, as we need to verify against the counterparty storage which uses proto.
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

        ChannelId counterpartyChannelId =
            msg_.channel.counterparty.channel_id.parseChannelIdCalldata();

        if (
            !verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofInit,
                msg_.channel.counterparty.port_id,
                counterpartyChannelId,
                IbcCoreChannelV1Channel.encode(expectedChannel)
            )
        ) {
            revert IBCChannelLib.ErrInvalidProof();
        }

        ChannelId channelId = generateChannelIdentifier();
        channels[msg_.portId][channelId] =
            IBCChannelTypes.optimizedChannel(msg_.channel);
        nextSequenceSends[msg_.portId][channelId] = 1;
        nextSequenceRecvs[msg_.portId][channelId] = 1;
        nextSequenceAcks[msg_.portId][channelId] = 1;
        updateChannelCommitment(msg_.portId, channelId);

        IIBCModule module = lookupModuleByPort(msg_.portId);

        claimCapability(channelId, address(module));

        module.onChanOpenTry(
            msg_.channel.ordering,
            msg_.channel.connection_hops,
            msg_.portId,
            channelId,
            IBCChannelTypes.optimizedCounterparty(msg_.channel.counterparty),
            msg_.channel.version,
            msg_.counterpartyVersion
        );

        emit IBCChannelLib.ChannelOpenTry(
            channelId,
            connectionId,
            msg_.portId,
            msg_.channel.counterparty.port_id,
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
        ChannelId channelId = msg_.channelId.parseChannelIdCalldata();
        ChannelId counterpartyChannelId =
            msg_.counterpartyChannelId.parseChannelIdCalldata();

        IBCChannelTypes.Channel storage channel =
            channels[msg_.portId][channelId];

        if (channel.state != IbcCoreChannelV1GlobalEnums.State.STATE_INIT) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }

        IbcCoreConnectionV1ConnectionEnd.Data memory connection =
            ensureConnectionState(channel.connectionHops[0]);

        // NOTE: We use the proto types here, as we need to verify against the counterparty storage which uses proto.
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
            connection_hops: getCounterpartyHops(channel.connectionHops[0]),
            version: msg_.counterpartyVersion
        });

        if (
            !verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofTry,
                channel.counterparty.portId,
                counterpartyChannelId,
                IbcCoreChannelV1Channel.encode(expectedChannel)
            )
        ) {
            revert IBCChannelLib.ErrInvalidProof();
        }

        channel.state = IbcCoreChannelV1GlobalEnums.State.STATE_OPEN;
        channel.version = msg_.counterpartyVersion;
        channel.counterparty.channelId =
            msg_.counterpartyChannelId.parseChannelIdCalldata();
        updateChannelCommitment(msg_.portId, channelId);

        lookupModuleByPort(msg_.portId).onChanOpenAck(
            msg_.portId,
            channelId,
            counterpartyChannelId,
            msg_.counterpartyVersion
        );

        emit IBCChannelLib.ChannelOpenAck(channelId, msg_.portId);
    }

    /**
     * @dev channelOpenConfirm is called by the counterparty module to close their end of the channel, since the other end has been closed.
     */
    function channelOpenConfirm(IBCMsgs.MsgChannelOpenConfirm calldata msg_)
        external
        override
    {
        ChannelId channelId = msg_.channelId.parseChannelIdCalldata();

        IBCChannelTypes.Channel storage channel =
            channels[msg_.portId][channelId];

        if (channel.state != IbcCoreChannelV1GlobalEnums.State.STATE_TRYOPEN) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }

        IbcCoreConnectionV1ConnectionEnd.Data memory connection =
            ensureConnectionState(channel.connectionHops[0]);

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
            connection_hops: getCounterpartyHops(channel.connectionHops[0]),
            version: channel.version
        });

        if (
            !verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofAck,
                channel.counterparty.portId,
                channel.counterparty.channelId,
                IbcCoreChannelV1Channel.encode(expectedChannel)
            )
        ) {
            revert IBCChannelLib.ErrInvalidProof();
        }

        channel.state = IbcCoreChannelV1GlobalEnums.State.STATE_OPEN;
        updateChannelCommitment(msg_.portId, channelId);

        lookupModuleByPort(msg_.portId).onChanOpenConfirm(
            msg_.portId, channelId
        );

        emit IBCChannelLib.ChannelOpenConfirm(channelId, msg_.portId);
    }

    /**
     * @dev channelCloseInit is called by either module to close their end of the channel. Once closed, channels cannot be reopened.
     */
    function channelCloseInit(IBCMsgs.MsgChannelCloseInit calldata msg_)
        external
        override
    {
        ChannelId channelId = msg_.channelId.parseChannelIdCalldata();

        IBCChannelTypes.Channel storage channel =
            channels[msg_.portId][channelId];

        if (channel.state != IbcCoreChannelV1GlobalEnums.State.STATE_OPEN) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }

        // IbcCoreConnectionV1ConnectionEnd.Data memory connection =
        ensureConnectionState(channel.connectionHops[0]);

        channel.state = IbcCoreChannelV1GlobalEnums.State.STATE_CLOSED;
        updateChannelCommitment(msg_.portId, channelId);

        lookupModuleByPort(msg_.portId).onChanCloseInit(msg_.portId, channelId);

        emit IBCChannelLib.ChannelCloseInit(channelId, msg_.portId);
    }

    /**
     * @dev channelCloseConfirm is called by the counterparty module to close their end of the
     * channel, since the other end has been closed.
     */
    function channelCloseConfirm(IBCMsgs.MsgChannelCloseConfirm calldata msg_)
        external
        override
    {
        ChannelId channelId = msg_.channelId.parseChannelIdCalldata();

        IBCChannelTypes.Channel storage channel =
            channels[msg_.portId][channelId];

        if (channel.state != IbcCoreChannelV1GlobalEnums.State.STATE_OPEN) {
            revert IBCChannelLib.ErrInvalidChannelState();
        }

        IbcCoreConnectionV1ConnectionEnd.Data memory connection =
            ensureConnectionState(channel.connectionHops[0]);

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
            connection_hops: getCounterpartyHops(channel.connectionHops[0]),
            version: channel.version
        });

        if (
            !verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofInit,
                channel.counterparty.portId,
                channel.counterparty.channelId,
                IbcCoreChannelV1Channel.encode(expectedChannel)
            )
        ) {
            revert IBCChannelLib.ErrInvalidProof();
        }

        channel.state = IbcCoreChannelV1GlobalEnums.State.STATE_CLOSED;
        updateChannelCommitment(msg_.portId, channelId);

        lookupModuleByPort(msg_.portId).onChanCloseConfirm(
            msg_.portId, channelId
        );

        emit IBCChannelLib.ChannelCloseConfirm(channelId, msg_.portId);
    }

    function updateChannelCommitment(
        string memory portId,
        ChannelId channelId
    ) private {
        commitments[IBCCommitment.channelCommitmentKey(portId, channelId)] =
            keccak256(abi.encode(channels[portId][channelId]));
    }

    /* Verification functions */

    function verifyChannelState(
        IbcCoreConnectionV1ConnectionEnd.Data memory connection,
        IbcCoreClientV1Height.Data calldata height,
        bytes calldata proof,
        string memory portId,
        ChannelId channelId,
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

    function generateChannelIdentifier() internal returns (ChannelId) {
        string memory identifier = string(
            abi.encodePacked(
                "channel-", Strings.toString(uint256(nextChannelSequence))
            )
        );
        nextChannelSequence++;
        return identifier.parseChannelIdMemory();
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
