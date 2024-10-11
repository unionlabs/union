package types

import "cosmossdk.io/collections"

const (
	// ModuleName defines the module name
	ModuleName = "evidence"

	// StoreKey defines the primary module store key
	StoreKey = ModuleName
)

// KVStore key prefixes
var (
	KeyPrefixEvidence = collections.NewPrefix(0)
)
