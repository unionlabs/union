pragma solidity ^0.8.23;

import "solidity-bytes-utils/BytesLib.sol";
import "solady/utils/LibString.sol";

import {CometblsClient} from "../../../contracts/clients/CometblsClientV2.sol";
import {IBCChannelLib} from
    "../../../contracts/core/04-channel/IBCChannelHandshake.sol";
import {ILightClient} from "../../../contracts/core/02-client/ILightClient.sol";
import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {
    IbcCoreConnectionV1ConnectionEnd as ConnectionEnd,
    IbcCoreConnectionV1Counterparty as ConnectionCounterparty,
    IbcCoreConnectionV1GlobalEnums as ConnectionEnums
} from "../../../contracts/proto/ibc/core/connection/v1/connection.sol";
import {
    IbcCoreChannelV1Channel as Channel,
    IbcCoreChannelV1GlobalEnums as ChannelEnums
} from "../../../contracts/proto/ibc/core/channel/v1/channel.sol";
import {IbcCoreClientV1Height} from
    "../../../contracts/proto/ibc/core/client/v1/client.sol";
import {IbcCoreCommitmentV1MerklePrefix as CommitmentMerklePrefix} from
    "../../../contracts/proto/ibc/core/commitment/v1/commitment.sol";
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

contract TestCometblsClient is CometblsClient {
    uint256 validProof = 0;

    function pushValidProof() public {
        validProof += 1;
    }

    uint256 validMembership = 0;

    function pushValidMembership() public {
        validMembership += 1;
    }

    constructor(address ibcHandler_) CometblsClient(ibcHandler_) {}

    function verifyZKP(
        bytes calldata zkpBytes,
        string memory chainId,
        bytes32 trustedValidatorsHash,
        UnionIbcLightclientsCometblsV1LightHeader.Data memory header
    ) public override returns (bool) {
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

contract IBCChannelHandlerTest is TestPlus {
    using BytesLib for *;
    using ConnectionCounterparty for *;
    using LibString for *;

    string constant CLIENT_TYPE = "mock";

    bytes32 constant ARBITRARY_INITIAL_APP_HASH =
        hex"A8158610DD6858F3D26149CC0DB3339ABD580EA217DE0A151C9C451DED418E35";

    IBCHandler_Testable handler;
    TestCometblsClient client;
    MockApp app;
    string portId;

    event ChannelOpenInit(
        string channelId,
        string connectionId,
        string portId,
        string counterpartyPortId
    );

    event ChannelOpenTry(
        string channelId,
        string connectionId,
        string portId,
        string counterpartyPortId,
        string version
    );

    function setUp() public {
        handler = new IBCHandler_Testable();
        client = new TestCometblsClient(address(handler));
        handler.registerClient(CLIENT_TYPE, client);
        app = new MockApp();
        portId = address(app).toHexString();
    }

    function getValidHeader()
        internal
        pure
        returns (TendermintTypesSignedHeader.Data memory)
    {
        TendermintTypesHeader.Data memory header = TendermintTypesHeader.Data({
            version: TendermintVersionConsensus.Data({block: 11, app: 0}),
            chain_id: "union-devnet-1",
            height: 139,
            time: Timestamp.Data({secs: 1691496777, nanos: 793918988}),
            last_block_id: TendermintTypesBlockID.Data({
                hash: hex"80DF3A892BF2586E3B22201D2AC5A65EDAB66ECE7BB6F51077F3B50CCE7526E1",
                part_set_header: TendermintTypesPartSetHeader.Data({
                    total: 1,
                    hash: hex"0468D541CAD891D571E2AD1DD9F43480993BDF18A1016F4C956555A417EFE681"
                })
            }),
            last_commit_hash: hex"DA6FCBD48131808D58B54E8B44737AB2B6F3A3DD1AFF946D0F6CEFD25306FD48",
            data_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            validators_hash: hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60",
            next_validators_hash: hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60",
            consensus_hash: hex"048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
            app_hash: hex"983EF85676937CEC783601B5B50865733A72C3DF88E4CC0B3F11C108C9688459",
            last_results_hash: hex"357B78587B9CD4469F1F63C29B96EAC1D7F643520B97D396B20A20505122AA01",
            evidence_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            proposer_address: hex"4CE57693C82B50F830731DAB14FA759327762456"
        });
        return TendermintTypesSignedHeader.Data({
            header: header,
            // NOTE: validators are signing the block root which is computed
            // from the above TendermintTypesHeader field. Relayers can tamper
            // the commit but the client ensure that the block_id.hash matches
            // hash(header). Signatures are not required as the ZKP is a proof
            // that the validators signed the correct hash.
            commit: TendermintTypesCommit.Data({
                height: header.height,
                round: 0,
                block_id: TendermintTypesBlockID.Data({
                    hash: hex"90548CD1E2BA8603261FE6400ADFD97EA48150CBD5B24B9828B2542BAB9E27EC",
                    part_set_header: TendermintTypesPartSetHeader.Data({
                        total: 1,
                        hash: hex"153E8B1F5B189A140FE5DA85DAB72FBD4A1DFA7E69C6FE5CE1FD66F0CCB5F6A1"
                    })
                }),
                signatures: new TendermintTypesCommitSig.Data[](0)
            })
        });
    }

    function assumeValidProofHeight(uint64 proofHeight) internal {
        vm.assume(
            0 < proofHeight
                && proofHeight < uint64(getValidHeader().header.height)
        );
    }

    function createClient(uint64 proofHeight)
        internal
        returns (string memory)
    {
        assumeValidProofHeight(proofHeight);
        TendermintTypesSignedHeader.Data memory signedHeader = getValidHeader();
        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            proofHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );
        return handler.createClient(m);
    }

    function test_handshake_init_ack_ok(uint64 proofHeight) public {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenInit memory msg_init =
            MsgMocks.channelOpenInit(connId, portId);
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenInit("", "", "", "");
        string memory channelId = handler.channelOpenInit(msg_init);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenAck memory msg_ack =
            MsgMocks.channelOpenAck(portId, channelId, proofHeight);
        client.pushValidMembership();
        handler.channelOpenAck(msg_ack);
    }

    function test_handshake_init_ack_invalidProof(uint64 proofHeight) public {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenInit memory msg_init =
            MsgMocks.channelOpenInit(connId, portId);
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenInit("", "", "", "");
        string memory channelId = handler.channelOpenInit(msg_init);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenAck memory msg_ack =
            MsgMocks.channelOpenAck(portId, channelId, proofHeight);
        vm.expectRevert(IBCChannelLib.ErrInvalidProof.selector);
        handler.channelOpenAck(msg_ack);
    }

    function test_handshake_init_noHop(uint64 proofHeight) public {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenInit memory msg_init =
            MsgMocks.channelOpenInit(connId, portId);
        msg_init.channel.connection_hops = new string[](0);
        vm.expectRevert(IBCChannelLib.ErrConnNotSingleHop.selector);
        handler.channelOpenInit(msg_init);
    }

    function test_handshake_init_noConnection(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);

        IBCMsgs.MsgChannelOpenInit memory msg_init =
            MsgMocks.channelOpenInit("invalid-connection", portId);
        vm.expectRevert(IBCChannelLib.ErrInvalidConnectionState.selector);
        handler.channelOpenInit(msg_init);
    }

    function test_handshake_init_unsupportedFeature(uint64 proofHeight)
        public
    {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenInit memory msg_init =
            MsgMocks.channelOpenInit(connId, portId);
        msg_init.channel.ordering = ChannelEnums.Order.ORDER_NONE_UNSPECIFIED;
        vm.expectRevert(IBCChannelLib.ErrUnsupportedFeature.selector);
        handler.channelOpenInit(msg_init);
    }

    function test_handshake_init_notInit(uint64 proofHeight) public {
        vm.assume(proofHeight > 0);
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenInit memory msg_init =
            MsgMocks.channelOpenInit(connId, portId);
        msg_init.channel.state = ChannelEnums.State.STATE_OPEN;
        vm.expectRevert(IBCChannelLib.ErrInvalidChannelState.selector);
        handler.channelOpenInit(msg_init);
    }

    function test_handshake_init_nonEmptyCounterpartyChannel(uint64 proofHeight)
        public
    {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenInit memory msg_init =
            MsgMocks.channelOpenInit(connId, portId);
        msg_init.channel.counterparty.channel_id = "invalid";
        vm.expectRevert(IBCChannelLib.ErrCounterpartyChannelNotEmpty.selector);
        handler.channelOpenInit(msg_init);
    }

    function test_handshake_init_ack_close_init_ok(uint64 proofHeight) public {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenInit memory msg_init =
            MsgMocks.channelOpenInit(connId, portId);
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenInit("", "", "", "");
        string memory channelId = handler.channelOpenInit(msg_init);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenAck memory msg_ack =
            MsgMocks.channelOpenAck(portId, channelId, proofHeight);
        client.pushValidMembership();
        handler.channelOpenAck(msg_ack);

        IBCMsgs.MsgChannelCloseInit memory msg_close =
            MsgMocks.channelCloseInit(portId, channelId);
        handler.channelCloseInit(msg_close);
    }

    function test_handshake_init_ack_close_confirm_ok(uint64 proofHeight)
        public
    {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenInit memory msg_init =
            MsgMocks.channelOpenInit(connId, portId);
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenInit("", "", "", "");
        string memory channelId = handler.channelOpenInit(msg_init);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenAck memory msg_ack =
            MsgMocks.channelOpenAck(portId, channelId, proofHeight);
        client.pushValidMembership();
        handler.channelOpenAck(msg_ack);

        IBCMsgs.MsgChannelCloseConfirm memory msg_close =
            MsgMocks.channelCloseConfirm(portId, channelId, proofHeight);
        client.pushValidMembership();
        handler.channelCloseConfirm(msg_close);
    }

    function test_handshake_init_ack_close_confirm_invalidProof(
        uint64 proofHeight
    ) public {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenInit memory msg_init =
            MsgMocks.channelOpenInit(connId, portId);
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenInit("", "", "", "");
        string memory channelId = handler.channelOpenInit(msg_init);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenAck memory msg_ack =
            MsgMocks.channelOpenAck(portId, channelId, proofHeight);
        client.pushValidMembership();
        handler.channelOpenAck(msg_ack);

        IBCMsgs.MsgChannelCloseConfirm memory msg_close =
            MsgMocks.channelCloseConfirm(portId, channelId, proofHeight);
        vm.expectRevert(IBCChannelLib.ErrInvalidProof.selector);
        handler.channelCloseConfirm(msg_close);
    }

    function test_handshake_try_confirm_ok(uint64 proofHeight) public {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenTry memory msg_try =
            MsgMocks.channelOpenTry(connId, portId, proofHeight);
        client.pushValidMembership();
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenTry("", "", "", "", "");
        string memory channelId = handler.channelOpenTry(msg_try);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenConfirm memory msg_confirm =
            MsgMocks.channelOpenConfirm(portId, channelId, proofHeight);
        client.pushValidMembership();
        handler.channelOpenConfirm(msg_confirm);
    }

    function test_handshake_try_confirm_invalidProof(uint64 proofHeight)
        public
    {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenTry memory msg_try =
            MsgMocks.channelOpenTry(connId, portId, proofHeight);
        client.pushValidMembership();
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenTry("", "", "", "", "");
        string memory channelId = handler.channelOpenTry(msg_try);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenConfirm memory msg_confirm =
            MsgMocks.channelOpenConfirm(portId, channelId, proofHeight);
        vm.expectRevert(IBCChannelLib.ErrInvalidProof.selector);
        handler.channelOpenConfirm(msg_confirm);
    }

    function test_handshake_try_invalidProof(uint64 proofHeight) public {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenTry memory msg_try =
            MsgMocks.channelOpenTry(connId, portId, proofHeight);
        vm.expectRevert(IBCChannelLib.ErrInvalidProof.selector);
        handler.channelOpenTry(msg_try);
    }

    function test_handshake_try_notTryOpen(
        uint64 proofHeight,
        string memory portId
    ) public {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenTry memory msg_try =
            MsgMocks.channelOpenTry(connId, portId, proofHeight);
        msg_try.channel.state = ChannelEnums.State.STATE_INIT;

        client.pushValidMembership();
        vm.expectRevert(IBCChannelLib.ErrInvalidChannelState.selector);
        handler.channelOpenTry(msg_try);
    }

    function test_handshake_try_confirm_close_init_ok(uint64 proofHeight)
        public
    {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenTry memory msg_try =
            MsgMocks.channelOpenTry(connId, portId, proofHeight);
        client.pushValidMembership();
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenTry("", "", "", "", "");
        string memory channelId = handler.channelOpenTry(msg_try);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenConfirm memory msg_confirm =
            MsgMocks.channelOpenConfirm(portId, channelId, proofHeight);
        client.pushValidMembership();
        handler.channelOpenConfirm(msg_confirm);

        IBCMsgs.MsgChannelCloseInit memory msg_close =
            MsgMocks.channelCloseInit(portId, channelId);
        handler.channelCloseInit(msg_close);
    }

    function test_handshake_try_confirm_close_confirm_ok(uint64 proofHeight)
        public
    {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenTry memory msg_try =
            MsgMocks.channelOpenTry(connId, portId, proofHeight);

        client.pushValidMembership();
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenTry("", "", "", "", "");
        string memory channelId = handler.channelOpenTry(msg_try);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenConfirm memory msg_confirm =
            MsgMocks.channelOpenConfirm(portId, channelId, proofHeight);
        client.pushValidMembership();
        handler.channelOpenConfirm(msg_confirm);

        IBCMsgs.MsgChannelCloseConfirm memory msg_close =
            MsgMocks.channelCloseConfirm(portId, channelId, proofHeight);
        client.pushValidMembership();
        handler.channelCloseConfirm(msg_close);
    }

    function test_handshake_try_confirm_close_confirm_invalidProof(
        uint64 proofHeight
    ) public {
        (, string memory connId) = setupConnection(proofHeight);

        IBCMsgs.MsgChannelOpenTry memory msg_try =
            MsgMocks.channelOpenTry(connId, portId, proofHeight);

        client.pushValidMembership();
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenTry("", "", "", "", "");
        string memory channelId = handler.channelOpenTry(msg_try);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenConfirm memory msg_confirm =
            MsgMocks.channelOpenConfirm(portId, channelId, proofHeight);
        client.pushValidMembership();
        handler.channelOpenConfirm(msg_confirm);

        IBCMsgs.MsgChannelCloseConfirm memory msg_close =
            MsgMocks.channelCloseConfirm(portId, channelId, proofHeight);
        vm.expectRevert(IBCChannelLib.ErrInvalidProof.selector);
        handler.channelCloseConfirm(msg_close);
    }

    function setupConnection(uint64 proofHeight)
        internal
        returns (string memory, string memory)
    {
        string memory clientId = createClient(proofHeight);
        IBCMsgs.MsgConnectionOpenInit memory msg_init =
            MsgMocks.connectionOpenInit(clientId);
        string memory connId = handler.connectionOpenInit(msg_init);
        IBCMsgs.MsgConnectionOpenAck memory msg_ack =
            MsgMocks.connectionOpenAck(clientId, connId, proofHeight);
        client.pushValidMembership();
        client.pushValidMembership();
        handler.connectionOpenAck(msg_ack);
        return (clientId, connId);
    }
}
