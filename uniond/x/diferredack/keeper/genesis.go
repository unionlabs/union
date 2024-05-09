package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	"union/x/diferredack/types"
)

func (k Keeper) InitGenesis(ctx sdk.Context, genState types.GenesisState) {
}

func (k Keeper) ExportGenesis(ctx sdk.Context) *types.GenesisState {
	return &types.GenesisState{
		Params: k.GetParams(ctx),
	}
}
