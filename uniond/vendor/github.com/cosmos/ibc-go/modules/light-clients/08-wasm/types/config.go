package types

import "path/filepath"

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

type WasmConfig struct {
	// DataDir is the directory for Wasm blobs and various caches
	DataDir string
	// SupportedCapabilities is a comma separated list of capabilities supported by the chain
	// See https://github.com/CosmWasm/wasmd/blob/e5049ba686ab71164a01f6e71e54347710a1f740/app/wasm.go#L3-L15
	// for more information.
	SupportedCapabilities string
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
		SupportedCapabilities: defaultSupportedCapabilities,
		ContractDebugMode:     defaultContractDebugMode,
	}
}
