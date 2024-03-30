pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {
    IbcLightclientsMockV1ClientState,
    IbcLightclientsMockV1Header,
    IbcLightclientsMockV1ConsensusState,
    IbcCoreClientV1Height
} from "../../../contracts/proto/MockClient.sol";
import {GoogleProtobufAny as Any} from
    "../../../contracts/proto/GoogleProtobufAny.sol";
import "../../../contracts/core/04-channel/IBCChannelTypes.sol";
import {
    IbcCoreChannelV1Counterparty,
    IbcCoreChannelV1Channel,
    IbcCoreChannelV1GlobalEnums
} from "../../../contracts/proto/ibc/core/channel/v1/channel.sol";
import {
    IbcCoreConnectionV1Counterparty,
    IbcCoreConnectionV1Version,
    IbcCoreConnectionV1ConnectionEnd,
    IbcCoreConnectionV1GlobalEnums
} from "../../../contracts/proto/ibc/core/connection/v1/connection.sol";
import {IbcCoreCommitmentV1MerklePrefix} from
    "../../../contracts/proto/ibc/core/commitment/v1/commitment.sol";

contract TestPlus is Test {
    function assertStrEq(string memory a, string memory b) internal pure {
        require(
            keccak256(abi.encodePacked(a)) == keccak256(abi.encodePacked(b)),
            "strings not equal"
        );
    }

    function assertStrNotEq(string memory a, string memory b) internal pure {
        require(
            keccak256(abi.encodePacked(a)) != keccak256(abi.encodePacked(b)),
            "strings equal"
        );
    }
}

library MsgMocks {
    function connectionOpenInit(string memory clientId)
        internal
        view
        returns (IBCMsgs.MsgConnectionOpenInit memory m)
    {
        m.clientId = clientId;
        m.counterparty.client_id = "counterparty-client-id";
        m.counterparty.connection_id = "counterparty-conn-id";
    }

    function connectionOpenTry(
        string memory clientId,
        string memory connId
    ) internal view returns (IBCMsgs.MsgConnectionOpenTry memory m) {
        m.clientId = clientId;
        m.counterparty.client_id = clientId;
        m.counterparty.connection_id = connId;
        m.counterpartyVersions = new IbcCoreConnectionV1Version.Data[](1);
        m.counterpartyVersions[0] = IbcCoreConnectionV1Version.Data({
            identifier: "1",
            features: new string[](0)
        });
    }

    function connectionOpenAck(
        string memory clientId,
        string memory connId,
        uint64 proofHeight
    ) internal view returns (IBCMsgs.MsgConnectionOpenAck memory m) {
        m.connectionId = connId;
        m.version = IbcCoreConnectionV1Version.Data({
            identifier: "1",
            features: new string[](0)
        });
        m.proofHeight.revision_height = proofHeight;

        IbcCoreConnectionV1Counterparty.Data memory expectedCounterparty =
        IbcCoreConnectionV1Counterparty.Data({
            client_id: clientId,
            connection_id: connId,
            prefix: IbcCoreCommitmentV1MerklePrefix.Data({
                key_prefix: bytes(commitment_prefix())
            })
        });

        IbcCoreConnectionV1ConnectionEnd.Data memory expectedConnection =
        IbcCoreConnectionV1ConnectionEnd.Data({
            client_id: "counterparty-client-id",
            versions: new IbcCoreConnectionV1Version.Data[](1),
            state: IbcCoreConnectionV1GlobalEnums.State.STATE_TRYOPEN,
            delay_period: 0,
            counterparty: expectedCounterparty
        });
        expectedConnection.versions[0] = m.version;

        bytes memory encodedConnection =
            IbcCoreConnectionV1ConnectionEnd.encode(expectedConnection);
        bytes32 proof = sha256(encodedConnection);
        m.proofTry = abi.encodePacked(proof);

        // m.proofClient = ...;
        // m.clientStateBytes = ...;

        // TODO: MockClient.verifyMembership ignores this value, but we probably need to fill it for CometblsClient I supose?
        // m.counterpartyConnectionId = ...;

        // TODO: what other fields should we fill here?
    }

    function createClient(
        string memory clientType,
        uint64 revisionHeight
    ) internal view returns (IBCMsgs.MsgCreateClient memory m) {
        m.clientType = clientType;
        m.clientStateBytes = wrapAnyMockClientState(
            IbcLightclientsMockV1ClientState.Data({
                latest_height: IbcCoreClientV1Height.Data({
                    revision_number: 0,
                    revision_height: revisionHeight
                })
            })
        );
        m.consensusStateBytes = wrapAnyMockConsensusState(
            IbcLightclientsMockV1ConsensusState.Data({
                timestamp: uint64(block.timestamp)
            })
        );
    }

    function updateClient(
        string memory clientId,
        uint64 nextRevisionHeight
    ) internal view returns (IBCMsgs.MsgUpdateClient memory m) {
        m.clientId = clientId;
        m.clientMessage = wrapAnyMockHeader(
            IbcLightclientsMockV1Header.Data({
                height: IbcCoreClientV1Height.Data({
                    revision_number: 0,
                    revision_height: nextRevisionHeight
                }),
                timestamp: uint64(block.timestamp)
            })
        );
    }

    function channelOpenInit(
        string memory portId,
        uint64 revisionHeight
    ) internal view returns (IBCMsgs.MsgChannelOpenInit memory m) {
        IbcCoreChannelV1Counterparty.Data memory counterparty;
        counterparty.port_id = "1";
        counterparty.channel_id = "1";
        string[] memory hops = new string[](1);
        hops[0] = "hop-1";

        m.portId = portId;
        m.channel = IbcCoreChannelV1Channel.Data({
            state: IbcCoreChannelV1GlobalEnums.State.STATE_UNINITIALIZED_UNSPECIFIED,
            ordering: IbcCoreChannelV1GlobalEnums.Order.ORDER_NONE_UNSPECIFIED,
            counterparty: counterparty,
            connection_hops: hops,
            version: "v1"
        });
    }

    function commitment_prefix() private pure returns (string memory) {
        return "ibc";
    }

    function wrapAnyMockHeader(IbcLightclientsMockV1Header.Data memory header)
        private
        pure
        returns (bytes memory)
    {
        Any.Data memory anyHeader;
        anyHeader.type_url = "/ibc.lightclients.mock.v1.Header";
        anyHeader.value = IbcLightclientsMockV1Header.encode(header);
        return Any.encode(anyHeader);
    }

    function wrapAnyMockClientState(
        IbcLightclientsMockV1ClientState.Data memory clientState
    ) private pure returns (bytes memory) {
        Any.Data memory anyClientState;
        anyClientState.type_url = "/ibc.lightclients.mock.v1.ClientState";
        anyClientState.value =
            IbcLightclientsMockV1ClientState.encode(clientState);
        return Any.encode(anyClientState);
    }

    function wrapAnyMockConsensusState(
        IbcLightclientsMockV1ConsensusState.Data memory consensusState
    ) private pure returns (bytes memory) {
        Any.Data memory anyConsensusState;
        anyConsensusState.type_url = "/ibc.lightclients.mock.v1.ConsensusState";
        anyConsensusState.value =
            IbcLightclientsMockV1ConsensusState.encode(consensusState);
        return Any.encode(anyConsensusState);
    }
}
