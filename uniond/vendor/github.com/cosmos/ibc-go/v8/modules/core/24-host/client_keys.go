package host

import (
	"fmt"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// KeyClientStorePrefix defines the KVStore key prefix for IBC clients
var KeyClientStorePrefix = []byte("clients")

const (
	KeyClientState          = "clientState"
	KeyConsensusStatePrefix = "consensusStates"
)

// FullClientPath returns the full path of a specific client path in the format:
// "clients/{clientID}/{path}" as a string.
func FullClientPath(clientID string, path string) string {
	return fmt.Sprintf("%s/%s/%s", KeyClientStorePrefix, clientID, path)
}

// FullClientKey returns the full path of specific client path in the format:
// "clients/{clientID}/{path}" as a byte array.
func FullClientKey(clientID string, path []byte) []byte {
	return []byte(FullClientPath(clientID, string(path)))
}

// PrefixedClientStorePath returns a key path which can be used for prefixed
// key store iteration. The prefix may be a clientType, clientID, or any
// valid key prefix which may be concatenated with the client store constant.
func PrefixedClientStorePath(prefix []byte) string {
	return fmt.Sprintf("%s/%s", KeyClientStorePrefix, prefix)
}

// PrefixedClientStoreKey returns a key which can be used for prefixed
// key store iteration. The prefix may be a clientType, clientID, or any
// valid key prefix which may be concatenated with the client store constant.
func PrefixedClientStoreKey(prefix []byte) []byte {
	return []byte(PrefixedClientStorePath(prefix))
}

// ICS02
// The following paths are the keys to the store as defined in https://github.com/cosmos/ibc/tree/master/spec/core/ics-002-client-semantics#path-space

// FullClientStatePath takes a client identifier and returns a Path under which to store a
// particular client state
func FullClientStatePath(clientID string) string {
	return FullClientPath(clientID, KeyClientState)
}

// FullClientStateKey takes a client identifier and returns a Key under which to store a
// particular client state.
func FullClientStateKey(clientID string) []byte {
	return FullClientKey(clientID, []byte(KeyClientState))
}

// ClientStateKey returns a store key under which a particular client state is stored
// in a client prefixed store
func ClientStateKey() []byte {
	return []byte(KeyClientState)
}

// FullConsensusStatePath takes a client identifier and returns a Path under which to
// store the consensus state of a client.
func FullConsensusStatePath(clientID string, height exported.Height) string {
	return FullClientPath(clientID, ConsensusStatePath(height))
}

// FullConsensusStateKey returns the store key for the consensus state of a particular
// client.
func FullConsensusStateKey(clientID string, height exported.Height) []byte {
	return []byte(FullConsensusStatePath(clientID, height))
}

// ConsensusStatePath returns the suffix store key for the consensus state at a
// particular height stored in a client prefixed store.
func ConsensusStatePath(height exported.Height) string {
	return fmt.Sprintf("%s/%s", KeyConsensusStatePrefix, height)
}

// ConsensusStateKey returns the store key for a the consensus state of a particular
// client stored in a client prefixed store.
func ConsensusStateKey(height exported.Height) []byte {
	return []byte(ConsensusStatePath(height))
}
