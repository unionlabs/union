package keeper

import (
	"context"
	"errors"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/event"
	"cosmossdk.io/math"
	"cosmossdk.io/x/distribution/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// initialize starting info for a new delegation
func (k Keeper) initializeDelegation(ctx context.Context, val sdk.ValAddress, del sdk.AccAddress) error {
	// period has already been incremented - we want to store the period ended by this delegation action
	valCurrentRewards, err := k.ValidatorCurrentRewards.Get(ctx, val)
	if err != nil {
		return err
	}
	previousPeriod := valCurrentRewards.Period - 1

	// increment reference count for the period we're going to track
	err = k.incrementReferenceCount(ctx, val, previousPeriod)
	if err != nil {
		return err
	}

	validator, err := k.stakingKeeper.Validator(ctx, val)
	if err != nil {
		return err
	}

	delegation, err := k.stakingKeeper.Delegation(ctx, del, val)
	if err != nil {
		return err
	}

	// calculate delegation stake in tokens
	// we don't store directly, so multiply delegation shares * (tokens per share)
	// note: necessary to truncate so we don't allow withdrawing more rewards than owed
	stake := validator.TokensFromSharesTruncated(delegation.GetShares())
	headerinfo := k.HeaderService.HeaderInfo(ctx)
	return k.DelegatorStartingInfo.Set(ctx, collections.Join(val, del), types.NewDelegatorStartingInfo(previousPeriod, stake, uint64(headerinfo.Height)))
}

// calculateDelegationRewardsBetween calculates the rewards accrued by a delegation between two periods
func (k Keeper) calculateDelegationRewardsBetween(ctx context.Context, val sdk.ValidatorI,
	startingPeriod, endingPeriod uint64, stake math.LegacyDec,
) (sdk.DecCoins, error) {
	// sanity check
	if startingPeriod > endingPeriod {
		return sdk.DecCoins{}, errors.New("startingPeriod cannot be greater than endingPeriod")
	}

	// sanity check
	if stake.IsNegative() {
		return sdk.DecCoins{}, errors.New("stake should not be negative")
	}

	valBz, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(val.GetOperator())
	if err != nil {
		return sdk.DecCoins{}, err
	}

	// return staking * (ending - starting)
	starting, err := k.ValidatorHistoricalRewards.Get(ctx, collections.Join(sdk.ValAddress(valBz), startingPeriod))
	if err != nil {
		return sdk.DecCoins{}, err
	}

	ending, err := k.ValidatorHistoricalRewards.Get(ctx, collections.Join(sdk.ValAddress(valBz), endingPeriod))
	if err != nil {
		return sdk.DecCoins{}, err
	}

	difference := ending.CumulativeRewardRatio.Sub(starting.CumulativeRewardRatio)
	if difference.IsAnyNegative() {
		return sdk.DecCoins{}, errors.New("negative rewards should not be possible")
	}
	// note: necessary to truncate so we don't allow withdrawing more rewards than owed
	rewards := difference.MulDecTruncate(stake)
	return rewards, nil
}

// CalculateDelegationRewards calculate the total rewards accrued by a delegation
func (k Keeper) CalculateDelegationRewards(ctx context.Context, val sdk.ValidatorI, del sdk.DelegationI, endingPeriod uint64) (rewards sdk.DecCoins, err error) {
	addrCodec := k.authKeeper.AddressCodec()
	delAddr, err := addrCodec.StringToBytes(del.GetDelegatorAddr())
	if err != nil {
		return sdk.DecCoins{}, err
	}

	valAddr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(del.GetValidatorAddr())
	if err != nil {
		return sdk.DecCoins{}, err
	}

	// fetch starting info for delegation
	startingInfo, err := k.DelegatorStartingInfo.Get(ctx, collections.Join(sdk.ValAddress(valAddr), sdk.AccAddress(delAddr)))
	if err != nil && !errors.Is(err, collections.ErrNotFound) {
		return sdk.DecCoins{}, err
	}

	headerinfo := k.HeaderService.HeaderInfo(ctx)
	if startingInfo.Height == uint64(headerinfo.Height) { // started this height, no rewards yet
		return sdk.DecCoins{}, nil
	}

	startingPeriod := startingInfo.PreviousPeriod
	stake := startingInfo.Stake

	// Iterate through slashes and withdraw with calculated staking for
	// distribution periods. These period offsets are dependent on *when* slashes
	// happen - namely, in BeginBlock, after rewards are allocated...
	// Slashes which happened in the first block would have been before this
	// delegation existed, UNLESS they were slashes of a redelegation to this
	// validator which was itself slashed (from a fault committed by the
	// redelegation source validator) earlier in the same BeginBlock.
	startingHeight := startingInfo.Height
	// Slashes this block happened after reward allocation, but we have to account
	// for them for the stake sanity check below.
	endingHeight := uint64(headerinfo.Height)
	var iterErr error
	if endingHeight > startingHeight {
		err = k.IterateValidatorSlashEventsBetween(ctx, valAddr, startingHeight, endingHeight,
			func(height uint64, event types.ValidatorSlashEvent) (stop bool) {
				endingPeriod := event.ValidatorPeriod
				if endingPeriod > startingPeriod {
					delRewards, err := k.calculateDelegationRewardsBetween(ctx, val, startingPeriod, endingPeriod, stake)
					if err != nil {
						iterErr = err
						return true
					}
					rewards = rewards.Add(delRewards...)

					// Note: It is necessary to truncate so we don't allow withdrawing
					// more rewards than owed.
					stake = stake.MulTruncate(math.LegacyOneDec().Sub(event.Fraction))
					startingPeriod = endingPeriod
				}
				return false
			},
		)
		if iterErr != nil {
			return sdk.DecCoins{}, iterErr
		}
		if err != nil {
			return sdk.DecCoins{}, err
		}
	}

	// A total stake sanity check; Recalculated final stake should be less than or
	// equal to current stake here. We cannot use Equals because stake is truncated
	// when multiplied by slash fractions (see above). We could only use equals if
	// we had arbitrary-precision rationals.
	currentStake := val.TokensFromShares(del.GetShares())

	if stake.GT(currentStake) {
		// AccountI for rounding inconsistencies between:
		//
		//     currentStake: calculated as in staking with a single computation
		//     stake:        calculated as an accumulation of stake
		//                   calculations across validator's distribution periods
		//
		// These inconsistencies are due to differing order of operations which
		// will inevitably have different accumulated rounding and may lead to
		// the smallest decimal place being one greater in stake than
		// currentStake. When we calculated slashing by period, even if we
		// round down for each slash fraction, it's possible due to how much is
		// being rounded that we slash less when slashing by period instead of
		// for when we slash without periods. In other words, the single slash,
		// and the slashing by period could both be rounding down but the
		// slashing by period is simply rounding down less, thus making stake >
		// currentStake
		//
		// A small amount of this error is tolerated and corrected for,
		// however any greater amount should be considered a breach in expected
		// behavior.
		marginOfErr := math.LegacySmallestDec().MulInt64(3)
		if stake.LTE(currentStake.Add(marginOfErr)) {
			stake = currentStake
		} else {
			return sdk.DecCoins{}, fmt.Errorf("calculated final stake for delegator %s greater than current stake"+
				"\n\tfinal stake:\t%s"+
				"\n\tcurrent stake:\t%s",
				del.GetDelegatorAddr(), stake, currentStake)
		}
	}

	// calculate rewards for final period
	delRewards, err := k.calculateDelegationRewardsBetween(ctx, val, startingPeriod, endingPeriod, stake)
	if err != nil {
		return sdk.DecCoins{}, err
	}

	rewards = rewards.Add(delRewards...)
	return rewards, nil
}

// withdrawDelegationRewards withdraws the rewards accrued by a delegation.
func (k Keeper) withdrawDelegationRewards(ctx context.Context, val sdk.ValidatorI, del sdk.DelegationI) (sdk.Coins, error) {
	addrCodec := k.authKeeper.AddressCodec()
	delAddr, err := addrCodec.StringToBytes(del.GetDelegatorAddr())
	if err != nil {
		return nil, err
	}

	valAddr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(del.GetValidatorAddr())
	if err != nil {
		return nil, err
	}

	// check existence of delegator starting info
	hasInfo, err := k.DelegatorStartingInfo.Has(ctx, collections.Join(sdk.ValAddress(valAddr), sdk.AccAddress(delAddr)))
	if err != nil {
		return nil, err
	}

	if !hasInfo {
		return nil, types.ErrEmptyDelegationDistInfo
	}

	// end current period and calculate rewards
	endingPeriod, err := k.IncrementValidatorPeriod(ctx, val)
	if err != nil {
		return nil, err
	}

	rewardsRaw, err := k.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	if err != nil {
		return nil, err
	}

	outstanding, err := k.GetValidatorOutstandingRewardsCoins(ctx, sdk.ValAddress(valAddr))
	if err != nil {
		return nil, err
	}

	// defensive edge case may happen on the very final digits
	// of the decCoins due to operation order of the distribution mechanism.
	rewards := rewardsRaw.Intersect(outstanding)
	if !rewards.Equal(rewardsRaw) {
		k.Logger.Info(
			"rounding error withdrawing rewards from validator",
			"delegator", del.GetDelegatorAddr(),
			"validator", val.GetOperator(),
			"got", rewards.String(),
			"expected", rewardsRaw.String(),
		)
	}

	// truncate reward dec coins, return remainder to decimal pool
	finalRewards, remainder := rewards.TruncateDecimal()

	// add coins to user account
	if !finalRewards.IsZero() {
		withdrawAddr, err := k.GetDelegatorWithdrawAddr(ctx, delAddr)
		if err != nil {
			return nil, err
		}

		err = k.bankKeeper.SendCoinsFromModuleToAccount(ctx, types.ModuleName, withdrawAddr, finalRewards)
		if err != nil {
			return nil, err
		}
	}

	// update the outstanding rewards and the decimal pool only if the transaction was successful
	if err := k.ValidatorOutstandingRewards.Set(ctx, sdk.ValAddress(valAddr), types.ValidatorOutstandingRewards{Rewards: outstanding.Sub(rewards)}); err != nil {
		return nil, err
	}

	feePool, err := k.FeePool.Get(ctx)
	if err != nil {
		return nil, err
	}

	feePool.DecimalPool = feePool.DecimalPool.Add(remainder...)
	err = k.FeePool.Set(ctx, feePool)
	if err != nil {
		return nil, err
	}

	// decrement reference count of starting period
	startingInfo, err := k.DelegatorStartingInfo.Get(ctx, collections.Join(sdk.ValAddress(valAddr), sdk.AccAddress(delAddr)))
	if err != nil && !errors.Is(err, collections.ErrNotFound) {
		return nil, err
	}

	startingPeriod := startingInfo.PreviousPeriod
	err = k.decrementReferenceCount(ctx, sdk.ValAddress(valAddr), startingPeriod)
	if err != nil {
		return nil, err
	}

	// remove delegator starting info
	err = k.DelegatorStartingInfo.Remove(ctx, collections.Join(sdk.ValAddress(valAddr), sdk.AccAddress(delAddr)))
	if err != nil {
		return nil, err
	}

	if finalRewards.IsZero() {
		baseDenom, err := k.stakingKeeper.BondDenom(ctx)
		if err != nil {
			return nil, err
		}

		// Note, we do not call the NewCoins constructor as we do not want the zero
		// coin removed.
		finalRewards = sdk.Coins{sdk.NewCoin(baseDenom, math.ZeroInt())}
	}

	err = k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeWithdrawRewards,
		event.NewAttribute(sdk.AttributeKeyAmount, finalRewards.String()),
		event.NewAttribute(types.AttributeKeyValidator, val.GetOperator()),
		event.NewAttribute(types.AttributeKeyDelegator, del.GetDelegatorAddr()),
	)
	if err != nil {
		return nil, err
	}

	return finalRewards, nil
}
