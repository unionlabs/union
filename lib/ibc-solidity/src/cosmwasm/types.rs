// Duplicated until we resolve whether we reuse the same types, or not.

pub mod ibc {
    alloy::sol! {
        // #![sol(cfg_attr(feature = "rpc", rpc))]
        #![sol(rpc, all_derives)]

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        enum ConnectionState {
            Init,
            TryOpen,
            Open
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct Connection {
            ConnectionState state;
            uint32 clientId;
            uint32 counterpartyClientId;
            uint32 counterpartyConnectionId;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        enum ChannelState {
            Init,
            TryOpen,
            Open,
            Closed
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct Channel {
            ChannelState state;
            uint32 connectionId;
            uint32 counterpartyChannelId;
            string counterpartyPortId;
            string version;
        }

        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct Packet {
            uint32 sourceChannel;
            uint32 destinationChannel;
            bytes data;
            uint64 timeoutHeight;
            uint64 timeoutTimestamp;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgCreateClient {
            string clientType;
            bytes clientStateBytes;
            bytes consensusStateBytes;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgUpdateClient {
            uint32 clientId;
            bytes clientMessage;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgConnectionOpenInit {
            uint32 clientId;
            uint32 counterpartyClientId;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgConnectionOpenTry {
            uint32 counterpartyClientId;
            uint32 counterpartyConnectionId;
            uint32 clientId;
            bytes proofInit;
            uint64 proofHeight;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgConnectionOpenAck {
            uint32 connectionId;
            uint32 counterpartyConnectionId;
            bytes proofTry;
            uint64 proofHeight;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgConnectionOpenConfirm {
            uint32 connectionId;
            bytes proofAck;
            uint64 proofHeight;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgChannelOpenInit {
            string portId;
            string counterpartyPortId;
            uint32 connectionId;
            string version;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgChannelOpenTry {
            string portId;
            Channel channel;
            string counterpartyVersion;
            bytes proofInit;
            uint64 proofHeight;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgChannelOpenAck {
            uint32 channelId;
            string counterpartyVersion;
            uint32 counterpartyChannelId;
            bytes proofTry;
            uint64 proofHeight;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgChannelOpenConfirm {
            uint32 channelId;
            bytes proofAck;
            uint64 proofHeight;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgChannelCloseInit {
            uint32 channelId;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgChannelCloseConfirm {
            uint32 channelId;
            bytes proofInit;
            uint64 proofHeight;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgPacketRecv {
            Packet[] packets;
            bytes[] relayerMsgs;
            string relayer;
            bytes proof;
            uint64 proofHeight;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgPacketAcknowledgement {
            Packet[] packets;
            bytes[] acknowledgements;
            bytes proof;
            uint64 proofHeight;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgPacketTimeout {
            Packet packet;
            bytes proof;
            uint64 proofHeight;
            string relayer;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgIntentPacketRecv {
            Packet[] packets;
            bytes[] marketMakerMsgs;
            string marketMaker;
            bytes emptyProof;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgBatchSend {
            uint32 sourceChannel;
            Packet[] packets;
        }

        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct MsgBatchAcks {
            uint32 sourceChannel;
            Packet[] packets;
            bytes[] acks;
        }
    }
}
