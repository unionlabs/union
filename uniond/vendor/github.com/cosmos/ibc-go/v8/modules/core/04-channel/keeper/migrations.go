package keeper

import (
	"context"
	sdk "github.com/cosmos/cosmos-sdk/types"

	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper *Keeper
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper *Keeper) Migrator {
	return Migrator{keeper: keeper}
}

// MigrateParams migrates params to the default channel params.
func (m Migrator) MigrateParams(bareCtx context.Context) error {
	ctx := sdk.UnwrapSDKContext(bareCtx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	params := channeltypes.DefaultParams()
	m.keeper.SetParams(ctx, params)
	m.keeper.Logger(ctx).Info("successfully migrated ibc channel params")
	return nil
}
