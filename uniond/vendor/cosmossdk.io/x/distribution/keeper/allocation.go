package keeper

import (
	"context"
	"errors"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/comet"
	"cosmossdk.io/core/event"
	"cosmossdk.io/math"
	"cosmossdk.io/x/distribution/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// AllocateTokens performs reward and fee distribution to all validators based
// on the F1 fee distribution specification.
func (k Keeper) AllocateTokens(ctx context.Context, totalPreviousPower int64, bondedVotes []comet.VoteInfo) error {
	// fetch and clear the collected fees for distribution, since this is
	// called in BeginBlock, collected fees will be from the previous block
	// (and distributed to the previous proposer)
	feeCollector := k.authKeeper.GetModuleAccount(ctx, k.feeCollectorName)
	feesCollectedInt := k.bankKeeper.GetAllBalances(ctx, feeCollector.GetAddress())
	// return early if no fees to distribute
	if feesCollectedInt.Empty() {
		return nil
	}
	feesCollected := sdk.NewDecCoinsFromCoins(feesCollectedInt...)

	// transfer collected fees to the distribution module account
	if err := k.bankKeeper.SendCoinsFromModuleToModule(ctx, k.feeCollectorName, types.ModuleName, feesCollectedInt); err != nil {
		return err
	}

	feePool, err := k.FeePool.Get(ctx)
	if err != nil {
		return err
	}

	if totalPreviousPower == 0 {
		if err := k.FeePool.Set(ctx, types.FeePool{DecimalPool: feePool.DecimalPool.Add(feesCollected...)}); err != nil {
			return err
		}
	}

	// calculate fraction allocated to validators
	remaining := feesCollected
	communityTax, err := k.GetCommunityTax(ctx)
	if err != nil {
		return err
	}

	voteMultiplier := math.LegacyOneDec().Sub(communityTax)
	feeMultiplier := feesCollected.MulDecTruncate(voteMultiplier)

	// allocate tokens proportionally to voting power
	//
	// TODO: Consider parallelizing later
	//
	// Ref: https://github.com/cosmos/cosmos-sdk/pull/3099#discussion_r246276376
	for _, vote := range bondedVotes {

		validator, err := k.stakingKeeper.ValidatorByConsAddr(ctx, vote.Validator.Address)
		if err != nil {
			return err
		}

		// TODO: Consider micro-slashing for missing votes.
		//
		// Ref: https://github.com/cosmos/cosmos-sdk/issues/2525#issuecomment-430838701
		powerFraction := math.LegacyNewDec(vote.Validator.Power).QuoTruncate(math.LegacyNewDec(totalPreviousPower))
		reward := feeMultiplier.MulDecTruncate(powerFraction)

		if err = k.AllocateTokensToValidator(ctx, validator, reward); err != nil {
			return err
		}

		remaining = remaining.Sub(reward)
	}
	// send to community pool and set remainder in fee pool
	amt, re := remaining.TruncateDecimal()
	if err := k.bankKeeper.SendCoinsFromModuleToModule(ctx, types.ModuleName, types.ProtocolPoolDistrAccount, amt); err != nil {
		return err
	}

	if err := k.FeePool.Set(ctx, types.FeePool{DecimalPool: feePool.DecimalPool.Add(re...)}); err != nil {
		return err
	}

	return nil
}

// AllocateTokensToValidator allocate tokens to a particular validator,
// splitting according to commission.
func (k Keeper) AllocateTokensToValidator(ctx context.Context, val sdk.ValidatorI, tokens sdk.DecCoins) error {
	// split tokens between validator and delegators according to commission
	commission := tokens.MulDec(val.GetCommission())
	shared := tokens.Sub(commission)

	valBz, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(val.GetOperator())
	if err != nil {
		return err
	}

	// update current commission
	if err = k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeCommission,
		event.NewAttribute(sdk.AttributeKeyAmount, commission.String()),
		event.NewAttribute(types.AttributeKeyValidator, val.GetOperator()),
	); err != nil {
		return err
	}
	currentCommission, err := k.ValidatorsAccumulatedCommission.Get(ctx, valBz)
	if err != nil && !errors.Is(err, collections.ErrNotFound) {
		return err
	}

	currentCommission.Commission = currentCommission.Commission.Add(commission...)
	err = k.ValidatorsAccumulatedCommission.Set(ctx, valBz, currentCommission)
	if err != nil {
		return err
	}

	// update current rewards
	currentRewards, err := k.ValidatorCurrentRewards.Get(ctx, valBz)
	// if the rewards do not exist it's fine, we will just add to zero.
	if err != nil && !errors.Is(err, collections.ErrNotFound) {
		return err
	}

	currentRewards.Rewards = currentRewards.Rewards.Add(shared...)
	err = k.ValidatorCurrentRewards.Set(ctx, valBz, currentRewards)
	if err != nil {
		return err
	}

	// update outstanding rewards
	if err = k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeRewards,
		event.NewAttribute(sdk.AttributeKeyAmount, tokens.String()),
		event.NewAttribute(types.AttributeKeyValidator, val.GetOperator()),
	); err != nil {
		return err
	}

	outstanding, err := k.ValidatorOutstandingRewards.Get(ctx, valBz)
	if err != nil && !errors.Is(err, collections.ErrNotFound) {
		return err
	}

	outstanding.Rewards = outstanding.Rewards.Add(tokens...)
	return k.ValidatorOutstandingRewards.Set(ctx, valBz, outstanding)
}

// sendDecimalPoolToCommunityPool sends the decimal pool to the community pool
// Any remainder stays in the decimal pool
func (k Keeper) sendDecimalPoolToCommunityPool(ctx context.Context) error {
	feePool, err := k.FeePool.Get(ctx)
	if err != nil {
		return err
	}

	if feePool.DecimalPool.IsZero() {
		return nil
	}

	amt, re := feePool.DecimalPool.TruncateDecimal()
	if err := k.bankKeeper.SendCoinsFromModuleToModule(ctx, types.ModuleName, types.ProtocolPoolDistrAccount, amt); err != nil {
		return err
	}

	return k.FeePool.Set(ctx, types.FeePool{DecimalPool: re})
}
