// This file contains the part of the API that is exposed when cgo is disabled.

package cosmwasm

import (
	"bytes"
	"crypto/sha256"
	"fmt"

	"github.com/CosmWasm/wasmvm/types"
)

// Checksum represents a hash of the Wasm bytecode that serves as an ID. Must be generated from this library.
type Checksum = types.Checksum

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

// CreateChecksum performs the hashing of Wasm bytes to obtain the CosmWasm checksum.
//
// Ony Wasm blobs are allowed as inputs and a magic byte check will be performed
// to avoid accidental misusage.
func CreateChecksum(wasm []byte) (Checksum, error) {
	if len(wasm) == 0 {
		return Checksum{}, fmt.Errorf("Wasm bytes nil or empty")
	}
	if len(wasm) < 4 {
		return Checksum{}, fmt.Errorf("Wasm bytes shorter than 4 bytes")
	}
	// magic number for Wasm is "\0asm"
	// See https://webassembly.github.io/spec/core/binary/modules.html#binary-module
	if !bytes.Equal(wasm[:4], []byte("\x00\x61\x73\x6D")) {
		return Checksum{}, fmt.Errorf("Wasm bytes do not not start with Wasm magic number")
	}
	hash := sha256.Sum256(wasm)
	return Checksum(hash[:]), nil
}
