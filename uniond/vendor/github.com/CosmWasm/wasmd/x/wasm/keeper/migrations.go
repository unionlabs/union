package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	v1 "github.com/CosmWasm/wasmd/x/wasm/migrations/v1"
	v2 "github.com/CosmWasm/wasmd/x/wasm/migrations/v2"

	"github.com/CosmWasm/wasmd/x/wasm/exported"
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
	return v1.NewMigrator(m.keeper, m.keeper.addToContractCreatorSecondaryIndex).Migrate1to2(ctx)
}

// Migrate2to3 migrates the x/wasm module state from the consensus
// version 2 to version 3.
func (m Migrator) Migrate2to3(ctx sdk.Context) error {
	return v2.MigrateStore(ctx, m.keeper.storeKey, m.legacySubspace, m.keeper.cdc)
}
