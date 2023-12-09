package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/distribution/exported"
	v2 "github.com/cosmos/cosmos-sdk/x/distribution/migrations/v2"
	v3 "github.com/cosmos/cosmos-sdk/x/distribution/migrations/v3"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper         Keeper
	legacySubspace exported.Subspace
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper Keeper, legacySubspace exported.Subspace) Migrator {
	return Migrator{keeper: keeper, legacySubspace: legacySubspace}
}

// Migrate1to2 migrates from version 1 to 2.
func (m Migrator) Migrate1to2(ctx sdk.Context) error {
	return v2.MigrateStore(ctx, m.keeper.storeService)
}

// Migrate2to3 migrates the x/distribution module state from the consensus
// version 2 to version 3. Specifically, it takes the parameters that are currently stored
// and managed by the x/params module and stores them directly into the x/distribution
// module state.
func (m Migrator) Migrate2to3(ctx sdk.Context) error {
	return v3.MigrateStore(ctx, m.keeper.storeService, m.legacySubspace, m.keeper.cdc)
}
