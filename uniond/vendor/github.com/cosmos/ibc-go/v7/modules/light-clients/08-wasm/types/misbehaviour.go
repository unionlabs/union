package types

import (
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"

	exported "github.com/cosmos/ibc-go/v7/modules/core/exported"
)

var _ exported.ClientMessage = (*Misbehaviour)(nil)

// ClientType is Wasm light client
func (m Misbehaviour) ClientType() string {
	return exported.Wasm
}

// ValidateBasic implements Misbehaviour interface
func (m Misbehaviour) ValidateBasic() error {
	if len(m.Data) == 0 {
		return sdkerrors.Wrap(ErrInvalidData, "data cannot be empty")
	}
	return nil
}
