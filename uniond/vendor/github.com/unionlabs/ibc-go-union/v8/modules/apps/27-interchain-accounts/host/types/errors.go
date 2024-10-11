package types

import (
	errorsmod "cosmossdk.io/errors"
)

// ICA Host sentinel errors
var (
	ErrHostSubModuleDisabled = errorsmod.Register(SubModuleName, 2, "host submodule is disabled")
)
