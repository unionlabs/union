package types

import (
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// IBC transfer sentinel errors
var (
	ErrInvalidPacketTimeout    = sdkerrors.Register(ModuleName, 2, "invalid packet timeout")
	ErrInvalidDenomForTransfer = sdkerrors.Register(ModuleName, 3, "invalid denomination for cross-chain transfer")
	ErrInvalidVersion          = sdkerrors.Register(ModuleName, 4, "invalid ICS20 version")
	ErrInvalidAmount           = sdkerrors.Register(ModuleName, 5, "invalid token amount")
	ErrTraceNotFound           = sdkerrors.Register(ModuleName, 6, "denomination trace not found")
	ErrSendDisabled            = sdkerrors.Register(ModuleName, 7, "fungible token transfers from this chain are disabled")
	ErrReceiveDisabled         = sdkerrors.Register(ModuleName, 8, "fungible token transfers to this chain are disabled")
	ErrMaxTransferChannels     = sdkerrors.Register(ModuleName, 9, "max transfer channels")
	ErrInvalidAuthorization    = sdkerrors.Register(ModuleName, 10, "invalid transfer authorization")
)
