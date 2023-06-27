pragma solidity ^0.8.18;

import "@openzeppelin/contracts/utils/Strings.sol";
import "../../proto/ibc/core/channel/v1/channel.sol";
import "../25-handler/IBCMsgs.sol";
import "../02-client/IBCHeight.sol";
import "../24-host/IBCStore.sol";
import "../24-host/IBCCommitment.sol";
import "../04-channel/IIBCChannel.sol";

/**
 * @dev IBCChannelHandshake is a contract that implements [ICS-4](https://github.com/cosmos/ibc/tree/main/spec/core/ics-004-channel-and-packet-semantics).
 */
contract IBCChannelHandshake is IBCStore, IIBCChannelHandshake {
    using IBCHeight for IbcCoreClientV1Height.Data;

    /* Handshake functions */

    /**
     * @dev channelOpenInit is called by a module to initiate a channel opening handshake with a module on another chain.
     */
    function channelOpenInit(
        IBCMsgs.MsgChannelOpenInit calldata msg_
    ) external returns (string memory) {
        require(
            msg_.channel.connection_hops.length == 1,
            "channelOpenInit: connection must have a single hop"
        );
        IbcCoreConnectionV1ConnectionEnd.Data storage connection = connections[
            msg_.channel.connection_hops[0]
        ];
        require(
            connection.versions.length == 1,
            "single version must be negotiated on connection before opening channel"
        );
        require(
            msg_.channel.state == IbcCoreChannelV1GlobalEnums.State.STATE_INIT,
            "channel state must STATE_INIT"
        );

        // TODO verifySupportedFeature

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
    function channelOpenTry(
        IBCMsgs.MsgChannelOpenTry calldata msg_
    ) external returns (string memory) {
        require(
            msg_.channel.connection_hops.length == 1,
            "channelOpenInit: connection must have a single hop"
        );
        IbcCoreConnectionV1ConnectionEnd.Data storage connection = connections[
            msg_.channel.connection_hops[0]
        ];
        require(
            connection.versions.length == 1,
            "channelOpenTry: single version must be negotiated on connection before opening channel"
        );
        require(
            msg_.channel.state ==
                IbcCoreChannelV1GlobalEnums.State.STATE_TRYOPEN,
            "channelOpenTry: channel state must be STATE_TRYOPEN"
        );

        // TODO verifySupportedFeature

        IbcCoreChannelV1Counterparty.Data
            memory expectedCounterparty = IbcCoreChannelV1Counterparty.Data({
                port_id: msg_.portId,
                channel_id: ""
            });
        IbcCoreChannelV1Channel.Data
            memory expectedChannel = IbcCoreChannelV1Channel.Data({
                state: IbcCoreChannelV1GlobalEnums.State.STATE_INIT,
                ordering: msg_.channel.ordering,
                counterparty: expectedCounterparty,
                connection_hops: getCounterpartyHops(
                    msg_.channel.connection_hops[0]
                ),
                version: msg_.counterpartyVersion
            });
        require(
            verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofInit,
                msg_.channel.counterparty.port_id,
                msg_.channel.counterparty.channel_id,
                IbcCoreChannelV1Channel.encode(expectedChannel)
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
        IbcCoreChannelV1Channel.Data storage channel = channels[msg_.portId][
            msg_.channelId
        ];
        require(
            channel.state == IbcCoreChannelV1GlobalEnums.State.STATE_INIT,
            "channelOpenAck: channel.state != STATE_INIT"
        );

        IbcCoreConnectionV1ConnectionEnd.Data storage connection = connections[
            channel.connection_hops[0]
        ];
        require(
            connection.state == IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN,
            "connection state is not OPEN"
        );
        require(channel.connection_hops.length == 1);

        IbcCoreChannelV1Counterparty.Data
            memory expectedCounterparty = IbcCoreChannelV1Counterparty.Data({
                port_id: msg_.portId,
                channel_id: msg_.channelId
            });
        IbcCoreChannelV1Channel.Data
            memory expectedChannel = IbcCoreChannelV1Channel.Data({
                state: IbcCoreChannelV1GlobalEnums.State.STATE_TRYOPEN,
                ordering: channel.ordering,
                counterparty: expectedCounterparty,
                connection_hops: getCounterpartyHops(
                    channel.connection_hops[0]
                ),
                version: msg_.counterpartyVersion
            });
        require(
            verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofTry,
                channel.counterparty.port_id,
                msg_.counterpartyChannelId,
                IbcCoreChannelV1Channel.encode(expectedChannel)
            ),
            "channelOpenAck: failed to verify channel state"
        );
        channel.state = IbcCoreChannelV1GlobalEnums.State.STATE_OPEN;
        channel.version = msg_.counterpartyVersion;
        channel.counterparty.channel_id = msg_.counterpartyChannelId;
        updateChannelCommitment(msg_.portId, msg_.channelId);
    }

    /**
     * @dev channelOpenConfirm is called by the counterparty module to close their end of the channel, since the other end has been closed.
     */
    function channelOpenConfirm(
        IBCMsgs.MsgChannelOpenConfirm calldata msg_
    ) external {
        IbcCoreChannelV1Channel.Data storage channel = channels[msg_.portId][
            msg_.channelId
        ];
        require(
            channel.state == IbcCoreChannelV1GlobalEnums.State.STATE_TRYOPEN,
            "channelOpenConfirm: channel state is not TRYOPEN"
        );

        IbcCoreConnectionV1ConnectionEnd.Data storage connection = connections[
            channel.connection_hops[0]
        ];
        require(
            connection.state == IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN,
            "channelOpenConfirm: connection state is not OPEN"
        );
        require(channel.connection_hops.length == 1);

        IbcCoreChannelV1Counterparty.Data
            memory expectedCounterparty = IbcCoreChannelV1Counterparty.Data({
                port_id: msg_.portId,
                channel_id: msg_.channelId
            });
        IbcCoreChannelV1Channel.Data
            memory expectedChannel = IbcCoreChannelV1Channel.Data({
                state: IbcCoreChannelV1GlobalEnums.State.STATE_OPEN,
                ordering: channel.ordering,
                counterparty: expectedCounterparty,
                connection_hops: getCounterpartyHops(
                    channel.connection_hops[0]
                ),
                version: channel.version
            });
        require(
            verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofAck,
                channel.counterparty.port_id,
                channel.counterparty.channel_id,
                IbcCoreChannelV1Channel.encode(expectedChannel)
            ),
            "channelOpenConfirm: failed to verify channel state"
        );
        channel.state = IbcCoreChannelV1GlobalEnums.State.STATE_OPEN;
        updateChannelCommitment(msg_.portId, msg_.channelId);
    }

    /**
     * @dev channelCloseInit is called by either module to close their end of the channel. Once closed, channels cannot be reopened.
     */
    function channelCloseInit(
        IBCMsgs.MsgChannelCloseInit calldata msg_
    ) external {
        IbcCoreChannelV1Channel.Data storage channel = channels[msg_.portId][
            msg_.channelId
        ];
        require(
            channel.state != IbcCoreChannelV1GlobalEnums.State.STATE_CLOSED,
            "channelCloseInit: channel state is already CLOSED"
        );

        IbcCoreConnectionV1ConnectionEnd.Data storage connection = connections[
            channel.connection_hops[0]
        ];
        require(
            connection.state == IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN,
            "channelCloseInit: connection state is not OPEN"
        );

        channel.state = IbcCoreChannelV1GlobalEnums.State.STATE_CLOSED;
        updateChannelCommitment(msg_.portId, msg_.channelId);
    }

    /**
     * @dev channelCloseConfirm is called by the counterparty module to close their end of the
     * channel, since the other end has been closed.
     */
    function channelCloseConfirm(
        IBCMsgs.MsgChannelCloseConfirm calldata msg_
    ) external {
        IbcCoreChannelV1Channel.Data storage channel = channels[msg_.portId][
            msg_.channelId
        ];
        require(
            channel.state != IbcCoreChannelV1GlobalEnums.State.STATE_CLOSED,
            "channelCloseConfirm: channel state is already CLOSED"
        );

        require(channel.connection_hops.length == 1);
        IbcCoreConnectionV1ConnectionEnd.Data storage connection = connections[
            channel.connection_hops[0]
        ];
        require(
            connection.state == IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN,
            "channelCloseConfirm: connection state is not OPEN"
        );

        IbcCoreChannelV1Counterparty.Data
            memory expectedCounterparty = IbcCoreChannelV1Counterparty.Data({
                port_id: msg_.portId,
                channel_id: msg_.channelId
            });
        IbcCoreChannelV1Channel.Data
            memory expectedChannel = IbcCoreChannelV1Channel.Data({
                state: IbcCoreChannelV1GlobalEnums.State.STATE_CLOSED,
                ordering: channel.ordering,
                counterparty: expectedCounterparty,
                connection_hops: getCounterpartyHops(
                    channel.connection_hops[0]
                ),
                version: channel.version
            });
        require(
            verifyChannelState(
                connection,
                msg_.proofHeight,
                msg_.proofInit,
                channel.counterparty.port_id,
                channel.counterparty.channel_id,
                IbcCoreChannelV1Channel.encode(expectedChannel)
            ),
            "channelCloseConfirm: failed to verify channel state"
        );
        channel.state = IbcCoreChannelV1GlobalEnums.State.STATE_CLOSED;
        updateChannelCommitment(msg_.portId, msg_.channelId);
    }

    function updateChannelCommitment(
        string memory portId,
        string memory channelId
    ) private {
        commitments[
            IBCCommitment.channelCommitmentKey(portId, channelId)
        ] = keccak256(
            IbcCoreChannelV1Channel.encode(channels[portId][channelId])
        );
    }

    /* Verification functions */

    function verifyChannelState(
        IbcCoreConnectionV1ConnectionEnd.Data storage connection,
        IbcCoreClientV1Height.Data calldata height,
        bytes calldata proof,
        string memory portId,
        string memory channelId,
        bytes memory channelBytes
    ) private returns (bool) {
        return
            getClient(connection.client_id).verifyMembership(
                connection.client_id,
                height,
                0,
                0,
                proof,
                connection.counterparty.prefix.key_prefix,
                IBCCommitment.channelPathMerkle(portId, channelId),
                channelBytes
            );
    }

    /* Internal functions */

    function getCounterpartyHops(
        string memory connectionId
    ) internal view returns (string[] memory hops) {
        hops = new string[](1);
        hops[0] = connections[connectionId].counterparty.connection_id;
        return hops;
    }

    function generateChannelIdentifier() private returns (string memory) {
        string memory identifier = string(
            abi.encodePacked("channel-", Strings.toString(nextChannelSequence))
        );
        nextChannelSequence++;
        return identifier;
    }
}
