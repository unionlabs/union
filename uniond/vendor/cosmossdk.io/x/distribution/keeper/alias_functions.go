package keeper

import (
	"context"
	"errors"

	"cosmossdk.io/collections"
	"cosmossdk.io/x/distribution/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// get outstanding rewards
func (k Keeper) GetValidatorOutstandingRewardsCoins(ctx context.Context, val sdk.ValAddress) (sdk.DecCoins, error) {
	rewards, err := k.ValidatorOutstandingRewards.Get(ctx, val)
	if err != nil && !errors.Is(err, collections.ErrNotFound) {
		return nil, err
	}

	return rewards.Rewards, nil
}

// GetDistributionAccount returns the distribution ModuleAccount
func (k Keeper) GetDistributionAccount(ctx context.Context) sdk.ModuleAccountI {
	return k.authKeeper.GetModuleAccount(ctx, types.ModuleName)
}
