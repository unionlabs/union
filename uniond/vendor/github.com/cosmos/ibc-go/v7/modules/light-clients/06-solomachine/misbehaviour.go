package solomachine

import (
	"bytes"

	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"

	clienttypes "github.com/cosmos/ibc-go/v7/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"
)

var _ exported.ClientMessage = &Misbehaviour{}

// ClientType is a Solo Machine light client.
func (misbehaviour Misbehaviour) ClientType() string {
	return exported.Solomachine
}

// Type implements Misbehaviour interface.
func (misbehaviour Misbehaviour) Type() string {
	return exported.TypeClientMisbehaviour
}

// ValidateBasic implements Misbehaviour interface.
func (misbehaviour Misbehaviour) ValidateBasic() error {
	if misbehaviour.Sequence == 0 {
		return sdkerrors.Wrap(clienttypes.ErrInvalidMisbehaviour, "sequence cannot be 0")
	}

	if err := misbehaviour.SignatureOne.ValidateBasic(); err != nil {
		return sdkerrors.Wrap(err, "signature one failed basic validation")
	}

	if err := misbehaviour.SignatureTwo.ValidateBasic(); err != nil {
		return sdkerrors.Wrap(err, "signature two failed basic validation")
	}

	// misbehaviour signatures cannot be identical.
	if bytes.Equal(misbehaviour.SignatureOne.Signature, misbehaviour.SignatureTwo.Signature) {
		return sdkerrors.Wrap(clienttypes.ErrInvalidMisbehaviour, "misbehaviour signatures cannot be equal")
	}

	// message data signed cannot be identical if both paths are the same.
	if bytes.Equal(misbehaviour.SignatureOne.Path, misbehaviour.SignatureTwo.Path) &&
		bytes.Equal(misbehaviour.SignatureOne.Data, misbehaviour.SignatureTwo.Data) {
		return sdkerrors.Wrap(clienttypes.ErrInvalidMisbehaviour, "misbehaviour signature data must be signed over different messages")
	}

	return nil
}

// ValidateBasic ensures that the signature and data fields are non-empty.
func (sd SignatureAndData) ValidateBasic() error {
	if len(sd.Signature) == 0 {
		return sdkerrors.Wrap(ErrInvalidSignatureAndData, "signature cannot be empty")
	}
	if len(sd.Data) == 0 {
		return sdkerrors.Wrap(ErrInvalidSignatureAndData, "data for signature cannot be empty")
	}
	if len(sd.Path) == 0 {
		return sdkerrors.Wrap(ErrInvalidSignatureAndData, "path for signature cannot be empty")
	}
	if sd.Timestamp == 0 {
		return sdkerrors.Wrap(ErrInvalidSignatureAndData, "timestamp cannot be 0")
	}

	return nil
}
