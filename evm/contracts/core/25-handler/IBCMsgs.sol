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
        address relayer;
    }

    struct MsgConnectionOpenTry {
        uint32 counterpartyClientId;
        uint32 counterpartyConnectionId;
        uint32 clientId;
        bytes proofInit;
        uint64 proofHeight;
        address relayer;
    }

    struct MsgConnectionOpenAck {
        uint32 connectionId;
        uint32 counterpartyConnectionId;
        bytes proofTry;
        uint64 proofHeight;
        address relayer;
    }

    struct MsgConnectionOpenConfirm {
        uint32 connectionId;
        bytes proofAck;
        uint64 proofHeight;
        address relayer;
    }

    struct MsgChannelOpenInit {
        address portId;
        string counterpartyPortId;
        uint32 connectionId;
        IBCChannelOrder ordering;
        string version;
        address relayer;
    }

    struct MsgChannelOpenTry {
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
        uint64 nextSequenceRecv;
        address relayer;
    }

    struct MsgIntentPacketRecv {
        IBCPacket[] packets;
        bytes[] marketMakerMsgs;
        address marketMaker;
        bytes emptyProof;
    }

    struct MsgBatchSend {
        uint32 sourceChannel;
        IBCPacket[] packets;
    }

    struct MsgBatchAcks {
        uint32 sourceChannel;
        IBCPacket[] packets;
        bytes[] acks;
    }
}
