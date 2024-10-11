package keeper

import (
	"context"

	"cosmossdk.io/x/epochs/types"
)

// InitGenesis sets epoch info from genesis
func (k Keeper) InitGenesis(ctx context.Context, genState types.GenesisState) error {
	for _, epoch := range genState.Epochs {
		err := k.AddEpochInfo(ctx, epoch)
		if err != nil {
			return err
		}
	}
	return nil
}

// ExportGenesis returns the capability module's exported genesis.
func (k Keeper) ExportGenesis(ctx context.Context) (*types.GenesisState, error) {
	genesis := types.DefaultGenesis()
	epochs, err := k.AllEpochInfos(ctx)
	if err != nil {
		return nil, err
	}
	genesis.Epochs = epochs
	return genesis, nil
}
