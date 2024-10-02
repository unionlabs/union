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
