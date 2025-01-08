pragma solidity ^0.8.27;

enum IBCConnectionState {
    Unspecified,
    Init,
    TryOpen,
    Open
}

struct IBCConnection {
    IBCConnectionState state;
    uint32 clientId;
    uint32 counterpartyClientId;
    uint32 counterpartyConnectionId;
}

enum IBCChannelState {
    Unspecified,
    Init,
    TryOpen,
    Open,
    Closed
}

struct IBCChannel {
    IBCChannelState state;
    uint32 connectionId;
    uint32 counterpartyChannelId;
    bytes counterpartyPortId;
    string version;
}

struct IBCPacket {
    uint32 sourceChannelId;
    uint32 destinationChannelId;
    bytes data;
    uint64 timeoutHeight;
    uint64 timeoutTimestamp;
}

library IBCErrors {
    error ErrClientTypeAlreadyExists();
    error ErrClientTypeNotFound();
    error ErrInvalidProof();
    error ErrInvalidConnectionState();
    error ErrInvalidChannelState();
    error ErrUnauthorized();
    error ErrLatestTimestampNotFound();
    error ErrTimeoutMustBeSet();
    error ErrHeightTimeout();
    error ErrTimestampTimeout();
    error ErrAcknowledgementIsEmpty();
    error ErrPacketNotReceived();
    error ErrAcknowledgementAlreadyExists();
    error ErrPacketCommitmentNotFound();
    error ErrTimeoutHeightNotReached();
    error ErrTimeoutTimestampNotReached();
    error ErrNotEnoughPackets();
    error ErrCommittedAckNotPresent();
    error ErrClientNotFound();
    error ErrModuleNotFound();
    error ErrPacketAlreadyExist();
}
