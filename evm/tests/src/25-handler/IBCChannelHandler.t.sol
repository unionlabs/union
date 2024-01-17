pragma solidity ^0.8.23;

import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {MockClient} from "../../../contracts/clients/MockClient.sol";
import {IbcCoreConnectionV1ConnectionEnd as ConnectionEnd, IbcCoreConnectionV1Counterparty as ConnectionCounterparty, IbcCoreConnectionV1GlobalEnums as ConnectionEnums} from "../../../contracts/proto/ibc/core/connection/v1/connection.sol";
import {IbcCoreChannelV1Channel as Channel} from "../../../contracts/proto/ibc/core/channel/v1/channel.sol";
import {ILightClient} from "../../../contracts/core/02-client/ILightClient.sol";
import {MockClient} from "../../../contracts/clients/MockClient.sol";
import {IbcCoreCommitmentV1MerklePrefix as CommitmentMerklePrefix} from "../../../contracts/proto/ibc/core/commitment/v1/commitment.sol";

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

    /// tests a full channel creation handshake, from the perspective of chain A
    function test_openingHandshake_chainA(
        uint64 proofHeight,
        string memory portId
    ) public {
        vm.assume(proofHeight > 0);
        (string memory clientId, string memory connId) = setupConnection_chainA(
            proofHeight
        );

        // 1. bindPort
        handler.bindPort(portId, address(app));

        // 2. channelOpenInit
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

        // 3. channelOpenAck
        IBCMsgs.MsgChannelOpenAck memory msg_ack = MsgMocks.channelOpenAck(
            portId,
            channelId,
            proofHeight
        );
        handler.channelOpenAck(msg_ack);

        // TODO: verify channel commitment
        // #526
    }

    /// tests a full connection creation handshake, from the perspective of chain B
    function test_openingHandshake_chainB(
        uint64 proofHeight,
        string memory portId
    ) public {
        // 1. bindPort
        vm.assume(proofHeight > 0);
        (string memory clientId, string memory connId) = setupConnection_chainB(
            proofHeight
        );
        handler.bindPort(portId, address(app));

        // 2. connOpenTry
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

        // 3. connOpenConfirm
        IBCMsgs.MsgChannelOpenConfirm memory msg_confirm = MsgMocks
            .channelOpenConfirm(portId, channelId, proofHeight);
        handler.channelOpenConfirm(msg_confirm);
    }

    // TODO: test other failure paths
    // #526

    /// sets up an IBC Connection from the perspective of chain A
    function setupConnection_chainA(
        uint64 proofHeight
    ) internal returns (string memory clientId, string memory connId) {
        // 1. createClient
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(
            CLIENT_TYPE,
            proofHeight
        );
        clientId = handler.createClient(m);

        // 2. ConnOpenInit
        IBCMsgs.MsgConnectionOpenInit memory msg_init = MsgMocks
            .connectionOpenInit(clientId);
        connId = handler.connectionOpenInit(msg_init);

        // 3. ConnOpenAck
        IBCMsgs.MsgConnectionOpenAck memory msg_ack = MsgMocks
            .connectionOpenAck(clientId, connId, proofHeight);
        handler.connectionOpenAck(msg_ack);
    }

    /// sets up an IBC Connection from the perspective of chain B
    function setupConnection_chainB(
        uint64 proofHeight
    ) internal returns (string memory clientId, string memory connId) {
        // 1. createClient
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(
            CLIENT_TYPE,
            proofHeight
        );
        clientId = handler.createClient(m);

        // 1. ConnOpenTry
        IBCMsgs.MsgConnectionOpenTry memory msg_try = MsgMocks
            .connectionOpenTry(clientId, proofHeight);
        connId = handler.connectionOpenTry(msg_try);

        // 2. ConnOpenConfirm
        IBCMsgs.MsgConnectionOpenConfirm memory msg_confirm = MsgMocks
            .connectionOpenConfirm(clientId, connId, proofHeight);
        handler.connectionOpenConfirm(msg_confirm);
    }
}
