package types

// DONTCOVER

import (
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// x/capability module sentinel errors
var (
	ErrInvalidCapabilityName    = sdkerrors.Register(ModuleName, 2, "capability name not valid")
	ErrNilCapability            = sdkerrors.Register(ModuleName, 3, "provided capability is nil")
	ErrCapabilityTaken          = sdkerrors.Register(ModuleName, 4, "capability name already taken")
	ErrOwnerClaimed             = sdkerrors.Register(ModuleName, 5, "given owner already claimed capability")
	ErrCapabilityNotOwned       = sdkerrors.Register(ModuleName, 6, "capability not owned by module")
	ErrCapabilityNotFound       = sdkerrors.Register(ModuleName, 7, "capability not found")
	ErrCapabilityOwnersNotFound = sdkerrors.Register(ModuleName, 8, "owners not found for capability")
)
