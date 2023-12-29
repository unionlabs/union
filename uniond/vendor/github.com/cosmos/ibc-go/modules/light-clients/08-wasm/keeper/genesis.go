package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

// InitGenesis initializes the 08-wasm module's state from a provided genesis
// state.
func (k Keeper) InitGenesis(ctx sdk.Context, gs types.GenesisState) error {
	for _, contract := range gs.Contracts {
		_, err := k.storeWasmCode(ctx, contract.CodeBytes, ibcwasm.GetVM().StoreCodeUnchecked)
		if err != nil {
			return err
		}
	}
	return nil
}

// ExportGenesis returns the 08-wasm module's exported genesis. This includes the code
// for all contracts previously stored.
func (Keeper) ExportGenesis(ctx sdk.Context) types.GenesisState {
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

	return genesisState
}
