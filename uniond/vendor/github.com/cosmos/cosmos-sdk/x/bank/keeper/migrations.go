package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/bank/exported"
	v2 "github.com/cosmos/cosmos-sdk/x/bank/migrations/v2"
	v3 "github.com/cosmos/cosmos-sdk/x/bank/migrations/v3"
	v4 "github.com/cosmos/cosmos-sdk/x/bank/migrations/v4"
	"github.com/cosmos/cosmos-sdk/x/bank/types"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper         BaseKeeper
	legacySubspace exported.Subspace
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper BaseKeeper, legacySubspace exported.Subspace) Migrator {
	return Migrator{keeper: keeper, legacySubspace: legacySubspace}
}

// Migrate1to2 migrates from version 1 to 2.
func (m Migrator) Migrate1to2(ctx sdk.Context) error {
	return v2.MigrateStore(ctx, m.keeper.storeService, m.keeper.cdc)
}

// Migrate2to3 migrates x/bank storage from version 2 to 3.
func (m Migrator) Migrate2to3(ctx sdk.Context) error {
	return v3.MigrateStore(ctx, m.keeper.storeService, m.keeper.cdc)
}

// Migrate3to4 migrates x/bank storage from version 3 to 4.
func (m Migrator) Migrate3to4(ctx sdk.Context) error {
	m.MigrateSendEnabledParams(ctx)
	return v4.MigrateStore(ctx, m.keeper.storeService, m.legacySubspace, m.keeper.cdc)
}

// MigrateSendEnabledParams get params from x/params and update the bank params.
func (m Migrator) MigrateSendEnabledParams(ctx sdk.Context) {
	sendEnabled := types.GetSendEnabledParams(ctx, m.legacySubspace)
	m.keeper.SetAllSendEnabled(ctx, sendEnabled)
}
