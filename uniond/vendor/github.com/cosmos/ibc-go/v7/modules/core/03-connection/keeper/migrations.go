package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	connectionv7 "github.com/cosmos/ibc-go/v7/modules/core/03-connection/migrations/v7"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper Keeper
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper Keeper) Migrator {
	return Migrator{keeper: keeper}
}

// Migrate3to4 migrates from version 3 to 4.
// This migration writes the sentinel localhost connection end to state.
func (m Migrator) Migrate3to4(ctx sdk.Context) error {
	connectionv7.MigrateLocalhostConnection(ctx, m.keeper)
	return nil
}
