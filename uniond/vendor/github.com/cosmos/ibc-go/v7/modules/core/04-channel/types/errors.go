package types

import (
	errorsmod "cosmossdk.io/errors"
)

// IBC channel sentinel errors
var (
	ErrChannelExists             = errorsmod.Register(SubModuleName, 2, "channel already exists")
	ErrChannelNotFound           = errorsmod.Register(SubModuleName, 3, "channel not found")
	ErrInvalidChannel            = errorsmod.Register(SubModuleName, 4, "invalid channel")
	ErrInvalidChannelState       = errorsmod.Register(SubModuleName, 5, "invalid channel state")
	ErrInvalidChannelOrdering    = errorsmod.Register(SubModuleName, 6, "invalid channel ordering")
	ErrInvalidCounterparty       = errorsmod.Register(SubModuleName, 7, "invalid counterparty channel")
	ErrInvalidChannelCapability  = errorsmod.Register(SubModuleName, 8, "invalid channel capability")
	ErrChannelCapabilityNotFound = errorsmod.Register(SubModuleName, 9, "channel capability not found")
	ErrSequenceSendNotFound      = errorsmod.Register(SubModuleName, 10, "sequence send not found")
	ErrSequenceReceiveNotFound   = errorsmod.Register(SubModuleName, 11, "sequence receive not found")
	ErrSequenceAckNotFound       = errorsmod.Register(SubModuleName, 12, "sequence acknowledgement not found")
	ErrInvalidPacket             = errorsmod.Register(SubModuleName, 13, "invalid packet")
	ErrPacketTimeout             = errorsmod.Register(SubModuleName, 14, "packet timeout")
	ErrTooManyConnectionHops     = errorsmod.Register(SubModuleName, 15, "too many connection hops")
	ErrInvalidAcknowledgement    = errorsmod.Register(SubModuleName, 16, "invalid acknowledgement")
	ErrAcknowledgementExists     = errorsmod.Register(SubModuleName, 17, "acknowledgement for packet already exists")
	ErrInvalidChannelIdentifier  = errorsmod.Register(SubModuleName, 18, "invalid channel identifier")

	// packets already relayed errors
	ErrPacketReceived           = errorsmod.Register(SubModuleName, 19, "packet already received")
	ErrPacketCommitmentNotFound = errorsmod.Register(SubModuleName, 20, "packet commitment not found") // may occur for already received acknowledgements or timeouts and in rare cases for packets never sent

	// ORDERED channel error
	ErrPacketSequenceOutOfOrder = errorsmod.Register(SubModuleName, 21, "packet sequence is out of order")

	// Antehandler error
	ErrRedundantTx = errorsmod.Register(SubModuleName, 22, "packet messages are redundant")

	// Perform a no-op on the current Msg
	ErrNoOpMsg = errorsmod.Register(SubModuleName, 23, "message is redundant, no-op will be performed")

	ErrInvalidChannelVersion = errorsmod.Register(SubModuleName, 24, "invalid channel version")
	ErrPacketNotSent         = errorsmod.Register(SubModuleName, 25, "packet has not been sent")
	ErrInvalidTimeout        = errorsmod.Register(SubModuleName, 26, "invalid packet timeout")
)
