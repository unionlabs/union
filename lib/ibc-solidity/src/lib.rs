pub mod ibc {
    alloy::sol! {
        // #![sol(cfg_attr(feature = "rpc", rpc))]
        #![sol(rpc, all_derives)]

        contract Ibc {
            // STORE

            mapping(bytes32 => bytes32) public commitments;

            // ClientType -> Address
            mapping(bytes32 => address) public clientRegistry;
            // ClientId -> ClientType
            mapping(uint32 => bytes32) public clientTypes;
            // ClientId -> Address
            mapping(uint32 => address) public clientImpls;
            // ConnectionId -> Connection
            mapping(uint32 => Connection) public connections;
            // ChannelId -> Channel
            mapping(uint32 => Channel) public channels;
            // ChannelId -> PortId
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

            event ClientRegistered(bytes32 clientType, address clientAddress);
            event ClientCreated(bytes32 clientType, uint32 clientId);
            event ClientUpdated(uint32 clientId, uint64 height);

            error ErrClientTypeAlreadyExists();
            error ErrClientTypeNotFound();

            // IBC CONNECTION

            event ConnectionOpenInit(
                uint32 connectionId, uint32 clientId, uint32 counterpartyClientId
            );
            event ConnectionOpenTry(
                uint32 connectionId,
                uint32 clientId,
                uint32 counterpartyClientId,
                uint32 counterpartyConnectionId
            );
            event ConnectionOpenAck(
                uint32 connectionId,
                uint32 clientId,
                uint32 counterpartyClientId,
                uint32 counterpartyConnectionId
            );
            event ConnectionOpenConfirm(
                uint32 connectionId,
                uint32 clientId,
                uint32 counterpartyClientId,
                uint32 counterpartyConnectionId
            );

            // error ErrInvalidProof();
            // error ErrInvalidConnectionState();

            // IBC CHANNEL

            event ChannelOpenInit(
                address portId, uint32 channelId, uint32 connectionId, bytes32 version
            );
            event ChannelOpenTry(
                address portId,
                uint32 channelId,
                uint32 counterpartyChannelId,
                uint32 connectionId,
                bytes32 version
            );
            event ChannelOpenAck(
                address portId,
                uint32 channelId,
                uint32 counterpartyChannelId,
                uint32 connectionId
            );
            event ChannelOpenConfirm(
                address portId,
                uint32 channelId,
                uint32 counterpartyChannelId,
                uint32 connectionId
            );
            event ChannelCloseInit(address portId, uint32 channelId);
            event ChannelCloseConfirm(address portId, uint32 channelId);

            // IBC PACKET

            event SendPacket(Packet packet);
            event RecvPacket(Packet packet, address relayer, bytes relayerMsg);
            event RecvIntentPacket(
                Packet packet, address marketMaker, bytes marketMakerMsg
            );
            event WriteAcknowledgement(Packet packet, bytes acknowledgement);
            event AcknowledgePacket(
                Packet packet, bytes acknowledgement, address relayer
            );
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

        struct ConnectionCounterparty {
            uint32 clientId;
            uint32 connectionId;
        }

        struct Connection {
            ConnectionState state;
            ConnectionCounterparty counterparty;
            uint32 clientId;
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

        struct ChannelCounterparty {
            uint32 channelId;
        }

        struct Channel {
            ChannelState state;
            ChannelOrder ordering;
            uint32 connectionId;
            ChannelCounterparty counterparty;
            bytes32 version;
        }

        struct Packet {
            uint64 sequence;
            uint32 sourceChannel;
            uint32 destinationChannel;
            bytes data;
            uint64 timeoutHeight;
            uint64 timeoutTimestamp;
        }

        struct MsgCreateClient {
            bytes32 clientType;
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
            ConnectionCounterparty counterparty;
            address relayer;
        }

        struct MsgConnectionOpenTry {
            ConnectionCounterparty counterparty;
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
            Channel channel;
            address relayer;
        }

        struct MsgChannelOpenTry {
            address portId;
            Channel channel;
            bytes32 counterpartyVersion;
            bytes proofInit;
            uint64 proofHeight;
            address relayer;
        }

        struct MsgChannelOpenAck {
            address portId;
            uint32 channelId;
            bytes32 counterpartyVersion;
            uint32 counterpartyChannelId;
            bytes proofTry;
            uint64 proofHeight;
            address relayer;
        }

        struct MsgChannelOpenConfirm {
            address portId;
            uint32 channelId;
            bytes proofAck;
            uint64 proofHeight;
            address relayer;
        }

        struct MsgChannelCloseInit {
            address portId;
            uint32 channelId;
            address relayer;
        }

        struct MsgChannelCloseConfirm {
            address portId;
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
