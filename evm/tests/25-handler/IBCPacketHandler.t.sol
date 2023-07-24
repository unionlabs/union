pragma solidity ^0.8.18;

import {IBCMsgs} from "contracts/core/25-handler/IBCMsgs.sol";
import {IbcCoreClientV1Height as ClientHeight} from "contracts/proto/MockClient.sol";
import {MockClient} from "contracts/clients/MockClient.sol";
import {
    IbcCoreConnectionV1ConnectionEnd as ConnectionEnd,
    IbcCoreConnectionV1Counterparty as ConnectionCounterparty,
    IbcCoreConnectionV1GlobalEnums as ConnectionEnums
} from "contracts/proto/ibc/core/connection/v1/connection.sol";
import {IbcCoreChannelV1Channel as Channel} from "contracts/proto/ibc/core/channel/v1/channel.sol";
import {ILightClient} from "contracts/core/02-client/ILightClient.sol";
import {MockClient} from "contracts/clients/MockClient.sol";
import {IbcCoreCommitmentV1MerklePrefix as CommitmentMerklePrefix} from
    "contracts/proto/ibc/core/commitment/v1/commitment.sol";
import {IBCCommitment} from "contracts/core/24-host/IBCCommitment.sol";

import "tests/TestPlus.sol";

contract IBCPacketHandlerTest is TestPlus {
    using ConnectionCounterparty for ConnectionCounterparty.Data;

    IBCHandler_Testable handler;
    ILightClient client;
    MockApp app;
    string constant CLIENT_TYPE = "mock";

    uint64 proofHeight = 1;
    string clientId;
    string connId;
    string portId;
    string channelId;

    event SendPacket(
        uint64 sequence,
        string sourcePort,
        string sourceChannel,
        ClientHeight.Data timeoutHeight,
        uint64 timeoutTimestamp,
        bytes data
    );

    constructor() {
        handler = new IBCHandler_Testable();
        client = new MockClient(address(handler));
        app = new MockApp();
        handler.registerClient(CLIENT_TYPE, client);
        setupConnection();
        setupChannel();
    }

    function test_sendPacket() public {
        ClientHeight.Data memory timeoutHeight =
            ClientHeight.Data({revision_number: 0, revision_height: type(uint64).max});
        uint64 timeoutTimestamp = type(uint64).max;

        vm.prank(address(app));
        vm.expectEmit(false, false, false, false);
        emit SendPacket(0, "", "", timeoutHeight, 0, hex"");
        handler.sendPacket(portId, channelId, timeoutHeight, timeoutTimestamp, hex"12345678");
    }

    function test_recvPacket() public {
        // TODO: read the TODO notes on IBCPacket.t.sol
    }

    /// sets up an IBC Connection from the perspective of chain A
    function setupConnection() internal {
        // 1. createClient
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(CLIENT_TYPE, proofHeight);
        clientId = handler.createClient(m);

        // 2. ConnOpenInit
        IBCMsgs.MsgConnectionOpenInit memory msg_init = MsgMocks.connectionOpenInit(clientId);
        connId = handler.connectionOpenInit(msg_init);

        // 3. ConnOpenAck
        IBCMsgs.MsgConnectionOpenAck memory msg_ack = MsgMocks.connectionOpenAck(clientId, connId, proofHeight);
        handler.connectionOpenAck(msg_ack);
    }

    /// sets up an IBC Connection from the perspective of chain B
    function setupChannel() internal {
        // 1. bindPort
        handler.bindPort(portId, address(app));

        // 2. channelOpenInit
        IBCMsgs.MsgChannelOpenInit memory msg_init = MsgMocks.channelOpenInit(connId, portId);
        channelId = handler.channelOpenInit(msg_init);

        // 3. channelOpenAck
        IBCMsgs.MsgChannelOpenAck memory msg_ack = MsgMocks.channelOpenAck(portId, channelId, proofHeight);
        handler.channelOpenAck(msg_ack);
    }
}
