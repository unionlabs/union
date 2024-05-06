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

	ErrInvalidChannelVersion           = errorsmod.Register(SubModuleName, 24, "invalid channel version")
	ErrPacketNotSent                   = errorsmod.Register(SubModuleName, 25, "packet has not been sent")
	ErrInvalidTimeout                  = errorsmod.Register(SubModuleName, 26, "invalid packet timeout")
	ErrUpgradeErrorNotFound            = errorsmod.Register(SubModuleName, 27, "upgrade error receipt not found")
	ErrInvalidUpgrade                  = errorsmod.Register(SubModuleName, 28, "invalid upgrade")
	ErrInvalidUpgradeSequence          = errorsmod.Register(SubModuleName, 29, "invalid upgrade sequence")
	ErrUpgradeNotFound                 = errorsmod.Register(SubModuleName, 30, "upgrade not found")
	ErrIncompatibleCounterpartyUpgrade = errorsmod.Register(SubModuleName, 31, "incompatible counterparty upgrade")
	ErrInvalidUpgradeError             = errorsmod.Register(SubModuleName, 32, "invalid upgrade error")
	ErrUpgradeRestoreFailed            = errorsmod.Register(SubModuleName, 33, "restore failed")
	ErrUpgradeTimeout                  = errorsmod.Register(SubModuleName, 34, "upgrade timed-out")
	ErrInvalidUpgradeTimeout           = errorsmod.Register(SubModuleName, 35, "upgrade timeout is invalid")
	ErrPendingInflightPackets          = errorsmod.Register(SubModuleName, 36, "pending inflight packets exist")
	ErrUpgradeTimeoutFailed            = errorsmod.Register(SubModuleName, 37, "upgrade timeout failed")
	ErrInvalidPruningLimit             = errorsmod.Register(SubModuleName, 38, "invalid pruning limit")
	ErrTimeoutNotReached               = errorsmod.Register(SubModuleName, 39, "timeout not reached")
	ErrTimeoutElapsed                  = errorsmod.Register(SubModuleName, 40, "timeout elapsed")
	ErrPruningSequenceStartNotFound    = errorsmod.Register(SubModuleName, 41, "pruning sequence start not found")
	ErrRecvStartSequenceNotFound       = errorsmod.Register(SubModuleName, 42, "recv start sequence not found")
)
