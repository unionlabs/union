package types

import (
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// IBC channel sentinel errors
var (
	ErrChannelExists             = sdkerrors.Register(SubModuleName, 2, "channel already exists")
	ErrChannelNotFound           = sdkerrors.Register(SubModuleName, 3, "channel not found")
	ErrInvalidChannel            = sdkerrors.Register(SubModuleName, 4, "invalid channel")
	ErrInvalidChannelState       = sdkerrors.Register(SubModuleName, 5, "invalid channel state")
	ErrInvalidChannelOrdering    = sdkerrors.Register(SubModuleName, 6, "invalid channel ordering")
	ErrInvalidCounterparty       = sdkerrors.Register(SubModuleName, 7, "invalid counterparty channel")
	ErrInvalidChannelCapability  = sdkerrors.Register(SubModuleName, 8, "invalid channel capability")
	ErrChannelCapabilityNotFound = sdkerrors.Register(SubModuleName, 9, "channel capability not found")
	ErrSequenceSendNotFound      = sdkerrors.Register(SubModuleName, 10, "sequence send not found")
	ErrSequenceReceiveNotFound   = sdkerrors.Register(SubModuleName, 11, "sequence receive not found")
	ErrSequenceAckNotFound       = sdkerrors.Register(SubModuleName, 12, "sequence acknowledgement not found")
	ErrInvalidPacket             = sdkerrors.Register(SubModuleName, 13, "invalid packet")
	ErrPacketTimeout             = sdkerrors.Register(SubModuleName, 14, "packet timeout")
	ErrTooManyConnectionHops     = sdkerrors.Register(SubModuleName, 15, "too many connection hops")
	ErrInvalidAcknowledgement    = sdkerrors.Register(SubModuleName, 16, "invalid acknowledgement")
	ErrAcknowledgementExists     = sdkerrors.Register(SubModuleName, 17, "acknowledgement for packet already exists")
	ErrInvalidChannelIdentifier  = sdkerrors.Register(SubModuleName, 18, "invalid channel identifier")

	// packets already relayed errors
	ErrPacketReceived           = sdkerrors.Register(SubModuleName, 19, "packet already received")
	ErrPacketCommitmentNotFound = sdkerrors.Register(SubModuleName, 20, "packet commitment not found") // may occur for already received acknowledgements or timeouts and in rare cases for packets never sent

	// ORDERED channel error
	ErrPacketSequenceOutOfOrder = sdkerrors.Register(SubModuleName, 21, "packet sequence is out of order")

	// Antehandler error
	ErrRedundantTx = sdkerrors.Register(SubModuleName, 22, "packet messages are redundant")

	// Perform a no-op on the current Msg
	ErrNoOpMsg = sdkerrors.Register(SubModuleName, 23, "message is redundant, no-op will be performed")

	ErrInvalidChannelVersion = sdkerrors.Register(SubModuleName, 24, "invalid channel version")
	ErrPacketNotSent         = sdkerrors.Register(SubModuleName, 25, "packet has not been sent")
	ErrInvalidTimeout        = sdkerrors.Register(SubModuleName, 26, "invalid packet timeout")
)
