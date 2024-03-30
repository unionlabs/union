pragma solidity ^0.8.23;

import "solady/utils/LibString.sol";
import "solidity-bytes-utils/BytesLib.sol";

import {CometblsHelp} from "../../../contracts/lib/CometblsHelp.sol";
import {CometblsClient} from "../../../contracts/clients/CometblsClientV2.sol";
import {IBCHandler} from "../../../contracts/core/25-handler/IBCHandler.sol";
import {IBCConnection} from
    "../../../contracts/core/03-connection/IBCConnection.sol";
import {IBCClient} from "../../../contracts/core/02-client/IBCClient.sol";
import {IBCChannelHandshake} from
    "../../../contracts/core/04-channel/IBCChannelHandshake.sol";
import {IIBCPacket} from "../../../contracts/core/04-channel/IIBCPacket.sol";
import {
    IBCPacket,
    IBCPacketLib
} from "../../../contracts/core/04-channel/IBCPacket.sol";
import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {IbcCoreClientV1Height as ClientHeight} from
    "../../../contracts/proto/MockClient.sol";
import {
    IbcCoreConnectionV1ConnectionEnd as ConnectionEnd,
    IbcCoreConnectionV1Counterparty as ConnectionCounterparty,
    IbcCoreConnectionV1GlobalEnums as ConnectionEnums
} from "../../../contracts/proto/ibc/core/connection/v1/connection.sol";
import {IbcCoreChannelV1Packet} from
    "../../../contracts/proto/ibc/core/channel/v1/channel.sol";
import {ILightClient} from "../../../contracts/core/02-client/ILightClient.sol";
import {IBCCommitment} from "../../../contracts/core/24-host/IBCCommitment.sol";
import {IbcCoreCommitmentV1MerklePrefix as CommitmentMerklePrefix} from
    "../../../contracts/proto/ibc/core/commitment/v1/commitment.sol";
import {IbcCoreClientV1Height} from
    "../../../contracts/proto/ibc/core/client/v1/client.sol";
import {TendermintTypesSignedHeader} from
    "../../../contracts/proto/tendermint/types/canonical.sol";
import {
    TendermintTypesCommit,
    TendermintTypesHeader,
    TendermintTypesSignedHeader,
    TendermintVersionConsensus,
    TendermintTypesCommitSig,
    TendermintTypesBlockID,
    TendermintTypesPartSetHeader
} from "../../../contracts/proto/tendermint/types/types.sol";
import
    "../../../contracts/proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "../../../contracts/core/04-channel/IBCChannelTypes.sol";

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

contract TestCometblsClient is CometblsClient {
    uint256 validMembership = 0;

    function pushValidMembership() public {
        validMembership += 1;
    }

    uint256 validProof = 0;

    function pushValidProof() public {
        validProof += 1;
    }

    constructor(address ibcHandler_) CometblsClient(ibcHandler_) {}

    function verifyProof(
        uint256[8] memory proof,
        uint256[2] memory proofCommitment,
        uint256[2] calldata proofCommitmentPOK,
        uint256[2] calldata input
    ) external override returns (bool) {
        bool ok = validProof > 0;
        if (validProof > 0) {
            validProof -= 1;
        }
        return ok;
    }

    function verifyMembership(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height,
        uint64 delayPeriodTime,
        uint64 delayPeriodBlocks,
        bytes calldata proof,
        bytes memory prefix,
        bytes calldata path,
        bytes calldata value
    ) external override returns (bool) {
        bool ok = validMembership > 0;
        if (validMembership > 0) {
            validMembership -= 1;
        }
        return ok;
    }

    function verifyNonMembership(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height,
        uint64 delayPeriodTime,
        uint64 delayPeriodBlocks,
        bytes calldata proof,
        bytes calldata prefix,
        bytes calldata path
    ) external override returns (bool) {
        bool ok = validMembership > 0;
        if (validMembership > 0) {
            validMembership -= 1;
        }
        return ok;
    }
}

contract IBCPacketHandlerTest is TestPlus {
    using CometblsHelp for *;
    using BytesLib for *;
    using LibString for *;
    using ConnectionCounterparty for *;
    using {parseChannelIdMemory} for string;

    string constant CLIENT_TYPE = "mock";

    bytes32 constant ARBITRARY_INITIAL_NEXT_VALIDATORS =
        hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60";
    bytes32 constant ARBITRARY_INITIAL_APP_HASH =
        hex"A8158610DD6858F3D26149CC0DB3339ABD580EA217DE0A151C9C451DED418E35";
    string constant CHAIN_ID = "testnet-1337";
    uint64 constant LATEST_HEIGHT = 0x1337;
    uint64 constant LATEST_TIMESTAMP = 0xCAFEBABE;

    uint64 constant LOCAL_HEIGHT = 0xC0DEC0DEC0DEC0DE;
    uint64 constant LOCAL_TIMESTAMP = 0xDEADBEEFDEADBEEF;

    bytes constant ARBITRARY_ZKP =
        hex"195562CC376E9265A7FD89A086855C100173B717B0DEA58AC9F50120E9CBDD7402D59ADAC8A274C5DDB199915B03B5CFB7A91032A71723876F946A7662135D4912EB1FAD1FCA5E88AD1D9097870391D1D477F4CD2A26F27DB3CFC8B511922C482F374A4821BEE34818589A052995CC5994CE787538207F1BA0D595890EB96D751D947274566F6338FC14BB1728C9E42F47F9D47A8A7F46CFA341D3EC71F0A8E80ECDAA9E38B4D6090989B165E536C4332BDF470E860D85001362EC7B369DE0092FD13C85FE2A16247E574B759B7B8EBFE8C7ED19CE7520A693BD09FD604CA54E2FA277AC176ACEC9626313DA7022E8B8DB599E1B02C25DA90AD508AA315DA67C0EAF8A0F41C4CDC897A4941F3BFA7D0E0C2BDD3030D5B0025FB4030A31C886F417B2509E9ECFEA86AA22F75402599E72C21623E9C32A499D7B14B6DBC3A1251E119244B7DC12B54A74FBC3B23E7954435491D89AFA7ABF6F07E1DADE0B28F0DA1978EC72A2C2C0F1FE8DEDA8DD8DDA7E82454618C3DFF1341C9901456F7E656A";

    IBCHandlerFake handler;
    TestCometblsClient client;
    MockApp app;

    string clientId;
    string connectionId;
    ChannelId channelId;

    function setUp() public {
        handler = new IBCHandlerFake();
        client = new TestCometblsClient(address(handler));
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
        IBCMsgs.MsgConnectionOpenInit memory msg_init =
            MsgMocks.connectionOpenInit(clientId);
        connectionId = handler.connectionOpenInit(msg_init);
        IBCMsgs.MsgConnectionOpenAck memory msg_ack =
            MsgMocks.connectionOpenAck(clientId, connectionId, LATEST_HEIGHT);
        client.pushValidMembership();
        client.pushValidMembership();
        handler.connectionOpenAck(msg_ack);
    }

    function setupChannel() internal {
        IBCMsgs.MsgChannelOpenInit memory msg_init =
            MsgMocks.channelOpenInit(connectionId, address(app).toHexString());
        channelId = handler.channelOpenInit(msg_init);
        IBCMsgs.MsgChannelOpenAck memory msg_ack = MsgMocks.channelOpenAck(
            address(app).toHexString(), channelId, LATEST_HEIGHT
        );
        client.pushValidMembership();
        handler.channelOpenAck(msg_ack);
    }

    function makeHeader(
        uint64 height,
        uint64 timestamp
    )
        internal
        pure
        returns (UnionIbcLightclientsCometblsV1LightHeader.Data memory)
    {
        return UnionIbcLightclientsCometblsV1LightHeader.Data({
            height: int64(height),
            time: Timestamp.Data({secs: int64(timestamp), nanos: 0}),
            validators_hash: hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60",
            next_validators_hash: hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60",
            app_hash: hex"983EF85676937CEC783601B5B50865733A72C3DF88E4CC0B3F11C108C9688459"
        });
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
            address(app).toHexString(),
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
            address(app).toHexString(),
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
                IBCCommitment.packetCommitmentKey(
                    address(app).toHexString(), channelId, 1
                )
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
        vm.expectRevert(IBCPacketLib.ErrUnauthorized.selector);
        vm.prank(malicious);
        handler.sendPacket(
            address(app).toHexString(),
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
        uint64 sequenceBefore =
            handler.nextSequenceSends(address(app).toHexString(), channelId);
        vm.prank(address(app));
        handler.sendPacket(
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        uint64 sequenceAfter =
            handler.nextSequenceSends(address(app).toHexString(), channelId);
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
        vm.expectRevert(IBCPacketLib.ErrInvalidTimeoutHeight.selector);
        handler.sendPacket(
            address(app).toHexString(),
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
        vm.expectRevert(IBCPacketLib.ErrInvalidTimeoutTimestamp.selector);
        handler.sendPacket(
            address(app).toHexString(),
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
        client.pushValidMembership();
        vm.prank(relayer);
        handler.recvPacket(
            MsgMocks.packetRecv(
                address(app).toHexString(),
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
            address(app).toHexString(),
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp,
            payload
        );
        assertEq(
            handler.packetReceipts(
                msg_.packet.destination_port,
                msg_.packet.destination_channel.parseChannelIdMemory(),
                msg_.packet.sequence
            ),
            0
        );
        client.pushValidMembership();
        vm.prank(relayer);
        handler.recvPacket(msg_);
        assertEq(
            handler.packetReceipts(
                msg_.packet.destination_port,
                msg_.packet.destination_channel.parseChannelIdMemory(),
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
        client.pushValidMembership();
        IBCMsgs.MsgPacketRecv memory msg_ = MsgMocks.packetRecv(
            address(app).toHexString(),
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp,
            payload
        );
        vm.prank(relayer);
        handler.recvPacket(msg_);
        client.pushValidMembership();
        vm.expectRevert(IBCPacketLib.ErrPacketAlreadyReceived.selector);
        vm.prank(relayer);
        handler.recvPacket(msg_);
    }

    function test_recvPacket_timeoutHeight(
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
        client.pushValidMembership();
        vm.expectRevert(IBCPacketLib.ErrHeightTimeout.selector);
        vm.prank(relayer);
        handler.recvPacket(
            MsgMocks.packetRecv(
                address(app).toHexString(),
                channelId,
                LATEST_HEIGHT,
                LOCAL_HEIGHT - timeoutHeight,
                timeoutTimestamp,
                payload
            )
        );
    }

    function test_recvPacket_timeoutTimestamp(
        address relayer,
        bytes memory payload,
        uint64 timeoutHeight,
        uint32 timeoutTimestamp
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(0 < timeoutHeight && timeoutHeight > vm.getBlockNumber());
        vm.assume(timeoutTimestamp > 0);
        client.pushValidMembership();
        vm.expectRevert(IBCPacketLib.ErrTimestampTimeout.selector);
        vm.prank(relayer);
        handler.recvPacket(
            MsgMocks.packetRecv(
                address(app).toHexString(),
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
        vm.expectRevert(IBCPacketLib.ErrInvalidProof.selector);
        vm.prank(relayer);
        handler.recvPacket(
            MsgMocks.packetRecv(
                address(app).toHexString(),
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
            address(app).toHexString(),
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp,
            payload
        );
        msg_.packet.source_port = "invalid";
        client.pushValidMembership();
        vm.expectRevert(
            IBCPacketLib.ErrSourceAndCounterpartyPortMismatch.selector
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
            address(app).toHexString(),
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp,
            payload
        );
        msg_.packet.source_channel = "invalid";
        client.pushValidMembership();
        vm.expectRevert(
            IBCPacketLib.ErrSourceAndCounterpartyChannelMismatch.selector
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
            address(app).toHexString(), channelId, sequence, acknowledgement
        );
    }

    function test_writeAcknowledgement_alreadyExist(
        uint64 sequence,
        bytes memory acknowledgement
    ) public {
        vm.assume(acknowledgement.length > 0);
        client.pushValidMembership();
        vm.prank(address(app));
        handler.writeAcknowledgement(
            address(app).toHexString(), channelId, sequence, acknowledgement
        );
        vm.prank(address(app));
        vm.expectRevert(IBCPacketLib.ErrAcknowledgementAlreadyExists.selector);
        handler.writeAcknowledgement(
            address(app).toHexString(), channelId, sequence, acknowledgement
        );
    }

    function test_writeAcknowledgement_emptyAcknowledgement(uint64 sequence)
        public
    {
        vm.prank(address(app));
        vm.expectRevert(IBCPacketLib.ErrAcknowledgementIsEmpty.selector);
        handler.writeAcknowledgement(
            address(app).toHexString(), channelId, sequence, bytes("")
        );
    }

    function test_writeAcknowledgement_unauthorized(
        address malicious,
        uint64 sequence,
        bytes memory acknowledgement
    ) public {
        vm.assume(malicious != address(0) && malicious != address(app));
        vm.assume(acknowledgement.length > 0);
        vm.prank(address(malicious));
        vm.expectRevert(IBCPacketLib.ErrUnauthorized.selector);
        handler.writeAcknowledgement(
            address(app).toHexString(), channelId, sequence, acknowledgement
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
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        client.pushValidMembership();
        vm.prank(relayer);
        handler.acknowledgePacket(
            MsgMocks.packetAck(
                address(app).toHexString(),
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
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        client.pushValidMembership();
        vm.prank(relayer);
        handler.acknowledgePacket(
            MsgMocks.packetAck(
                address(app).toHexString(),
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                timeoutTimestamp,
                payload,
                acknowledgement
            )
        );
        client.pushValidMembership();
        vm.prank(relayer);
        vm.expectRevert(IBCPacketLib.ErrPacketCommitmentNotFound.selector);
        handler.acknowledgePacket(
            MsgMocks.packetAck(
                address(app).toHexString(),
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
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        vm.prank(relayer);
        vm.expectRevert(IBCPacketLib.ErrInvalidProof.selector);
        handler.acknowledgePacket(
            MsgMocks.packetAck(
                address(app).toHexString(),
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
        vm.expectRevert(IBCPacketLib.ErrPacketCommitmentNotFound.selector);
        handler.acknowledgePacket(
            MsgMocks.packetAck(
                address(app).toHexString(),
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
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        vm.prank(relayer);
        vm.expectRevert(IBCPacketLib.ErrInvalidPacketCommitment.selector);
        handler.acknowledgePacket(
            MsgMocks.packetAck(
                address(app).toHexString(),
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
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        client.pushValidMembership();
        IBCMsgs.MsgPacketAcknowledgement memory msg_ = MsgMocks.packetAck(
            address(app).toHexString(),
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
            IBCPacketLib.ErrDestinationAndCounterpartyPortMismatch.selector
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
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        client.pushValidMembership();
        IBCMsgs.MsgPacketAcknowledgement memory msg_ = MsgMocks.packetAck(
            address(app).toHexString(),
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
            IBCPacketLib.ErrDestinationAndCounterpartyChannelMismatch.selector
        );
        handler.acknowledgePacket(msg_);
    }

    // TODO: acknowledgePacket tests against ORDERED channel

    function test_timeoutPacket_payloadTampered(
        address relayer,
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.prank(address(app));
        handler.sendPacket(
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({revision_number: 0, revision_height: 0}),
            0,
            payload
        );
        client.pushValidMembership();
        vm.prank(relayer);
        vm.expectRevert(IBCPacketLib.ErrInvalidPacketCommitment.selector);
        handler.timeoutPacket(
            MsgMocks.packetTimeout(
                address(app).toHexString(),
                channelId,
                LATEST_HEIGHT,
                0,
                0,
                abi.encodePacked(payload, hex"00")
            )
        );
    }

    function test_timeoutPacket_notSent(
        address relayer,
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        client.pushValidMembership();
        vm.prank(relayer);
        vm.expectRevert(IBCPacketLib.ErrPacketCommitmentNotFound.selector);
        handler.timeoutPacket(
            MsgMocks.packetTimeout(
                address(app).toHexString(),
                channelId,
                LATEST_HEIGHT,
                0,
                0,
                payload
            )
        );
    }

    function test_timeoutPacket_invalidDestinationPort(
        address relayer,
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        IBCMsgs.MsgPacketTimeout memory msg_ = MsgMocks.packetTimeout(
            address(app).toHexString(), channelId, LATEST_HEIGHT, 0, 0, payload
        );
        msg_.packet.destination_port = "invalid";
        client.pushValidMembership();
        vm.prank(relayer);
        vm.expectRevert(
            IBCPacketLib.ErrDestinationAndCounterpartyPortMismatch.selector
        );
        handler.timeoutPacket(msg_);
    }

    function test_timeoutPacket_invalidDestinationChannel(
        address relayer,
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        IBCMsgs.MsgPacketTimeout memory msg_ = MsgMocks.packetTimeout(
            address(app).toHexString(), channelId, LATEST_HEIGHT, 0, 0, payload
        );
        msg_.packet.destination_channel = "invalid";
        client.pushValidMembership();
        vm.prank(relayer);
        vm.expectRevert(
            IBCPacketLib.ErrDestinationAndCounterpartyChannelMismatch.selector
        );
        handler.timeoutPacket(msg_);
    }

    function test_timeoutPacket_noTimeout(
        address relayer,
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.prank(address(app));
        handler.sendPacket(
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({revision_number: 0, revision_height: 0}),
            0,
            payload
        );
        client.pushValidMembership();
        vm.prank(relayer);
        vm.expectRevert(IBCPacketLib.ErrPacketWithoutTimeout.selector);
        handler.timeoutPacket(
            MsgMocks.packetTimeout(
                address(app).toHexString(),
                channelId,
                LATEST_HEIGHT,
                0,
                0,
                payload
            )
        );
    }

    function test_timeoutPacket_height_ok(
        address relayer,
        uint64 timeoutHeight,
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > LATEST_HEIGHT + 1);
        vm.prank(address(app));
        handler.sendPacket(
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight - 1
            }),
            0,
            payload
        );
        client.pushValidProof();
        vm.prank(relayer);
        handler.updateClient(
            Cometbls.updateClient(
                clientId,
                makeHeader(timeoutHeight, LATEST_TIMESTAMP + 1),
                LATEST_HEIGHT,
                ARBITRARY_ZKP
            )
        );
        client.pushValidMembership();
        vm.prank(relayer);
        handler.timeoutPacket(
            MsgMocks.packetTimeout(
                address(app).toHexString(),
                channelId,
                timeoutHeight,
                timeoutHeight - 1,
                0,
                payload
            )
        );
    }

    function test_timeoutPacket_alreadyTimedout(
        address relayer,
        uint64 timeoutHeight,
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > LATEST_HEIGHT + 1);
        vm.prank(address(app));
        handler.sendPacket(
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight - 1
            }),
            0,
            payload
        );
        client.pushValidProof();
        vm.prank(relayer);
        handler.updateClient(
            Cometbls.updateClient(
                clientId,
                makeHeader(timeoutHeight, LATEST_TIMESTAMP + 1),
                LATEST_HEIGHT,
                ARBITRARY_ZKP
            )
        );
        client.pushValidMembership();
        vm.prank(relayer);
        handler.timeoutPacket(
            MsgMocks.packetTimeout(
                address(app).toHexString(),
                channelId,
                timeoutHeight,
                timeoutHeight - 1,
                0,
                payload
            )
        );
        client.pushValidMembership();
        vm.prank(relayer);
        vm.expectRevert(IBCPacketLib.ErrPacketCommitmentNotFound.selector);
        handler.timeoutPacket(
            MsgMocks.packetTimeout(
                address(app).toHexString(),
                channelId,
                timeoutHeight,
                timeoutHeight - 1,
                0,
                payload
            )
        );
    }

    function test_timeoutPacket_invalidProof(
        address relayer,
        uint64 timeoutHeight,
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > LATEST_HEIGHT + 1);
        vm.prank(address(app));
        handler.sendPacket(
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight - 1
            }),
            0,
            payload
        );
        client.pushValidProof();
        vm.prank(relayer);
        handler.updateClient(
            Cometbls.updateClient(
                clientId,
                makeHeader(timeoutHeight, LATEST_TIMESTAMP + 1),
                LATEST_HEIGHT,
                ARBITRARY_ZKP
            )
        );
        vm.prank(relayer);
        vm.expectRevert(IBCPacketLib.ErrInvalidProof.selector);
        handler.timeoutPacket(
            MsgMocks.packetTimeout(
                address(app).toHexString(),
                channelId,
                timeoutHeight,
                timeoutHeight - 1,
                0,
                payload
            )
        );
    }

    function test_timeoutPacket_height_notReached(
        address relayer,
        uint64 timeoutHeight,
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > LATEST_HEIGHT + 1);
        vm.prank(address(app));
        handler.sendPacket(
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight - 1
            }),
            0,
            payload
        );
        client.pushValidProof();
        vm.prank(relayer);
        handler.updateClient(
            Cometbls.updateClient(
                clientId,
                makeHeader(timeoutHeight - 1, LATEST_TIMESTAMP + 1),
                LATEST_HEIGHT,
                ARBITRARY_ZKP
            )
        );
        client.pushValidMembership();
        vm.prank(relayer);
        vm.expectRevert(IBCPacketLib.ErrTimeoutHeightNotReached.selector);
        handler.timeoutPacket(
            MsgMocks.packetTimeout(
                address(app).toHexString(),
                channelId,
                timeoutHeight - 1,
                timeoutHeight - 1,
                0,
                payload
            )
        );
    }

    function test_timeoutPacket_timestamp_ok(
        address relayer,
        // avoid overflowing uint64.max
        uint32 timeoutTimestamp,
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(LATEST_TIMESTAMP + 1 < timeoutTimestamp);
        vm.prank(address(app));
        handler.sendPacket(
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({revision_number: 0, revision_height: 0}),
            timeoutTimestamp - 1,
            payload
        );
        client.pushValidProof();
        vm.prank(relayer);
        handler.updateClient(
            Cometbls.updateClient(
                clientId,
                makeHeader(LATEST_HEIGHT + 1, timeoutTimestamp),
                LATEST_HEIGHT,
                ARBITRARY_ZKP
            )
        );
        client.pushValidMembership();
        vm.prank(relayer);
        handler.timeoutPacket(
            MsgMocks.packetTimeout(
                address(app).toHexString(),
                channelId,
                LATEST_HEIGHT + 1,
                0,
                timeoutTimestamp - 1,
                payload
            )
        );
    }

    function test_timeoutPacket_timestamp_notReached(
        address relayer,
        // avoid overflowing uint64.max
        uint32 timeoutTimestamp,
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(LATEST_TIMESTAMP + 1 < timeoutTimestamp);
        vm.prank(address(app));
        handler.sendPacket(
            address(app).toHexString(),
            channelId,
            ClientHeight.Data({revision_number: 0, revision_height: 0}),
            timeoutTimestamp - 1,
            payload
        );
        client.pushValidProof();
        vm.prank(relayer);
        handler.updateClient(
            Cometbls.updateClient(
                clientId,
                makeHeader(LATEST_HEIGHT + 1, timeoutTimestamp - 1),
                LATEST_HEIGHT,
                ARBITRARY_ZKP
            )
        );
        client.pushValidMembership();
        vm.prank(relayer);
        vm.expectRevert(IBCPacketLib.ErrTimeoutTimestampNotReached.selector);
        handler.timeoutPacket(
            MsgMocks.packetTimeout(
                address(app).toHexString(),
                channelId,
                LATEST_HEIGHT + 1,
                0,
                timeoutTimestamp - 1,
                payload
            )
        );
    }
}
