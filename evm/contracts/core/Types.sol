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

library IBCErrors {
    error ErrClientTypeAlreadyExists();
    error ErrClientTypeNotFound();
    error ErrInvalidProof();
    error ErrInvalidConnectionState();
    error ErrPortIdMustBeLowercase();
    error ErrConnNotSingleHop();
    error ErrConnNotSingleVersion();
    error ErrUnsupportedFeature();
    error ErrInvalidChannelState();
    error ErrCounterpartyChannelNotEmpty();
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
