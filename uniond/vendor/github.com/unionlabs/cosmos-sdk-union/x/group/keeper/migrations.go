package keeper

import (
	"context"

	v2 "cosmossdk.io/x/group/migrations/v2"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper Keeper
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper Keeper) Migrator {
	return Migrator{keeper: keeper}
}

// Migrate1to2 migrates from version 1 to 2.
func (m Migrator) Migrate1to2(ctx context.Context) error {
	return v2.Migrate(
		ctx,
		m.keeper.KVStoreService,
		m.keeper.accKeeper,
		m.keeper.groupPolicySeq,
		m.keeper.groupPolicyTable,
	)
}
