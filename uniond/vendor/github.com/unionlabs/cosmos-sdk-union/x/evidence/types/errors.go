package types

import "cosmossdk.io/errors"

// x/evidence module sentinel errors
var (
	ErrNoEvidenceHandlerExists = errors.Register(ModuleName, 2, "unregistered handler for evidence type")
	ErrInvalidEvidence         = errors.Register(ModuleName, 3, "invalid evidence")
	ErrEvidenceExists          = errors.Register(ModuleName, 5, "evidence already exists")
)
