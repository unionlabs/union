pragma solidity ^0.8.18;

import {IBCMsgs} from "contracts/core/25-handler/IBCMsgs.sol";
import {MockClient} from "contracts/clients/MockClient.sol";
import {
    IbcCoreConnectionV1ConnectionEnd as ConnectionEnd,
    IbcCoreConnectionV1Counterparty as ConnectionCounterparty,
    IbcCoreConnectionV1GlobalEnums as ConnectionEnums
} from "contracts/proto/ibc/core/connection/v1/connection.sol";
import {ILightClient} from "contracts/core/02-client/ILightClient.sol";
import {MockClient} from "contracts/clients/MockClient.sol";
import {IbcCoreCommitmentV1MerklePrefix as CommitmentMerklePrefix} from
    "contracts/proto/ibc/core/commitment/v1/commitment.sol";

import "tests/TestPlus.sol";
import {IBCHandler_Testable} from "tests/utils/IBCHandler_Testable.sol";

contract IBCConnectionTest is TestPlus {
    using ConnectionCounterparty for ConnectionCounterparty.Data;

    IBCHandler_Testable handler;
    ILightClient client;
    string constant CLIENT_TYPE = "mock";

    constructor() {
        handler = new IBCHandler_Testable();
        client = new MockClient(address(handler));
        handler.registerClient(CLIENT_TYPE, client);
    }

    /// tests a full connection creation handshake, from the perspective of chain A
    function test_openingHandshake_chainA(uint64 proofHeight) public {
        vm.assume(proofHeight > 0);

        // 1. createClient
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(CLIENT_TYPE, proofHeight);
        string memory clientId = handler.createClient(m);

        // 2. ConnOpenInit
        IBCMsgs.MsgConnectionOpenInit memory msg_init = MsgMocks.connectionOpenInit(clientId);
        string memory connId = handler.connectionOpenInit(msg_init);

        (ConnectionEnd.Data memory connection,) = handler.getConnection(connId);
        assertEq(connection.client_id, clientId, "clientId mismatch");
        assertEq(connection.delay_period, msg_init.delayPeriod, "delayPeriod mismatch");
        assertEq(connection.counterparty.encode(), msg_init.counterparty.encode(), "counterparty mismatch");
        assert(connection.state == ConnectionEnums.State.STATE_INIT);
        assertEq(connection.versions.length, 1);
        assertEq(connection.versions[0].features.length, 2);
        assertEq(connection.versions[0].identifier, "1");
        assertEq(connection.versions[0].features[0], "ORDER_ORDERED");
        assertEq(connection.versions[0].features[1], "ORDER_UNORDERED");

        // 3. ConnOpenAck
        IBCMsgs.MsgConnectionOpenAck memory msg_ack = MsgMocks.connectionOpenAck(clientId, connId, proofHeight);
        handler.connectionOpenAck(msg_ack);

        // compute the expected counterparty after ack
        ConnectionCounterparty.Data memory expectedCounterparty = msg_init.counterparty;
        expectedCounterparty.connection_id = msg_ack.counterpartyConnectionID;

        (connection,) = handler.getConnection(connId);
        assertEq(connection.client_id, clientId, "clientId mismatch");
        assertEq(connection.delay_period, msg_init.delayPeriod, "delayPeriod mismatch");
        assertEq(connection.counterparty.encode(), expectedCounterparty.encode(), "counterparty mismatch");
        assert(connection.state == ConnectionEnums.State.STATE_OPEN);
        assertEq(connection.versions.length, 1);
        assertEq(connection.versions[0].features.length, 2);
        assertEq(connection.versions[0].identifier, "1");
        assertEq(connection.versions[0].features[0], "ORDER_ORDERED");
        assertEq(connection.versions[0].features[1], "ORDER_UNORDERED");
    }

    /// tests a full connection creation handshake, from the perspective of chain B
    function test_openingHandshake_chainB(uint64 proofHeight) public {
        vm.assume(proofHeight > 0);

        // 1. createClient
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(CLIENT_TYPE, proofHeight);
        string memory clientId = handler.createClient(m);

        // 1. ConnOpenTry
        IBCMsgs.MsgConnectionOpenTry memory msg_try = MsgMocks.connectionOpenTry(clientId, proofHeight);
        string memory connId = handler.connectionOpenTry(msg_try);

        (ConnectionEnd.Data memory connection,) = handler.getConnection(connId);
        assertEq(connection.client_id, clientId, "clientId mismatch");
        assertEq(connection.delay_period, msg_try.delayPeriod, "delayPeriod mismatch");
        assertEq(connection.counterparty.encode(), msg_try.counterparty.encode(), "counterparty mismatch");
        assert(connection.state == ConnectionEnums.State.STATE_TRYOPEN);
        assertEq(connection.versions.length, 1);
        assertEq(connection.versions[0].features.length, 2);
        assertEq(connection.versions[0].identifier, "1");
        assertEq(connection.versions[0].features[0], "ORDER_ORDERED");
        assertEq(connection.versions[0].features[1], "ORDER_UNORDERED");

        // 2. ConnOpenConfirm
        IBCMsgs.MsgConnectionOpenConfirm memory msg_confirm =
            MsgMocks.connectionOpenConfirm(clientId, connId, proofHeight);
        handler.connectionOpenConfirm(msg_confirm);

        (connection,) = handler.getConnection(connId);
        assertEq(connection.client_id, clientId, "clientId mismatch");
        assertEq(connection.delay_period, msg_try.delayPeriod, "delayPeriod mismatch");
        assertEq(connection.counterparty.encode(), msg_try.counterparty.encode(), "counterparty mismatch");
        assert(connection.state == ConnectionEnums.State.STATE_OPEN);
        assertEq(connection.versions.length, 1);
        assertEq(connection.versions[0].features.length, 2);
        assertEq(connection.versions[0].identifier, "1");
        assertEq(connection.versions[0].features[0], "ORDER_ORDERED");
        assertEq(connection.versions[0].features[1], "ORDER_UNORDERED");
    }

    function test_openingHandshake_chainA_duplicateIds() public {
        IBCMsgs.MsgConnectionOpenInit memory m = MsgMocks.connectionOpenInit("client-1");

        string memory id = handler.connectionOpenInit(m);
        string memory id2 = handler.connectionOpenInit(m);
        assertStrNotEq(id, id2);
    }

    // TODO: test other failure paths
}
