package types

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	// ModuleName defines the module name
	ModuleName = "capability"

	// StoreKey defines the primary module store key
	StoreKey = ModuleName

	// MemStoreKey defines the in-memory store key
	MemStoreKey = "memory:capability"
)

var (
	// KeyIndex defines the key that stores the current globally unique capability
	// index.
	KeyIndex = []byte("index")

	// KeyPrefixIndexCapability defines a key prefix that stores index to capability
	// owners mappings.
	KeyPrefixIndexCapability = []byte("capability_index")

	// KeyMemInitialized defines the key that stores the initialized flag in the memory store
	KeyMemInitialized = []byte("mem_initialized")
)

// RevCapabilityKey returns a reverse lookup key for a given module and capability
// name.
func RevCapabilityKey(module, name string) []byte {
	return []byte(fmt.Sprintf("%s/rev/%s", module, name))
}

// FwdCapabilityKey returns a forward lookup key for a given module and capability
// reference.
func FwdCapabilityKey(module string, cap *Capability) []byte {
	return []byte(fmt.Sprintf("%s/fwd/%#016p", module, cap))
}

// IndexToKey returns bytes to be used as a key for a given capability index.
func IndexToKey(index uint64) []byte {
	return sdk.Uint64ToBigEndian(index)
}

// IndexFromKey returns an index from a call to IndexToKey for a given capability
// index.
func IndexFromKey(key []byte) uint64 {
	return sdk.BigEndianToUint64(key)
}
