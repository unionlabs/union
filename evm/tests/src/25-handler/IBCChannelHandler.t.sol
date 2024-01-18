pragma solidity ^0.8.23;

import "solidity-bytes-utils/BytesLib.sol";

import {IMembershipVerifier} from "../../../contracts/core/IMembershipVerifier.sol";
import {IZKVerifierV2} from "../../../contracts/core/IZKVerifierV2.sol";
import {CometblsClient} from "../../../contracts/clients/CometblsClientV2.sol";
import {ILightClient} from "../../../contracts/core/02-client/ILightClient.sol";
import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {IbcCoreConnectionV1ConnectionEnd as ConnectionEnd, IbcCoreConnectionV1Counterparty as ConnectionCounterparty, IbcCoreConnectionV1GlobalEnums as ConnectionEnums} from "../../../contracts/proto/ibc/core/connection/v1/connection.sol";
import {IbcCoreChannelV1Channel as Channel, IbcCoreChannelV1GlobalEnums as ChannelEnums} from "../../../contracts/proto/ibc/core/channel/v1/channel.sol";
import {IbcCoreCommitmentV1MerklePrefix as CommitmentMerklePrefix} from "../../../contracts/proto/ibc/core/commitment/v1/commitment.sol";
import {TendermintTypesSignedHeader} from "../../../contracts/proto/tendermint/types/canonical.sol";
import {TendermintTypesCommit, TendermintTypesHeader, TendermintTypesSignedHeader, TendermintVersionConsensus, TendermintTypesCommitSig, TendermintTypesBlockID, TendermintTypesPartSetHeader} from "../../../contracts/proto/tendermint/types/types.sol";

import "../TestPlus.sol";

contract IBCChannelHandlerTest is TestPlus {
    using ConnectionCounterparty for ConnectionCounterparty.Data;

    IBCHandler_Testable handler;
    ILightClient client;
    MockApp app;
    string constant CLIENT_TYPE = "mock";

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

    constructor() {
        handler = new IBCHandler_Testable();
        client = new MockClient(address(handler));
        app = new MockApp();
        handler.registerClient(CLIENT_TYPE, client);
    }

    function test_handshake_init_ack_ok(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);
        (
            string memory clientId,
            string memory connId
        ) = setupConnection_init_ack(proofHeight);

        handler.bindPort(portId, address(app));

        IBCMsgs.MsgChannelOpenInit memory msg_init = MsgMocks.channelOpenInit(
            connId,
            portId
        );
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenInit("", "", "", "");
        string memory channelId = handler.channelOpenInit(msg_init);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenAck memory msg_ack = MsgMocks.channelOpenAck(
            portId,
            channelId,
            proofHeight
        );
        handler.channelOpenAck(msg_ack);
    }

    function test_handshake_init_noHop(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);
        (, string memory connId) = setupConnection_init_ack(proofHeight);
        handler.bindPort(portId, address(app));

        IBCMsgs.MsgChannelOpenInit memory msg_init = MsgMocks.channelOpenInit(
            connId,
            portId
        );
        msg_init.channel.connection_hops = new string[](0);
        vm.expectRevert("channelOpenInit: connection must have a single hop");
        handler.channelOpenInit(msg_init);
    }

    function test_handshake_init_noConnection(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);
        handler.bindPort(portId, address(app));

        IBCMsgs.MsgChannelOpenInit memory msg_init = MsgMocks.channelOpenInit(
            "invalid-connection",
            portId
        );
        vm.expectRevert(
            "channelOpenInit: single version must be negotiated on connection before opening channel"
        );
        handler.channelOpenInit(msg_init);
    }

    function test_handshake_init_unsupportedFeature(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);
        (, string memory connId) = setupConnection_init_ack(proofHeight);
        handler.bindPort(portId, address(app));

        IBCMsgs.MsgChannelOpenInit memory msg_init = MsgMocks.channelOpenInit(
            connId,
            portId
        );
        msg_init.channel.ordering = ChannelEnums.Order.ORDER_NONE_UNSPECIFIED;
        vm.expectRevert("channelOpenInit: feature not supported");
        handler.channelOpenInit(msg_init);
    }

    function test_handshake_init_notInit(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);
        (, string memory connId) = setupConnection_init_ack(proofHeight);
        handler.bindPort(portId, address(app));

        IBCMsgs.MsgChannelOpenInit memory msg_init = MsgMocks.channelOpenInit(
            connId,
            portId
        );
        msg_init.channel.state = ChannelEnums.State.STATE_OPEN;
        vm.expectRevert("channelOpenInit: channel state is not INIT");
        handler.channelOpenInit(msg_init);
    }

    function test_handshake_init_nonEmptyCounterpartyChannel(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);
        (, string memory connId) = setupConnection_init_ack(proofHeight);
        handler.bindPort(portId, address(app));

        IBCMsgs.MsgChannelOpenInit memory msg_init = MsgMocks.channelOpenInit(
            connId,
            portId
        );
        msg_init.channel.counterparty.channel_id = "invalid";
        vm.expectRevert(
            "channelOpenInit: counterparty channel_id must be empty"
        );
        handler.channelOpenInit(msg_init);
    }

    function test_handshake_init_ack_close_init_ok(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);
        (
            string memory clientId,
            string memory connId
        ) = setupConnection_init_ack(proofHeight);

        handler.bindPort(portId, address(app));

        IBCMsgs.MsgChannelOpenInit memory msg_init = MsgMocks.channelOpenInit(
            connId,
            portId
        );
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenInit("", "", "", "");
        string memory channelId = handler.channelOpenInit(msg_init);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenAck memory msg_ack = MsgMocks.channelOpenAck(
            portId,
            channelId,
            proofHeight
        );
        handler.channelOpenAck(msg_ack);

        IBCMsgs.MsgChannelCloseInit memory msg_close = MsgMocks
            .channelCloseInit(portId, channelId);
        handler.channelCloseInit(msg_close);
    }

    function test_handshake_init_ack_close_confirm_ok(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);
        (
            string memory clientId,
            string memory connId
        ) = setupConnection_init_ack(proofHeight);

        handler.bindPort(portId, address(app));

        IBCMsgs.MsgChannelOpenInit memory msg_init = MsgMocks.channelOpenInit(
            connId,
            portId
        );
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenInit("", "", "", "");
        string memory channelId = handler.channelOpenInit(msg_init);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenAck memory msg_ack = MsgMocks.channelOpenAck(
            portId,
            channelId,
            proofHeight
        );
        handler.channelOpenAck(msg_ack);

        IBCMsgs.MsgChannelCloseConfirm memory msg_close = MsgMocks
            .channelCloseConfirm(portId, channelId, proofHeight);
        handler.channelCloseConfirm(msg_close);
    }

    function test_handshake_try_confirm_ok(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);
        (
            string memory clientId,
            string memory connId
        ) = setupConnection_try_confirm(proofHeight);
        handler.bindPort(portId, address(app));

        IBCMsgs.MsgChannelOpenTry memory msg_try = MsgMocks.channelOpenTry(
            connId,
            portId,
            proofHeight
        );
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenTry("", "", "", "", "");
        string memory channelId = handler.channelOpenTry(msg_try);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenConfirm memory msg_confirm = MsgMocks
            .channelOpenConfirm(portId, channelId, proofHeight);
        handler.channelOpenConfirm(msg_confirm);
    }

    function test_handshake_try_notTryOpen(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);
        (
            string memory clientId,
            string memory connId
        ) = setupConnection_try_confirm(proofHeight);
        handler.bindPort(portId, address(app));

        IBCMsgs.MsgChannelOpenTry memory msg_try = MsgMocks.channelOpenTry(
            connId,
            portId,
            proofHeight
        );
        msg_try.channel.state = ChannelEnums.State.STATE_INIT;
        vm.expectRevert("channelOpenTry: channel state is not TRYOPEN");
        handler.channelOpenTry(msg_try);
    }

    function test_handshake_try_confirm_close_init_ok(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);
        (
            string memory clientId,
            string memory connId
        ) = setupConnection_try_confirm(proofHeight);
        handler.bindPort(portId, address(app));

        IBCMsgs.MsgChannelOpenTry memory msg_try = MsgMocks.channelOpenTry(
            connId,
            portId,
            proofHeight
        );
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenTry("", "", "", "", "");
        string memory channelId = handler.channelOpenTry(msg_try);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenConfirm memory msg_confirm = MsgMocks
            .channelOpenConfirm(portId, channelId, proofHeight);
        handler.channelOpenConfirm(msg_confirm);

        IBCMsgs.MsgChannelCloseInit memory msg_close = MsgMocks
            .channelCloseInit(portId, channelId);
        handler.channelCloseInit(msg_close);
    }

    function test_handshake_try_confirm_close_confirm_ok(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);
        (
            string memory clientId,
            string memory connId
        ) = setupConnection_try_confirm(proofHeight);
        handler.bindPort(portId, address(app));

        IBCMsgs.MsgChannelOpenTry memory msg_try = MsgMocks.channelOpenTry(
            connId,
            portId,
            proofHeight
        );
        vm.expectEmit(false, false, false, false);
        emit ChannelOpenTry("", "", "", "", "");
        string memory channelId = handler.channelOpenTry(msg_try);

        assertEq(
            handler.capabilities(string.concat(portId, "/", channelId)),
            address(app)
        );

        IBCMsgs.MsgChannelOpenConfirm memory msg_confirm = MsgMocks
            .channelOpenConfirm(portId, channelId, proofHeight);
        handler.channelOpenConfirm(msg_confirm);

        IBCMsgs.MsgChannelCloseConfirm memory msg_close = MsgMocks
            .channelCloseConfirm(portId, channelId, proofHeight);
        handler.channelCloseConfirm(msg_close);
    }

    function setupConnection_init_ack(
        uint64 proofHeight
    ) internal returns (string memory clientId, string memory connId) {
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(
            CLIENT_TYPE,
            proofHeight
        );
        clientId = handler.createClient(m);
        IBCMsgs.MsgConnectionOpenInit memory msg_init = MsgMocks
            .connectionOpenInit(clientId);
        connId = handler.connectionOpenInit(msg_init);
        IBCMsgs.MsgConnectionOpenAck memory msg_ack = MsgMocks
            .connectionOpenAck(clientId, connId, proofHeight);
        handler.connectionOpenAck(msg_ack);
    }

    function setupConnection_try_confirm(
        uint64 proofHeight
    ) internal returns (string memory clientId, string memory connId) {
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(
            CLIENT_TYPE,
            proofHeight
        );
        clientId = handler.createClient(m);
        IBCMsgs.MsgConnectionOpenTry memory msg_try = MsgMocks
            .connectionOpenTry(clientId, proofHeight);
        connId = handler.connectionOpenTry(msg_try);
        IBCMsgs.MsgConnectionOpenConfirm memory msg_confirm = MsgMocks
            .connectionOpenConfirm(clientId, connId, proofHeight);
        handler.connectionOpenConfirm(msg_confirm);
    }
}
