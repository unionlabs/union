package simulation

import (
	"github.com/cosmos/cosmos-sdk/types/module"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// RandomizeGenState generates a random GenesisState for wasm
func RandomizedGenState(simstate *module.SimulationState) {
	params := types.DefaultParams()
	wasmGenesis := types.GenesisState{
		Params:    params,
		Codes:     nil,
		Contracts: nil,
		Sequences: []types.Sequence{
			{IDKey: types.KeySequenceCodeID, Value: simstate.Rand.Uint64() % 1_000_000_000},
			{IDKey: types.KeySequenceInstanceID, Value: simstate.Rand.Uint64() % 1_000_000_000},
		},
	}

	_, err := simstate.Cdc.MarshalJSON(&wasmGenesis)
	if err != nil {
		panic(err)
	}

	simstate.GenState[types.ModuleName] = simstate.Cdc.MustMarshalJSON(&wasmGenesis)
}
