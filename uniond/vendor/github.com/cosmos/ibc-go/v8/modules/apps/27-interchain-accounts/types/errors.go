package types

import (
	errorsmod "cosmossdk.io/errors"
)

var (
	ErrUnknownDataType             = errorsmod.Register(ModuleName, 2, "unknown data type")
	ErrAccountAlreadyExist         = errorsmod.Register(ModuleName, 3, "account already exist")
	ErrPortAlreadyBound            = errorsmod.Register(ModuleName, 4, "port is already bound")
	ErrInvalidChannelFlow          = errorsmod.Register(ModuleName, 5, "invalid message sent to channel end")
	ErrInvalidOutgoingData         = errorsmod.Register(ModuleName, 6, "invalid outgoing data")
	ErrInvalidRoute                = errorsmod.Register(ModuleName, 7, "invalid route")
	ErrInterchainAccountNotFound   = errorsmod.Register(ModuleName, 8, "interchain account not found")
	ErrInterchainAccountAlreadySet = errorsmod.Register(ModuleName, 9, "interchain account is already set")
	ErrActiveChannelAlreadySet     = errorsmod.Register(ModuleName, 10, "active channel already set for this owner")
	ErrActiveChannelNotFound       = errorsmod.Register(ModuleName, 11, "no active channel for this owner")
	ErrInvalidVersion              = errorsmod.Register(ModuleName, 12, "invalid interchain accounts version")
	ErrInvalidAccountAddress       = errorsmod.Register(ModuleName, 13, "invalid account address")
	ErrUnsupported                 = errorsmod.Register(ModuleName, 14, "interchain account does not support this action")
	ErrInvalidControllerPort       = errorsmod.Register(ModuleName, 15, "invalid controller port")
	ErrInvalidHostPort             = errorsmod.Register(ModuleName, 16, "invalid host port")
	ErrInvalidTimeoutTimestamp     = errorsmod.Register(ModuleName, 17, "timeout timestamp must be in the future")
	ErrInvalidCodec                = errorsmod.Register(ModuleName, 18, "codec is not supported")
	ErrInvalidAccountReopening     = errorsmod.Register(ModuleName, 19, "invalid account reopening")
)
