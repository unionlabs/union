package types

import (
	errorsmod "cosmossdk.io/errors"
)

// IBC transfer sentinel errors
var (
	ErrInvalidPacketTimeout    = errorsmod.Register(ModuleName, 2, "invalid packet timeout")
	ErrInvalidDenomForTransfer = errorsmod.Register(ModuleName, 3, "invalid denomination for cross-chain transfer")
	ErrInvalidVersion          = errorsmod.Register(ModuleName, 4, "invalid ICS20 version")
	ErrInvalidAmount           = errorsmod.Register(ModuleName, 5, "invalid token amount")
	ErrTraceNotFound           = errorsmod.Register(ModuleName, 6, "denomination trace not found")
	ErrSendDisabled            = errorsmod.Register(ModuleName, 7, "fungible token transfers from this chain are disabled")
	ErrReceiveDisabled         = errorsmod.Register(ModuleName, 8, "fungible token transfers to this chain are disabled")
	ErrMaxTransferChannels     = errorsmod.Register(ModuleName, 9, "max transfer channels")
	ErrInvalidAuthorization    = errorsmod.Register(ModuleName, 10, "invalid transfer authorization")
	ErrInvalidMemo             = errorsmod.Register(ModuleName, 11, "invalid memo")
)
