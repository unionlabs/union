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
            event RegisterClient(
                string indexed client_type_index, string client_type, address client_address
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event CreateClient(
                string indexed client_type_index,
                string clientType,
                uint32 indexed client_id,
                string counterparty_chain_id
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event UpdateClient(uint32 indexed client_id, uint64 height);
            // #[cfg_attr(
            //     feature = "serde", derive(serde::Serialize, serde::Deserialize),
            //     serde(deny_unknown_fields)
            // )]
            // event Misbehaviour(uint32 indexed clientId);

            error ErrClientTypeAlreadyExists();
            error ErrClientTypeNotFound();

            // IBC CONNECTION

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event ConnectionOpenInit(
                uint32 indexed connection_id,
                uint32 indexed client_id,
                uint32 counterparty_client_id
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event ConnectionOpenTry(
                uint32 indexed connection_id,
                uint32 indexed client_id,
                uint32 counterparty_client_id,
                uint32 counterparty_connection_id
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event ConnectionOpenAck(
                uint32 indexed connection_id,
                uint32 indexed client_id,
                uint32 counterparty_client_id,
                uint32 counterparty_connection_id
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event ConnectionOpenConfirm(
                uint32 indexed connection_id,
                uint32 indexed client_id,
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
                address indexed port_id,
                uint32 indexed channel_id,
                bytes counterparty_port_id,
                uint32 connection_id,
                string indexed version_index,
                string version
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event ChannelOpenTry(
                address indexed port_id,
                uint32 indexed channel_id,
                bytes counterparty_port_id,
                uint32 counterparty_channel_id,
                uint32 connection_id,
                string indexed counterparty_version_index,
                string counterparty_version
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event ChannelOpenAck(
                address indexed port_id,
                uint32 indexed channel_id,
                bytes counterparty_port_id,
                uint32 counterparty_channel_id,
                uint32 connection_id
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event ChannelOpenConfirm(
                address indexed port_id,
                uint32 indexed channel_id,
                bytes counterparty_portId,
                uint32 counterparty_channel_id,
                uint32 connection_id
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event ChannelCloseInit(
                address indexed port_id,
                uint32 indexed channel_id,
                bytes counterparty_port_id,
                uint32 counterparty_channel_id
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event ChannelCloseConfirm(
                address indexed port_id,
                uint32 indexed channel_id,
                bytes counterparty_port_id,
                uint32 counterparty_channel_id
            );

            // IBC PACKET

            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event PacketSend(
                uint32 indexed channel_id, bytes32 indexed packet_hash, Packet packet
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event PacketRecv(
                uint32 indexed channel_id,
                bytes32 indexed packet_hash,
                address indexed maker,
                bytes maker_msg
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event IntentPacketRecv(
                uint32 indexed channel_id,
                bytes32 indexed packet_hash,
                address indexed maker,
                bytes maker_msg
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event WriteAck(
                uint32 indexed channel_id,
                bytes32 indexed packet_hash,
                bytes acknowledgement
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event PacketAck(
                uint32 indexed channel_id,
                bytes32 indexed packet_hash,
                bytes acknowledgement,
                address indexed maker
            );
            #[cfg_attr(
                feature = "serde", derive(serde::Serialize, serde::Deserialize),
                serde(deny_unknown_fields)
            )]
            event PacketTimeout(
                uint32 indexed channel_id,
                bytes32 indexed packet_hash,
                address indexed maker
            );
            // #[cfg_attr(
            //     feature = "serde", derive(serde::Serialize, serde::Deserialize),
            //     serde(deny_unknown_fields)
            // )]
            // event BatchedPreviouslySent(
            //     uint32 indexed channel_id,
            //     bytes32 indexed batch_hash,
            //     bytes32 indexed packet_hash
            // );
            // #[cfg_attr(
            //     feature = "serde", derive(serde::Serialize, serde::Deserialize),
            //     serde(deny_unknown_fields)
            // )]
            // event BatchedPreviouslyAcked(
            //     uint32 indexed channel_id,
            //     bytes32 indexed batch_hash,
            //     bytes32 indexed packet_hash
            // );

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

        enum ConnectionState {
            Unspecified,
            Init,
            TryOpen,
            Open
        }

        struct Connection {
            ConnectionState state;
            uint32 client_id;
            uint32 counterparty_client_id;
            uint32 counterparty_connection_id;
        }

        enum ChannelState {
            Unspecified,
            Init,
            TryOpen,
            Open,
            Closed
        }

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
            uint32 source_channel_id;
            uint32 destination_channel_id;
            bytes data;
            uint64 timeout_height;
            uint64 timeout_timestamp;
        }

        struct MsgCreateClient {
            string client_type;
            bytes client_state_bytes;
            bytes consensus_state_bytes;
            address relayer;
        }

        struct MsgUpdateClient {
            uint32 client_id;
            bytes client_message;
            address relayer;
        }

        struct MsgConnectionOpenInit {
            uint32 client_id;
            uint32 counterparty_client_id;
        }

        struct MsgConnectionOpenTry {
            uint32 counterparty_client_id;
            uint32 counterparty_connection_id;
            uint32 client_id;
            bytes proof_init;
            uint64 proof_height;
        }

        struct MsgConnectionOpenAck {
            uint32 connection_id;
            uint32 counterparty_connection_id;
            bytes proof_try;
            uint64 proof_height;
        }

        struct MsgConnectionOpenConfirm {
            uint32 connection_id;
            bytes proof_ack;
            uint64 proof_height;
        }

        struct MsgChannelOpenInit {
            address port_id;
            bytes counterparty_port_id;
            uint32 connection_id;
            string version;
            address relayer;
        }

        struct MsgChannelOpenTry {
            address port_id;
            Channel channel;
            string counterparty_version;
            bytes proof_init;
            uint64 proof_height;
            address relayer;
        }

        struct MsgChannelOpenAck {
            uint32 channel_id;
            string counterparty_version;
            uint32 counterparty_channel_id;
            bytes proof_try;
            uint64 proof_height;
            address relayer;
        }

        struct MsgChannelOpenConfirm {
            uint32 channel_id;
            bytes proof_ack;
            uint64 proof_height;
            address relayer;
        }

        struct MsgChannelCloseInit {
            uint32 channel_id;
            address relayer;
        }

        struct MsgChannelCloseConfirm {
            uint32 channel_id;
            bytes proof_init;
            uint64 proof_height;
            address relayer;
        }

        struct MsgPacketRecv {
            Packet[] packets;
            bytes[] relayer_msgs;
            address relayer;
            bytes proof;
            uint64 proof_height;
        }

        struct MsgPacketAcknowledgement {
            Packet[] packets;
            bytes[] acknowledgements;
            bytes proof;
            uint64 proof_height;
            address relayer;
        }

        struct MsgPacketTimeout {
            Packet packet;
            bytes proof;
            uint64 proof_height;
            address relayer;
        }

        struct MsgIntentPacketRecv {
            Packet[] packets;
            bytes[] market_maker_msgs;
            address market_maker;
            bytes emptyProof;
        }

        struct MsgBatchSend {
            uint32 source_channel;
            Packet[] packets;
        }

        struct MsgBatchAcks {
            uint32 source_channel;
            Packet[] packets;
            bytes[] acks;
        }

        interface ILightClient {
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

impl Clone for Ibc::IbcEvents {
    fn clone(&self) -> Self {
        match self {
            Ibc::IbcEvents::RegisterClient(client_registered) => {
                Ibc::IbcEvents::RegisterClient(client_registered.clone())
            }
            Ibc::IbcEvents::CreateClient(client_created) => {
                Ibc::IbcEvents::CreateClient(client_created.clone())
            }
            Ibc::IbcEvents::UpdateClient(client_updated) => {
                Ibc::IbcEvents::UpdateClient(client_updated.clone())
            }
            Ibc::IbcEvents::ConnectionOpenInit(connection_open_init) => {
                Ibc::IbcEvents::ConnectionOpenInit(connection_open_init.clone())
            }
            Ibc::IbcEvents::ConnectionOpenTry(connection_open_try) => {
                Ibc::IbcEvents::ConnectionOpenTry(connection_open_try.clone())
            }
            Ibc::IbcEvents::ConnectionOpenAck(connection_open_ack) => {
                Ibc::IbcEvents::ConnectionOpenAck(connection_open_ack.clone())
            }
            Ibc::IbcEvents::ConnectionOpenConfirm(connection_open_confirm) => {
                Ibc::IbcEvents::ConnectionOpenConfirm(connection_open_confirm.clone())
            }
            Ibc::IbcEvents::ChannelOpenInit(channel_open_init) => {
                Ibc::IbcEvents::ChannelOpenInit(channel_open_init.clone())
            }
            Ibc::IbcEvents::ChannelOpenTry(channel_open_try) => {
                Ibc::IbcEvents::ChannelOpenTry(channel_open_try.clone())
            }
            Ibc::IbcEvents::ChannelOpenAck(channel_open_ack) => {
                Ibc::IbcEvents::ChannelOpenAck(channel_open_ack.clone())
            }
            Ibc::IbcEvents::ChannelOpenConfirm(channel_open_confirm) => {
                Ibc::IbcEvents::ChannelOpenConfirm(channel_open_confirm.clone())
            }
            Ibc::IbcEvents::ChannelCloseInit(channel_close_init) => {
                Ibc::IbcEvents::ChannelCloseInit(channel_close_init.clone())
            }
            Ibc::IbcEvents::ChannelCloseConfirm(channel_close_confirm) => {
                Ibc::IbcEvents::ChannelCloseConfirm(channel_close_confirm.clone())
            }
            Ibc::IbcEvents::PacketSend(send_packet) => {
                Ibc::IbcEvents::PacketSend(send_packet.clone())
            }
            Ibc::IbcEvents::PacketRecv(recv_packet) => {
                Ibc::IbcEvents::PacketRecv(recv_packet.clone())
            }
            Ibc::IbcEvents::IntentPacketRecv(recv_intent_packet) => {
                Ibc::IbcEvents::IntentPacketRecv(recv_intent_packet.clone())
            }
            Ibc::IbcEvents::WriteAck(write_acknowledgement) => {
                Ibc::IbcEvents::WriteAck(write_acknowledgement.clone())
            }
            Ibc::IbcEvents::PacketAck(acknowledge_packet) => {
                Ibc::IbcEvents::PacketAck(acknowledge_packet.clone())
            }
            Ibc::IbcEvents::PacketTimeout(timeout_packet) => {
                Ibc::IbcEvents::PacketTimeout(timeout_packet.clone())
            }
        }
    }
}

/// Conversions between [`alloy::sol!`] generated types and the canonical tyeps in [`ibc_union_spec`].
pub mod compat {
    use ibc_union_spec::{ChannelId, ClientId, ConnectionId, Timestamp};

    use super::*;

    impl From<ibc_union_spec::Packet> for Packet {
        fn from(value: ibc_union_spec::Packet) -> Self {
            Self {
                source_channel_id: value.source_channel_id.raw(),
                destination_channel_id: value.destination_channel_id.raw(),
                data: value.data.into(),
                timeout_height: value.timeout_height,
                timeout_timestamp: value.timeout_timestamp.as_nanos(),
            }
        }
    }

    impl TryFrom<Packet> for ibc_union_spec::Packet {
        type Error = InvalidPacketError;

        fn try_from(value: Packet) -> Result<Self, Self::Error> {
            Ok(Self {
                source_channel_id: ChannelId::from_raw(value.source_channel_id)
                    .ok_or(InvalidPacketError::InvalidSourceChannelId)?,
                destination_channel_id: ChannelId::from_raw(value.destination_channel_id)
                    .ok_or(InvalidPacketError::InvalidDestinationChannelId)?,
                data: value.data.into(),
                timeout_height: value.timeout_height,
                timeout_timestamp: Timestamp::from_nanos(value.timeout_timestamp),
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum InvalidPacketError {
        #[error("invalid source channel id")]
        InvalidSourceChannelId,
        #[error("invalid destination channel id")]
        InvalidDestinationChannelId,
    }

    impl From<ibc_union_spec::Connection> for Connection {
        fn from(value: ibc_union_spec::Connection) -> Self {
            Self {
                state: match value.state {
                    ibc_union_spec::ConnectionState::Init => ConnectionState::Init,
                    ibc_union_spec::ConnectionState::TryOpen => ConnectionState::TryOpen,
                    ibc_union_spec::ConnectionState::Open => ConnectionState::Open,
                },
                client_id: value.client_id.raw(),
                counterparty_client_id: value.counterparty_client_id.raw(),
                counterparty_connection_id: value
                    .counterparty_connection_id
                    .map(|counterparty_connection_id| counterparty_connection_id.raw())
                    .unwrap_or_default(),
            }
        }
    }

    impl TryFrom<Connection> for ibc_union_spec::Connection {
        type Error = InvalidConnectionError;

        fn try_from(value: Connection) -> Result<Self, Self::Error> {
            Ok(Self {
                state: match value.state {
                    ConnectionState::Init => ibc_union_spec::ConnectionState::Init,
                    ConnectionState::TryOpen => ibc_union_spec::ConnectionState::TryOpen,
                    ConnectionState::Open => ibc_union_spec::ConnectionState::Open,
                    ConnectionState::Unspecified | ConnectionState::__Invalid => {
                        return Err(InvalidConnectionError::ConnectionState)
                    }
                },
                client_id: ClientId::from_raw(value.client_id)
                    .ok_or(InvalidConnectionError::ClientId)?,
                counterparty_client_id: ClientId::from_raw(value.counterparty_client_id)
                    .ok_or(InvalidConnectionError::CounterpartyClientId)?,
                counterparty_connection_id: ConnectionId::from_raw(
                    value.counterparty_connection_id,
                ),
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum InvalidConnectionError {
        #[error("invalid connection state")]
        ConnectionState,
        #[error("invalid client id")]
        ClientId,
        #[error("invalid counterparty client id")]
        CounterpartyClientId,
    }

    impl From<ibc_union_spec::Channel> for Channel {
        fn from(value: ibc_union_spec::Channel) -> Self {
            Self {
                state: match value.state {
                    ibc_union_spec::ChannelState::Init => ChannelState::Init,
                    ibc_union_spec::ChannelState::TryOpen => ChannelState::TryOpen,
                    ibc_union_spec::ChannelState::Open => ChannelState::Open,
                    ibc_union_spec::ChannelState::Closed => ChannelState::Closed,
                },
                connection_id: value.connection_id.raw(),
                counterparty_channel_id: value
                    .counterparty_channel_id
                    .map(|counterparty_channel_id| counterparty_channel_id.raw())
                    .unwrap_or_default(),
                counterparty_port_id: value.counterparty_port_id.into(),
                version: value.version,
            }
        }
    }

    impl TryFrom<Channel> for ibc_union_spec::Channel {
        type Error = InvalidChannelError;

        fn try_from(value: Channel) -> Result<Self, Self::Error> {
            Ok(Self {
                state: match value.state {
                    ChannelState::Init => ibc_union_spec::ChannelState::Init,
                    ChannelState::TryOpen => ibc_union_spec::ChannelState::TryOpen,
                    ChannelState::Open => ibc_union_spec::ChannelState::Open,
                    ChannelState::Closed => ibc_union_spec::ChannelState::Closed,
                    ChannelState::Unspecified | ChannelState::__Invalid => {
                        return Err(InvalidChannelError::InvalidChannelState)
                    }
                },
                connection_id: ConnectionId::from_raw(value.connection_id)
                    .ok_or(InvalidChannelError::InvalidConnectionId)?,
                counterparty_channel_id: ChannelId::from_raw(value.counterparty_channel_id),
                counterparty_port_id: value.counterparty_port_id.into(),
                version: value.version,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum InvalidChannelError {
        #[error("invalid channel state")]
        InvalidChannelState,
        #[error("invalid connection id")]
        InvalidConnectionId,
    }
}

#[cfg(test)]
mod tests {
    use alloy::{
        primitives::{bytes, keccak256},
        sol_types::SolValue,
    };

    use super::*;
    use crate::Packet;

    #[test]
    fn packet_hash() {
        dbg!(keccak256(
            Packet {
                source_channel_id: 1,
                destination_channel_id: 1,
                data: bytes!("0000000000000000000000000000000000000000000000000000000000000000"),
                timeout_height: 0,
                timeout_timestamp: 1733160153000000000
            }
            .abi_encode()
        ));
    }

    mod connection {
        use ibc_union_spec::{ClientId, ConnectionId};

        use super::*;

        #[test]
        fn abi_encode() {
            let ibc_solidity_connection = Connection {
                state: ConnectionState::Init,
                client_id: 1,
                counterparty_client_id: 1,
                counterparty_connection_id: 1,
            };

            let connection = ibc_union_spec::Connection {
                state: ibc_union_spec::ConnectionState::Init,
                client_id: ClientId!(1),
                counterparty_client_id: ClientId!(1),
                counterparty_connection_id: Some(ConnectionId!(1)),
            };

            let ibc_solidity_bz = ibc_solidity_connection.abi_encode_params();
            let bz = connection.abi_encode_params();
            assert_eq!(ibc_solidity_bz, bz);

            let ibc_solidity_bz = ibc_solidity_connection.abi_encode();
            let bz = connection.abi_encode();
            assert_eq!(ibc_solidity_bz, bz);
        }

        #[test]
        fn abi_decode() {
            let ibc_solidity_connection = Connection {
                state: ConnectionState::Init,
                client_id: 1,
                counterparty_client_id: 1,
                counterparty_connection_id: 1,
            };

            let connection = ibc_union_spec::Connection {
                state: ibc_union_spec::ConnectionState::Init,
                client_id: ClientId!(1),
                counterparty_client_id: ClientId!(1),
                counterparty_connection_id: Some(ConnectionId!(1)),
            };

            let ibc_solidity_bz = ibc_solidity_connection.abi_encode();
            let decoded_connection =
                ibc_union_spec::Connection::abi_decode(&ibc_solidity_bz, true).unwrap();
            assert_eq!(connection, decoded_connection);

            let ibc_solidity_bz = ibc_solidity_connection.abi_encode_params();
            let decoded_connection =
                ibc_union_spec::Connection::abi_decode_params(&ibc_solidity_bz, true).unwrap();
            assert_eq!(connection, decoded_connection);
        }

        #[test]
        fn abi_decode_invalid() {
            let ibc_solidity_connection = Connection {
                state: ConnectionState::Unspecified,
                client_id: 1,
                counterparty_client_id: 1,
                counterparty_connection_id: 1,
            };

            let expected_err =
                alloy::sol_types::Error::type_check_fail_token::<ibc_union_spec::Connection>(&(
                    alloy::sol_types::private::U256::from(0_u8).into(),
                    alloy::sol_types::private::U256::from(1_u32).into(),
                    alloy::sol_types::private::U256::from(1_u32).into(),
                    alloy::sol_types::private::U256::from(1_u32).into(),
                ));

            let ibc_solidity_bz = ibc_solidity_connection.abi_encode_params();
            let err =
                ibc_union_spec::Connection::abi_decode_params(&ibc_solidity_bz, true).unwrap_err();
            assert_eq!(expected_err, err);

            let ibc_solidity_bz = ibc_solidity_connection.abi_encode();
            let err = ibc_union_spec::Connection::abi_decode(&ibc_solidity_bz, true).unwrap_err();
            assert_eq!(expected_err, err);
        }
    }

    mod channel {
        use alloy::primitives::U256;
        use ibc_union_spec::{ChannelId, ConnectionId};

        use super::*;

        #[test]
        fn abi_encode() {
            let ibc_solidity_connection = Channel {
                state: ChannelState::Init,
                connection_id: 1,
                counterparty_channel_id: 1,
                counterparty_port_id: b"port".into(),
                version: "version".into(),
            };

            let connection = ibc_union_spec::Channel {
                state: ibc_union_spec::ChannelState::Init,
                connection_id: ConnectionId!(1),
                counterparty_channel_id: Some(ChannelId!(1)),
                counterparty_port_id: b"port".into(),
                version: "version".into(),
            };

            let ibc_solidity_bz = ibc_solidity_connection.abi_encode_params();
            let bz = connection.abi_encode_params();
            assert_eq!(ibc_solidity_bz, bz);

            let ibc_solidity_bz = ibc_solidity_connection.abi_encode();
            let bz = connection.abi_encode();
            assert_eq!(ibc_solidity_bz, bz);
        }

        #[test]
        fn abi_decode() {
            let ibc_solidity_connection = Channel {
                state: ChannelState::Init,
                connection_id: 1,
                counterparty_channel_id: 1,
                counterparty_port_id: b"port".into(),
                version: "version".into(),
            };

            let connection = ibc_union_spec::Channel {
                state: ibc_union_spec::ChannelState::Init,
                connection_id: ConnectionId!(1),
                counterparty_channel_id: Some(ChannelId!(1)),
                counterparty_port_id: b"port".into(),
                version: "version".into(),
            };

            let ibc_solidity_bz = ibc_solidity_connection.abi_encode();
            let decoded_connection =
                ibc_union_spec::Channel::abi_decode(&ibc_solidity_bz, true).unwrap();
            assert_eq!(connection, decoded_connection);

            let ibc_solidity_bz = ibc_solidity_connection.abi_encode_params();
            let decoded_connection =
                ibc_union_spec::Channel::abi_decode_params(&ibc_solidity_bz, true).unwrap();
            assert_eq!(connection, decoded_connection);
        }

        #[test]
        fn abi_decode_invalid() {
            let ibc_solidity_channel = Channel {
                state: ChannelState::Unspecified,
                connection_id: 1,
                counterparty_channel_id: 1,
                counterparty_port_id: b"port".into(),
                version: "version".into(),
            };

            let expected_err =
                alloy::sol_types::Error::type_check_fail_token::<ibc_union_spec::Channel>(&(
                    U256::from(0_u32).into(),
                    U256::from(1_u32).into(),
                    U256::from(1_u32).into(),
                    b"port".as_slice().into(),
                    b"version".as_slice().into(),
                ));

            let ibc_solidity_bz = ibc_solidity_channel.abi_encode_params();
            let err =
                ibc_union_spec::Channel::abi_decode_params(&ibc_solidity_bz, true).unwrap_err();
            assert_eq!(expected_err, err);

            let ibc_solidity_bz = ibc_solidity_channel.abi_encode();
            let err = ibc_union_spec::Channel::abi_decode(&ibc_solidity_bz, true).unwrap_err();
            assert_eq!(expected_err, err);
        }
    }

    mod packet {
        use alloy::sol_types::{private::U256, SolValue};
        use ibc_union_spec::{ChannelId, Timestamp};

        use super::*;

        #[test]
        fn abi_encode() {
            let ibc_solidity_packet = Packet {
                source_channel_id: 1,
                destination_channel_id: 1,
                data: b"data".into(),
                timeout_height: 1,
                timeout_timestamp: 0,
            };

            let packet = ibc_union_spec::Packet {
                source_channel_id: ChannelId::from_raw(1).unwrap(),
                destination_channel_id: ChannelId::from_raw(1).unwrap(),
                data: b"data".into(),
                timeout_height: 1,
                timeout_timestamp: Timestamp::ZERO,
            };

            let ibc_solidity_bz = ibc_solidity_packet.abi_encode_params();
            let bz = packet.abi_encode_params();
            assert_eq!(ibc_solidity_bz, bz);

            let ibc_solidity_bz = ibc_solidity_packet.abi_encode();
            let bz = packet.abi_encode();
            assert_eq!(ibc_solidity_bz, bz);
        }

        #[test]
        fn abi_decode() {
            let ibc_solidity_packet = Packet {
                source_channel_id: 1,
                destination_channel_id: 1,
                data: b"data".into(),
                timeout_height: 1,
                timeout_timestamp: 0,
            };

            let packet = ibc_union_spec::Packet {
                source_channel_id: ChannelId::from_raw(1).unwrap(),
                destination_channel_id: ChannelId::from_raw(1).unwrap(),
                data: b"data".into(),
                timeout_height: 1,
                timeout_timestamp: Timestamp::ZERO,
            };

            let ibc_solidity_bz = ibc_solidity_packet.abi_encode();
            let decoded_packet =
                ibc_union_spec::Packet::abi_decode(&ibc_solidity_bz, true).unwrap();
            assert_eq!(packet, decoded_packet);

            let ibc_solidity_bz = ibc_solidity_packet.abi_encode_params();
            let decoded_packet =
                ibc_union_spec::Packet::abi_decode_params(&ibc_solidity_bz, true).unwrap();
            assert_eq!(packet, decoded_packet);
        }

        #[test]
        fn abi_decode_invalid() {
            let ibc_solidity_packet = Packet {
                source_channel_id: 0,
                destination_channel_id: 0,
                data: b"data".into(),
                timeout_height: 0,
                timeout_timestamp: 0,
            };

            let expected_err =
                alloy::sol_types::Error::type_check_fail_token::<ibc_union_spec::Packet>(&(
                    U256::from(0_u32).into(),
                    U256::from(0_u32).into(),
                    b"data".as_slice().into(),
                    U256::from(0_u64).into(),
                    U256::from(0_u64).into(),
                ));

            let ibc_solidity_bz = ibc_solidity_packet.abi_encode_params();
            let err =
                ibc_union_spec::Packet::abi_decode_params(&ibc_solidity_bz, true).unwrap_err();
            assert_eq!(expected_err, err);

            let ibc_solidity_bz = ibc_solidity_packet.abi_encode();
            let err = ibc_union_spec::Packet::abi_decode(&ibc_solidity_bz, true).unwrap_err();
            assert_eq!(expected_err, err);
        }
    }
}
