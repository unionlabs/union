package keeper

import (
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/x/distribution/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// register all distribution invariants
func RegisterInvariants(ir sdk.InvariantRegistry, k Keeper) {
	ir.RegisterRoute(types.ModuleName, "nonnegative-outstanding",
		NonNegativeOutstandingInvariant(k))
	ir.RegisterRoute(types.ModuleName, "can-withdraw",
		CanWithdrawInvariant(k))
	ir.RegisterRoute(types.ModuleName, "reference-count",
		ReferenceCountInvariant(k))
	ir.RegisterRoute(types.ModuleName, "module-account",
		ModuleAccountInvariant(k))
}

// AllInvariants runs all invariants of the distribution module
func AllInvariants(k Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		res, stop := CanWithdrawInvariant(k)(ctx)
		if stop {
			return res, stop
		}
		res, stop = NonNegativeOutstandingInvariant(k)(ctx)
		if stop {
			return res, stop
		}
		res, stop = ReferenceCountInvariant(k)(ctx)
		if stop {
			return res, stop
		}
		return ModuleAccountInvariant(k)(ctx)
	}
}

// NonNegativeOutstandingInvariant checks that outstanding unwithdrawn fees are never negative
func NonNegativeOutstandingInvariant(k Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		var msg string
		var count int
		var outstanding sdk.DecCoins

		err := k.ValidatorOutstandingRewards.Walk(ctx, nil, func(addr sdk.ValAddress, rewards types.ValidatorOutstandingRewards) (stop bool, err error) {
			outstanding = rewards.GetRewards()
			if outstanding.IsAnyNegative() {
				count++
				msg += fmt.Sprintf("\t%v has negative outstanding coins: %v\n", addr, outstanding)
			}
			return false, nil
		})
		if err != nil {
			return sdk.FormatInvariant(types.ModuleName, "nonnegative outstanding", err.Error()), true
		}
		broken := count != 0

		return sdk.FormatInvariant(types.ModuleName, "nonnegative outstanding",
			fmt.Sprintf("found %d validators with negative outstanding rewards\n%s", count, msg)), broken
	}
}

// CanWithdrawInvariant checks that current rewards can be completely withdrawn
func CanWithdrawInvariant(k Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		// cache, we don't want to write changes
		ctx, _ = ctx.CacheContext()

		var remaining sdk.DecCoins

		valDelegationAddrs := make(map[string][][]byte)
		allDelegations, err := k.stakingKeeper.GetAllSDKDelegations(ctx)
		if err != nil {
			panic(err)
		}

		for _, del := range allDelegations {
			delAddr, err := k.authKeeper.AddressCodec().StringToBytes(del.GetDelegatorAddr())
			if err != nil {
				panic(err)
			}
			valAddr := del.GetValidatorAddr()
			valDelegationAddrs[valAddr] = append(valDelegationAddrs[valAddr], delAddr)
		}

		// iterate over all validators
		err = k.stakingKeeper.IterateValidators(ctx, func(_ int64, val sdk.ValidatorI) (stop bool) {
			valBz, err1 := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(val.GetOperator())
			if err != nil {
				panic(err1)
			}
			_, _ = k.WithdrawValidatorCommission(ctx, valBz)

			delegationAddrs, ok := valDelegationAddrs[val.GetOperator()]
			if ok {
				for _, delAddr := range delegationAddrs {
					if _, err := k.WithdrawDelegationRewards(ctx, delAddr, valBz); err != nil {
						panic(err)
					}
				}
			}

			var err error
			remaining, err = k.GetValidatorOutstandingRewardsCoins(ctx, valBz)
			if err != nil {
				panic(err)
			}

			if len(remaining) > 0 && remaining[0].Amount.IsNegative() {
				return true
			}

			return false
		})
		if err != nil {
			panic(err)
		}

		broken := len(remaining) > 0 && remaining[0].Amount.IsNegative()
		return sdk.FormatInvariant(types.ModuleName, "can withdraw",
			fmt.Sprintf("remaining coins: %v\n", remaining)), broken
	}
}

// ReferenceCountInvariant checks that the number of historical rewards records is correct
func ReferenceCountInvariant(k Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		valCount := uint64(0)
		err := k.stakingKeeper.IterateValidators(ctx, func(_ int64, val sdk.ValidatorI) (stop bool) {
			valCount++
			return false
		})
		if err != nil {
			panic(err)
		}

		dels, err := k.stakingKeeper.GetAllSDKDelegations(ctx)
		if err != nil {
			panic(err)
		}

		slashCount := uint64(0)
		err = k.ValidatorSlashEvents.Walk(
			ctx,
			nil,
			func(k collections.Triple[sdk.ValAddress, uint64, uint64], event types.ValidatorSlashEvent) (stop bool, err error) {
				slashCount++
				return false, nil
			},
		)
		if err != nil {
			panic(err)
		}

		// one record per validator (last tracked period), one record per
		// delegation (previous period), one record per slash (previous period)
		expected := valCount + uint64(len(dels)) + slashCount
		count := uint64(0)
		err = k.ValidatorHistoricalRewards.Walk(
			ctx, nil, func(key collections.Pair[sdk.ValAddress, uint64], rewards types.ValidatorHistoricalRewards) (stop bool, err error) {
				count += uint64(rewards.ReferenceCount)
				return false, nil
			},
		)
		if err != nil {
			panic(err)
		}

		broken := count != expected

		return sdk.FormatInvariant(types.ModuleName, "reference count",
			fmt.Sprintf("expected historical reference count: %d = %v validators + %v delegations + %v slashes\n"+
				"total validator historical reference count: %d\n",
				expected, valCount, len(dels), slashCount, count)), broken
	}
}

// ModuleAccountInvariant checks that the coins held by the distr ModuleAccount
// is consistent with the sum of validator outstanding rewards
func ModuleAccountInvariant(k Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		var expectedCoins sdk.DecCoins
		err := k.ValidatorOutstandingRewards.Walk(ctx, nil, func(_ sdk.ValAddress, rewards types.ValidatorOutstandingRewards) (stop bool, err error) {
			expectedCoins = expectedCoins.Add(rewards.Rewards...)
			return false, nil
		})
		if err != nil {
			return sdk.FormatInvariant(types.ModuleName, "module account coins", err.Error()), true
		}

		expectedInt, _ := expectedCoins.TruncateDecimal()

		balances := k.bankKeeper.GetAllBalances(ctx, k.GetDistributionAccount(ctx).GetAddress())
		broken := !balances.Equal(expectedInt)
		return sdk.FormatInvariant(
			types.ModuleName, "ModuleAccount coins",
			fmt.Sprintf("\texpected ModuleAccount coins:     %s\n"+
				"\tdistribution ModuleAccount coins: %s\n",
				expectedInt, balances,
			),
		), broken
	}
}
