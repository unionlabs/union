pragma solidity ^0.8.18;

import "forge-std/Test.sol";
import "../../../contracts/core/25-handler/IBCHandler.sol";
import "../../../contracts/core/02-client/IBCClient.sol";
import "../../../contracts/core/03-connection/IBCConnection.sol";
import "../../../contracts/core/04-channel/IBCChannelHandshake.sol";
import "../../../contracts/core/04-channel/IBCPacket.sol";
import "../../../contracts/core/24-host/IBCCommitment.sol";
import "../../../contracts/lib/CometblsHelp.sol";
import "../../../contracts/lib/Encoder.sol";
import "../../../contracts/clients/MockClient.sol";
import "../../../contracts/proto/MockClient.sol";
import "../../../contracts/proto/ibc/core/connection/v1/connection.sol";
import "../../../contracts/proto/ibc/core/channel/v1/channel.sol";
import "../../../contracts/proto/tendermint/types/canonical.sol";
import "./TestableIBCHandler.t.sol";
import "./MockApp.t.sol";

// TODO split setup code into other contracts
contract IBCTest is Test {
    using CometblsHelp for TendermintTypesHeader.Data;

    TestableIBCHandler handler;
    MockClient mockClient;
    MockApp mockApp;

    string private constant mockClientType = "mock-client";
    string private constant portId = "mock";
    bytes32 private testPacketCommitment;

    function setUp() public {
        address ibcClient = address(new IBCClient());
        address ibcConnection = address(new IBCConnection());
        address ibcChannelHandshake = address(new IBCChannelHandshake());
        address ibcPacket = address(new IBCPacket());
        handler = new TestableIBCHandler(ibcClient, ibcConnection, ibcChannelHandshake, ibcPacket);

        mockClient = new MockClient(address(handler));
        handler.registerClient(mockClientType, mockClient);

        setUpMockClient();
        setUpConnection();
        setUpChannel();
        setUpMockApp();
    }

    function setUpMockClient() internal {
        createMockClient(1);
    }

    function setUpConnection() internal {
        IbcCoreConnectionV1ConnectionEnd.Data memory connection = IbcCoreConnectionV1ConnectionEnd.Data({
            client_id: "mock-client-0",
            versions: getConnectionVersions(),
            state: IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN,
            delay_period: 0,
            counterparty: IbcCoreConnectionV1Counterparty.Data({
                client_id: "mock-client-0",
                connection_id: "connection-0",
                prefix: IbcCoreCommitmentV1MerklePrefix.Data({key_prefix: bytes("ibc")})
            })
        });
        handler.setConnection("connection-0", connection);
        handler.setNextConnectionSequence(1);
    }

    function setUpChannel() internal {
        string[] memory hops = new string[](1);
        hops[0] = "connection-0";
        IbcCoreChannelV1Channel.Data memory channel = IbcCoreChannelV1Channel.Data({
            state: IbcCoreChannelV1GlobalEnums.State.STATE_OPEN,
            ordering: IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED,
            counterparty: IbcCoreChannelV1Counterparty.Data({port_id: portId, channel_id: "channel-0"}),
            connection_hops: hops,
            version: "1"
        });
        handler.setChannel(portId, "channel-0", channel);
        handler.setNextChannelSequence(1);
        handler.setNextSequenceSend(portId, "channel-0", 1);
        handler.setNextSequenceRecv(portId, "channel-0", 1);
        handler.setNextSequenceAck(portId, "channel-0", 1);

        testPacketCommitment = makePacketCommitment(getPacket());
    }

    function setUpMockApp() internal {
        mockApp = new MockApp();
        handler.bindPort(portId, address(mockApp));
        handler.claimCapabilityDirectly(handler.channelCapabilityPath(portId, "channel-0"), address(mockApp));
        handler.claimCapabilityDirectly(handler.channelCapabilityPath(portId, "channel-0"), address(this));
    }

    /* test cases */

    function testConnectionOpenInit() public {
        IBCMsgs.MsgConnectionOpenInit memory msg_ = IBCMsgs.MsgConnectionOpenInit({
            clientId: "mock-client-1",
            counterparty: IbcCoreConnectionV1Counterparty.Data({
                client_id: "mock-client-1",
                connection_id: "",
                prefix: IbcCoreCommitmentV1MerklePrefix.Data({key_prefix: bytes("ibc")})
            }),
            delayPeriod: 0
        });
        string memory connectionId = handler.connectionOpenInit(msg_);
        assertEq(connectionId, "connection-1");
    }

    function testBenchmarkCreateMockClient() public {
        createMockClient(1);
    }

    function testBenchmarkUpdateMockClient() public {
        updateMockClient(2);
    }

    function testBenchmarkSendPacket() public {
        IbcCoreChannelV1Packet.Data memory packet = getPacket();
        handler.sendPacket(packet);
    }

    event MockRecv(bool ok);

    function testBenchmarkRecvPacket() public {
        IbcCoreChannelV1Packet.Data memory packet = getPacket();
        vm.expectEmit(false, false, false, true);
        emit MockRecv(true);
        handler.recvPacket(
            IBCMsgs.MsgPacketRecv({
                packet: packet,
                proof: abi.encodePacked(sha256(abi.encodePacked(testPacketCommitment))),
                proofHeight: IbcCoreClientV1Height.Data({revision_number: 0, revision_height: 1})
            })
        );
    }

    /* internal functions */

    function createMockClient(uint64 revision_height) internal {
        handler.createClient(
            IBCMsgs.MsgCreateClient({
                clientType: mockClientType,
                clientStateBytes: wrapAnyMockClientState(
                    IbcLightclientsMockV1ClientState.Data({
                        latest_height: IbcCoreClientV1Height.Data({revision_number: 0, revision_height: revision_height})
                    })
                    ),
                consensusStateBytes: wrapAnyMockConsensusState(
                    IbcLightclientsMockV1ConsensusState.Data({timestamp: uint64(block.timestamp)})
                    )
            })
        );
    }

    function updateMockClient(uint64 next_revision_height) internal {
        handler.updateClient(
            IBCMsgs.MsgUpdateClient({
                clientId: "mock-client-0",
                clientMessage: wrapAnyMockHeader(
                    IbcLightclientsMockV1Header.Data({
                        height: IbcCoreClientV1Height.Data({revision_number: 0, revision_height: next_revision_height}),
                        timestamp: uint64(block.timestamp)
                    })
                    )
            })
        );
    }

    function wrapAnyMockHeader(IbcLightclientsMockV1Header.Data memory header) internal pure returns (bytes memory) {
        Any.Data memory anyHeader;
        anyHeader.type_url = "/ibc.lightclients.mock.v1.Header";
        anyHeader.value = IbcLightclientsMockV1Header.encode(header);
        return Any.encode(anyHeader);
    }

    function wrapAnyMockClientState(IbcLightclientsMockV1ClientState.Data memory clientState)
        internal
        pure
        returns (bytes memory)
    {
        Any.Data memory anyClientState;
        anyClientState.type_url = "/ibc.lightclients.mock.v1.ClientState";
        anyClientState.value = IbcLightclientsMockV1ClientState.encode(clientState);
        return Any.encode(anyClientState);
    }

    function wrapAnyMockConsensusState(IbcLightclientsMockV1ConsensusState.Data memory consensusState)
        internal
        pure
        returns (bytes memory)
    {
        Any.Data memory anyConsensusState;
        anyConsensusState.type_url = "/ibc.lightclients.mock.v1.ConsensusState";
        anyConsensusState.value = IbcLightclientsMockV1ConsensusState.encode(consensusState);
        return Any.encode(anyConsensusState);
    }

    function getConnectionVersions() internal pure returns (IbcCoreConnectionV1Version.Data[] memory) {
        IbcCoreConnectionV1Version.Data[] memory versions = new IbcCoreConnectionV1Version.Data[](1);
        string[] memory features = new string[](2);
        features[0] = "ORDER_ORDERED";
        features[1] = "ORDER_UNORDERED";
        versions[0] = IbcCoreConnectionV1Version.Data({identifier: "1", features: features});
        return versions;
    }

    function getPacket() internal pure returns (IbcCoreChannelV1Packet.Data memory packet) {
        return IbcCoreChannelV1Packet.Data({
            sequence: 1,
            source_port: portId,
            source_channel: "channel-0",
            destination_port: portId,
            destination_channel: "channel-0",
            data: bytes("{\"amount\": \"100\"}"),
            timeout_height: IbcCoreClientV1Height.Data({revision_number: 0, revision_height: 100}),
            timeout_timestamp: 0
        });
    }

    function makePacketCommitment(IbcCoreChannelV1Packet.Data memory packet) internal pure returns (bytes32) {
        return sha256(
            abi.encodePacked(
                packet.timeout_timestamp,
                packet.timeout_height.revision_number,
                packet.timeout_height.revision_height,
                sha256(packet.data)
            )
        );
    }
}
