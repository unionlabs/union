pragma solidity ^0.8.23;

import "solady/utils/LibString.sol";
import "solidity-bytes-utils/BytesLib.sol";
import "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/utils/math/Math.sol";

import {CometblsHelp} from "../../../contracts/lib/CometblsHelp.sol";
import {CometblsClient} from "../../../contracts/clients/CometblsClientV2.sol";
import {IBCHandler} from "../../../contracts/core/25-handler/IBCHandler.sol";
import {IBCConnection} from
    "../../../contracts/core/03-connection/IBCConnection.sol";
import {IBCClient} from "../../../contracts/core/02-client/IBCClient.sol";
import {IBCChannelHandshake} from
    "../../../contracts/core/04-channel/IBCChannelHandshake.sol";
import {
    IBCPacket,
    IBCPacketLib
} from "../../../contracts/core/04-channel/IBCPacket.sol";
import {IIBCPacket} from "../../../contracts/core/04-channel/IIBCPacket.sol";
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

import "../TestPlus.sol";

contract IBCHandlerFake is IBCHandler {}

contract TestCometblsClient is CometblsClient {
    uint256 validMembership = 0;

    function pushValidMembership() public {
        validMembership += 1;
    }

    uint256 validProof = 0;

    function pushValidProof() public {
        validProof += 1;
    }

    function internalVerifyZKP(
        bytes calldata,
        string memory,
        bytes32,
        UnionIbcLightclientsCometblsV1LightHeader.Data calldata
    ) internal override returns (bool) {
        bool ok = validProof > 0;
        if (validProof > 0) {
            validProof -= 1;
        }
        return ok;
    }

    function verifyMembership(
        string calldata,
        IbcCoreClientV1Height.Data calldata,
        uint64,
        uint64,
        bytes calldata,
        bytes memory,
        bytes calldata,
        bytes calldata
    ) external override returns (bool) {
        bool ok = validMembership > 0;
        if (validMembership > 0) {
            validMembership -= 1;
        }
        return ok;
    }

    function verifyNonMembership(
        string calldata,
        IbcCoreClientV1Height.Data calldata,
        uint64,
        uint64,
        bytes calldata,
        bytes calldata,
        bytes calldata
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

    string constant CLIENT_TYPE = "mock";

    bytes32 constant ARBITRARY_INITIAL_NEXT_VALIDATORS =
        hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60";
    bytes32 constant ARBITRARY_INITIAL_APP_HASH =
        hex"A8158610DD6858F3D26149CC0DB3339ABD580EA217DE0A151C9C451DED418E35";
    string constant CHAIN_ID = "testnet-1337";
    uint64 constant LATEST_HEIGHT = 0x1337;
    uint64 constant LATEST_TIMESTAMP = 0xCAFEBABE;

    uint64 constant LOCAL_HEIGHT = LATEST_HEIGHT + 0xDEAD;
    uint64 constant LOCAL_TIMESTAMP = LATEST_TIMESTAMP + 0xC0DE;

    bytes constant ARBITRARY_ZKP =
        hex"195562CC376E9265A7FD89A086855C100173B717B0DEA58AC9F50120E9CBDD7402D59ADAC8A274C5DDB199915B03B5CFB7A91032A71723876F946A7662135D4912EB1FAD1FCA5E88AD1D9097870391D1D477F4CD2A26F27DB3CFC8B511922C482F374A4821BEE34818589A052995CC5994CE787538207F1BA0D595890EB96D751D947274566F6338FC14BB1728C9E42F47F9D47A8A7F46CFA341D3EC71F0A8E80ECDAA9E38B4D6090989B165E536C4332BDF470E860D85001362EC7B369DE0092FD13C85FE2A16247E574B759B7B8EBFE8C7ED19CE7520A693BD09FD604CA54E2FA277AC176ACEC9626313DA7022E8B8DB599E1B02C25DA90AD508AA315DA67C0EAF8A0F41C4CDC897A4941F3BFA7D0E0C2BDD3030D5B0025FB4030A31C886F417B2509E9ECFEA86AA22F75402599E72C21623E9C32A499D7B14B6DBC3A1251E119244B7DC12B54A74FBC3B23E7954435491D89AFA7ABF6F07E1DADE0B28F0DA1978EC72A2C2C0F1FE8DEDA8DD8DDA7E82454618C3DFF1341C9901456F7E656A";

    IBCHandlerFake handler;
    TestCometblsClient client;
    MockApp app;

    string clientId;
    string connectionId;
    string channelId;

    function mkWriteAckPacket(
        string memory destinationChannel,
        uint64 sequence
    ) public view returns (IbcCoreChannelV1Packet.Data memory) {
        return IbcCoreChannelV1Packet.Data({
            sequence: sequence,
            source_port: "",
            source_channel: "",
            destination_port: address(app).toHexString(),
            destination_channel: destinationChannel,
            data: bytes(""),
            timeout_height: IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: 0
            }),
            timeout_timestamp: 0
        });
    }

    function setUp() public {
        handler = IBCHandlerFake(
            address(
                new ERC1967Proxy(
                    address(new IBCHandlerFake()),
                    abi.encodeCall(
                        IBCHandler.initialize,
                        (
                            address(new IBCClient()),
                            address(new IBCConnection()),
                            address(new IBCChannelHandshake()),
                            address(new IBCPacket()),
                            address(this)
                        )
                    )
                )
            )
        );
        client = TestCometblsClient(
            address(
                new ERC1967Proxy(
                    address(new TestCometblsClient()),
                    abi.encodeCall(
                        CometblsClient.initialize,
                        (address(handler), address(this))
                    )
                )
            )
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
        uint64 sequenceBefore = uint64(
            uint256(
                handler.commitments(
                    IBCCommitment.nextSequenceSendCommitmentKey(
                        address(app).toHexString(), channelId
                    )
                )
            )
        );
        vm.prank(address(app));
        handler.sendPacket(
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp,
            payload
        );
        uint64 sequenceAfter = uint64(
            uint256(
                handler.commitments(
                    IBCCommitment.nextSequenceSendCommitmentKey(
                        address(app).toHexString(), channelId
                    )
                )
            )
        );
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
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
        client.pushValidMembership();
        vm.prank(relayer);
        handler.recvPacket(
            MsgMocks.packetRecv(
                address(app).toHexString(),
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                timeoutTimestamp * 1e9,
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
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
        IBCMsgs.MsgPacketRecv memory msg_ = MsgMocks.packetRecv(
            address(app).toHexString(),
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp * 1e9,
            payload
        );
        assertEq(
            handler.commitments(
                IBCCommitment.packetReceiptCommitmentKey(
                    msg_.packet.destination_port,
                    msg_.packet.destination_channel,
                    msg_.packet.sequence
                )
            ),
            bytes32(uint256(0))
        );
        client.pushValidMembership();
        vm.prank(relayer);
        handler.recvPacket(msg_);
        assertEq(
            handler.commitments(
                IBCCommitment.packetReceiptCommitmentKey(
                    msg_.packet.destination_port,
                    msg_.packet.destination_channel,
                    msg_.packet.sequence
                )
            ),
            bytes32(uint256(1))
        );
    }

    function test_recvPacket_alreadyReceived(
        address relayer,
        bytes memory payload,
        uint64 timeoutHeight
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > vm.getBlockNumber());
        uint64 timeoutTimestamp = uint64(vm.getBlockTimestamp()) + 1;
        client.pushValidMembership();
        IBCMsgs.MsgPacketRecv memory msg_ = MsgMocks.packetRecv(
            address(app).toHexString(),
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp * 1e9,
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
        uint32 timeoutHeightDelta,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeightDelta > 0);
        vm.assume(timeoutTimestamp > vm.getBlockTimestamp());
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
        client.pushValidMembership();
        vm.prank(relayer);
        vm.expectRevert(IBCPacketLib.ErrHeightTimeout.selector);
        handler.recvPacket(
            MsgMocks.packetRecv(
                address(app).toHexString(),
                channelId,
                LATEST_HEIGHT,
                // If the height is zero it's considered not to be a timeout on height, we need to ensure its properly clamped
                LOCAL_HEIGHT
                    - uint64(Math.min(LOCAL_HEIGHT - 1, timeoutHeightDelta)),
                timeoutTimestamp * 1e9,
                payload
            )
        );
    }

    function test_recvPacket_timeoutTimestamp(
        address relayer,
        bytes memory payload,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(0 < timeoutHeight && timeoutHeight > vm.getBlockNumber());
        vm.assume(timeoutTimestamp < LOCAL_TIMESTAMP);
        client.pushValidMembership();
        vm.expectRevert(IBCPacketLib.ErrTimestampTimeout.selector);
        vm.prank(relayer);
        handler.recvPacket(
            MsgMocks.packetRecv(
                address(app).toHexString(),
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                (LOCAL_TIMESTAMP - timeoutTimestamp) * 1e9,
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
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
        vm.expectRevert(IBCPacketLib.ErrInvalidProof.selector);
        vm.prank(relayer);
        handler.recvPacket(
            MsgMocks.packetRecv(
                address(app).toHexString(),
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                timeoutTimestamp * 1e9,
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
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
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
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
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
            mkWriteAckPacket(channelId, sequence), acknowledgement
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
            mkWriteAckPacket(channelId, sequence), acknowledgement
        );
        vm.prank(address(app));
        vm.expectRevert(IBCPacketLib.ErrAcknowledgementAlreadyExists.selector);
        handler.writeAcknowledgement(
            mkWriteAckPacket(channelId, sequence), acknowledgement
        );
    }

    function test_writeAcknowledgement_emptyAcknowledgement(uint64 sequence)
        public
    {
        vm.prank(address(app));
        vm.expectRevert(IBCPacketLib.ErrAcknowledgementIsEmpty.selector);
        handler.writeAcknowledgement(
            mkWriteAckPacket(channelId, sequence), bytes("")
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
            mkWriteAckPacket(channelId, sequence), acknowledgement
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
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
        vm.prank(address(app));
        handler.sendPacket(
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp * 1e9,
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
                timeoutTimestamp * 1e9,
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
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
        vm.prank(address(app));
        handler.sendPacket(
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp * 1e9,
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
                timeoutTimestamp * 1e9,
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
                timeoutTimestamp * 1e9,
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
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
        vm.prank(address(app));
        handler.sendPacket(
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp * 1e9,
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
                timeoutTimestamp * 1e9,
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
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
        vm.prank(relayer);
        vm.expectRevert(IBCPacketLib.ErrPacketCommitmentNotFound.selector);
        handler.acknowledgePacket(
            MsgMocks.packetAck(
                address(app).toHexString(),
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                timeoutTimestamp * 1e9,
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
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
        vm.prank(address(app));
        handler.sendPacket(
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp * 1e9,
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
                timeoutTimestamp * 1e9,
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
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
        vm.prank(address(app));
        handler.sendPacket(
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp * 1e9,
            payload
        );
        client.pushValidMembership();
        IBCMsgs.MsgPacketAcknowledgement memory msg_ = MsgMocks.packetAck(
            address(app).toHexString(),
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp * 1e9,
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
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
        vm.prank(address(app));
        handler.sendPacket(
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp * 1e9,
            payload
        );
        client.pushValidMembership();
        IBCMsgs.MsgPacketAcknowledgement memory msg_ = MsgMocks.packetAck(
            address(app).toHexString(),
            channelId,
            LATEST_HEIGHT,
            timeoutHeight,
            timeoutTimestamp * 1e9,
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
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.assume(timeoutHeight > LATEST_HEIGHT);
        vm.assume(timeoutTimestamp > LATEST_TIMESTAMP);
        vm.assume(timeoutTimestamp < type(uint64).max / 1e9);
        vm.prank(address(app));
        handler.sendPacket(
            channelId,
            ClientHeight.Data({
                revision_number: 0,
                revision_height: timeoutHeight
            }),
            timeoutTimestamp * 1e9,
            payload
        );
        vm.warp(timeoutTimestamp);
        client.pushValidMembership();
        vm.prank(relayer);
        vm.expectRevert(IBCPacketLib.ErrInvalidPacketCommitment.selector);
        handler.timeoutPacket(
            MsgMocks.packetTimeout(
                address(app).toHexString(),
                channelId,
                LATEST_HEIGHT,
                timeoutHeight,
                timeoutTimestamp * 1e9,
                abi.encodePacked(hex"00", payload)
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
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        vm.prank(address(app));
        uint64 timeoutTimestamp = LATEST_TIMESTAMP + 2;
        handler.sendPacket(
            channelId,
            ClientHeight.Data({revision_number: 0, revision_height: 0}),
            (timeoutTimestamp - 1) * 1e9,
            payload
        );
        vm.warp(timeoutTimestamp);
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
                (timeoutTimestamp - 1) * 1e9,
                payload
            )
        );
    }

    function test_timeoutPacket_timestamp_notReached(
        address relayer,
        bytes memory payload
    ) public {
        vm.assume(relayer != address(0) && relayer != address(app));
        uint64 timeoutTimestamp = LATEST_TIMESTAMP + 2;
        vm.prank(address(app));
        handler.sendPacket(
            channelId,
            ClientHeight.Data({revision_number: 0, revision_height: 0}),
            (timeoutTimestamp - 1) * 1e9,
            payload
        );
        vm.warp(timeoutTimestamp - 1);
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
                (timeoutTimestamp - 1) * 1e9,
                payload
            )
        );
    }
}
