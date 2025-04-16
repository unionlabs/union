pragma solidity ^0.8.27;

import "../Types.sol";

/**
 * @dev IBCMsgs provides datagram types in [ICS-26](https://github.com/cosmos/ibc/tree/main/spec/core/ics-026-routing-module#datagram-handlers-write)
 */
library IBCMsgs {
    struct MsgCreateClient {
        string clientType;
        bytes clientStateBytes;
        bytes consensusStateBytes;
        address relayer;
    }

    struct MsgUpdateClient {
        uint32 clientId;
        bytes clientMessage;
        address relayer;
    }

    struct MsgConnectionOpenInit {
        uint32 clientId;
        uint32 counterpartyClientId;
    }

    struct MsgConnectionOpenTry {
        uint32 counterpartyClientId;
        uint32 counterpartyConnectionId;
        uint32 clientId;
        bytes proofInit;
        uint64 proofHeight;
    }

    struct MsgConnectionOpenAck {
        uint32 connectionId;
        uint32 counterpartyConnectionId;
        bytes proofTry;
        uint64 proofHeight;
    }

    struct MsgConnectionOpenConfirm {
        uint32 connectionId;
        bytes proofAck;
        uint64 proofHeight;
    }

    struct MsgChannelOpenInit {
        address portId;
        bytes counterpartyPortId;
        uint32 connectionId;
        string version;
        address relayer;
    }

    struct MsgChannelOpenTry {
        address portId;
        IBCChannel channel;
        string counterpartyVersion;
        bytes proofInit;
        uint64 proofHeight;
        address relayer;
    }

    struct MsgChannelOpenAck {
        uint32 channelId;
        string counterpartyVersion;
        uint32 counterpartyChannelId;
        bytes proofTry;
        uint64 proofHeight;
        address relayer;
    }

    struct MsgChannelOpenConfirm {
        uint32 channelId;
        bytes proofAck;
        uint64 proofHeight;
        address relayer;
    }

    struct MsgChannelCloseInit {
        uint32 channelId;
        address relayer;
    }

    struct MsgChannelCloseConfirm {
        uint32 channelId;
        bytes proofInit;
        uint64 proofHeight;
        address relayer;
    }

    struct MsgPacketRecv {
        IBCPacket[] packets;
        bytes[] relayerMsgs;
        address relayer;
        bytes proof;
        uint64 proofHeight;
    }

    struct MsgPacketAcknowledgement {
        IBCPacket[] packets;
        bytes[] acknowledgements;
        bytes proof;
        uint64 proofHeight;
        address relayer;
    }

    struct MsgPacketTimeout {
        IBCPacket packet;
        bytes proof;
        uint64 proofHeight;
        address relayer;
    }

    struct MsgIntentPacketRecv {
        IBCPacket[] packets;
        bytes[] marketMakerMsgs;
        address marketMaker;
        bytes emptyProof;
    }

    struct MsgBatchSend {
        IBCPacket[] packets;
    }

    struct MsgBatchAcks {
        IBCPacket[] packets;
        bytes[] acks;
    }

    struct MsgMisbehaviour {
        uint32 clientId;
        bytes clientMessage;
        address relayer;
    }
}
