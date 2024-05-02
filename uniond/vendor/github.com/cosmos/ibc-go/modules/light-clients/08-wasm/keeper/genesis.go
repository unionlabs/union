package keeper

import (
	wasmvm "github.com/CosmWasm/wasmvm/v2"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

// InitGenesis initializes the 08-wasm module's state from a provided genesis
// state.
func (k Keeper) InitGenesis(ctx sdk.Context, gs types.GenesisState) error {
	storeFn := func(code wasmvm.WasmCode, _ uint64) (wasmvm.Checksum, uint64, error) {
		checksum, err := k.GetVM().StoreCodeUnchecked(code)
		return checksum, 0, err
	}

	for _, contract := range gs.Contracts {
		_, err := k.storeWasmCode(ctx, contract.CodeBytes, storeFn)
		if err != nil {
			return err
		}
	}
	return nil
}

// ExportGenesis returns the 08-wasm module's exported genesis. This includes the code
// for all contracts previously stored.
func (k Keeper) ExportGenesis(ctx sdk.Context) types.GenesisState {
	checksums, err := k.GetAllChecksums(ctx)
	if err != nil {
		panic(err)
	}

	// Grab code from wasmVM and add to genesis state.
	var genesisState types.GenesisState
	for _, checksum := range checksums {
		code, err := k.GetVM().GetCode(checksum)
		if err != nil {
			panic(err)
		}
		genesisState.Contracts = append(genesisState.Contracts, types.Contract{
			CodeBytes: code,
		})
	}

	return genesisState
}
