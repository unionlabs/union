package keeper

import (
	"context"

	wasmvm "github.com/CosmWasm/wasmvm/v2"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

// InitGenesis initializes the 08-wasm module's state from a provided genesis
// state.
func (k Keeper) InitGenesis(ctx context.Context, state types.GenesisState) {
	storeFn := func(code wasmvm.WasmCode, _ uint64) (wasmvm.Checksum, uint64, error) {
		checksum, err := ibcwasm.GetVM().StoreCodeUnchecked(code)
		return checksum, 0, err
	}

	for _, contract := range state.Contracts {
		_, err := k.storeWasmCode(ctx, contract.CodeBytes, storeFn)
		if err != nil {
			panic(err)
		}
	}
}

// ExportGenesis returns the 08-wasm module's exported genesis. This includes the code
// for all contracts previously stored.
func (k Keeper) ExportGenesis(ctx context.Context) *types.GenesisState {
	checksums, err := types.GetAllChecksums(ctx)
	if err != nil {
		panic(err)
	}

	// Grab code from wasmVM and add to genesis state.
	var genesisState types.GenesisState
	for _, checksum := range checksums {
		code, err := ibcwasm.GetVM().GetCode(checksum)
		if err != nil {
			panic(err)
		}
		genesisState.Contracts = append(genesisState.Contracts, types.Contract{
			CodeBytes: code,
		})
	}

	return &genesisState
}
