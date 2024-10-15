package keeper

// BuiltInCapabilities returns all capabilities currently supported by this version of x/wasm.
// See also https://github.com/CosmWasm/cosmwasm/blob/main/docs/CAPABILITIES-BUILT-IN.md.
//
// Use this directly or together with your chain's custom capabilities (if any):
//
//	append(wasmkeeper.BuiltInCapabilities(), "token_factory")
func BuiltInCapabilities() []string {
	return []string{
		"iterator",
		"staking",
		"stargate",
		"cosmwasm_1_1",
		"cosmwasm_1_2",
		"cosmwasm_1_3",
		"cosmwasm_1_4",
		"cosmwasm_2_0",
		"cosmwasm_2_1",
	}
}
