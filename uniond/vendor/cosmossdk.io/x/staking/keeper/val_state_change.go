package keeper

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"sort"

	gogotypes "github.com/cosmos/gogoproto/types"

	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/event"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math"
	"cosmossdk.io/x/staking/types"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// BlockValidatorUpdates calculates the ValidatorUpdates for the current block
// Called in each EndBlock
func (k Keeper) BlockValidatorUpdates(ctx context.Context) ([]appmodule.ValidatorUpdate, error) {
	var (
		validatorUpdates []appmodule.ValidatorUpdate
		err              error
	)
	isValidatorSetRotation, err := k.isValidatorSetRotation(ctx)
	if isValidatorSetRotation {
		// NOTE: ApplyAndReturnValidatorSetUpdates has to come before
		// UnbondAllMatureValidatorQueue.
		// This fixes a bug when the unbonding period is instant (is the case in
		// some of the tests). The test expected the validator to be completely
		// unbonded after the Endblocker (go from Bonded -> Unbonding during
		// ApplyAndReturnValidatorSetUpdates and then Unbonding -> Unbonded during
		// UnbondAllMatureValidatorQueue).
		validatorUpdates, err = k.ApplyAndReturnValidatorSetUpdates(ctx)
		if err != nil {
			return nil, err
		}
	}

	// unbond all mature validators from the unbonding queue
	err = k.UnbondAllMatureValidators(ctx)
	if err != nil {
		return nil, err
	}

	time := k.HeaderService.HeaderInfo(ctx).Time
	// Remove all mature unbonding delegations from the ubd queue.
	matureUnbonds, err := k.DequeueAllMatureUBDQueue(ctx, time)
	if err != nil {
		return nil, err
	}

	for _, dvPair := range matureUnbonds {
		addr, err := k.validatorAddressCodec.StringToBytes(dvPair.ValidatorAddress)
		if err != nil {
			return nil, err
		}
		delegatorAddress, err := k.authKeeper.AddressCodec().StringToBytes(dvPair.DelegatorAddress)
		if err != nil {
			return nil, err
		}

		balances, err := k.CompleteUnbonding(ctx, delegatorAddress, addr)
		if err != nil {
			continue
		}

		if err := k.EventService.EventManager(ctx).EmitKV(
			types.EventTypeCompleteUnbonding,
			event.NewAttribute(sdk.AttributeKeyAmount, balances.String()),
			event.NewAttribute(types.AttributeKeyValidator, dvPair.ValidatorAddress),
			event.NewAttribute(types.AttributeKeyDelegator, dvPair.DelegatorAddress),
		); err != nil {
			return nil, err
		}
	}

	// Remove all mature redelegations from the red queue.
	matureRedelegations, err := k.DequeueAllMatureRedelegationQueue(ctx, time)
	if err != nil {
		return nil, err
	}

	for _, dvvTriplet := range matureRedelegations {
		valSrcAddr, err := k.validatorAddressCodec.StringToBytes(dvvTriplet.ValidatorSrcAddress)
		if err != nil {
			return nil, err
		}
		valDstAddr, err := k.validatorAddressCodec.StringToBytes(dvvTriplet.ValidatorDstAddress)
		if err != nil {
			return nil, err
		}
		delegatorAddress, err := k.authKeeper.AddressCodec().StringToBytes(dvvTriplet.DelegatorAddress)
		if err != nil {
			return nil, err
		}

		balances, err := k.CompleteRedelegation(
			ctx,
			delegatorAddress,
			valSrcAddr,
			valDstAddr,
		)
		if err != nil {
			continue
		}

		if err := k.EventService.EventManager(ctx).EmitKV(
			types.EventTypeCompleteRedelegation,
			event.NewAttribute(sdk.AttributeKeyAmount, balances.String()),
			event.NewAttribute(types.AttributeKeyDelegator, dvvTriplet.DelegatorAddress),
			event.NewAttribute(types.AttributeKeySrcValidator, dvvTriplet.ValidatorSrcAddress),
			event.NewAttribute(types.AttributeKeyDstValidator, dvvTriplet.ValidatorDstAddress),
		); err != nil {
			return nil, err
		}
	}

	err = k.PurgeAllMaturedConsKeyRotatedKeys(ctx, time)
	if err != nil {
		return nil, err
	}

	return validatorUpdates, nil
}

// ApplyAndReturnValidatorSetUpdates applies and return accumulated updates to the bonded validator set. Also,
// * Updates the active valset as keyed by LastValidatorPowerKey.
// * Updates the total power as keyed by LastTotalPowerKey.
// * Updates validator status' according to updated powers.
// * Updates the fee pool bonded vs not-bonded tokens.
// * Updates relevant indices.
// It gets called once after genesis, another time maybe after genesis transactions,
// then once at every EndBlock.
//
// CONTRACT: Only validators with non-zero power or zero-power that were bonded
// at the previous block height or were removed from the validator set entirely
// are returned to CometBFT.
func (k Keeper) ApplyAndReturnValidatorSetUpdates(ctx context.Context) ([]appmodule.ValidatorUpdate, error) {
	params, err := k.Params.Get(ctx)
	if err != nil {
		return nil, err
	}
	maxValidators := params.MaxValidators
	powerReduction := k.PowerReduction(ctx)
	totalPower := math.ZeroInt()
	amtFromBondedToNotBonded, amtFromNotBondedToBonded := math.ZeroInt(), math.ZeroInt()

	// Retrieve the last validator set.
	// The persistent set is updated later in this function.
	// (see LastValidatorPowerKey).
	last, err := k.getLastValidatorsByAddr(ctx)
	if err != nil {
		return nil, err
	}

	// Iterate over validators, highest power to lowest.
	iterator, err := k.ValidatorsPowerStoreIterator(ctx)
	if err != nil {
		return nil, err
	}
	defer iterator.Close()

	var updates []appmodule.ValidatorUpdate

	count := 0

	for ; iterator.Valid() && count < int(maxValidators); iterator.Next() {
		// everything that is iterated in this loop is becoming or already a
		// part of the bonded validator set
		valAddr := sdk.ValAddress(iterator.Value())
		validator, err := k.GetValidator(ctx, valAddr)
		if err != nil {
			return nil, fmt.Errorf("validator record not found for address: %X", valAddr)
		}

		if validator.Jailed {
			return nil, errors.New("should never retrieve a jailed validator from the power store")
		}

		// if we get to a zero-power validator (which we don't bond),
		// there are no more possible bonded validators
		if validator.PotentialConsensusPower(k.PowerReduction(ctx)) == 0 {
			break
		}

		// apply the appropriate state change if necessary
		switch {
		case validator.IsUnbonded():
			validator, err = k.unbondedToBonded(ctx, validator)
			if err != nil {
				return nil, err
			}
			amtFromNotBondedToBonded = amtFromNotBondedToBonded.Add(validator.GetTokens())
		case validator.IsUnbonding():
			validator, err = k.unbondingToBonded(ctx, validator)
			if err != nil {
				return nil, err
			}
			amtFromNotBondedToBonded = amtFromNotBondedToBonded.Add(validator.GetTokens())
		case validator.IsBonded():
			// no state change
		default:
			return nil, errors.New("unexpected validator status")
		}

		// fetch the old power bytes
		valAddrStr, err := k.validatorAddressCodec.BytesToString(valAddr)
		if err != nil {
			return nil, err
		}
		oldPowerBytes, found := last[valAddrStr]
		newPower := validator.ConsensusPower(powerReduction)
		newPowerBytes := k.cdc.MustMarshal(&gogotypes.Int64Value{Value: newPower})

		// update the validator set if power has changed
		if !found || !bytes.Equal(oldPowerBytes, newPowerBytes) {
			updates = append(updates, validator.ModuleValidatorUpdate(powerReduction))
			if err = k.SetLastValidatorPower(ctx, valAddr, newPower); err != nil {
				return nil, err
			}
		}

		delete(last, valAddrStr)
		count++

		totalPower = totalPower.Add(math.NewInt(newPower))
	}

	// Update the size of the new epochs validator set
	if err = k.NumberOfValidatorsInEpoch.Set(ctx, uint32(count)); err != nil {
		return nil, err
	}
	// Reset the number of jailed validators from the new epoch to 0
	if err = k.ResetNumberofValidatorsInJail(ctx); err != nil {
		return nil, err
	}

	noLongerBonded, err := sortNoLongerBonded(last, k.validatorAddressCodec)
	if err != nil {
		return nil, err
	}

	for _, valAddrBytes := range noLongerBonded {
		validator, err := k.GetValidator(ctx, sdk.ValAddress(valAddrBytes))
		if err != nil {
			return nil, fmt.Errorf("validator record not found for address: %X", sdk.ValAddress(valAddrBytes))
		}
		validator, err = k.bondedToUnbonding(ctx, validator)
		if err != nil {
			return nil, err
		}
		str, err := k.validatorAddressCodec.StringToBytes(validator.GetOperator())
		if err != nil {
			return nil, err
		}
		amtFromBondedToNotBonded = amtFromBondedToNotBonded.Add(validator.GetTokens())
		if err = k.DeleteLastValidatorPower(ctx, str); err != nil {
			return nil, err
		}

		updates = append(updates, validator.ModuleValidatorUpdateZero())
	}

	// ApplyAndReturnValidatorSetUpdates checks if there is ConsPubKeyRotationHistory
	// with ConsPubKeyRotationHistory.RotatedHeight == ctx.BlockHeight() and if so, generates 2 ValidatorUpdate,
	// one for a remove validator and one for create new validator
	historyObjects, err := k.GetBlockConsPubKeyRotationHistory(ctx)
	if err != nil {
		return nil, err
	}

	for _, history := range historyObjects {
		valAddr := history.OperatorAddress
		validator, err := k.GetValidator(ctx, valAddr)
		if err != nil {
			return nil, err
		}

		oldPkCached := history.OldConsPubkey.GetCachedValue()
		if oldPkCached == nil {
			return nil, errorsmod.Wrap(sdkerrors.ErrInvalidType, "OldConsPubkey cached value is nil")
		}
		oldPk, ok := oldPkCached.(cryptotypes.PubKey)
		if !ok {
			return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidType, "Expecting cryptotypes.PubKey, got %T", oldPkCached)
		}

		newPkCached := history.NewConsPubkey.GetCachedValue()
		if newPkCached == nil {
			return nil, errorsmod.Wrap(sdkerrors.ErrInvalidType, "NewConsPubkey cached value is nil")
		}
		newPk, ok := newPkCached.(cryptotypes.PubKey)
		if !ok {
			return nil, errorsmod.Wrapf(sdkerrors.ErrInvalidType, "Expecting cryptotypes.PubKey, got %T", newPkCached)
		}

		// a validator cannot rotate keys if it's not bonded or if it's jailed
		// - a validator can be unbonding state but jailed status false
		// - a validator can be jailed and status can be unbonding
		if !(validator.Jailed || validator.Status != types.Bonded) {
			updates = append(updates, appmodule.ValidatorUpdate{
				PubKey:     oldPk.Bytes(),
				PubKeyType: oldPk.Type(),
				Power:      0,
			})

			updates = append(updates, appmodule.ValidatorUpdate{
				PubKey:     newPk.Bytes(),
				PubKeyType: newPk.Type(),
				Power:      validator.ConsensusPower(powerReduction),
			})

			if err := k.updateToNewPubkey(ctx, validator, history.OldConsPubkey, history.NewConsPubkey, history.Fee); err != nil {
				return nil, err
			}
		}
	}

	// Update the pools based on the recent updates in the validator set:
	// - The tokens from the non-bonded candidates that enter the new validator set need to be transferred
	// to the Bonded pool.
	// - The tokens from the bonded validators that are being kicked out from the validator set
	// need to be transferred to the NotBonded pool.
	switch {
	// Compare and subtract the respective amounts to only perform one transfer.
	// This is done in order to avoid doing multiple updates inside each iterator/loop.
	case amtFromNotBondedToBonded.GT(amtFromBondedToNotBonded):
		if err = k.notBondedTokensToBonded(ctx, amtFromNotBondedToBonded.Sub(amtFromBondedToNotBonded)); err != nil {
			return nil, err
		}
	case amtFromNotBondedToBonded.LT(amtFromBondedToNotBonded):
		if err = k.bondedTokensToNotBonded(ctx, amtFromBondedToNotBonded.Sub(amtFromNotBondedToBonded)); err != nil {
			return nil, err
		}
	default: // equal amounts of tokens; no update required
	}

	// set total power on lookup index if there are any updates
	if len(updates) > 0 {
		if err = k.LastTotalPower.Set(ctx, totalPower); err != nil {
			return nil, err
		}
	}

	return updates, err
}

// Validator state transitions

func (k Keeper) bondedToUnbonding(ctx context.Context, validator types.Validator) (types.Validator, error) {
	if !validator.IsBonded() {
		return types.Validator{}, fmt.Errorf("bad state transition bondedToUnbonding, validator: %v", validator)
	}

	return k.BeginUnbondingValidator(ctx, validator)
}

func (k Keeper) unbondingToBonded(ctx context.Context, validator types.Validator) (types.Validator, error) {
	if !validator.IsUnbonding() {
		return types.Validator{}, fmt.Errorf("bad state transition unbondingToBonded, validator: %v", validator)
	}

	return k.bondValidator(ctx, validator)
}

func (k Keeper) unbondedToBonded(ctx context.Context, validator types.Validator) (types.Validator, error) {
	if !validator.IsUnbonded() {
		return types.Validator{}, fmt.Errorf("bad state transition unbondedToBonded, validator: %v", validator)
	}

	return k.bondValidator(ctx, validator)
}

// UnbondingToUnbonded switches a validator from unbonding state to unbonded state
func (k Keeper) UnbondingToUnbonded(ctx context.Context, validator types.Validator) (types.Validator, error) {
	if !validator.IsUnbonding() {
		return types.Validator{}, fmt.Errorf("bad state transition unbondingToUnbonded, validator: %v", validator)
	}

	return k.completeUnbondingValidator(ctx, validator)
}

// send a validator to jail
func (k Keeper) jailValidator(ctx context.Context, validator types.Validator) error {
	if validator.Jailed {
		return types.ErrValidatorJailed.Wrapf("cannot jail already jailed validator, validator: %v", validator)
	}

	validator.Jailed = true
	if err := k.SetValidator(ctx, validator); err != nil {
		return err
	}

	if err := k.IncermentNumberofValidatorsInJail(ctx); err != nil {
		return err
	}

	return k.DeleteValidatorByPowerIndex(ctx, validator)
}

// remove a validator from jail
func (k Keeper) unjailValidator(ctx context.Context, validator types.Validator) error {
	if !validator.Jailed {
		return fmt.Errorf("cannot unjail already unjailed validator, validator: %v", validator)
	}

	validator.Jailed = false
	if err := k.SetValidator(ctx, validator); err != nil {
		return err
	}

	if err := k.DecrementNumberofValidatorsInJail(ctx); err != nil {
		return err
	}

	return k.SetValidatorByPowerIndex(ctx, validator)
}

// perform all the store operations for when a validator status becomes bonded
func (k Keeper) bondValidator(ctx context.Context, validator types.Validator) (types.Validator, error) {
	// delete the validator by power index, as the key will change
	if err := k.DeleteValidatorByPowerIndex(ctx, validator); err != nil {
		return validator, err
	}

	validator = validator.UpdateStatus(types.Bonded)

	// save the now bonded validator record to the two referenced stores
	if err := k.SetValidator(ctx, validator); err != nil {
		return validator, err
	}

	if err := k.SetValidatorByPowerIndex(ctx, validator); err != nil {
		return validator, err
	}

	// delete from queue if present
	if err := k.DeleteValidatorQueue(ctx, validator); err != nil {
		return validator, err
	}

	// trigger hook
	consAddr, err := validator.GetConsAddr()
	if err != nil {
		return validator, err
	}

	str, err := k.validatorAddressCodec.StringToBytes(validator.GetOperator())
	if err != nil {
		return validator, err
	}

	if err := k.Hooks().AfterValidatorBonded(ctx, consAddr, str); err != nil {
		return validator, err
	}

	return validator, err
}

// BeginUnbondingValidator performs all the store operations for when a validator begins unbonding
func (k Keeper) BeginUnbondingValidator(ctx context.Context, validator types.Validator) (types.Validator, error) {
	params, err := k.Params.Get(ctx)
	if err != nil {
		return validator, err
	}

	// delete the validator by power index, as the key will change
	if err = k.DeleteValidatorByPowerIndex(ctx, validator); err != nil {
		return validator, err
	}

	// sanity check
	if validator.Status != types.Bonded {
		return validator, fmt.Errorf("should not already be unbonded or unbonding, validator: %v", validator)
	}

	id, err := k.IncrementUnbondingID(ctx)
	if err != nil {
		return validator, err
	}

	validator = validator.UpdateStatus(types.Unbonding)

	headerInfo := k.HeaderService.HeaderInfo(ctx)
	// set the unbonding completion time and completion height appropriately
	validator.UnbondingTime = headerInfo.Time.Add(params.UnbondingTime)
	validator.UnbondingHeight = headerInfo.Height

	validator.UnbondingIds = append(validator.UnbondingIds, id)

	// save the now unbonded validator record and power index
	if err = k.SetValidator(ctx, validator); err != nil {
		return validator, err
	}

	if err = k.SetValidatorByPowerIndex(ctx, validator); err != nil {
		return validator, err
	}

	// Adds to unbonding validator queue
	if err = k.InsertUnbondingValidatorQueue(ctx, validator); err != nil {
		return validator, err
	}

	// trigger hook
	consAddr, err := validator.GetConsAddr()
	if err != nil {
		return validator, err
	}

	str, err := k.validatorAddressCodec.StringToBytes(validator.GetOperator())
	if err != nil {
		return validator, err
	}

	if err := k.Hooks().AfterValidatorBeginUnbonding(ctx, consAddr, str); err != nil {
		return validator, err
	}

	if err := k.SetValidatorByUnbondingID(ctx, validator, id); err != nil {
		return validator, err
	}

	if err := k.Hooks().AfterUnbondingInitiated(ctx, id); err != nil {
		return validator, err
	}

	return validator, nil
}

// perform all the store operations for when a validator status becomes unbonded
func (k Keeper) completeUnbondingValidator(ctx context.Context, validator types.Validator) (types.Validator, error) {
	validator = validator.UpdateStatus(types.Unbonded)
	if err := k.SetValidator(ctx, validator); err != nil {
		return validator, err
	}

	return validator, nil
}

// map of operator bech32-addresses to serialized power
// We use bech32 strings here, because we can't have slices as keys: map[[]byte][]byte
type validatorsByAddr map[string][]byte

// get the last validator set
func (k Keeper) getLastValidatorsByAddr(ctx context.Context) (validatorsByAddr, error) {
	last := make(validatorsByAddr)

	err := k.LastValidatorPower.Walk(ctx, nil, func(key []byte, value gogotypes.Int64Value) (bool, error) {
		valAddrStr, err := k.validatorAddressCodec.BytesToString(key)
		if err != nil {
			return true, err
		}

		intV := value.GetValue()
		bz := k.cdc.MustMarshal(&gogotypes.Int64Value{Value: intV})
		last[valAddrStr] = bz
		return false, nil
	})
	if err != nil {
		return nil, err
	}

	return last, nil
}

// given a map of remaining validators to previous bonded power
// returns the list of validators to be unbonded, sorted by operator address
func sortNoLongerBonded(last validatorsByAddr, ac address.Codec) ([][]byte, error) {
	// sort the map keys for determinism
	noLongerBonded := make([][]byte, len(last))
	index := 0

	for valAddrStr := range last {
		valAddrBytes, err := ac.StringToBytes(valAddrStr)
		if err != nil {
			return nil, err
		}
		noLongerBonded[index] = valAddrBytes
		index++
	}
	// sorted by address - order doesn't matter
	sort.SliceStable(noLongerBonded, func(i, j int) bool {
		// -1 means strictly less than
		return bytes.Compare(noLongerBonded[i], noLongerBonded[j]) == -1
	})

	return noLongerBonded, nil
}

// Returns a boolean indicating if we should conduct a validator set rotation.
//
// True only if it's the end of an epoch or if the jailed validator threshold has been exceeded.
func (k Keeper) isValidatorSetRotation(goCtx context.Context) (bool, error) {
	ctx := goCtx.(sdk.Context)

	epochLength, err := k.EpochLength(ctx)
	if err != nil {
		return false, err
	}
	if isEndOfEpoch(ctx, epochLength) {
		ctx.Logger().Info("Rotating validator set due to end of epoch.")
		return true, nil
	}

	isOverJailedThreshold, err := k.isOverJailedThreshold(ctx)
	if err != nil {
		return false, err
	}
	if isOverJailedThreshold {
		ctx.Logger().Info("Rotating validator set due to exceeding the threshold of jailed validators.")
		return true, nil
	}

	return false, nil
}

// Determines if the number of jailed validators is over the configured threshold.
//
// Treats `JailedValidatorThreshold` as a percentage (`JailedvalidatorThreshold/100`)
func (k Keeper) isOverJailedThreshold(ctx context.Context) (bool, error) {
	epochValidatorSetSize, err := k.NumberOfValidatorsInEpoch.Get(ctx)
	if err != nil {
		return false, err
	}
	numberOfJailedValidators, err := k.NumberOfValidatorsInJail.Get(ctx)
	if err != nil {
		return false, err
	}

	mul := math.LegacyNewDecFromIntWithPrec(math.NewIntFromUint64(1), 2) // 0.01
	jailedThresholdRaw, err := k.JailedValidatorThreshold(ctx)
	if err != nil {
		return false, err
	}
	jailedThresholdPercentage := math.LegacyNewDecFromInt(math.NewIntFromUint64(uint64(jailedThresholdRaw))).Mul(mul)
	jailedThreshold := jailedThresholdPercentage.MulInt64(int64(epochValidatorSetSize)).RoundInt64()

	if jailedThreshold == 0 {
		jailedThreshold = 1
	}

	return jailedThreshold <= int64(numberOfJailedValidators), nil
}

// Returns true if the current block is the end of an epoch
func isEndOfEpoch(goCtx context.Context, epochLength int64) bool {
	ctx := goCtx.(sdk.Context)

	currentHeight := ctx.BlockHeight()

	return currentHeight%epochLength == 0
}
