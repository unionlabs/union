package solomachine

import (
	errorsmod "cosmossdk.io/errors"
)

var (
	ErrInvalidHeader               = errorsmod.Register(ModuleName, 2, "invalid header")
	ErrInvalidSequence             = errorsmod.Register(ModuleName, 3, "invalid sequence")
	ErrInvalidSignatureAndData     = errorsmod.Register(ModuleName, 4, "invalid signature and data")
	ErrSignatureVerificationFailed = errorsmod.Register(ModuleName, 5, "signature verification failed")
	ErrInvalidProof                = errorsmod.Register(ModuleName, 6, "invalid solo machine proof")
)
