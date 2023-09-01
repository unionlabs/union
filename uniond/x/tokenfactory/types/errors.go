package types

// DONTCOVER

import (
	fmt "fmt"

	errorsmod "cosmossdk.io/errors"
)

// x/tokenfactory module sentinel errors
var (
	ErrDenomExists              = errorsmod.Register(ModuleName, 2, "attempting to create a denom that already exists (has bank metadata)")
	ErrUnauthorized             = errorsmod.Register(ModuleName, 3, "unauthorized account")
	ErrInvalidDenom             = errorsmod.Register(ModuleName, 4, "invalid denom")
	ErrInvalidCreator           = errorsmod.Register(ModuleName, 5, "invalid creator")
	ErrInvalidAuthorityMetadata = errorsmod.Register(ModuleName, 6, "invalid authority metadata")
	ErrInvalidGenesis           = errorsmod.Register(ModuleName, 7, "invalid genesis")
	ErrSubdenomTooLong          = errorsmod.Register(ModuleName, 8, fmt.Sprintf("subdenom too long, max length is %d bytes", MaxSubdenomLength))
	ErrCreatorTooLong           = errorsmod.Register(ModuleName, 9, fmt.Sprintf("creator too long, max length is %d bytes", MaxCreatorLength))
	ErrDenomDoesNotExist        = errorsmod.Register(ModuleName, 10, "denom does not exist")
	ErrCapabilityNotEnabled     = errorsmod.Register(ModuleName, 11, "this capability is not enabled on chain")
)
