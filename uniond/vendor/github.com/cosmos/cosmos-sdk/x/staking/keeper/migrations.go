package keeper

import (
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/exported"
	v2 "github.com/cosmos/cosmos-sdk/x/staking/migrations/v2"
	v3 "github.com/cosmos/cosmos-sdk/x/staking/migrations/v3"
	v4 "github.com/cosmos/cosmos-sdk/x/staking/migrations/v4"
	v5 "github.com/cosmos/cosmos-sdk/x/staking/migrations/v5"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper         *Keeper
	legacySubspace exported.Subspace
}

// NewMigrator returns a new Migrator instance.
func NewMigrator(keeper *Keeper, legacySubspace exported.Subspace) Migrator {
	return Migrator{
		keeper:         keeper,
		legacySubspace: legacySubspace,
	}
}

// Migrate1to2 migrates from version 1 to 2.
func (m Migrator) Migrate1to2(ctx sdk.Context) error {
	store := runtime.KVStoreAdapter(m.keeper.storeService.OpenKVStore(ctx))
	return v2.MigrateStore(ctx, store)
}

// Migrate2to3 migrates x/staking state from consensus version 2 to 3.
func (m Migrator) Migrate2to3(ctx sdk.Context) error {
	store := runtime.KVStoreAdapter(m.keeper.storeService.OpenKVStore(ctx))
	return v3.MigrateStore(ctx, store, m.keeper.cdc, m.legacySubspace)
}

// Migrate3to4 migrates x/staking state from consensus version 3 to 4.
func (m Migrator) Migrate3to4(ctx sdk.Context) error {
	store := runtime.KVStoreAdapter(m.keeper.storeService.OpenKVStore(ctx))
	return v4.MigrateStore(ctx, store, m.keeper.cdc, m.legacySubspace)
}

// Migrate4to5 migrates x/staking state from consensus version 4 to 5.
func (m Migrator) Migrate4to5(ctx sdk.Context) error {
	store := runtime.KVStoreAdapter(m.keeper.storeService.OpenKVStore(ctx))
	return v5.MigrateStore(ctx, store, m.keeper.cdc)
}
