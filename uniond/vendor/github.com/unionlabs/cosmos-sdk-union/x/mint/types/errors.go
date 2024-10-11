package types

import "cosmossdk.io/errors"

var ErrInvalidSigner = errors.Register(ModuleName, 1, "expected authority account as only signer for proposal message")
