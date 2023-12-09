package types

import (
	errorsmod "cosmossdk.io/errors"
)

// ICA Controller sentinel errors
var (
	ErrControllerSubModuleDisabled = errorsmod.Register(SubModuleName, 2, "controller submodule is disabled")
)
