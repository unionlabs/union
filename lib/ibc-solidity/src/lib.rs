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
                ) external returns (uint32 client_id);

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

                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event ClientRegistered(string clientType, address clientAddress);
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event ClientCreated(string clientType, uint32 client_id);
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event ClientUpdated(uint32 client_id, uint64 height);

                error ErrClientTypeAlreadyExists();
                error ErrClientTypeNotFound();

                // IBC CONNECTION

                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event ConnectionOpenInit(
                    uint32 connection_id, uint32 client_id, uint32 counterparty_client_id
                );
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event ConnectionOpenTry(
                    uint32 connection_id,
                    uint32 client_id,
                    uint32 counterparty_client_id,
                    uint32 counterparty_connection_id
                );
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event ConnectionOpenAck(
                    uint32 connection_id,
                    uint32 client_id,
                    uint32 counterparty_client_id,
                    uint32 counterparty_connection_id
                );
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event ConnectionOpenConfirm(
                    uint32 connection_id,
                    uint32 client_id,
                    uint32 counterparty_client_id,
                    uint32 counterparty_connection_id
                );

                // error ErrInvalidProof();
                // error ErrInvalidConnectionState();

                // IBC CHANNEL

                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event ChannelOpenInit(
                    address port_id,
                    uint32 channel_id,
                    bytes counterparty_port_id,
                    uint32 connection_id,
                    string version
                );
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event ChannelOpenTry(
                    address port_id,
                    uint32 channel_id,
                    bytes counterparty_port_id,
                    uint32 counterparty_channel_id,
                    uint32 connection_id,
                    string version
                );
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event ChannelOpenAck(
                    address port_id,
                    uint32 channel_id,
                    bytes counterparty_port_id,
                    uint32 counterparty_channel_id,
                    uint32 connection_id
                );
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event ChannelOpenConfirm(
                    address port_id,
                    uint32 channel_id,
                    bytes counterparty_port_id,
                    uint32 counterparty_channel_id,
                    uint32 connection_id
                );
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event ChannelCloseInit(
                    address port_id,
                    uint32 channel_id,
                    bytes counterparty_port_id,
                    uint32 counterparty_channel_id
                );
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event ChannelCloseConfirm(
                    address port_id,
                    uint32 channel_id,
                    bytes counterparty_port_id,
                    uint32 counterparty_channel_id
                );

                // IBC PACKET

                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event SendPacket(Packet packet);
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event RecvPacket(Packet packet, address relayer, bytes relayerMsg);
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event RecvIntentPacket(
                    Packet packet, address market_maker, bytes market_maker_msg
                );
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event WriteAcknowledgement(Packet packet, bytes acknowledgement);
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event AcknowledgePacket(
                    Packet packet, bytes acknowledgement, address relayer
                );
                #[cfg_attr(
                    feature = "serde", derive(serde::Serialize, serde::Deserialize),
                    serde(deny_unknown_fields)
                )]
                event TimeoutPacket(Packet packet, address relayer);

                error ErrUnauthorized();
                error ErrLatestTimestampNotFound();
                error ErrTimeoutMustBeSet();
                error ErrHeightTimeout();
                error ErrTimestampTimeout();
                error ErrInvalidProof();
                error ErrAcknowledgementIsEmpty();
                error ErrPacketNotReceived();
                error ErrAcknowledgementAlreadyExists();
                error ErrPacketCommitmentNotFound();
                error ErrTimeoutHeightNotReached();
                error ErrTimeoutTimestampNotReached();
                error ErrNotEnoughPackets();
                error ErrCommittedAckNotPresent();

                // COMETBLS CLIENT

                error ErrNotIBC();
                error ErrTrustedConsensusStateNotFound();
                error ErrUntrustedHeightLTETrustedHeight();
                error ErrUntrustedTimestampLTETrustedTimestamp();
                error ErrHeaderExpired();
                error ErrMaxClockDriftExceeded();
                error ErrInvalidZKP();
                error ErrInvalidUntrustedValidatorsHash();
                error ErrInvalidMisbehaviourHeadersSequence();
                error ErrInvalidMisbehaviour();
                error ErrClientFrozen();
                error ErrInvalidInitialConsensusState();
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            enum ConnectionState {
                Unspecified,
                Init,
                TryOpen,
                Open
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct Connection {
                ConnectionState state;
                uint32 client_id;
                uint32 counterparty_client_id;
                uint32 counterparty_connection_id;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            enum ChannelState {
                Unspecified,
                Init,
                TryOpen,
                Open,
                Closed
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct Channel {
                ChannelState state;
                uint32 connection_id;
                uint32 counterparty_channel_id;
                bytes counterparty_port_id;
                string version;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct Packet {
                uint32 source_channel;
                uint32 destination_channel;
                bytes data;
                uint64 timeout_height;
                uint64 timeout_timestamp;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgCreateClient {
                string client_type;
                bytes client_state_bytes;
                bytes consensus_state_bytes;
                address relayer;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgUpdateClient {
                uint32 client_id;
                bytes client_message;
                address relayer;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgConnectionOpenInit {
                uint32 client_id;
                uint32 counterparty_client_id;
                address relayer;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgConnectionOpenTry {
                uint32 counterparty_client_id;
                uint32 counterparty_connection_id;
                uint32 client_id;
                bytes proof_init;
                uint64 proof_height;
                address relayer;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgConnectionOpenAck {
                uint32 connection_id;
                uint32 counterparty_connection_id;
                bytes proof_try;
                uint64 proof_height;
                address relayer;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgConnectionOpenConfirm {
                uint32 connection_id;
                bytes proof_ack;
                uint64 proof_height;
                address relayer;
            }

            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(deny_unknown_fields))]
            struct MsgChannelOpenInit {
                address port_id;
                bytes counterparty_port_id;
                uint32 connection_id;
                string version;
                address relayer;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgChannelOpenTry {
                Channel channel;
                string counterparty_version;
                bytes proof_init;
                uint64 proof_height;
                address relayer;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgChannelOpenAck {
                uint32 channel_id;
                string counterparty_version;
                uint32 counterparty_channel_id;
                bytes proof_try;
                uint64 proof_height;
                address relayer;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgChannelOpenConfirm {
                uint32 channel_id;
                bytes proof_ack;
                uint64 proof_height;
                address relayer;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgChannelCloseInit {
                uint32 channel_id;
                address relayer;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgChannelCloseConfirm {
                uint32 channel_id;
                bytes proof_init;
                uint64 proof_height;
                address relayer;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgPacketRecv {
                Packet[] packets;
                bytes[] relayer_msgs;
                address relayer;
                bytes proof;
                uint64 proof_height;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgPacketAcknowledgement {
                Packet[] packets;
                bytes[] acknowledgements;
                bytes proof;
                uint64 proof_height;
                address relayer;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgPacketTimeout {
                Packet packet;
                bytes proof;
                uint64 proof_height;
                address relayer;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgIntentPacketRecv {
                Packet[] packets;
                bytes[] market_maker_msgs;
                address market_maker;
                bytes emptyProof;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgBatchSend {
                uint32 source_channel;
                Packet[] packets;
            }

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            struct MsgBatchAcks {
                uint32 source_channel;
                Packet[] packets;
                bytes[] acks;
            }

            interface ILightClient {
                function getTimestampAtHeight(
                    uint32 client_id,
                    uint64 height
                ) external view returns (uint64);

                function getLatestHeight(
                    uint32 client_id
                ) external view returns (uint64 height);

                function verifyMembership(
                    uint32 client_id,
                    uint64 height,
                    bytes calldata proof,
                    bytes calldata path,
                    bytes calldata value
                ) external returns (bool);

                function verifyNonMembership(
                    uint32 client_id,
                    uint64 height,
                    bytes calldata proof,
                    bytes calldata path
                ) external returns (bool);

                function getClientState(
                    uint32 client_id
                ) external view returns (bytes memory);

                function getConsensusState(
                    uint32 client_id,
                    uint64 height
                ) external view returns (bytes memory);

                function isFrozen(
                    uint32 client_id
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
