// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/utils/Strings.sol";
import "../../proto/Channel.sol";
import "../25-handler/IBCMsgs.sol";
import "../02-client/IBCHeight.sol";
import "../24-host/IBCStore.sol";
import "../24-host/IBCCommitment.sol";
import "../04-channel/IIBCChannel.sol";

/**
 * @dev IBCChannelHandshake is a contract that implements [ICS-4](https://github.com/cosmos/ibc/tree/main/spec/core/ics-004-channel-and-packet-semantics).
 */
contract IBCChannelHandshake is IBCStore, IIBCChannelHandshake {
    using IBCHeight for Height.Data;

    /* Handshake functions */

    /**
     * @dev channelOpenInit is called by a module to initiate a channel opening handshake with a module on another chain.
     */
    function channelOpenInit(IBCMsgs.MsgChannelOpenInit calldata msg_) external returns (string memory) {
        require(msg_.channel.connection_hops.length == 1, "connection_hops length must be 1");
        ConnectionEnd.Data storage connection = connections[msg_.channel.connection_hops[0]];
        require(
            connection.versions.length == 1, "single version must be negotiated on connection before opening channel"
        );
        require(msg_.channel.state == Channel.State.STATE_INIT, "channel state must STATE_INIT");

        // TODO verifySupportedFeature

        // TODO authenticates a port binding

        string memory channelId = generateChannelIdentifier();
        channels[msg_.portId][channelId] = msg_.channel;
        nextSequenceSends[msg_.portId][channelId] = 1;
        nextSequenceRecvs[msg_.portId][channelId] = 1;
        nextSequenceAcks[msg_.portId][channelId] = 1;
        updateChannelCommitment(msg_.portId, channelId);
        return channelId;
    }

    /**
     * @dev channelOpenTry is called by a module to accept the first step of a channel opening handshake initiated by a module on another chain.
     */
    function channelOpenTry(IBCMsgs.MsgChannelOpenTry calldata msg_) external returns (string memory) {
        require(msg_.channel.connection_hops.length == 1, "connection_hops length must be 1");
        ConnectionEnd.Data storage connection = connections[msg_.channel.connection_hops[0]];
        require(
            connection.versions.length == 1, "single version must be negotiated on connection before opening channel"
        );
        require(msg_.channel.state == Channel.State.STATE_TRYOPEN, "channel state must be STATE_TRYOPEN");
        require(msg_.channel.connection_hops.length == 1);

        // TODO verifySupportedFeature

        // TODO authenticates a port binding

        ChannelCounterparty.Data memory expectedCounterparty =
            ChannelCounterparty.Data({port_id: msg_.portId, channel_id: ""});
        Channel.Data memory expectedChannel = Channel.Data({
            state: Channel.State.STATE_INIT,
            ordering: msg_.channel.ordering,
            counterparty: expectedCounterparty,
            connection_hops: getCounterpartyHops(msg_.channel.connection_hops[0]),
            version: msg_.counterpartyVersion
        });
        require(
            verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofInit,
                msg_.channel.counterparty.port_id,
                msg_.channel.counterparty.channel_id,
                Channel.encode(expectedChannel)
            ),
            "failed to verify channel state"
        );

        string memory channelId = generateChannelIdentifier();
        channels[msg_.portId][channelId] = msg_.channel;
        nextSequenceSends[msg_.portId][channelId] = 1;
        nextSequenceRecvs[msg_.portId][channelId] = 1;
        nextSequenceAcks[msg_.portId][channelId] = 1;
        updateChannelCommitment(msg_.portId, channelId);
        return channelId;
    }

    /**
     * @dev channelOpenAck is called by the handshake-originating module to acknowledge the acceptance of the initial request by the counterparty module on the other chain.
     */
    function channelOpenAck(IBCMsgs.MsgChannelOpenAck calldata msg_) external {
        Channel.Data storage channel = channels[msg_.portId][msg_.channelId];
        require(
            channel.state == Channel.State.STATE_INIT || channel.state == Channel.State.STATE_TRYOPEN,
            "invalid channel state"
        );

        // TODO authenticates a port binding

        ConnectionEnd.Data storage connection = connections[channel.connection_hops[0]];
        require(connection.state == ConnectionEnd.State.STATE_OPEN, "connection state is not OPEN");
        require(channel.connection_hops.length == 1);

        ChannelCounterparty.Data memory expectedCounterparty =
            ChannelCounterparty.Data({port_id: msg_.portId, channel_id: msg_.channelId});
        Channel.Data memory expectedChannel = Channel.Data({
            state: Channel.State.STATE_TRYOPEN,
            ordering: channel.ordering,
            counterparty: expectedCounterparty,
            connection_hops: getCounterpartyHops(channel.connection_hops[0]),
            version: msg_.counterpartyVersion
        });
        require(
            verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofTry,
                channel.counterparty.port_id,
                msg_.counterpartyChannelId,
                Channel.encode(expectedChannel)
            ),
            "failed to verify channel state"
        );
        channel.state = Channel.State.STATE_OPEN;
        channel.version = msg_.counterpartyVersion;
        channel.counterparty.channel_id = msg_.counterpartyChannelId;
        updateChannelCommitment(msg_.portId, msg_.channelId);
    }

    /**
     * @dev channelOpenConfirm is called by the counterparty module to close their end of the channel, since the other end has been closed.
     */
    function channelOpenConfirm(IBCMsgs.MsgChannelOpenConfirm calldata msg_) external {
        Channel.Data storage channel = channels[msg_.portId][msg_.channelId];
        require(channel.state == Channel.State.STATE_TRYOPEN, "channel state is not TRYOPEN");

        // TODO authenticates a port binding

        ConnectionEnd.Data storage connection = connections[channel.connection_hops[0]];
        require(connection.state == ConnectionEnd.State.STATE_OPEN, "connection state is not OPEN");
        require(channel.connection_hops.length == 1);

        ChannelCounterparty.Data memory expectedCounterparty =
            ChannelCounterparty.Data({port_id: msg_.portId, channel_id: msg_.channelId});
        Channel.Data memory expectedChannel = Channel.Data({
            state: Channel.State.STATE_OPEN,
            ordering: channel.ordering,
            counterparty: expectedCounterparty,
            connection_hops: getCounterpartyHops(channel.connection_hops[0]),
            version: channel.version
        });
        require(
            verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofAck,
                channel.counterparty.port_id,
                channel.counterparty.channel_id,
                Channel.encode(expectedChannel)
            ),
            "failed to verify channel state"
        );
        channel.state = Channel.State.STATE_OPEN;
        updateChannelCommitment(msg_.portId, msg_.channelId);
    }

    /**
     * @dev channelCloseInit is called by either module to close their end of the channel. Once closed, channels cannot be reopened.
     */
    function channelCloseInit(IBCMsgs.MsgChannelCloseInit calldata msg_) external {
        Channel.Data storage channel = channels[msg_.portId][msg_.channelId];
        require(channel.state != Channel.State.STATE_CLOSED, "channel state is already CLOSED");

        // TODO authenticates a port binding

        ConnectionEnd.Data storage connection = connections[channel.connection_hops[0]];
        require(connection.state == ConnectionEnd.State.STATE_OPEN, "connection state is not OPEN");

        channel.state = Channel.State.STATE_CLOSED;
        updateChannelCommitment(msg_.portId, msg_.channelId);
    }

    /**
     * @dev channelCloseConfirm is called by the counterparty module to close their end of the
     * channel, since the other end has been closed.
     */
    function channelCloseConfirm(IBCMsgs.MsgChannelCloseConfirm calldata msg_) external {
        Channel.Data storage channel = channels[msg_.portId][msg_.channelId];
        require(channel.state != Channel.State.STATE_CLOSED, "channel state is already CLOSED");

        // TODO authenticates a port binding

        require(channel.connection_hops.length == 1);
        ConnectionEnd.Data storage connection = connections[channel.connection_hops[0]];
        require(connection.state == ConnectionEnd.State.STATE_OPEN, "connection state is not OPEN");

        ChannelCounterparty.Data memory expectedCounterparty =
            ChannelCounterparty.Data({port_id: msg_.portId, channel_id: msg_.channelId});
        Channel.Data memory expectedChannel = Channel.Data({
            state: Channel.State.STATE_CLOSED,
            ordering: channel.ordering,
            counterparty: expectedCounterparty,
            connection_hops: getCounterpartyHops(channel.connection_hops[0]),
            version: channel.version
        });
        require(
            verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofInit,
                channel.counterparty.port_id,
                channel.counterparty.channel_id,
                Channel.encode(expectedChannel)
            ),
            "failed to verify channel state"
        );
        channel.state = Channel.State.STATE_CLOSED;
        updateChannelCommitment(msg_.portId, msg_.channelId);
    }

    function updateChannelCommitment(string memory portId, string memory channelId) private {
        commitments[IBCCommitment.channelCommitmentKey(portId, channelId)] =
            keccak256(Channel.encode(channels[portId][channelId]));
    }

    /* Verification functions */

    function verifyChannelState(
        ConnectionEnd.Data storage connection,
        Height.Data calldata height,
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

    function getCounterpartyHops(string memory connectionId) internal view returns (string[] memory hops) {
        hops = new string[](1);
        hops[0] = connections[connectionId].counterparty.connection_id;
        return hops;
    }

    function generateChannelIdentifier() private returns (string memory) {
        string memory identifier = string(abi.encodePacked("channel-", Strings.toString(nextChannelSequence)));
        nextChannelSequence++;
        return identifier;
    }
}
