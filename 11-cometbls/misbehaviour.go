package cometbls

import (
	"time"

	errorsmod "cosmossdk.io/errors"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var _ exported.ClientMessage = (*Misbehaviour)(nil)

// FrozenHeight is same for all misbehaviour
var FrozenHeight = clienttypes.NewHeight(0, 1)

// NewMisbehaviour creates a new Misbehaviour instance.
func NewMisbehaviour(clientID string, header1, header2 *Header) *Misbehaviour {
	return &Misbehaviour{
		Header_1: header1,
		Header_2: header2,
	}
}

// ClientType is Tendermint light client
func (Misbehaviour) ClientType() string {
	return ClientType
}

// GetTime returns the timestamp at which misbehaviour occurred. It uses the
// maximum value from both headers to prevent producing an invalid header outside
// of the misbehaviour age range.
func (misbehaviour Misbehaviour) GetTime() time.Time {
	t1, t2 := misbehaviour.Header_1.GetTime(), misbehaviour.Header_2.GetTime()
	if t1.After(t2) {
		return t1
	}
	return t2
}

// ValidateBasic implements Misbehaviour interface
func (misbehaviour Misbehaviour) ValidateBasic() error {
	if misbehaviour.Header_1 == nil {
		return errorsmod.Wrap(ErrInvalidHeader, "misbehaviour Header_1 cannot be nil")
	}
	if misbehaviour.Header_2 == nil {
		return errorsmod.Wrap(ErrInvalidHeader, "misbehaviour Header_2 cannot be nil")
	}
	if misbehaviour.Header_1.TrustedHeight.RevisionHeight == 0 {
		return errorsmod.Wrapf(ErrInvalidHeaderHeight, "misbehaviour Header_1 cannot have zero revision height")
	}
	if misbehaviour.Header_2.TrustedHeight.RevisionHeight == 0 {
		return errorsmod.Wrapf(ErrInvalidHeaderHeight, "misbehaviour Header_2 cannot have zero revision height")
	}

	// ValidateBasic on both validators
	if err := misbehaviour.Header_1.ValidateBasic(); err != nil {
		return errorsmod.Wrap(
			clienttypes.ErrInvalidMisbehaviour,
			errorsmod.Wrap(err, "header 1 failed validation").Error(),
		)
	}
	if err := misbehaviour.Header_2.ValidateBasic(); err != nil {
		return errorsmod.Wrap(
			clienttypes.ErrInvalidMisbehaviour,
			errorsmod.Wrap(err, "header 2 failed validation").Error(),
		)
	}
	// Ensure that Height1 is greater than or equal to Height2
	if misbehaviour.Header_1.GetHeight().LT(misbehaviour.Header_2.GetHeight()) {
		return errorsmod.Wrapf(clienttypes.ErrInvalidMisbehaviour, "Header_1 height is less than Header_2 height (%s < %s)", misbehaviour.Header_1.GetHeight(), misbehaviour.Header_2.GetHeight())
	}

	return nil
}
