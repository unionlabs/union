package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	v7 "github.com/cosmos/ibc-go/v7/modules/core/02-client/migrations/v7"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper Keeper
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper Keeper) Migrator {
	return Migrator{keeper: keeper}
}

// Migrate2to3 migrates from consensus version 2 to 3.
// This migration
// - migrates solo machine client states from v2 to v3 protobuf definition
// - prunes solo machine consensus states
// - removes the localhost client
// - asserts that existing tendermint clients are properly registered on the chain codec
func (m Migrator) Migrate2to3(ctx sdk.Context) error {
	return v7.MigrateStore(ctx, m.keeper.storeKey, m.keeper.cdc, m.keeper)
}

// Migrate3to4 migrates from consensus version 3 to 4.
// This migration enables the localhost client.
func (m Migrator) Migrate3to4(ctx sdk.Context) error {
	return v7.MigrateLocalhostClient(ctx, m.keeper)
}
