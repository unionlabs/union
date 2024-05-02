package types

import (
	"path/filepath"
	"strings"
)

const (
	// ContractMemoryLimit is the memory limit of each contract execution (in MiB)
	// constant value so all nodes run with the same limit.
	ContractMemoryLimit = 32
	// MemoryCacheSize is the size of the wasm vm cache (in MiB), it is set to 0 to reduce unnecessary memory usage.
	// See: https://github.com/CosmWasm/cosmwasm/pull/1925
	MemoryCacheSize = 0

	defaultDataDir               string = "ibc_08-wasm_client_data"
	defaultSupportedCapabilities string = "iterator"
	defaultContractDebugMode            = false
)

// WasmConfig defines configuration parameters for the 08-wasm wasm virtual machine instance.
// It includes the `dataDir` intended to be used for wasm blobs and internal caches, as well as a comma separated list
// of features or capabilities the user wishes to enable. A boolean flag is provided to enable debug mode.
type WasmConfig struct {
	// DataDir is the directory for Wasm blobs and various caches
	DataDir string
	// SupportedCapabilities is a slice of capabilities supported by the chain
	// See https://github.com/CosmWasm/wasmd/blob/9e44af168570391b0b69822952f206d35320d473/app/wasm.go#L3-L16
	// for more information.
	SupportedCapabilities []string
	// ContractDebugMode is a flag to log what contracts print. It must be false on all
	// production nodes, and only enabled in test environments or debug non-validating nodes.
	ContractDebugMode bool
}

// DefaultWasmConfig returns the default settings for WasmConfig.
// The homePath is the path to the directory where the data directory for
// Wasm blobs and caches will be stored.
func DefaultWasmConfig(homePath string) WasmConfig {
	return WasmConfig{
		DataDir:               filepath.Join(homePath, defaultDataDir),
		SupportedCapabilities: strings.Split(defaultSupportedCapabilities, ","),
		ContractDebugMode:     defaultContractDebugMode,
	}
}
