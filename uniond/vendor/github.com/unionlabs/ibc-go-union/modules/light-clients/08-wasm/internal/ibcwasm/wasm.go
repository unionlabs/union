package ibcwasm

import (
	"errors"

	"cosmossdk.io/collections"
	storetypes "cosmossdk.io/core/store"
)

var (
	vm WasmEngine

	queryRouter  QueryRouter
	queryPlugins QueryPluginsI

	// state management
	Schema    collections.Schema
	Checksums collections.KeySet[[]byte]

	// ChecksumsKey is the key under which all checksums are stored
	ChecksumsKey = collections.NewPrefix(0)
)

// SetVM sets the wasm VM for the 08-wasm module.
// It panics if the wasm VM is nil.
func SetVM(wasmVM WasmEngine) {
	if wasmVM == nil {
		panic(errors.New("wasm VM must be not nil"))
	}
	vm = wasmVM
}

// GetVM returns the wasm VM for the 08-wasm module.
func GetVM() WasmEngine {
	return vm
}

// SetQueryRouter sets the custom wasm query router for the 08-wasm module.
// Panics if the queryRouter is nil.
func SetQueryRouter(router QueryRouter) {
	if router == nil {
		panic(errors.New("query router must be not nil"))
	}
	queryRouter = router
}

// GetQueryRouter returns the custom wasm query router for the 08-wasm module.
func GetQueryRouter() QueryRouter {
	return queryRouter
}

// SetQueryPlugins sets the current query plugins
func SetQueryPlugins(plugins QueryPluginsI) {
	if plugins == nil {
		panic(errors.New("query plugins must be not nil"))
	}
	queryPlugins = plugins
}

// GetQueryPlugins returns the current query plugins
func GetQueryPlugins() QueryPluginsI {
	return queryPlugins
}

// SetupWasmStoreService sets up the 08-wasm module's collections.
func SetupWasmStoreService(storeService storetypes.KVStoreService) {
	sb := collections.NewSchemaBuilder(storeService)

	Checksums = collections.NewKeySet(sb, ChecksumsKey, "checksums", collections.BytesKey)

	schema, err := sb.Build()
	if err != nil {
		panic(err)
	}

	Schema = schema
}
