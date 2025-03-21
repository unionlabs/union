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
                string indexed version
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
                string indexed counterparty_version
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

#[cfg(test)]
mod tests {
    use alloy::{
        primitives::{bytes, keccak256},
        sol_types::SolValue,
    };

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
}
