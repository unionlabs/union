pragma solidity ^0.8.18;

import {IBCMsgs} from "contracts/core/25-handler/IBCMsgs.sol";
import {MockClient} from "contracts/clients/MockClient.sol";
import {
    IbcCoreConnectionV1ConnectionEnd,
    IbcCoreConnectionV1Counterparty,
    IbcCoreConnectionV1GlobalEnums
} from "contracts/proto/ibc/core/connection/v1/connection.sol";
import {ILightClient} from "contracts/core/02-client/ILightClient.sol";
import {MockClient} from "contracts/clients/MockClient.sol";

import "forge-std/Test.sol";
import {TestPlus, MsgMocks} from "tests/utils/TestUtils.sol";
import {IBCHandler_Testable} from "tests/utils/IBCHandler_Testable.sol";

contract IBCConnectionTest is TestPlus {
    using IbcCoreConnectionV1Counterparty for IbcCoreConnectionV1Counterparty.Data;

    IBCHandler_Testable handler;
    ILightClient client;
    string constant CLIENT_TYPE = "mock";

    constructor() {
        handler = new IBCHandler_Testable();
        client = new MockClient(address(handler));
        handler.registerClient(CLIENT_TYPE, client);
    }

    function test_openingHandshake_chainA(string memory connId, uint64 proofHeight) public {
        vm.assume(proofHeight > 0);

        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(CLIENT_TYPE, proofHeight);

        string memory clientId = handler.createClient(m);

        // 1. ConnOpenInit
        IBCMsgs.MsgConnectionOpenInit memory msg_init = MsgMocks.connectionOpenInit(clientId);
        string memory id = handler.connectionOpenInit(msg_init);

        (IbcCoreConnectionV1ConnectionEnd.Data memory connection,) = handler.getConnection(id);
        assertStrEq(connection.client_id, clientId);
        assertEq(connection.versions.length, 1);
        assertStrEq(connection.versions[0].identifier, "1");
        assertEq(connection.versions[0].features.length, 2);
        assertStrEq(connection.versions[0].features[0], "ORDER_ORDERED");
        assertStrEq(connection.versions[0].features[1], "ORDER_UNORDERED");
        assertEq(connection.delay_period, msg_init.delayPeriod);
        assertEq(connection.counterparty.encode(), msg_init.counterparty.encode());
        assert(connection.state == IbcCoreConnectionV1GlobalEnums.State.STATE_INIT);

        // 2. ConnOpenAck
        IBCMsgs.MsgConnectionOpenAck memory msg_ack = MsgMocks.connectionOpenAck(clientId, id, proofHeight);
        handler.connectionOpenAck(msg_ack);
        // 1
    }

    function test_openingHandshake_chainB(string memory clientId, string memory connId) public {
        // 1. ConnOpenTry
        // 2. ConnOpenConfirm
    }

    function test_openingHandshake_chainA_duplicateIds() public {
        IBCMsgs.MsgConnectionOpenInit memory m = MsgMocks.connectionOpenInit("client-1");

        string memory id = handler.connectionOpenInit(m);
        string memory id2 = handler.connectionOpenInit(m);
        assertStrNotEq(id, id2);
    }
}
