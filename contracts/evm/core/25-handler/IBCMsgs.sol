// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "../../proto/Client.sol";
import "../../proto/Connection.sol";
import "../../proto/Channel.sol";

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
        Counterparty.Data counterparty;
        uint64 delayPeriod;
    }

    struct MsgConnectionOpenTry {
        string previousConnectionId;
        Counterparty.Data counterparty; // counterpartyConnectionIdentifier, counterpartyPrefix and counterpartyClientIdentifier
        uint64 delayPeriod;
        string clientId; // clientID of chainA
        bytes clientStateBytes; // clientState that chainA has for chainB
        Version.Data[] counterpartyVersions; // supported versions of chain A
        bytes proofInit; // proof that chainA stored connectionEnd in state (on ConnOpenInit)
        bytes proofClient; // proof that chainA stored a light client of chainB
        bytes proofConsensus; // proof that chainA stored chainB's consensus state at consensus height
        Height.Data proofHeight; // height at which relayer constructs proof of A storing connectionEnd in state
        Height.Data consensusHeight; // latest height of chain B which chain A has stored in its chain B client
    }

    struct MsgConnectionOpenAck {
        string connectionId;
        bytes clientStateBytes; // client state for chainA on chainB
        Version.Data version; // version that ChainB chose in ConnOpenTry
        string counterpartyConnectionID;
        bytes proofTry; // proof that connectionEnd was added to ChainB state in ConnOpenTry
        bytes proofClient; // proof of client state on chainB for chainA
        bytes proofConsensus; // proof that chainB has stored ConsensusState of chainA on its client
        Height.Data proofHeight; // height that relayer constructed proofTry
        Height.Data consensusHeight; // latest height of chainA that chainB has stored on its chainA client
    }

    struct MsgConnectionOpenConfirm {
        string connectionId;
        bytes proofAck;
        Height.Data proofHeight;
    }

    /* Channel */

    struct MsgChannelOpenInit {
        string portId;
        Channel.Data channel;
    }

    struct MsgChannelOpenTry {
        string portId;
        string previousChannelId;
        Channel.Data channel;
        string counterpartyVersion;
        bytes proofInit;
        Height.Data proofHeight;
    }

    struct MsgChannelOpenAck {
        string portId;
        string channelId;
        string counterpartyVersion;
        string counterpartyChannelId;
        bytes proofTry;
        Height.Data proofHeight;
    }

    struct MsgChannelOpenConfirm {
        string portId;
        string channelId;
        bytes proofAck;
        Height.Data proofHeight;
    }

    struct MsgChannelCloseInit {
        string portId;
        string channelId;
    }

    struct MsgChannelCloseConfirm {
        string portId;
        string channelId;
        bytes proofInit;
        Height.Data proofHeight;
    }

    /* Packet relay */

    struct MsgPacketRecv {
        Packet.Data packet;
        bytes proof;
        Height.Data proofHeight;
    }

    struct MsgPacketAcknowledgement {
        Packet.Data packet;
        bytes acknowledgement;
        bytes proof;
        Height.Data proofHeight;
    }
}
