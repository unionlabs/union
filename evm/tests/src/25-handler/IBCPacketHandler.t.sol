pragma solidity ^0.8.23;

import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {IbcCoreClientV1Height as ClientHeight} from "../../../contracts/proto/MockClient.sol";
import {MockClient} from "../../../contracts/clients/MockClient.sol";
import {IbcCoreConnectionV1ConnectionEnd as ConnectionEnd, IbcCoreConnectionV1Counterparty as ConnectionCounterparty, IbcCoreConnectionV1GlobalEnums as ConnectionEnums} from "../../../contracts/proto/ibc/core/connection/v1/connection.sol";
import {IbcCoreChannelV1Channel as Channel} from "../../../contracts/proto/ibc/core/channel/v1/channel.sol";
import {ILightClient} from "../../../contracts/core/02-client/ILightClient.sol";
import {MockClient} from "../../../contracts/clients/MockClient.sol";
import {IbcCoreCommitmentV1MerklePrefix as CommitmentMerklePrefix} from "../../../contracts/proto/ibc/core/commitment/v1/commitment.sol";
import {IBCCommitment} from "../../../contracts/core/24-host/IBCCommitment.sol";

import "../TestPlus.sol";

contract IBCPacketHandlerTest is TestPlus {
    using ConnectionCounterparty for ConnectionCounterparty.Data;

    IBCHandler_Testable handler;
    ILightClient client;
    MockApp app;
    string constant CLIENT_TYPE = "mock";

    string clientId;

    event SendPacket(
        uint64 sequence,
        string sourcePort,
        string sourceChannel,
        ClientHeight.Data timeoutHeight,
        uint64 timeoutTimestamp,
        bytes data
    );

    function setUp() public {
        handler = new IBCHandler_Testable();
        client = new MockClient(address(handler));
        app = new MockApp();
        handler.registerClient(CLIENT_TYPE, client);
    }

    function createClient(uint64 proofHeight) internal returns (string memory) {
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(
            CLIENT_TYPE,
            proofHeight
        );
        return handler.createClient(m);
    }

    function setupConnection(
        uint64 proofHeight
    ) internal returns (string memory) {
        IBCMsgs.MsgConnectionOpenInit memory msg_init = MsgMocks
            .connectionOpenInit(clientId);
        string memory connId = handler.connectionOpenInit(msg_init);
        IBCMsgs.MsgConnectionOpenAck memory msg_ack = MsgMocks
            .connectionOpenAck(clientId, connId, proofHeight);
        handler.connectionOpenAck(msg_ack);
        return connId;
    }

    function setupChannel(
        uint64 proofHeight,
        string memory portId,
        string memory connId
    ) internal returns (string memory) {
        handler.bindPort(portId, address(app));
        IBCMsgs.MsgChannelOpenInit memory msg_init = MsgMocks.channelOpenInit(
            connId,
            portId
        );
        string memory channelId = handler.channelOpenInit(msg_init);
        IBCMsgs.MsgChannelOpenAck memory msg_ack = MsgMocks.channelOpenAck(
            portId,
            channelId,
            proofHeight
        );
        handler.channelOpenAck(msg_ack);
        return channelId;
    }
}
