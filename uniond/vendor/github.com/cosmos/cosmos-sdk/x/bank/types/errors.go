package types

import (
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// x/bank module sentinel errors
var (
	ErrNoInputs              = sdkerrors.Register(ModuleName, 2, "no inputs to send transaction")
	ErrNoOutputs             = sdkerrors.Register(ModuleName, 3, "no outputs to send transaction")
	ErrInputOutputMismatch   = sdkerrors.Register(ModuleName, 4, "sum inputs != sum outputs")
	ErrSendDisabled          = sdkerrors.Register(ModuleName, 5, "send transactions are disabled")
	ErrDenomMetadataNotFound = sdkerrors.Register(ModuleName, 6, "client denom metadata not found")
	ErrInvalidKey            = sdkerrors.Register(ModuleName, 7, "invalid key")
	ErrDuplicateEntry        = sdkerrors.Register(ModuleName, 8, "duplicate entry")
	ErrMultipleSenders       = sdkerrors.Register(ModuleName, 9, "multiple senders not allowed")
)
