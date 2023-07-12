pragma solidity ^0.8.18;

import "forge-std/Test.sol";
import {IBCMsgs} from "contracts/core/25-handler/IBCMsgs.sol";
import {
    IbcLightclientsMockV1ClientState as MockClientState,
    IbcLightclientsMockV1Header as MockHeader,
    IbcLightclientsMockV1ConsensusState as MockConsensusState,
    IbcCoreClientV1Height as ClientHeight
} from "contracts/proto/MockClient.sol";
import {GoogleProtobufAny as Any} from "contracts/proto/GoogleProtobufAny.sol";
import {
    IbcCoreChannelV1Counterparty as ChannelCounterparty,
    IbcCoreChannelV1Channel as Channel,
    IbcCoreChannelV1GlobalEnums as ChannelEnums
} from "contracts/proto/ibc/core/channel/v1/channel.sol";
import {
    IbcCoreConnectionV1Counterparty as ConnectionCounterparty,
    IbcCoreConnectionV1Version as ConnectionVersion,
    IbcCoreConnectionV1ConnectionEnd as ConnectionEnd,
    IbcCoreConnectionV1GlobalEnums as ConnectionEnums
} from "contracts/proto/ibc/core/connection/v1/connection.sol";
import {IbcCoreCommitmentV1MerklePrefix as CommitmentMerklePrefix} from
    "contracts/proto/ibc/core/commitment/v1/commitment.sol";

library MsgMocks {
    //
    // IBCClient msgs
    //

    /// Builds a MsgCreateClient
    /// TODO: should we fuzz over height.revision_number?
    function createClient(string memory clientType, uint64 revisionHeight)
        internal
        view
        returns (IBCMsgs.MsgCreateClient memory m)
    {
        m.clientType = clientType;
        m.clientStateBytes = wrapAnyMockClientState(
            MockClientState.Data({
                latest_height: ClientHeight.Data({revision_number: 0, revision_height: revisionHeight})
            })
        );
        m.consensusStateBytes = wrapAnyMockConsensusState(MockConsensusState.Data({timestamp: uint64(block.timestamp)}));
    }

    /// Builds a MsgUpdateClient
    /// TODO: should we fuzz over height.revision_number?
    function updateClient(string memory clientId, uint64 nextRevisionHeight)
        internal
        view
        returns (IBCMsgs.MsgUpdateClient memory m)
    {
        m.clientId = clientId;
        m.clientMessage = wrapAnyMockHeader(
            MockHeader.Data({
                height: ClientHeight.Data({revision_number: 0, revision_height: nextRevisionHeight}),
                timestamp: uint64(block.timestamp)
            })
        );
    }

    //
    // IBCConnection msgs
    //

    /// Builds a MsgConnectionOpenInit
    function connectionOpenInit(string memory clientId)
        internal
        view
        returns (IBCMsgs.MsgConnectionOpenInit memory m)
    {
        m.clientId = clientId;
        m.counterparty.client_id = "counterparty-client-id";
        m.counterparty.connection_id = "counterparty-conn-id";
    }

    /// Builds a MsgConnectionOpenTry
    /// TODO: should we fuzz over version.identifier?
    function connectionOpenTry(string memory clientId, string memory connId)
        internal
        view
        returns (IBCMsgs.MsgConnectionOpenTry memory m)
    {
        m.clientId = clientId;
        m.counterparty.client_id = clientId;
        m.counterparty.connection_id = connId;
        m.counterpartyVersions = new ConnectionVersion.Data[](1);
        m.counterpartyVersions[0] = ConnectionVersion.Data({identifier: "1", features: new string[](0)});
    }

    /// Builds a MsgConnectionOpenAck
    /// TODO: what should msg.clientStateBytes be?
    //. TODO: msg.counterpartyConnectionId is ignored by MockClient. but probably should be set for CometblsClient
    /// TODO: what other fields should we fill here?
    function connectionOpenAck(string memory clientId, string memory connId, uint64 proofHeight)
        internal
        view
        returns (IBCMsgs.MsgConnectionOpenAck memory m)
    {
        m.connectionId = connId;
        m.version = ConnectionVersion.Data({identifier: "1", features: new string[](0)});
        m.proofHeight.revision_height = proofHeight;

        // mocking connection data
        ConnectionEnd.Data memory connection = ConnectionEnd.Data({
            client_id: "counterparty-client-id",
            versions: new ConnectionVersion.Data[](1),
            state: ConnectionEnums.State.STATE_TRYOPEN,
            delay_period: 0,
            counterparty: ConnectionCounterparty.Data({
                client_id: clientId,
                connection_id: connId,
                prefix: CommitmentMerklePrefix.Data({key_prefix: bytes(commitment_prefix())})
            })
        });
        connection.versions[0] = m.version;

        bytes memory encodedConnection = ConnectionEnd.encode(connection);
        m.proofTry = abi.encodePacked(sha256(encodedConnection));

        // for MockClient, it seems this value doesn't matter
        // it just checks sha256(clientStateBytes) == proofClient
        m.clientStateBytes = abi.encodePacked(bytes32(uint256(0x1)));
        m.proofClient = abi.encodePacked(sha256(m.clientStateBytes));
    }

    /// Builds a MsgChannelOpenInit
    function channelOpenInit(string memory portId, uint64 revisionHeight)
        internal
        view
        returns (IBCMsgs.MsgChannelOpenInit memory m)
    {
        ChannelCounterparty.Data memory counterparty;
        counterparty.port_id = "1";
        counterparty.channel_id = "1";
        string[] memory hops = new string[](1);
        hops[0] = "hop-1";

        m.portId = portId;
        m.channel = Channel.Data({
            state: ChannelEnums.State.STATE_UNINITIALIZED_UNSPECIFIED,
            ordering: ChannelEnums.Order.ORDER_NONE_UNSPECIFIED,
            counterparty: counterparty,
            connection_hops: hops,
            version: "v1"
        });
    }
}

function wrapAnyMockHeader(MockHeader.Data memory header) pure returns (bytes memory) {
    Any.Data memory anyHeader;
    anyHeader.type_url = "/ibc.lightclients.mock.v1.Header";
    anyHeader.value = MockHeader.encode(header);
    return Any.encode(anyHeader);
}

function wrapAnyMockClientState(MockClientState.Data memory clientState) pure returns (bytes memory) {
    Any.Data memory anyClientState;
    anyClientState.type_url = "/ibc.lightclients.mock.v1.ClientState";
    anyClientState.value = MockClientState.encode(clientState);
    return Any.encode(anyClientState);
}

function wrapAnyMockConsensusState(MockConsensusState.Data memory consensusState) pure returns (bytes memory) {
    Any.Data memory anyConsensusState;
    anyConsensusState.type_url = "/ibc.lightclients.mock.v1.ConsensusState";
    anyConsensusState.value = MockConsensusState.encode(consensusState);
    return Any.encode(anyConsensusState);
}

function commitment_prefix() pure returns (string memory) {
    return "ibc";
}
