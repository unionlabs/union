package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v7/modules/light-clients/08-wasm/types"
)

// InitGenesis initializes the 08-wasm module's state from a provided genesis
// state.
func (k Keeper) InitGenesis(ctx sdk.Context, gs types.GenesisState) error {
	for _, contract := range gs.Contracts {
		err := k.importWasmCode(ctx, contract.CodeIdKey, contract.ContractCode)
		if err != nil {
			return err
		}
	}
	return nil
}

// ExportGenesis returns the 08-wasm module's exported genesis.
func (k Keeper) ExportGenesis(ctx sdk.Context) types.GenesisState {
	store := ctx.KVStore(k.storeKey)
	iterator := sdk.KVStorePrefixIterator(store, []byte(types.KeyCodeIDPrefix))
	defer iterator.Close()

	var genesisState types.GenesisState
	for ; iterator.Valid(); iterator.Next() {
		genesisState.Contracts = append(genesisState.Contracts, types.GenesisContract{
			CodeIdKey:    iterator.Key(),
			ContractCode: iterator.Value(),
		})
	}
	return genesisState
}
