pragma solidity ^0.8.23;

import "solidity-bytes-utils/BytesLib.sol";

import {IMembershipVerifier} from "../../../contracts/core/IMembershipVerifier.sol";
import {IZKVerifierV2} from "../../../contracts/core/IZKVerifierV2.sol";
import {CometblsClient} from "../../../contracts/clients/CometblsClientV2.sol";
import {IBCHandler} from "../../../contracts/core/25-handler/IBCHandler.sol";
import {IBCConnection} from "../../../contracts/core/03-connection/IBCConnection.sol";
import {IBCClient} from "../../../contracts/core/02-client/IBCClient.sol";
import {IBCChannelHandshake} from "../../../contracts/core/04-channel/IBCChannelHandshake.sol";
import {IIBCPacket} from "../../../contracts/core/04-channel/IIBCChannel.sol";
import {IBCPacket} from "../../../contracts/core/04-channel/IBCPacket.sol";
import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {IbcCoreClientV1Height as ClientHeight} from "../../../contracts/proto/MockClient.sol";
import {IbcCoreConnectionV1ConnectionEnd as ConnectionEnd, IbcCoreConnectionV1Counterparty as ConnectionCounterparty, IbcCoreConnectionV1GlobalEnums as ConnectionEnums} from "../../../contracts/proto/ibc/core/connection/v1/connection.sol";
import {IbcCoreChannelV1Packet} from "../../../contracts/proto/ibc/core/channel/v1/channel.sol";
import {ILightClient} from "../../../contracts/core/02-client/ILightClient.sol";
import {IBCCommitment} from "../../../contracts/core/24-host/IBCCommitment.sol";
import {IbcCoreCommitmentV1MerklePrefix as CommitmentMerklePrefix} from "../../../contracts/proto/ibc/core/commitment/v1/commitment.sol";
import {IbcCoreClientV1Height} from "../../../contracts/proto/ibc/core/client/v1/client.sol";

import "../TestPlus.sol";

contract IBCHandlerFake is IBCHandler {
    constructor()
        IBCHandler(
            address(new IBCClient()),
            address(new IBCConnection()),
            address(new IBCChannelHandshake()),
            address(new IBCPacket())
        )
    {}
}

contract TestVerifier is IZKVerifierV2 {
    uint256 valid = 0;

    function pushValid() public {
        valid += 1;
    }

    function verifyProof(
        uint256[8] memory proof,
        uint256[2] memory proofCommitment,
        uint256[5] calldata input
    ) external returns (bool) {
        bool ok = valid > 0;
        if (valid > 0) {
            valid -= 1;
        }
        return ok;
    }
}

contract TestMembershipVerifier is IMembershipVerifier {
    uint256 valid = 0;

    function pushValid() public {
        valid += 1;
    }

    function verifyMembership(
        bytes memory root,
        bytes calldata proof,
        bytes memory prefix,
        bytes calldata path,
        bytes calldata value
    ) external returns (bool) {
        bool ok = valid > 0;
        if (valid > 0) {
            valid -= 1;
        }
        return ok;
    }

    function verifyNonMembership(
        bytes memory root,
        bytes calldata proof,
        bytes calldata prefix,
        bytes calldata path
    ) external returns (bool) {
        bool ok = valid > 0;
        if (valid > 0) {
            valid -= 1;
        }
        return ok;
    }
}

contract IBCPacketHandlerTest is TestPlus {
    using BytesLib for bytes;
    using ConnectionCounterparty for ConnectionCounterparty.Data;

    string constant CLIENT_TYPE = "mock";
    string constant PORT_ID = "app";

    bytes32 constant ARBITRARY_INITIAL_NEXT_VALIDATORS =
        hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60";
    bytes32 constant ARBITRARY_INITIAL_APP_HASH =
        hex"A8158610DD6858F3D26149CC0DB3339ABD580EA217DE0A151C9C451DED418E35";
    string constant CHAIN_ID = "testnet-1337";
    uint64 constant LATEST_HEIGHT = 0x1337;
    uint64 constant LATEST_TIMESTAMP = 0xCAFEBABE;

    uint64 constant LOCAL_HEIGHT = 0xC0DEC0DEC0DEC0DE;
    uint64 constant LOCAL_TIMESTAMP = 0xDEADBEEFDEADBEEF;

    IBCHandlerFake handler;
    ILightClient client;
    TestVerifier verifier;
    TestMembershipVerifier membershipVerifier;
    MockApp app;

    string clientId;
    string connectionId;
    string channelId;

    function setUp() public {
        handler = new IBCHandlerFake();
        membershipVerifier = new TestMembershipVerifier();
        verifier = new TestVerifier();
        client = new CometblsClient(
            address(handler),
            verifier,
            membershipVerifier
        );
        handler.registerClient(CLIENT_TYPE, client);
        app = new MockApp();
        createClient();
        setupConnection();
        setupChannel();

        vm.warp(LOCAL_TIMESTAMP);
        vm.roll(LOCAL_HEIGHT);
    }

    function createClient() internal {
        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            CHAIN_ID,
            LATEST_HEIGHT,
            ARBITRARY_INITIAL_APP_HASH,
            ARBITRARY_INITIAL_NEXT_VALIDATORS,
            LATEST_TIMESTAMP
        );
        clientId = handler.createClient(m);
    }

    function setupConnection() internal {
        IBCMsgs.MsgConnectionOpenInit memory msg_init = MsgMocks
            .connectionOpenInit(clientId);
        connectionId = handler.connectionOpenInit(msg_init);
        IBCMsgs.MsgConnectionOpenAck memory msg_ack = MsgMocks
            .connectionOpenAck(clientId, connectionId, LATEST_HEIGHT);
        membershipVerifier.pushValid();
        membershipVerifier.pushValid();
        handler.connectionOpenAck(msg_ack);
    }

    function setupChannel() internal {
        handler.bindPort(PORT_ID, address(app));
        IBCMsgs.MsgChannelOpenInit memory msg_init = MsgMocks.channelOpenInit(
            connectionId,
            PORT_ID
        );
        channelId = handler.channelOpenInit(msg_init);
        IBCMsgs.MsgChannelOpenAck memory msg_ack = MsgMocks.channelOpenAck(
            PORT_ID,
            channelId,
            LATEST_HEIGHT
        );
        membershipVerifier.pushValid();
        handler.channelOpenAck(msg_ack);
    }

    function test_sendPacket_ok(
        bytes memory payload,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(timeoutHeight > LATEST_HEIGHT);
        vm.assume(timeoutTimestamp > LATEST_TIMESTAMP);
        vm.prank(address(app));
        handler.sendPacket(
            PORT_ID,
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
    }

    function test_sendPacket_newCommitment(
        bytes memory payload,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(timeoutHeight > LATEST_HEIGHT);
        vm.assume(timeoutTimestamp > LATEST_TIMESTAMP);
        vm.prank(address(app));
        handler.sendPacket(
            PORT_ID,
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        assertEq(
            handler.commitments(
                IBCCommitment.packetCommitmentKey(PORT_ID, channelId, 1)
            ),
            keccak256(
                abi.encodePacked(
                    sha256(
                        abi.encodePacked(
                            timeoutTimestamp,
                            uint64(0),
                            timeoutHeight,
                            sha256(payload)
                        )
                    )
                )
            )
        );
    }

    function test_sendPacket_unauthorized(
        address malicious,
        bytes memory payload,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(malicious != address(0) && malicious != address(app));
        vm.assume(timeoutHeight > LATEST_HEIGHT);
        vm.assume(timeoutTimestamp > LATEST_TIMESTAMP);
        vm.expectRevert("sendPacket: unauthorized");
        vm.prank(malicious);
        handler.sendPacket(
            PORT_ID,
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
    }

    function test_sendPacket_incrementSequence(
        bytes memory payload,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(timeoutHeight > LATEST_HEIGHT);
        vm.assume(timeoutTimestamp > LATEST_TIMESTAMP);
        uint64 sequenceBefore = handler.nextSequenceSends(PORT_ID, channelId);
        vm.prank(address(app));
        handler.sendPacket(
            PORT_ID,
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        uint64 sequenceAfter = handler.nextSequenceSends(PORT_ID, channelId);
        assertEq(sequenceAfter, sequenceBefore + 1);
    }

    function test_sendPacket_invalidTimeoutHeight(
        bytes memory payload,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(0 < timeoutHeight && timeoutHeight <= LATEST_HEIGHT);
        vm.assume(timeoutTimestamp > LATEST_TIMESTAMP);
        vm.prank(address(app));
        vm.expectRevert(
            "sendPacket: receiving chain block height >= packet timeout height"
        );
        handler.sendPacket(
            PORT_ID,
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
    }

    function test_sendPacket_invalidTimeoutTimestamp(
        bytes memory payload,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(timeoutHeight > LATEST_HEIGHT);
        vm.assume(0 < timeoutTimestamp && timeoutTimestamp <= LATEST_TIMESTAMP);
        vm.prank(address(app));
        vm.expectRevert(
            "sendPacket: receiving chain block timestamp >= packet timeout timestamp"
        );
        handler.sendPacket(
            PORT_ID,
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
    }

    function test_recvPacket_ok(
        address relayer,
        bytes memory payload,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > vm.getBlockNumber());
        vm.assume(timeoutTimestamp > vm.getBlockTimestamp());
        membershipVerifier.pushValid();
        vm.prank(relayer);
        handler.recvPacket(
            MsgMocks.packetRecv(
                PORT_ID,
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                timeoutTimestamp,
                payload
            )
        );
    }

    function test_recvPacket_receiptSet(
        address relayer,
        bytes memory payload,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > vm.getBlockNumber());
        vm.assume(timeoutTimestamp > vm.getBlockTimestamp());
        IBCMsgs.MsgPacketRecv memory msg_ = MsgMocks.packetRecv(
            PORT_ID,
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp,
            payload
        );
        assertEq(
            handler.packetReceipts(
                msg_.packet.destination_port,
                msg_.packet.destination_channel,
                msg_.packet.sequence
            ),
            0
        );
        membershipVerifier.pushValid();
        vm.prank(relayer);
        handler.recvPacket(msg_);
        assertEq(
            handler.packetReceipts(
                msg_.packet.destination_port,
                msg_.packet.destination_channel,
                msg_.packet.sequence
            ),
            1
        );
    }

    function test_recvPacket_alreadyReceived(
        address relayer,
        bytes memory payload,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > vm.getBlockNumber());
        vm.assume(timeoutTimestamp > vm.getBlockTimestamp());
        membershipVerifier.pushValid();
        IBCMsgs.MsgPacketRecv memory msg_ = MsgMocks.packetRecv(
            PORT_ID,
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp,
            payload
        );
        vm.prank(relayer);
        handler.recvPacket(msg_);
        membershipVerifier.pushValid();
        vm.expectRevert(
            "recvPacket: packet sequence already has been received"
        );
        vm.prank(relayer);
        handler.recvPacket(msg_);
    }

    function test_recvPacket_invalidTimeoutHeight(
        address relayer,
        bytes memory payload,
        uint32 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(
            0 < timeoutTimestamp && timeoutTimestamp > vm.getBlockTimestamp()
        );
        vm.assume(timeoutHeight > 0);
        membershipVerifier.pushValid();
        vm.expectRevert("recvPacket: block height >= packet timeout height");
        vm.prank(relayer);
        handler.recvPacket(
            MsgMocks.packetRecv(
                PORT_ID,
                channelId,
                LATEST_HEIGHT,
                LOCAL_HEIGHT - timeoutHeight,
                timeoutTimestamp,
                payload
            )
        );
    }

    function test_recvPacket_invalidTimeoutTimestamp(
        address relayer,
        bytes memory payload,
        uint64 timeoutHeight,
        uint32 timeoutTimestamp
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(0 < timeoutHeight && timeoutHeight > vm.getBlockNumber());
        vm.assume(timeoutTimestamp > 0);
        membershipVerifier.pushValid();
        vm.expectRevert(
            "recvPacket: block timestamp >= packet timeout timestamp"
        );
        vm.prank(relayer);
        handler.recvPacket(
            MsgMocks.packetRecv(
                PORT_ID,
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                LOCAL_TIMESTAMP - timeoutTimestamp,
                payload
            )
        );
    }

    function test_recvPacket_invalidProof(
        address relayer,
        bytes memory payload,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > vm.getBlockNumber());
        vm.assume(timeoutTimestamp > vm.getBlockTimestamp());
        vm.expectRevert("recvPacket: failed to verify packet commitment");
        vm.prank(relayer);
        handler.recvPacket(
            MsgMocks.packetRecv(
                PORT_ID,
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                timeoutTimestamp,
                payload
            )
        );
    }

    function test_recvPacket_invalidOriginPort(
        address relayer,
        bytes memory payload,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(relayer != address(0));
        vm.assume(timeoutHeight > vm.getBlockNumber());
        vm.assume(timeoutTimestamp > vm.getBlockTimestamp());
        IBCMsgs.MsgPacketRecv memory msg_ = MsgMocks.packetRecv(
            PORT_ID,
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp,
            payload
        );
        msg_.packet.source_port = "invalid";
        membershipVerifier.pushValid();
        vm.expectRevert(
            "recvPacket: packet source port doesn't match the counterparty's port"
        );
        vm.prank(relayer);
        handler.recvPacket(msg_);
    }

    function test_recvPacket_invalidOriginChannel(
        address relayer,
        bytes memory payload,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > vm.getBlockNumber());
        vm.assume(timeoutTimestamp > vm.getBlockTimestamp());
        IBCMsgs.MsgPacketRecv memory msg_ = MsgMocks.packetRecv(
            PORT_ID,
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp,
            payload
        );
        msg_.packet.source_channel = "invalid";
        membershipVerifier.pushValid();
        vm.expectRevert(
            "recvPacket: packet source channel doesn't match the counterparty's channel"
        );
        vm.prank(relayer);
        handler.recvPacket(msg_);
    }

    function test_writeAcknowledgement_ok(
        uint64 sequence,
        bytes memory acknowledgement
    ) public {
        vm.assume(acknowledgement.length > 0);
        vm.prank(address(app));
        handler.writeAcknowledgement(
            PORT_ID,
            channelId,
            sequence,
            acknowledgement
        );
    }

    function test_writeAcknowledgement_alreadyExist(
        uint64 sequence,
        bytes memory acknowledgement
    ) public {
        vm.assume(acknowledgement.length > 0);
        membershipVerifier.pushValid();
        vm.prank(address(app));
        handler.writeAcknowledgement(
            PORT_ID,
            channelId,
            sequence,
            acknowledgement
        );
        vm.prank(address(app));
        vm.expectRevert(
            "writeAcknowlegement: acknowledgement for packet already exists"
        );
        handler.writeAcknowledgement(
            PORT_ID,
            channelId,
            sequence,
            acknowledgement
        );
    }

    function test_writeAcknowledgement_emptyAcknowledgement(
        uint64 sequence
    ) public {
        vm.prank(address(app));
        vm.expectRevert("writeAcknowlegement: acknowledgement cannot be empty");
        handler.writeAcknowledgement(PORT_ID, channelId, sequence, bytes(""));
    }

    function test_writeAcknowledgement_unauthorized(
        address malicious,
        uint64 sequence,
        bytes memory acknowledgement
    ) public {
        vm.assume(malicious != address(0) && malicious != address(app));
        vm.assume(acknowledgement.length > 0);
        vm.prank(address(malicious));
        vm.expectRevert("writeAcknowledgement: unauthorized");
        handler.writeAcknowledgement(
            PORT_ID,
            channelId,
            sequence,
            acknowledgement
        );
    }

    function test_acknowledgePacket_ok(
        address relayer,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes memory payload,
        bytes memory acknowledgement
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > LATEST_HEIGHT);
        vm.assume(timeoutTimestamp > LATEST_TIMESTAMP);
        vm.prank(address(app));
        handler.sendPacket(
            PORT_ID,
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        membershipVerifier.pushValid();
        vm.prank(relayer);
        handler.acknowledgePacket(
            MsgMocks.packetAck(
                PORT_ID,
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                timeoutTimestamp,
                payload,
                acknowledgement
            )
        );
    }

    function test_acknowledgePacket_alreadyAcknowledged(
        address relayer,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes memory payload,
        bytes memory acknowledgement
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > LATEST_HEIGHT);
        vm.assume(timeoutTimestamp > LATEST_TIMESTAMP);
        vm.prank(address(app));
        handler.sendPacket(
            PORT_ID,
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        membershipVerifier.pushValid();
        vm.prank(relayer);
        handler.acknowledgePacket(
            MsgMocks.packetAck(
                PORT_ID,
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                timeoutTimestamp,
                payload,
                acknowledgement
            )
        );
        membershipVerifier.pushValid();
        vm.prank(relayer);
        vm.expectRevert("acknowledgePacket: packet commitment not found");
        handler.acknowledgePacket(
            MsgMocks.packetAck(
                PORT_ID,
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                timeoutTimestamp,
                payload,
                acknowledgement
            )
        );
    }

    function test_acknowledgePacket_invalidProof(
        address relayer,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes memory payload,
        bytes memory acknowledgement
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > LATEST_HEIGHT);
        vm.assume(timeoutTimestamp > LATEST_TIMESTAMP);
        vm.prank(address(app));
        handler.sendPacket(
            PORT_ID,
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        vm.prank(relayer);
        vm.expectRevert(
            "acknowledgePacket: failed to verify packet acknowledgement commitment"
        );
        handler.acknowledgePacket(
            MsgMocks.packetAck(
                PORT_ID,
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                timeoutTimestamp,
                payload,
                acknowledgement
            )
        );
    }

    function test_acknowledgePacket_notSent(
        address relayer,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes memory payload,
        bytes memory acknowledgement
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > LATEST_HEIGHT);
        vm.assume(timeoutTimestamp > LATEST_TIMESTAMP);
        vm.prank(relayer);
        vm.expectRevert("acknowledgePacket: packet commitment not found");
        handler.acknowledgePacket(
            MsgMocks.packetAck(
                PORT_ID,
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                timeoutTimestamp,
                payload,
                acknowledgement
            )
        );
    }

    function test_acknowledgePacket_payloadTampered(
        address relayer,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes memory payload,
        bytes memory acknowledgement
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > LATEST_HEIGHT);
        vm.assume(timeoutTimestamp > LATEST_TIMESTAMP);
        vm.prank(address(app));
        handler.sendPacket(
            PORT_ID,
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        vm.prank(relayer);
        vm.expectRevert("acknowledgePacket: commitment bytes are not equal");
        handler.acknowledgePacket(
            MsgMocks.packetAck(
                PORT_ID,
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                timeoutTimestamp,
                abi.encodePacked(payload, hex"00"),
                acknowledgement
            )
        );
    }

    function test_acknowledgePacket_invalidDestinationPort(
        address relayer,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes memory payload,
        bytes memory acknowledgement
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > LATEST_HEIGHT);
        vm.assume(timeoutTimestamp > LATEST_TIMESTAMP);
        vm.prank(address(app));
        handler.sendPacket(
            PORT_ID,
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        membershipVerifier.pushValid();
        IBCMsgs.MsgPacketAcknowledgement memory msg_ = MsgMocks.packetAck(
            PORT_ID,
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp,
            payload,
            acknowledgement
        );
        msg_.packet.destination_port = "invalid";
        vm.prank(relayer);
        vm.expectRevert(
            "acknowledgePacket: packet destination port doesn't match the counterparty's port"
        );
        handler.acknowledgePacket(msg_);
    }

    function test_acknowledgePacket_invalidDestinationChannel(
        address relayer,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes memory payload,
        bytes memory acknowledgement
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > LATEST_HEIGHT);
        vm.assume(timeoutTimestamp > LATEST_TIMESTAMP);
        vm.prank(address(app));
        handler.sendPacket(
            PORT_ID,
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        membershipVerifier.pushValid();
        IBCMsgs.MsgPacketAcknowledgement memory msg_ = MsgMocks.packetAck(
            PORT_ID,
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp,
            payload,
            acknowledgement
        );
        msg_.packet.destination_channel = "invalid";
        vm.prank(relayer);
        vm.expectRevert(
            "acknowledgePacket: packet destination channel doesn't match the counterparty's channel"
        );
        handler.acknowledgePacket(msg_);
    }

    // TODO: acknowledgePacket tests against ORDERED channel
}
