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

enum IBCChannelOrder {
    Unspecified,
    Unordered,
    Ordered
}

struct IBCChannel {
    IBCChannelState state;
    IBCChannelOrder ordering;
    uint32 connectionId;
    uint32 counterpartyChannelId;
    string portId;
    string counterpartyPortId;
    string version;
}

struct IBCPacket {
    uint64 sequence;
    uint32 sourceChannel;
    uint32 destinationChannel;
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
    error ErrInvalidChannelOrdering();
    error ErrUnauthorized();
    error ErrLatestTimestampNotFound();
    error ErrTimeoutMustBeSet();
    error ErrHeightTimeout();
    error ErrTimestampTimeout();
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
    error ErrClientNotFound();
    error ErrModuleNotFound();
}
