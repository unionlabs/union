package types

import (
	"cosmossdk.io/collections"
)

const (
	// ModuleName defines the module name.
	ModuleName = "epochs"

	// StoreKey defines the primary module store key.
	StoreKey = ModuleName
)

// KeyPrefixEpoch defines prefix key for storing epochs.
var KeyPrefixEpoch = collections.NewPrefix(1)
