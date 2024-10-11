package types

import (
	errorsmod "cosmossdk.io/errors"
)

// SubModuleName is the error codespace
const SubModuleName string = "commitment"

// IBC connection sentinel errors
var (
	ErrInvalidProof       = errorsmod.Register(SubModuleName, 2, "invalid proof")
	ErrInvalidPrefix      = errorsmod.Register(SubModuleName, 3, "invalid prefix")
	ErrInvalidMerkleProof = errorsmod.Register(SubModuleName, 4, "invalid merkle proof")
)
