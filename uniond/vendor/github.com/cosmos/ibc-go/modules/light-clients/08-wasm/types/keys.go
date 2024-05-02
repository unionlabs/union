package types

import "cosmossdk.io/collections"

const (
	// ModuleName for the wasm client
	ModuleName = "08-wasm"

	// StoreKey is the store key string for 08-wasm
	StoreKey = ModuleName

	// Wasm is the client type for IBC light clients created using 08-wasm
	Wasm = ModuleName

	// KeyChecksums is the key under which all checksums are stored
	// Deprecated: in favor of collections.KeySet
	KeyChecksums = "checksums"
)

// ChecksumsKey is the key under which all checksums are stored
var ChecksumsKey = collections.NewPrefix(0)
