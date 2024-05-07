pragma solidity ^0.8.23;

import "../../proto/ibc/core/client/v1/client.sol";
import "../../proto/ibc/core/connection/v1/connection.sol";
import "../../proto/ibc/core/channel/v1/channel.sol";

/**
 * @dev IBCMsgs provides datagram types in [ICS-26](https://github.com/cosmos/ibc/tree/main/spec/core/ics-026-routing-module#datagram-handlers-write)
 */
library IBCMsgs {
    /* Client */

    struct MsgCreateClient {
        string clientType;
        bytes clientStateBytes;
        bytes consensusStateBytes;
    }

    struct MsgUpdateClient {
        string clientId;
        bytes clientMessage;
    }

    /* Connection */

    struct MsgConnectionOpenInit {
        string clientId;
        IbcCoreConnectionV1Counterparty.Data counterparty;
        uint64 delayPeriod;
    }

    struct MsgConnectionOpenTry {
        IbcCoreConnectionV1Counterparty.Data counterparty; // counterpartyConnectionIdentifier, counterpartyPrefix and counterpartyClientIdentifier
        uint64 delayPeriod;
        string clientId; // clientID of chainA
        bytes clientStateBytes; // clientState that chainA has for chainB
        IbcCoreConnectionV1Version.Data[] counterpartyVersions; // supported versions of chain A
        bytes proofInit; // proof that chainA stored connectionEnd in state (on ConnOpenInit)
        bytes proofClient; // proof that chainA stored a light client of chainB
        bytes proofConsensus; // proof that chainA stored chainB's consensus state at consensus height
        IbcCoreClientV1Height.Data proofHeight; // height at which relayer constructs proof of A storing connectionEnd in state
        IbcCoreClientV1Height.Data consensusHeight; // latest height of chain B which chain A has stored in its chain B client
    }

    struct MsgConnectionOpenAck {
        string connectionId;
        bytes clientStateBytes; // client state for chainA on chainB
        IbcCoreConnectionV1Version.Data version; // version that ChainB chose in ConnOpenTry
        string counterpartyConnectionID;
        bytes proofTry; // proof that connectionEnd was added to ChainB state in ConnOpenTry
        bytes proofClient; // proof of client state on chainB for chainA
        bytes proofConsensus; // proof that chainB has stored ConsensusState of chainA on its client
        IbcCoreClientV1Height.Data proofHeight; // height that relayer constructed proofTry
        IbcCoreClientV1Height.Data consensusHeight; // latest height of chainA that chainB has stored on its chainA client
    }

    struct MsgConnectionOpenConfirm {
        string connectionId;
        bytes proofAck;
        IbcCoreClientV1Height.Data proofHeight;
    }

    /* Channel */

    struct MsgChannelOpenInit {
        string portId;
        IbcCoreChannelV1Channel.Data channel;
    }

    struct MsgChannelOpenTry {
        string portId;
        IbcCoreChannelV1Channel.Data channel;
        string counterpartyVersion;
        bytes proofInit;
        IbcCoreClientV1Height.Data proofHeight;
    }

    struct MsgChannelOpenAck {
        string portId;
        string channelId;
        string counterpartyVersion;
        string counterpartyChannelId;
        bytes proofTry;
        IbcCoreClientV1Height.Data proofHeight;
    }

    struct MsgChannelOpenConfirm {
        string portId;
        string channelId;
        bytes proofAck;
        IbcCoreClientV1Height.Data proofHeight;
    }

    struct MsgChannelCloseInit {
        string portId;
        string channelId;
    }

    struct MsgChannelCloseConfirm {
        string portId;
        string channelId;
        bytes proofInit;
        IbcCoreClientV1Height.Data proofHeight;
    }

    /* Packet relay */

    struct MsgPacketRecv {
        IbcCoreChannelV1Packet.Data packet;
        bytes proof;
        IbcCoreClientV1Height.Data proofHeight;
    }

    struct MsgPacketAcknowledgement {
        IbcCoreChannelV1Packet.Data packet;
        bytes acknowledgement;
        bytes proof;
        IbcCoreClientV1Height.Data proofHeight;
    }

    struct MsgPacketTimeout {
        IbcCoreChannelV1Packet.Data packet;
        bytes proof;
        IbcCoreClientV1Height.Data proofHeight;
        uint64 nextSequenceRecv;
    }
}
