pragma solidity ^0.8.23;

import {ILightClient} from "../../../contracts/core/02-client/ILightClient.sol";
import {MockClient} from "../../../contracts/clients/MockClient.sol";
import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";

import "../TestPlus.sol";
import {IBCHandler_Testable} from "../utils/IBCHandler_Testable.sol";

contract IBCConnectionHandlerTest is TestPlus {
    IBCHandler_Testable handler;

    string constant CLIENT_TYPE = "mock";
    ILightClient client;
    ILightClient client2;

    event ConnectionOpenInit(string connectionId);
    event ConnectionOpenTry(string connectionId);
    event ConnectionOpenAck(string connectionId);
    event ConnectionOpenConfirm(string connectionId);

    constructor() {
        vm.warp(1);
        handler = new IBCHandler_Testable();
        client = new MockClient(address(handler));
        handler.registerClient(CLIENT_TYPE, client);
    }

    /// tests a full connection creation handshake, from the perspective of chain A
    /// TODO: no event is emitted on connectionOpenAck. should we?
    function test_openingHandshake_chainA(uint64 proofHeight) public {
        vm.assume(proofHeight > 0);

        // 1. createClient
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(
            CLIENT_TYPE,
            proofHeight
        );
        string memory clientId = handler.createClient(m);

        // 2. ConnOpenInit
        IBCMsgs.MsgConnectionOpenInit memory msg_init = MsgMocks
            .connectionOpenInit(clientId);
        vm.expectEmit(false, false, false, false);
        emit ConnectionOpenInit("");
        string memory connId = handler.connectionOpenInit(msg_init);

        // 3. ConnOpenAck
        IBCMsgs.MsgConnectionOpenAck memory msg_ack = MsgMocks
            .connectionOpenAck(clientId, connId, proofHeight);
        handler.connectionOpenAck(msg_ack);
    }

    /// tests a full connection creation handshake, from the perspective of chain B
    /// TODO: no event is emitted on connectionOpenConfirm. should we?
    function test_openingHandshake_chainB(uint64 proofHeight) public {
        vm.assume(proofHeight > 0);

        // 1. createClient
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(
            CLIENT_TYPE,
            proofHeight
        );
        string memory clientId = handler.createClient(m);

        // 1. ConnOpenTry
        IBCMsgs.MsgConnectionOpenTry memory msg_try = MsgMocks
            .connectionOpenTry(clientId, proofHeight);
        vm.expectEmit(false, false, false, false);
        emit ConnectionOpenTry("");
        string memory connId = handler.connectionOpenTry(msg_try);

        // 2. ConnOpenConfirm
        IBCMsgs.MsgConnectionOpenConfirm memory msg_confirm = MsgMocks
            .connectionOpenConfirm(clientId, connId, proofHeight);
        handler.connectionOpenConfirm(msg_confirm);
    }
}
