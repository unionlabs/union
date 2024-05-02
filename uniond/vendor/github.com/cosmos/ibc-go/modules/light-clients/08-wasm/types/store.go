package types

import (
	"fmt"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
)

// GetClientState retrieves the client state from the store using the provided KVStore and codec.
// It returns the unmarshaled ClientState and a boolean indicating if the state was found.
func GetClientState(store storetypes.KVStore, cdc codec.BinaryCodec) (*ClientState, bool) {
	bz := store.Get(host.ClientStateKey())
	if len(bz) == 0 {
		return nil, false
	}

	clientStateI := clienttypes.MustUnmarshalClientState(cdc, bz)
	var clientState *ClientState
	clientState, ok := clientStateI.(*ClientState)
	if !ok {
		panic(fmt.Errorf("cannot convert %T into %T", clientStateI, clientState))
	}
	return clientState, ok
}

// Checksum is a type alias used for wasm byte code checksums.
type Checksum = wasmvmtypes.Checksum

// CreateChecksum creates a sha256 checksum from the given wasm code, it forwards the
// call to the wasmvm package. The code is checked for the following conditions:
// - code length is zero.
// - code length is less than 4 bytes (magic number length).
// - code does not start with the wasm magic number.
func CreateChecksum(code []byte) (Checksum, error) {
	return wasmvm.CreateChecksum(code)
}
