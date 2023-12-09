package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	clientkeeper "github.com/cosmos/ibc-go/v8/modules/core/02-client/keeper"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper Keeper
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper Keeper) Migrator {
	return Migrator{keeper: keeper}
}

// Migrate2to3 migrates from version 2 to 3. See 02-client keeper function Migrate2to3.
func (m Migrator) Migrate2to3(ctx sdk.Context) error {
	clientMigrator := clientkeeper.NewMigrator(m.keeper.ClientKeeper)
	return clientMigrator.Migrate2to3(ctx)
}
