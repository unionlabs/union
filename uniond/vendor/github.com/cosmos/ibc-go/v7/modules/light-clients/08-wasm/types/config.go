package types

import "path/filepath"

const (
	// contractMemoryLimit is the memory limit of each contract execution (in MiB)
	// constant value so all nodes run with the same limit.
	ContractMemoryLimit = 32

	defaultDataDir           string = "ibc_08-wasm_client_data"
	defaultSupportedFeatures string = "iterator"
	defaultMemoryCacheSize   uint32 = 256 // in MiB
	defaultContractDebugMode        = false
)

type WasmConfig struct {
	// DataDir is the directory for Wasm blobs and various caches
	DataDir string
	// SupportedFeatures is a comma separated list of capabilities supported by the chain
	// See https://github.com/CosmWasm/wasmd/blob/e5049ba686ab71164a01f6e71e54347710a1f740/app/wasm.go#L3-L15
	// for more information.
	SupportedFeatures string
	// MemoryCacheSize in MiB not bytes. It is not consensus-critical and should
	// be defined on a per-node basis, often 100-1000 MB.
	MemoryCacheSize uint32
	// ContractDebugMode is a flag to log what contracts print. It must be false on all
	// production nodes, and only enabled in test environments or debug non-validating nodes.
	ContractDebugMode bool
}

// DefaultWasmConfig returns the default settings for WasmConfig.
// The homePath is the path to the directory where the data directory for
// Wasm blobs and caches will be stored.
func DefaultWasmConfig(homePath string) WasmConfig {
	return WasmConfig{
		DataDir:           filepath.Join(homePath, defaultDataDir),
		SupportedFeatures: defaultSupportedFeatures,
		MemoryCacheSize:   defaultMemoryCacheSize,
		ContractDebugMode: defaultContractDebugMode,
	}
}
