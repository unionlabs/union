package types

import (
	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var _ exported.ClientMessage = &ClientMessage{}

// ClientType is a Wasm light client.
func (ClientMessage) ClientType() string {
	return exported.Wasm
}

// ValidateBasic defines a basic validation for the wasm client message.
func (c ClientMessage) ValidateBasic() error {
	if len(c.Data) == 0 {
		return errorsmod.Wrap(ErrInvalidData, "data cannot be empty")
	}

	return nil
}
