package host

import (
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// SubModuleName defines the ICS 24 host
const SubModuleName = "host"

// IBC client sentinel errors
var (
	ErrInvalidID     = sdkerrors.Register(SubModuleName, 2, "invalid identifier")
	ErrInvalidPath   = sdkerrors.Register(SubModuleName, 3, "invalid path")
	ErrInvalidPacket = sdkerrors.Register(SubModuleName, 4, "invalid packet")
)
