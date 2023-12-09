package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/crisis/types"
)

// new crisis genesis
func (k *Keeper) InitGenesis(ctx sdk.Context, data *types.GenesisState) {
	if err := k.ConstantFee.Set(ctx, data.ConstantFee); err != nil {
		panic(err)
	}
}

// ExportGenesis returns a GenesisState for a given context and keeper.
func (k *Keeper) ExportGenesis(ctx sdk.Context) *types.GenesisState {
	constantFee, err := k.ConstantFee.Get(ctx)
	if err != nil {
		panic(err)
	}
	return types.NewGenesisState(constantFee)
}
