package keeper

import (
	"context"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
)

// Migrator is a struct for handling in-place state migrations.
type Migrator struct {
	keeper *Keeper
}

// NewMigrator returns Migrator instance for the state migration.
func NewMigrator(k *Keeper) Migrator {
	return Migrator{
		keeper: k,
	}
}

// MigrateParams migrates the host submodule's parameters from the x/params to self store.
func (m Migrator) MigrateParams(bareCtx context.Context) error {
	ctx := sdk.UnwrapSDKContext(bareCtx) // TODO: https://github.com/cosmos/ibc-go/issues/7223
	if m.keeper != nil {
		params := types.DefaultParams()
		if m.keeper.legacySubspace != nil {
			m.keeper.legacySubspace.GetParamSetIfExists(ctx, &params)
		}
		if err := params.Validate(); err != nil {
			return err
		}
		m.keeper.SetParams(ctx, params)
		m.keeper.Logger(ctx).Info("successfully migrated ica/host submodule to self-manage params")
	}
	return nil
}
