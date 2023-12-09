package types

import "cosmossdk.io/errors"

// x/bank module sentinel errors
var (
	ErrNoInputs              = errors.Register(ModuleName, 2, "no inputs to send transaction")
	ErrNoOutputs             = errors.Register(ModuleName, 3, "no outputs to send transaction")
	ErrInputOutputMismatch   = errors.Register(ModuleName, 4, "sum inputs != sum outputs")
	ErrSendDisabled          = errors.Register(ModuleName, 5, "send transactions are disabled")
	ErrDenomMetadataNotFound = errors.Register(ModuleName, 6, "client denom metadata not found")
	ErrInvalidKey            = errors.Register(ModuleName, 7, "invalid key")
	ErrDuplicateEntry        = errors.Register(ModuleName, 8, "duplicate entry")
	ErrMultipleSenders       = errors.Register(ModuleName, 9, "multiple senders not allowed")
)
