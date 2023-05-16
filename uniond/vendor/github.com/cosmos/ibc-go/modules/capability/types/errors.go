package types

import (
	errorsmod "cosmossdk.io/errors"
)

// TODO: revert these codes to 2-8. These are incremented to avoid panics due to duplicate code registration in the
// sdk. Once we are using a version of the sdk which has removed the capability module, we can go back to the old
// codes.

var (
	ErrInvalidCapabilityName    = errorsmod.Register(ModuleName, 9, "capability name not valid")
	ErrNilCapability            = errorsmod.Register(ModuleName, 10, "provided capability is nil")
	ErrCapabilityTaken          = errorsmod.Register(ModuleName, 11, "capability name already taken")
	ErrOwnerClaimed             = errorsmod.Register(ModuleName, 12, "given owner already claimed capability")
	ErrCapabilityNotOwned       = errorsmod.Register(ModuleName, 13, "capability not owned by module")
	ErrCapabilityNotFound       = errorsmod.Register(ModuleName, 14, "capability not found")
	ErrCapabilityOwnersNotFound = errorsmod.Register(ModuleName, 15, "owners not found for capability")
)
