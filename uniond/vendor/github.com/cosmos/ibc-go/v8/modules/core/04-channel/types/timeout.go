package types

import (
	errorsmod "cosmossdk.io/errors"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
)

// NewTimeout returns a new Timeout instance.
func NewTimeout(height clienttypes.Height, timestamp uint64) Timeout {
	return Timeout{
		Height:    height,
		Timestamp: timestamp,
	}
}

// IsValid returns true if either the height or timestamp is non-zero.
func (t Timeout) IsValid() bool {
	return !t.Height.IsZero() || t.Timestamp != 0
}

// Elapsed returns true if either the provided height or timestamp is past the
// respective absolute timeout values.
func (t Timeout) Elapsed(height clienttypes.Height, timestamp uint64) bool {
	return t.heightElapsed(height) || t.timestampElapsed(timestamp)
}

// ErrTimeoutElapsed returns a timeout elapsed error indicating which timeout value
// has elapsed.
func (t Timeout) ErrTimeoutElapsed(height clienttypes.Height, timestamp uint64) error {
	if t.heightElapsed(height) {
		return errorsmod.Wrapf(ErrTimeoutElapsed, "current height: %s, timeout height %s", height, t.Height)
	}

	return errorsmod.Wrapf(ErrTimeoutElapsed, "current timestamp: %d, timeout timestamp %d", timestamp, t.Timestamp)
}

// ErrTimeoutNotReached returns a timeout not reached error indicating which timeout value
// has not been reached.
func (t Timeout) ErrTimeoutNotReached(height clienttypes.Height, timestamp uint64) error {
	// only return height information if the height is set
	// t.heightElapsed() will return false when it is empty
	if !t.Height.IsZero() && !t.heightElapsed(height) {
		return errorsmod.Wrapf(ErrTimeoutNotReached, "current height: %s, timeout height %s", height, t.Height)
	}

	return errorsmod.Wrapf(ErrTimeoutNotReached, "current timestamp: %d, timeout timestamp %d", timestamp, t.Timestamp)
}

// heightElapsed returns true if the timeout height is non empty
// and the timeout height is greater than or equal to the relative height.
func (t Timeout) heightElapsed(height clienttypes.Height) bool {
	return !t.Height.IsZero() && height.GTE(t.Height)
}

// timestampElapsed returns true if the timeout timestamp is non empty
// and the timeout timestamp is greater than or equal to the relative timestamp.
func (t Timeout) timestampElapsed(timestamp uint64) bool {
	return t.Timestamp != 0 && timestamp >= t.Timestamp
}
