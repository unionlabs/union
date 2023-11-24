package types

import "cosmossdk.io/errors"

// x/crisis module sentinel errors
var (
	ErrNoSender         = errors.Register(ModuleName, 2, "sender address is empty")
	ErrUnknownInvariant = errors.Register(ModuleName, 3, "unknown invariant")
)
