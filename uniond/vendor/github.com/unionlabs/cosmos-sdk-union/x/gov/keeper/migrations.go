package keeper

import (
	"context"

	v5 "cosmossdk.io/x/gov/migrations/v5"
	v6 "cosmossdk.io/x/gov/migrations/v6"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper *Keeper
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper *Keeper) Migrator {
	return Migrator{
		keeper: keeper,
	}
}

// Migrate1to2 migrates from version 1 to 2.
func (m Migrator) Migrate1to2(ctx context.Context) error {
	return nil
}

// Migrate2to3 migrates from version 2 to 3.
func (m Migrator) Migrate2to3(ctx context.Context) error {
	return nil
}

// Migrate3to4 migrates from version 3 to 4.
func (m Migrator) Migrate3to4(ctx context.Context) error {
	return nil
}

// Migrate4to5 migrates from version 4 to 5.
func (m Migrator) Migrate4to5(ctx context.Context) error {
	return v5.MigrateStore(ctx, m.keeper.KVStoreService, m.keeper.cdc, m.keeper.Constitution)
}

// Migrate5to6 migrates from version 5 to 6.
func (m Migrator) Migrate5to6(ctx context.Context) error {
	return v6.MigrateStore(ctx, m.keeper.KVStoreService, m.keeper.Params, m.keeper.Proposals)
}
