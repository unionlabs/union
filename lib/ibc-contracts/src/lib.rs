alloy::sol! {
    interface Ibc {
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
            IBCPacket calldata packet,
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
    }

    enum IBCConnectionState {
        Unspecified,
        Init,
        TryOpen,
        Open
    }

    struct IBCConnectionCounterparty {
        uint32 clientId;
        uint32 connectionId;
    }

    struct IBCConnection {
        IBCConnectionState state;
        IBCConnectionCounterparty counterparty;
        uint32 clientId;
    }

    enum IBCChannelState {
        Unspecified,
        Init,
        TryOpen,
        Open,
        Closed
    }

    enum IBCChannelOrder {
        Unspecified,
        Unordered,
        Ordered
    }

    struct IBCChannelCounterparty {
        uint32 channelId;
    }

    struct IBCChannel {
        IBCChannelState state;
        IBCChannelOrder ordering;
        uint32 connectionId;
        IBCChannelCounterparty counterparty;
        bytes32 version;
    }

    struct IBCPacket {
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
        IBCConnectionCounterparty counterparty;
        address relayer;
    }

    struct MsgConnectionOpenTry {
        IBCConnectionCounterparty counterparty;
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
        IBCChannel channel;
        address relayer;
    }

    struct MsgChannelOpenTry {
        address portId;
        IBCChannel channel;
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

alloy::sol! {
    struct Call3 {
        address target;
        bool allowFailure;
        bytes callData;
    }

    struct Result {
        bool success;
        bytes returnData;
    }

    event MulticallResult(Result[]);

    contract Multicall {
        function multicall(
            Call3[] calldata calls
        ) public payable returns (Result[] memory returnData);
    }
}
