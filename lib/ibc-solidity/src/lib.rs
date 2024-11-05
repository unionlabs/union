macro_rules! maybe_sol_attr {
    (
        alloy::sol! {
            #![sol(rpc)]
            $($tt:tt)*
        }
    ) => {
        #[cfg(feature = "rpc")]
        alloy::sol! {
            #![sol(rpc)]
            $($tt)*
        }

        #[cfg(not(feature = "rpc"))]
        alloy::sol! {
            $($tt)*
        }
    };
}

pub mod ibc {
    maybe_sol_attr! {
        alloy::sol! {
            #![sol(rpc)]
            #![sol(all_derives)]

            contract Ibc {
                // STORE

                mapping(bytes32 => bytes32) public commitments;

                /// ClientType -> Address
                mapping(bytes32 => address) public clientRegistry;
                /// ClientId -> ClientType
                mapping(uint32 => string) public clientTypes;
                /// ClientId -> Address
                mapping(uint32 => address) public clientImpls;
                /// ConnectionId -> Connection
                mapping(uint32 => Connection) public connections;
                /// ChannelId -> Channel
                mapping(uint32 => Channel) public channels;
                /// ChannelId -> PortId
                mapping(uint32 => address) public channelOwner;

                function registerClient(bytes32 clientType, address client) external;

                function createClient(
                    MsgCreateClient calldata msg_
                ) external returns (uint32 clientId);

                function updateClient(
                    MsgUpdateClient calldata msg_
                ) external;

                // CONNECTION

                function connectionOpenInit(
                    MsgConnectionOpenInit calldata msg_
                ) external returns (uint32);

                function connectionOpenTry(
                    MsgConnectionOpenTry calldata msg_
                ) external returns (uint32);

                function connectionOpenAck(
                    MsgConnectionOpenAck calldata msg_
                ) external;

                function connectionOpenConfirm(
                    MsgConnectionOpenConfirm calldata msg_
                ) external;

                // CHANNEL

                function channelOpenInit(
                    MsgChannelOpenInit calldata msg_
                ) external returns (uint32);

                function channelOpenTry(
                    MsgChannelOpenTry calldata msg_
                ) external returns (uint32);

                function channelOpenAck(
                    MsgChannelOpenAck calldata msg_
                ) external;

                function channelOpenConfirm(
                    MsgChannelOpenConfirm calldata msg_
                ) external;

                function channelCloseInit(
                    MsgChannelCloseInit calldata msg_
                ) external;

                function channelCloseConfirm(
                    MsgChannelCloseConfirm calldata msg_
                ) external;

                // PACKET

                function sendPacket(
                    uint32 sourceChannel,
                    uint64 timeoutHeight,
                    uint64 timeoutTimestamp,
                    bytes calldata data
                ) external returns (uint64);

                function recvPacket(
                    MsgPacketRecv calldata msg_
                ) external;

                function recvIntentPacket(
                    MsgIntentPacketRecv calldata msg_
                ) external;

                function writeAcknowledgement(
                    Packet calldata packet,
                    bytes memory acknowledgement
                ) external;

                function acknowledgePacket(
                    MsgPacketAcknowledgement calldata msg_
                ) external;

                function timeoutPacket(
                    MsgPacketTimeout calldata msg_
                ) external;

                function batchSend(
                    MsgBatchSend calldata msg_
                ) external;

                function batchAcks(
                    MsgBatchAcks calldata msg_
                ) external;

                // IBC MODULE

                error ErrClientNotFound();
                error ErrModuleNotFound();
                error ErrInvalidConnectionState();
                error ErrInvalidChannelState();

                // IBC CLIENT

                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event ClientRegistered(string clientType, address clientAddress);
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event ClientCreated(string clientType, uint32 clientId);
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event ClientUpdated(uint32 clientId, uint64 height);

                error ErrClientTypeAlreadyExists();
                error ErrClientTypeNotFound();

                // IBC CONNECTION

                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event ConnectionOpenInit(
                    uint32 connectionId, uint32 clientId, uint32 counterpartyClientId
                );
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event ConnectionOpenTry(
                    uint32 connectionId,
                    uint32 clientId,
                    uint32 counterpartyClientId,
                    uint32 counterpartyConnectionId
                );
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event ConnectionOpenAck(
                    uint32 connectionId,
                    uint32 clientId,
                    uint32 counterpartyClientId,
                    uint32 counterpartyConnectionId
                );
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event ConnectionOpenConfirm(
                    uint32 connectionId,
                    uint32 clientId,
                    uint32 counterpartyClientId,
                    uint32 counterpartyConnectionId
                );

                // error ErrInvalidProof();
                // error ErrInvalidConnectionState();

                // IBC CHANNEL

                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event ChannelOpenInit(
                    string portId,
                    uint32 channelId,
                    string counterpartyPortId,
                    uint32 connectionId,
                    string version
                );
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event ChannelOpenTry(
                    string portId,
                    uint32 channelId,
                    string counterpartyPortId,
                    uint32 counterpartyChannelId,
                    uint32 connectionId,
                    string version
                );
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event ChannelOpenAck(
                    string portId,
                    uint32 channelId,
                    string counterpartyPortId,
                    uint32 counterpartyChannelId,
                    uint32 connectionId
                );
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event ChannelOpenConfirm(
                    string portId,
                    uint32 channelId,
                    string counterpartyPortId,
                    uint32 counterpartyChannelId,
                    uint32 connectionId
                );
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event ChannelCloseInit(
                    string portId,
                    uint32 channelId,
                    string counterpartyPortId,
                    uint32 counterpartyChannelId
                );
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event ChannelCloseConfirm(
                    string portId,
                    uint32 channelId,
                    string counterpartyPortId,
                    uint32 counterpartyChannelId
                );

                // IBC PACKET

                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event SendPacket(Packet packet);
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event RecvPacket(Packet packet, address relayer, bytes relayerMsg);
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event RecvIntentPacket(
                    Packet packet, address marketMaker, bytes marketMakerMsg
                );
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event WriteAcknowledgement(Packet packet, bytes acknowledgement);
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event AcknowledgePacket(
                    Packet packet, bytes acknowledgement, address relayer
                );
                #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
                event TimeoutPacket(Packet packet, address relayer);

                error ErrUnauthorized();
                error ErrLatestTimestampNotFound();
                error ErrTimeoutMustBeSet();
                error ErrHeightTimeout();
                error ErrTimestampTimeout();
                error ErrInvalidProof();
                error ErrPacketSequenceNextSequenceMismatch();
                error ErrPacketSequenceAckSequenceMismatch();
                error ErrAcknowledgementIsEmpty();
                error ErrPacketNotReceived();
                error ErrAcknowledgementAlreadyExists();
                error ErrPacketCommitmentNotFound();
                error ErrTimeoutHeightNotReached();
                error ErrTimeoutTimestampNotReached();
                error ErrNextSequenceMustBeLEQThanTimeoutSequence();
                error ErrNotEnoughPackets();
                error ErrCommittedAckNotPresent();
                error ErrCannotIntentOrderedPacket();

                // COMETBLS CLIENT

                error ErrNotIBC();
                error ErrTrustedConsensusStateNotFound();
                error ErrUntrustedHeightLTETrustedHeight();
                error ErrUntrustedTimestampLTETrustedTimestamp();
                error ErrHeaderExpired();
                error ErrMaxClockDriftExceeded();
                error ErrInvalidZKP();
                error ErrInvalidUntrustedValidatorsHash();
                error ErrInvalidMisbehaviorHeadersSequence();
                error ErrInvalidMisbehavior();
                error ErrClientFrozen();
                error ErrInvalidInitialConsensusState();
            }

            enum ConnectionState {
                Unspecified,
                Init,
                TryOpen,
                Open
            }

            struct Connection {
                ConnectionState state;
                uint32 clientId;
                uint32 counterpartyClientId;
                uint32 counterpartyConnectionId;
            }

            enum ChannelState {
                Unspecified,
                Init,
                TryOpen,
                Open,
                Closed
            }

            enum ChannelOrder {
                Unspecified,
                Unordered,
                Ordered
            }

            struct Channel {
                ChannelState state;
                ChannelOrder ordering;
                uint32 connectionId;
                uint32 counterpartyChannelId;
                string counterpartyPortId;
                string version;
            }

            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            struct Packet {
                uint64 sequence;
                uint32 sourceChannel;
                uint32 destinationChannel;
                bytes data;
                uint64 timeoutHeight;
                uint64 timeoutTimestamp;
            }

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
                ChannelOrder ordering;
                string version;
                address relayer;
            }

            struct MsgChannelOpenTry {
                Channel channel;
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
                Packet[] packets;
                bytes[] relayerMsgs;
                address relayer;
                bytes proof;
                uint64 proofHeight;
            }

            struct MsgPacketAcknowledgement {
                Packet[] packets;
                bytes[] acknowledgements;
                bytes proof;
                uint64 proofHeight;
                address relayer;
            }

            struct MsgPacketTimeout {
                Packet packet;
                bytes proof;
                uint64 proofHeight;
                uint64 nextSequenceRecv;
                address relayer;
            }

            struct MsgIntentPacketRecv {
                Packet[] packets;
                bytes[] marketMakerMsgs;
                address marketMaker;
                bytes emptyProof;
            }

            struct MsgBatchSend {
                uint32 sourceChannel;
                Packet[] packets;
            }

            struct MsgBatchAcks {
                uint32 sourceChannel;
                Packet[] packets;
                bytes[] acks;
            }

            struct ConsensusStateUpdate {
                bytes32 clientStateCommitment;
                bytes32 consensusStateCommitment;
                uint64 height;
            }

            interface ILightClient {
                function createClient(
                    uint32 clientId,
                    bytes calldata clientStateBytes,
                    bytes calldata consensusStateBytes
                ) external returns (ConsensusStateUpdate memory update);

                function getTimestampAtHeight(
                    uint32 clientId,
                    uint64 height
                ) external view returns (uint64);

                function getLatestHeight(
                    uint32 clientId
                ) external view returns (uint64 height);

                function updateClient(
                    uint32 clientId,
                    bytes calldata clientMessageBytes
                ) external returns (ConsensusStateUpdate memory update);

                function verifyMembership(
                    uint32 clientId,
                    uint64 height,
                    bytes calldata proof,
                    bytes calldata path,
                    bytes calldata value
                ) external returns (bool);

                function verifyNonMembership(
                    uint32 clientId,
                    uint64 height,
                    bytes calldata proof,
                    bytes calldata path
                ) external returns (bool);

                function getClientState(
                    uint32 clientId
                ) external view returns (bytes memory);

                function getConsensusState(
                    uint32 clientId,
                    uint64 height
                ) external view returns (bytes memory);

                function isFrozen(
                    uint32 clientId
                ) external view returns (bool);
            }
        }
    }
}

pub mod ics23 {
    alloy::sol! {
        struct ExistenceProof {
            bytes key;
            bytes value;
            bytes leafPrefix;
            InnerOp[] path;
        }

        struct NonExistenceProof {
            bytes key;
            ExistenceProof left;
            ExistenceProof right;
        }

        struct InnerOp {
            bytes prefix;
            bytes suffix;
        }

        struct ProofSpec {
            uint256 childSize;
            uint256 minPrefixLength;
            uint256 maxPrefixLength;
        }
    }
}

pub mod cometbls {
    alloy::sol! {
        struct SignedHeader {
            uint64 height;
            uint64 secs;
            uint64 nanos;
            bytes32 validatorsHash;
            bytes32 nextValidatorsHash;
            bytes32 appHash;
        }

        struct Header {
            SignedHeader signedHeader;
            uint64 trustedHeight;
            bytes zeroKnowledgeProof;
        }

        struct ClientState {
            bytes31 chainId;
            uint64 trustingPeriod;
            uint64 maxClockDrift;
            uint64 frozenHeight;
            uint64 latestHeight;
        }

        struct ConsensusState {
            uint64 timestamp;
            bytes32 appHash;
            bytes32 nextValidatorsHash;
        }
    }
}

impl Clone for ibc::Ibc::IbcEvents {
    fn clone(&self) -> Self {
        match self {
            ibc::Ibc::IbcEvents::ClientRegistered(client_registered) => {
                ibc::Ibc::IbcEvents::ClientRegistered(client_registered.clone())
            }
            ibc::Ibc::IbcEvents::ClientCreated(client_created) => {
                ibc::Ibc::IbcEvents::ClientCreated(client_created.clone())
            }
            ibc::Ibc::IbcEvents::ClientUpdated(client_updated) => {
                ibc::Ibc::IbcEvents::ClientUpdated(client_updated.clone())
            }
            ibc::Ibc::IbcEvents::ConnectionOpenInit(connection_open_init) => {
                ibc::Ibc::IbcEvents::ConnectionOpenInit(connection_open_init.clone())
            }
            ibc::Ibc::IbcEvents::ConnectionOpenTry(connection_open_try) => {
                ibc::Ibc::IbcEvents::ConnectionOpenTry(connection_open_try.clone())
            }
            ibc::Ibc::IbcEvents::ConnectionOpenAck(connection_open_ack) => {
                ibc::Ibc::IbcEvents::ConnectionOpenAck(connection_open_ack.clone())
            }
            ibc::Ibc::IbcEvents::ConnectionOpenConfirm(connection_open_confirm) => {
                ibc::Ibc::IbcEvents::ConnectionOpenConfirm(connection_open_confirm.clone())
            }
            ibc::Ibc::IbcEvents::ChannelOpenInit(channel_open_init) => {
                ibc::Ibc::IbcEvents::ChannelOpenInit(channel_open_init.clone())
            }
            ibc::Ibc::IbcEvents::ChannelOpenTry(channel_open_try) => {
                ibc::Ibc::IbcEvents::ChannelOpenTry(channel_open_try.clone())
            }
            ibc::Ibc::IbcEvents::ChannelOpenAck(channel_open_ack) => {
                ibc::Ibc::IbcEvents::ChannelOpenAck(channel_open_ack.clone())
            }
            ibc::Ibc::IbcEvents::ChannelOpenConfirm(channel_open_confirm) => {
                ibc::Ibc::IbcEvents::ChannelOpenConfirm(channel_open_confirm.clone())
            }
            ibc::Ibc::IbcEvents::ChannelCloseInit(channel_close_init) => {
                ibc::Ibc::IbcEvents::ChannelCloseInit(channel_close_init.clone())
            }
            ibc::Ibc::IbcEvents::ChannelCloseConfirm(channel_close_confirm) => {
                ibc::Ibc::IbcEvents::ChannelCloseConfirm(channel_close_confirm.clone())
            }
            ibc::Ibc::IbcEvents::SendPacket(send_packet) => {
                ibc::Ibc::IbcEvents::SendPacket(send_packet.clone())
            }
            ibc::Ibc::IbcEvents::RecvPacket(recv_packet) => {
                ibc::Ibc::IbcEvents::RecvPacket(recv_packet.clone())
            }
            ibc::Ibc::IbcEvents::RecvIntentPacket(recv_intent_packet) => {
                ibc::Ibc::IbcEvents::RecvIntentPacket(recv_intent_packet.clone())
            }
            ibc::Ibc::IbcEvents::WriteAcknowledgement(write_acknowledgement) => {
                ibc::Ibc::IbcEvents::WriteAcknowledgement(write_acknowledgement.clone())
            }
            ibc::Ibc::IbcEvents::AcknowledgePacket(acknowledge_packet) => {
                ibc::Ibc::IbcEvents::AcknowledgePacket(acknowledge_packet.clone())
            }
            ibc::Ibc::IbcEvents::TimeoutPacket(timeout_packet) => {
                ibc::Ibc::IbcEvents::TimeoutPacket(timeout_packet.clone())
            }
        }
    }
}
