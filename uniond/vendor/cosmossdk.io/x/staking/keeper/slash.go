package keeper

import (
	"context"
	"errors"
	"fmt"

	st "cosmossdk.io/api/cosmos/staking/v1beta1"
	"cosmossdk.io/collections"
	"cosmossdk.io/math"
	"cosmossdk.io/x/staking/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// Slash a validator for an infraction committed at a known height
// Find the contributing stake at that height and burn the specified slashFactor
// of it, updating unbonding delegations & redelegations appropriately
//
// CONTRACT:
//
//	slashFactor is non-negative
//
// CONTRACT:
//
//	Infraction was committed equal to or less than an unbonding period in the past,
//	so all unbonding delegations and redelegations from that height are stored
//
// CONTRACT:
//
//	Slash will not slash unbonded validators (for the above reason)
//
// CONTRACT:
//
//	Infraction was committed at the current height or at a past height,
//	but not at a height in the future
func (k Keeper) Slash(ctx context.Context, consAddr sdk.ConsAddress, infractionHeight, power int64, slashFactor math.LegacyDec) (math.Int, error) {
	if slashFactor.IsNegative() {
		return math.NewInt(0), fmt.Errorf("attempted to slash with a negative slash factor: %v", slashFactor)
	}

	// Amount of slashing = slash slashFactor * power at time of infraction
	amount := k.TokensFromConsensusPower(ctx, power)
	slashAmountDec := math.LegacyNewDecFromInt(amount).Mul(slashFactor)
	slashAmount := slashAmountDec.TruncateInt()

	// ref https://github.com/cosmos/cosmos-sdk/issues/1348

	validator, err := k.GetValidatorByConsAddr(ctx, consAddr)
	if errors.Is(err, types.ErrNoValidatorFound) {
		// If not found, the validator must have been overslashed and removed - so we don't need to do anything
		// NOTE:  Correctness dependent on invariant that unbonding delegations / redelegations must also have been completely
		//        slashed in this case - which we don't explicitly check, but should be true.
		// Log the slash attempt for future reference (maybe we should tag it too)
		conStr, err := k.consensusAddressCodec.BytesToString(consAddr)
		if err != nil {
			return math.NewInt(0), err
		}

		k.Logger.Error(
			"WARNING: ignored attempt to slash a nonexistent validator; we recommend you investigate immediately",
			"validator", conStr,
		)
		return math.NewInt(0), nil
	} else if err != nil {
		return math.NewInt(0), err
	}

	// should not be slashing an unbonded validator
	if validator.IsUnbonded() {
		return math.NewInt(0), fmt.Errorf("should not be slashing unbonded validator: %s", validator.GetOperator())
	}

	operatorAddress, err := k.ValidatorAddressCodec().StringToBytes(validator.GetOperator())
	if err != nil {
		return math.NewInt(0), err
	}

	// call the before-modification hook
	if err := k.Hooks().BeforeValidatorModified(ctx, operatorAddress); err != nil {
		return math.NewInt(0), fmt.Errorf("failed to call before validator modified hook: %w", err)
	}

	// Track remaining slash amount for the validator
	// This will decrease when we slash unbondings and
	// redelegations, as that stake has since unbonded
	remainingSlashAmount := slashAmount

	headerInfo := k.HeaderService.HeaderInfo(ctx)
	height := headerInfo.Height
	switch {
	case infractionHeight > height:
		// Can't slash infractions in the future
		return math.NewInt(0), fmt.Errorf(
			"impossible attempt to slash future infraction at height %d but we are at height %d",
			infractionHeight, height)

	case infractionHeight == height:
		// Special-case slash at current height for efficiency - we don't need to
		// look through unbonding delegations or redelegations.
		k.Logger.Info(
			"slashing at current height; not scanning unbonding delegations & redelegations",
			"height", infractionHeight,
		)

	case infractionHeight < height:
		// Iterate through unbonding delegations from slashed validator
		unbondingDelegations, err := k.GetUnbondingDelegationsFromValidator(ctx, operatorAddress)
		if err != nil {
			return math.NewInt(0), err
		}

		for _, unbondingDelegation := range unbondingDelegations {
			amountSlashed, err := k.SlashUnbondingDelegation(ctx, unbondingDelegation, infractionHeight, slashFactor)
			if err != nil {
				return math.ZeroInt(), err
			}
			if amountSlashed.IsZero() {
				continue
			}

			remainingSlashAmount = remainingSlashAmount.Sub(amountSlashed)
		}

		// Iterate through redelegations from slashed source validator
		redelegations, err := k.GetRedelegationsFromSrcValidator(ctx, operatorAddress)
		if err != nil {
			return math.NewInt(0), err
		}

		for _, redelegation := range redelegations {
			amountSlashed, err := k.SlashRedelegation(ctx, validator, redelegation, infractionHeight, slashFactor)
			if err != nil {
				return math.NewInt(0), err
			}

			if amountSlashed.IsZero() {
				continue
			}

			remainingSlashAmount = remainingSlashAmount.Sub(amountSlashed)
		}
	}

	// cannot decrease balance below zero
	tokensToBurn := math.MinInt(remainingSlashAmount, validator.Tokens)
	tokensToBurn = math.MaxInt(tokensToBurn, math.ZeroInt()) // defensive.

	if tokensToBurn.IsZero() {
		// Nothing to burn, we can end this route immediately! We also don't
		// need to call the k.Hooks().BeforeValidatorSlashed hook as we won't
		// be slashing at all.
		k.Logger.Info(
			"no validator slashing because slash amount is zero",
			"validator", validator.GetOperator(),
			"slash_factor", slashFactor.String(),
			"burned", tokensToBurn,
			"validatorTokens", validator.Tokens,
		)
		return math.NewInt(0), nil
	}

	// we need to calculate the *effective* slash fraction for distribution
	if validator.Tokens.IsPositive() {
		effectiveFraction := math.LegacyNewDecFromInt(tokensToBurn).QuoRoundUp(math.LegacyNewDecFromInt(validator.Tokens))
		// possible if power has changed
		if oneDec := math.LegacyOneDec(); effectiveFraction.GT(oneDec) {
			effectiveFraction = oneDec
		}
		// call the before-slashed hook
		if err := k.Hooks().BeforeValidatorSlashed(ctx, operatorAddress, effectiveFraction); err != nil {
			return math.NewInt(0), fmt.Errorf("failed to call before validator slashed hook: %w", err)
		}
	}

	// Deduct from validator's bonded tokens and update the validator.
	// Burn the slashed tokens from the pool account and decrease the total supply.
	validator, err = k.RemoveValidatorTokens(ctx, validator, tokensToBurn)
	if err != nil {
		return math.NewInt(0), err
	}

	switch validator.GetStatus() {
	case sdk.Bonded:
		if err := k.burnBondedTokens(ctx, tokensToBurn); err != nil {
			return math.NewInt(0), err
		}
	case sdk.Unbonding, sdk.Unbonded:
		if err := k.burnNotBondedTokens(ctx, tokensToBurn); err != nil {
			return math.NewInt(0), err
		}
	default:
		return math.NewInt(0), errors.New("invalid validator status")
	}

	k.Logger.Info(
		"validator slashed by slash factor",
		"validator", validator.GetOperator(),
		"slash_factor", slashFactor.String(),
		"burned", tokensToBurn,
	)
	return tokensToBurn, nil
}

// SlashWithInfractionReason implementation doesn't require the infraction (types.Infraction) to work but is required by Interchain Security.
func (k Keeper) SlashWithInfractionReason(ctx context.Context, consAddr sdk.ConsAddress, infractionHeight, power int64, slashFactor math.LegacyDec, _ st.Infraction) (math.Int, error) {
	return k.Slash(ctx, consAddr, infractionHeight, power, slashFactor)
}

// jail a validator
func (k Keeper) Jail(ctx context.Context, consAddr sdk.ConsAddress) error {
	validator, err := k.GetValidatorByConsAddr(ctx, consAddr)
	if err != nil {
		return fmt.Errorf("validator with consensus-Address %s not found", consAddr)
	}
	if err := k.jailValidator(ctx, validator); err != nil {
		return err
	}

	k.Logger.Info("validator jailed", "validator", consAddr)
	return nil
}

// unjail a validator
func (k Keeper) Unjail(ctx context.Context, consAddr sdk.ConsAddress) error {
	validator, err := k.GetValidatorByConsAddr(ctx, consAddr)
	if err != nil {
		return fmt.Errorf("validator with consensus-Address %s not found", consAddr)
	}
	if err := k.unjailValidator(ctx, validator); err != nil {
		return err
	}

	k.Logger.Info("validator un-jailed", "validator", consAddr)
	return nil
}

// slash an unbonding delegation and update the pool
// return the amount that would have been slashed assuming
// the unbonding delegation had enough stake to slash
// (the amount actually slashed may be less if there's
// insufficient stake remaining)
func (k Keeper) SlashUnbondingDelegation(ctx context.Context, unbondingDelegation types.UnbondingDelegation,
	infractionHeight int64, slashFactor math.LegacyDec,
) (totalSlashAmount math.Int, err error) {
	now := k.HeaderService.HeaderInfo(ctx).Time
	totalSlashAmount = math.ZeroInt()
	burnedAmount := math.ZeroInt()

	// perform slashing on all entries within the unbonding delegation
	for i, entry := range unbondingDelegation.Entries {
		// If unbonding started before this height, stake didn't contribute to infraction
		if entry.CreationHeight < infractionHeight {
			continue
		}

		if entry.IsMature(now) && !entry.OnHold() {
			// Unbonding delegation no longer eligible for slashing, skip it
			continue
		}

		// Calculate slash amount proportional to stake contributing to infraction
		slashAmountDec := slashFactor.MulInt(entry.InitialBalance)
		slashAmount := slashAmountDec.TruncateInt()
		totalSlashAmount = totalSlashAmount.Add(slashAmount)

		// Don't slash more tokens than held
		// Possible since the unbonding delegation may already
		// have been slashed, and slash amounts are calculated
		// according to stake held at time of infraction
		unbondingSlashAmount := math.MinInt(slashAmount, entry.Balance)

		// Update unbonding delegation if necessary
		if unbondingSlashAmount.IsZero() {
			continue
		}

		burnedAmount = burnedAmount.Add(unbondingSlashAmount)
		entry.Balance = entry.Balance.Sub(unbondingSlashAmount)
		unbondingDelegation.Entries[i] = entry
		if err = k.SetUnbondingDelegation(ctx, unbondingDelegation); err != nil {
			return math.ZeroInt(), err
		}
	}

	if err := k.burnNotBondedTokens(ctx, burnedAmount); err != nil {
		return math.ZeroInt(), err
	}

	return totalSlashAmount, nil
}

// slash a redelegation and update the pool
// return the amount that would have been slashed assuming
// the unbonding delegation had enough stake to slash
// (the amount actually slashed may be less if there's
// insufficient stake remaining)
// NOTE this is only slashing for prior infractions from the source validator
func (k Keeper) SlashRedelegation(ctx context.Context, srcValidator types.Validator, redelegation types.Redelegation,
	infractionHeight int64, slashFactor math.LegacyDec,
) (totalSlashAmount math.Int, err error) {
	now := k.HeaderService.HeaderInfo(ctx).Time
	totalSlashAmount = math.ZeroInt()
	bondedBurnedAmount, notBondedBurnedAmount := math.ZeroInt(), math.ZeroInt()

	valDstAddr, err := k.validatorAddressCodec.StringToBytes(redelegation.ValidatorDstAddress)
	if err != nil {
		return math.ZeroInt(), fmt.Errorf("SlashRedelegation: could not parse validator destination address: %w", err)
	}

	delegatorAddress, err := k.authKeeper.AddressCodec().StringToBytes(redelegation.DelegatorAddress)
	if err != nil {
		return math.ZeroInt(), fmt.Errorf("SlashRedelegation: could not parse delegator address: %w", err)
	}

	// perform slashing on all entries within the redelegation
	for _, entry := range redelegation.Entries {
		// If redelegation started before this height, stake didn't contribute to infraction
		if entry.CreationHeight < infractionHeight {
			continue
		}

		if entry.IsMature(now) && !entry.OnHold() {
			// Redelegation no longer eligible for slashing, skip it
			continue
		}

		// Calculate slash amount proportional to stake contributing to infraction
		slashAmountDec := slashFactor.MulInt(entry.InitialBalance)
		slashAmount := slashAmountDec.TruncateInt()
		totalSlashAmount = totalSlashAmount.Add(slashAmount)

		// Handle undelegation after redelegation
		// Prioritize slashing unbondingDelegation than delegation
		unbondingDelegation, err := k.UnbondingDelegations.Get(ctx, collections.Join(delegatorAddress, valDstAddr))
		if err == nil {
			for i, entry := range unbondingDelegation.Entries {
				// slash with the amount of `slashAmount` if possible, else slash all unbonding token
				unbondingSlashAmount := math.MinInt(slashAmount, entry.Balance)

				switch {
				// There's no token to slash
				case unbondingSlashAmount.IsZero():
					continue
				// If unbonding started before this height, stake didn't contribute to infraction
				case entry.CreationHeight < infractionHeight:
					continue
				// Unbonding delegation no longer eligible for slashing, skip it
				case entry.IsMature(now) && !entry.OnHold():
					continue
				// Slash the unbonding delegation
				default:
					// update remaining slashAmount
					slashAmount = slashAmount.Sub(unbondingSlashAmount)

					notBondedBurnedAmount = notBondedBurnedAmount.Add(unbondingSlashAmount)
					entry.Balance = entry.Balance.Sub(unbondingSlashAmount)
					unbondingDelegation.Entries[i] = entry
					if err = k.SetUnbondingDelegation(ctx, unbondingDelegation); err != nil {
						return math.ZeroInt(), err
					}
				}
			}
		}

		// Slash the moved delegation
		// Unbond from target validator
		if slashAmount.IsZero() {
			continue
		}

		dstVal, err := k.GetValidator(ctx, valDstAddr)
		if err != nil {
			return math.ZeroInt(), err
		}

		sharesToUnbond, err := dstVal.SharesFromTokensTruncated(slashAmount)
		if sharesToUnbond.IsZero() {
			continue
		} else if err != nil {
			return math.ZeroInt(), err
		}

		// Delegations can be dynamic hence need to be looked up on every redelegation entry loop.
		delegation, err := k.Delegations.Get(ctx, collections.Join(sdk.AccAddress(delegatorAddress), sdk.ValAddress(valDstAddr)))
		if err != nil {
			// If deleted, delegation has zero shares, and we can't unbond any more
			continue
		}

		if sharesToUnbond.GT(delegation.Shares) {
			sharesToUnbond = delegation.Shares
		}

		tokensToBurn, err := k.Unbond(ctx, delegatorAddress, valDstAddr, sharesToUnbond)
		if err != nil {
			return math.ZeroInt(), err
		}

		dstValidator, err := k.GetValidator(ctx, valDstAddr)
		isNotFoundErr := errors.Is(err, types.ErrNoValidatorFound)
		if err != nil && !isNotFoundErr {
			return math.ZeroInt(), err
		}

		// tokens of a redelegation currently live in the destination validator
		// therefore we must burn tokens from the destination-validator's bonding status
		switch {
		// this case covers for when a validator is removed from the set when his bond drops down to 0.
		case isNotFoundErr:
			notBondedBurnedAmount = notBondedBurnedAmount.Add(tokensToBurn)
		case dstValidator.IsBonded():
			bondedBurnedAmount = bondedBurnedAmount.Add(tokensToBurn)
		case dstValidator.IsUnbonded() || dstValidator.IsUnbonding():
			notBondedBurnedAmount = notBondedBurnedAmount.Add(tokensToBurn)
		default:
			return math.ZeroInt(), errors.New("unknown validator status")
		}
	}

	if err := k.burnBondedTokens(ctx, bondedBurnedAmount); err != nil {
		return math.ZeroInt(), err
	}

	if err := k.burnNotBondedTokens(ctx, notBondedBurnedAmount); err != nil {
		return math.ZeroInt(), err
	}

	return totalSlashAmount, nil
}
