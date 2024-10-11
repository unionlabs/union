package keeper

import (
	"context"

	"cosmossdk.io/x/mint/types"
)

// Migrator is a struct for handling in-place state migrations.
type Migrator struct {
	keeper Keeper
}

// NewMigrator returns Migrator instance for the state migration.
func NewMigrator(k Keeper) Migrator {
	return Migrator{
		keeper: k,
	}
}

// Migrate1to2 migrates the x/mint module state from the consensus version 1 to
// version 2. Specifically, it takes the parameters that are currently stored
// and managed by the x/params modules and stores them directly into the x/mint
// module state.
func (m Migrator) Migrate1to2(_ context.Context) error {
	return nil
}

// Migrate2to3 migrates the x/mint module state from the consensus version 2 to
// version 3.
func (m Migrator) Migrate2to3(ctx context.Context) error {
	params, err := m.keeper.Params.Get(ctx)
	if err != nil {
		return err
	}

	// Initialize the new MaxSupply parameter with the default value
	defaultParams := types.DefaultParams()
	params.MaxSupply = defaultParams.MaxSupply

	// Set the updated params
	err = m.keeper.Params.Set(ctx, params)
	if err != nil {
		return err
	}

	return nil
}
