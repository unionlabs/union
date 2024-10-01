pragma solidity ^0.8.27;

enum IBCConnectionState {
    Unspecified,
    Init,
    TryOpen,
    Open
}

struct IBCConnectionCounterparty {
    uint32 clientId;
    uint32 connectionId;
    bytes32 merklePrefix;
}

struct IBCConnection {
    // We often read clientId/state
    uint32 clientId;
    IBCConnectionState state;
    IBCConnectionCounterparty counterparty;
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
    bytes32 portId;
    uint32 channelId;
}

struct IBCChannel {
    // We often read state/connectionId
    IBCChannelState state;
    uint32 connectionId;
    IBCChannelOrder ordering;
    IBCChannelCounterparty counterparty;
    bytes32 version;
}

struct IBCPacket {
    uint64 sequence;
    bytes32 sourcePort;
    uint32 sourceChannel;
    bytes32 destinationPort;
    uint32 destinationChannel;
    bytes data;
    uint64 timeoutHeight;
    uint64 timeoutTimestamp;
}
