package types

import (
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

/*
	This file is to allow for unexported functions and fields to be accessible to the testing package.
*/

// MaxWasmSize is the maximum size of a wasm code in bytes.
const MaxWasmSize = maxWasmSize

var (
	SubjectPrefix    = subjectPrefix
	SubstitutePrefix = substitutePrefix
)

// GetClientID is a wrapper around getClientID to allow the function to be directly called in tests.
func GetClientID(clientStore storetypes.KVStore) (string, error) {
	return getClientID(clientStore)
}

// NewMigrateProposalWrappedStore is a wrapper around newMigrateProposalWrappedStore to allow the function to be directly called in tests.
//
//nolint:revive // Returning unexported type for testing purposes.
func NewMigrateProposalWrappedStore(subjectStore, substituteStore storetypes.KVStore) migrateClientWrappedStore {
	return newMigrateClientWrappedStore(subjectStore, substituteStore)
}

// GetStore is a wrapper around getStore to allow the function to be directly called in tests.
func (ws migrateClientWrappedStore) GetStore(key []byte) (storetypes.KVStore, bool) {
	return ws.getStore(key)
}

// SplitPrefix is a wrapper around splitKey to allow the function to be directly called in tests.
func SplitPrefix(key []byte) ([]byte, []byte) {
	return splitPrefix(key)
}

// WasmQuery wraps wasmQuery and is used solely for testing.
func WasmQuery[T ContractResult](ctx sdk.Context, clientStore storetypes.KVStore, cs *ClientState, payload QueryMsg) (T, error) {
	return wasmQuery[T](ctx, clientStore, cs, payload)
}

// WasmSudo wraps wasmCall and is used solely for testing.
func WasmSudo[T ContractResult](ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, cs *ClientState, payload SudoMsg) (T, error) {
	return wasmSudo[T](ctx, cdc, clientStore, cs, payload)
}

// WasmInstantiate wraps wasmInstantiate and is used solely for testing.
func WasmInstantiate(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, cs *ClientState, payload InstantiateMessage) error {
	return wasmInstantiate(ctx, cdc, clientStore, cs, payload)
}

// WasmMigrate wraps wasmMigrate and is used solely for testing.
func WasmMigrate(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, cs *ClientState, clientID string, payload []byte) error {
	return wasmMigrate(ctx, cdc, clientStore, cs, clientID, payload)
}
