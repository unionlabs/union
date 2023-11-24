package types

import (
	errorsmod "cosmossdk.io/errors"
)

// IBC port sentinel errors
var (
	ErrPortExists   = errorsmod.Register(SubModuleName, 2, "port is already binded")
	ErrPortNotFound = errorsmod.Register(SubModuleName, 3, "port not found")
	ErrInvalidPort  = errorsmod.Register(SubModuleName, 4, "invalid port")
	ErrInvalidRoute = errorsmod.Register(SubModuleName, 5, "route not found")
)
