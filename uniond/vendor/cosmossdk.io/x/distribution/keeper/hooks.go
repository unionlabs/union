package keeper

import (
	"context"
	"errors"

	"cosmossdk.io/collections"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/distribution/types"
	stakingtypes "cosmossdk.io/x/staking/types"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// Wrapper struct
type Hooks struct {
	k Keeper
}

var _ stakingtypes.StakingHooks = Hooks{}

// Hooks creates new distribution hooks
func (k Keeper) Hooks() Hooks {
	return Hooks{k}
}

// AfterValidatorCreated initialize validator distribution record
func (h Hooks) AfterValidatorCreated(ctx context.Context, valAddr sdk.ValAddress) error {
	val, err := h.k.stakingKeeper.Validator(ctx, valAddr)
	if err != nil {
		return err
	}
	return h.k.initializeValidator(ctx, val)
}

// AfterValidatorRemoved performs clean up after a validator is removed
func (h Hooks) AfterValidatorRemoved(ctx context.Context, _ sdk.ConsAddress, valAddr sdk.ValAddress) error {
	// fetch outstanding
	outstanding, err := h.k.GetValidatorOutstandingRewardsCoins(ctx, valAddr)
	if err != nil {
		return err
	}

	// force-withdraw commission
	valCommission, err := h.k.ValidatorsAccumulatedCommission.Get(ctx, valAddr)
	if err != nil && !errors.Is(err, collections.ErrNotFound) {
		return err
	}

	commission := valCommission.Commission

	if !commission.IsZero() {
		// subtract from outstanding
		outstanding = outstanding.Sub(commission)

		// split into integral & remainder
		coins, remainder := commission.TruncateDecimal()

		// remainder to decimal pool
		feePool, err := h.k.FeePool.Get(ctx)
		if err != nil {
			return err
		}

		feePool.DecimalPool = feePool.DecimalPool.Add(remainder...)
		err = h.k.FeePool.Set(ctx, feePool)
		if err != nil {
			return err
		}

		// add to validator account
		if !coins.IsZero() {
			accAddr := sdk.AccAddress(valAddr)
			withdrawAddr, err := h.k.GetDelegatorWithdrawAddr(ctx, accAddr)
			if err != nil {
				return err
			}

			if err := h.k.bankKeeper.SendCoinsFromModuleToAccount(ctx, types.ModuleName, withdrawAddr, coins); err != nil {
				return err
			}
		}
	}

	// Add outstanding to decimal pool
	// The validator is removed only after it has no more delegations.
	// This operation sends only the remaining dust to the decimal pool.
	feePool, err := h.k.FeePool.Get(ctx)
	if err != nil {
		return err
	}

	feePool.DecimalPool = feePool.DecimalPool.Add(outstanding...)
	err = h.k.FeePool.Set(ctx, feePool)
	if err != nil {
		return err
	}

	// delete outstanding
	err = h.k.ValidatorOutstandingRewards.Remove(ctx, valAddr)
	if err != nil {
		return err
	}

	// remove commission record
	err = h.k.ValidatorsAccumulatedCommission.Remove(ctx, valAddr)
	if err != nil {
		return err
	}

	// clear slashes
	err = h.k.ValidatorSlashEvents.Clear(ctx, collections.NewPrefixedTripleRange[sdk.ValAddress, uint64, uint64](valAddr))
	if err != nil {
		return err
	}

	// clear historical rewards
	err = h.k.ValidatorHistoricalRewards.Clear(ctx, collections.NewPrefixedPairRange[sdk.ValAddress, uint64](valAddr))
	if err != nil {
		return err
	}
	// clear current rewards
	err = h.k.ValidatorCurrentRewards.Remove(ctx, valAddr)
	if err != nil {
		return err
	}

	return nil
}

// BeforeDelegationCreated increment period
func (h Hooks) BeforeDelegationCreated(ctx context.Context, _ sdk.AccAddress, valAddr sdk.ValAddress) error {
	val, err := h.k.stakingKeeper.Validator(ctx, valAddr)
	if err != nil {
		return err
	}

	_, err = h.k.IncrementValidatorPeriod(ctx, val)
	return err
}

// BeforeDelegationSharesModified withdraws delegation rewards (which also increments period)
func (h Hooks) BeforeDelegationSharesModified(ctx context.Context, delAddr sdk.AccAddress, valAddr sdk.ValAddress) error {
	val, err := h.k.stakingKeeper.Validator(ctx, valAddr)
	if err != nil {
		return err
	}

	del, err := h.k.stakingKeeper.Delegation(ctx, delAddr, valAddr)
	if err != nil {
		return err
	}

	if _, err := h.k.withdrawDelegationRewards(ctx, val, del); err != nil {
		return err
	}

	return nil
}

// AfterDelegationModified create new delegation period record
func (h Hooks) AfterDelegationModified(ctx context.Context, delAddr sdk.AccAddress, valAddr sdk.ValAddress) error {
	return h.k.initializeDelegation(ctx, valAddr, delAddr)
}

// BeforeValidatorSlashed record the slash event
func (h Hooks) BeforeValidatorSlashed(ctx context.Context, valAddr sdk.ValAddress, fraction sdkmath.LegacyDec) error {
	return h.k.updateValidatorSlashFraction(ctx, valAddr, fraction)
}

func (h Hooks) BeforeValidatorModified(_ context.Context, _ sdk.ValAddress) error {
	return nil
}

func (h Hooks) AfterValidatorBonded(_ context.Context, _ sdk.ConsAddress, _ sdk.ValAddress) error {
	return nil
}

func (h Hooks) AfterValidatorBeginUnbonding(_ context.Context, _ sdk.ConsAddress, _ sdk.ValAddress) error {
	return nil
}

func (h Hooks) BeforeDelegationRemoved(_ context.Context, _ sdk.AccAddress, _ sdk.ValAddress) error {
	return nil
}

func (h Hooks) AfterUnbondingInitiated(_ context.Context, _ uint64) error {
	return nil
}

func (h Hooks) AfterConsensusPubKeyUpdate(_ context.Context, _, _ cryptotypes.PubKey, _ sdk.Coin) error {
	return nil
}
