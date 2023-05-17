// This file contains the part of the API that is exposed when cgo is disabled.

package cosmwasm

import (
	"github.com/CosmWasm/wasmvm/types"
)

// Checksum represents a hash of the Wasm bytecode that serves as an ID. Must be generated from this library.
type Checksum []byte

// WasmCode is an alias for raw bytes of the wasm compiled code
type WasmCode []byte

// KVStore is a reference to some sub-kvstore that is valid for one instance of a code
type KVStore = types.KVStore

// GoAPI is a reference to some "precompiles", go callbacks
type GoAPI = types.GoAPI

// Querier lets us make read-only queries on other modules
type Querier = types.Querier

// GasMeter is a read-only version of the sdk gas meter
type GasMeter = types.GasMeter

// LibwasmvmVersion returns the version of the loaded library
// at runtime. This can be used for debugging to verify the loaded version
// matches the expected version.
//
// When cgo is disabled at build time, this returns an error at runtime.
func LibwasmvmVersion() (string, error) {
	return libwasmvmVersionImpl()
}
