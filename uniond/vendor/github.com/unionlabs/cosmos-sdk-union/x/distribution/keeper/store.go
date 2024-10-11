package keeper

import (
	"context"
	"errors"
	"math"

	"cosmossdk.io/collections"
	"cosmossdk.io/x/distribution/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// get the delegator withdraw address, defaulting to the delegator address
func (k Keeper) GetDelegatorWithdrawAddr(ctx context.Context, delAddr sdk.AccAddress) (sdk.AccAddress, error) {
	addr, err := k.DelegatorsWithdrawAddress.Get(ctx, delAddr)
	if err != nil && errors.Is(err, collections.ErrNotFound) {
		return delAddr, nil
	}
	return addr, err
}

// iterate over slash events between heights, inclusive
func (k Keeper) IterateValidatorSlashEventsBetween(ctx context.Context, val sdk.ValAddress, startingHeight, endingHeight uint64,
	handler func(height uint64, event types.ValidatorSlashEvent) (stop bool),
) error {
	rng := new(collections.Range[collections.Triple[sdk.ValAddress, uint64, uint64]]).
		StartInclusive(collections.Join3(val, startingHeight, uint64(0))).
		EndExclusive(collections.Join3(val, endingHeight+1, uint64(math.MaxUint64)))

	err := k.ValidatorSlashEvents.Walk(ctx, rng, func(k collections.Triple[sdk.ValAddress, uint64, uint64], ev types.ValidatorSlashEvent) (stop bool, err error) {
		height := k.K2()
		if handler(height, ev) {
			return true, nil
		}
		return false, nil
	})
	if err != nil {
		return err
	}

	return nil
}
