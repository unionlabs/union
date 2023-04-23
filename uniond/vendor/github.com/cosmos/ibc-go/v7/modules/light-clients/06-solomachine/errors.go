package solomachine

import (
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var (
	ErrInvalidHeader               = sdkerrors.Register(ModuleName, 2, "invalid header")
	ErrInvalidSequence             = sdkerrors.Register(ModuleName, 3, "invalid sequence")
	ErrInvalidSignatureAndData     = sdkerrors.Register(ModuleName, 4, "invalid signature and data")
	ErrSignatureVerificationFailed = sdkerrors.Register(ModuleName, 5, "signature verification failed")
	ErrInvalidProof                = sdkerrors.Register(ModuleName, 6, "invalid solo machine proof")
)
