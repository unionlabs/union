package host

import (
	errorsmod "cosmossdk.io/errors"
)

// SubModuleName defines the ICS 24 host
const SubModuleName = "host"

// IBC client sentinel errors
var (
	ErrInvalidID     = errorsmod.Register(SubModuleName, 2, "invalid identifier")
	ErrInvalidPath   = errorsmod.Register(SubModuleName, 3, "invalid path")
	ErrInvalidPacket = errorsmod.Register(SubModuleName, 4, "invalid packet")
)
