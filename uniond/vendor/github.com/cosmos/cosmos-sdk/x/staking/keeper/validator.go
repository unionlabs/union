package keeper

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"time"

	cmtprotocrypto "github.com/cometbft/cometbft/proto/tendermint/crypto"
	gogotypes "github.com/cosmos/gogoproto/types"

	corestore "cosmossdk.io/core/store"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

// GetValidator gets a single validator
func (k Keeper) GetValidator(ctx context.Context, addr sdk.ValAddress) (validator types.Validator, err error) {
	store := k.storeService.OpenKVStore(ctx)
	value, err := store.Get(types.GetValidatorKey(addr))
	if err != nil {
		return validator, err
	}

	if value == nil {
		return validator, types.ErrNoValidatorFound
	}

	return types.UnmarshalValidator(k.cdc, value)
}

func (k Keeper) mustGetValidator(ctx context.Context, addr sdk.ValAddress) types.Validator {
	validator, err := k.GetValidator(ctx, addr)
	if err != nil {
		panic(fmt.Sprintf("validator record not found for address: %X\n", addr))
	}

	return validator
}

// GetValidatorByConsAddr gets a single validator by consensus address
func (k Keeper) GetValidatorByConsAddr(ctx context.Context, consAddr sdk.ConsAddress) (validator types.Validator, err error) {
	store := k.storeService.OpenKVStore(ctx)
	opAddr, err := store.Get(types.GetValidatorByConsAddrKey(consAddr))
	if err != nil {
		return validator, err
	}

	if opAddr == nil {
		return validator, types.ErrNoValidatorFound
	}

	return k.GetValidator(ctx, opAddr)
}

// Increments the number of jailed validators by one.
//
// Will assume a stored value of `nil` should be read as zero and will increment
// by 1 (will not fail).
func (k Keeper) IncermentNumberofValidatorsInJail(ctx context.Context) {
	var bz []byte
	store := k.storeService.OpenKVStore(ctx)

	has, err := store.Has(types.NumberOfValidatorsInJail)
	if err != nil {
		// TODO(aeryz): Seems like panics are changed into errors, check if it is okay to panic here
		panic(err)
	}

	if has {
		// value found, increment
		intV := gogotypes.UInt32Value{}
		val, err2 := store.Get(types.NumberOfValidatorsInJail)
		if err2 != nil {
			// TODO(aeryz): Seems like panics are changed into errors, check if it is okay to panic here
			panic(err)
		}
		k.cdc.MustUnmarshal(val, &intV)
		bz = k.cdc.MustMarshal(&gogotypes.UInt32Value{Value: intV.Value + 1})
	} else {
		// no value stored yet, assume it should be zero and add one
		bz = k.cdc.MustMarshal(&gogotypes.UInt32Value{Value: 1})
	}

	store.Set(types.NumberOfValidatorsInJail, bz)
}

// Decrement the number of jailed validators by one.
//
// There must be some number of jailed validators greater than zero, otherwise this function will return an error.
//
// # Errors
//
// - `ErrInvalidRequest` The value stored for `NumberOfJailedValidators` is either `nil` or zero
func (k Keeper) DecrementNumberofValidatorsInJail(ctx context.Context) error {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(types.NumberOfValidatorsInJail)
	if err != nil {
		return err
	}

	if bz == nil {
		return errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "No Jailed validators exists")
	}

	intV := gogotypes.UInt32Value{}
	k.cdc.MustUnmarshal(bz, &intV)

	if intV.Value == 0 {
		return errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "No Jailed validators exists")
	}

	bz = k.cdc.MustMarshal(&gogotypes.UInt32Value{Value: intV.Value - 1})

	return nil
}

func (k Keeper) mustGetValidatorByConsAddr(ctx context.Context, consAddr sdk.ConsAddress) types.Validator {
	validator, err := k.GetValidatorByConsAddr(ctx, consAddr)
	if err != nil {
		panic(fmt.Errorf("validator with consensus-Address %s not found", consAddr))
	}

	return validator
}

func (k Keeper) ResetNumberofValidatorsInJail(ctx context.Context) {
	store := k.storeService.OpenKVStore(ctx)

	bz := k.cdc.MustMarshal(&gogotypes.UInt32Value{Value: 0})

	store.Set(types.NumberOfValidatorsInJail, bz)
}

// SetValidator sets the main record holding validator details
func (k Keeper) SetValidator(ctx context.Context, validator types.Validator) error {
	store := k.storeService.OpenKVStore(ctx)
	bz := types.MustMarshalValidator(k.cdc, &validator)
	str, err := k.ValidatorAddressCodec().StringToBytes(validator.GetOperator())
	if err != nil {
		return err
	}
	return store.Set(types.GetValidatorKey(str), bz)
}

// SetValidatorByConsAddr sets a validator by conesensus address
func (k Keeper) SetValidatorByConsAddr(ctx context.Context, validator types.Validator) error {
	consPk, err := validator.GetConsAddr()
	if err != nil {
		return err
	}
	store := k.storeService.OpenKVStore(ctx)

	bz, err := k.validatorAddressCodec.StringToBytes(validator.GetOperator())
	if err != nil {
		return err
	}

	return store.Set(types.GetValidatorByConsAddrKey(consPk), bz)
}

// SetValidatorByPowerIndex sets a validator by power index
func (k Keeper) SetValidatorByPowerIndex(ctx context.Context, validator types.Validator) error {
	// jailed validators are not kept in the power index
	if validator.Jailed {
		return nil
	}

	store := k.storeService.OpenKVStore(ctx)
	str, err := k.validatorAddressCodec.StringToBytes(validator.GetOperator())
	if err != nil {
		return err
	}
	return store.Set(types.GetValidatorsByPowerIndexKey(validator, k.PowerReduction(ctx), k.validatorAddressCodec), str)
}

// DeleteValidatorByPowerIndex deletes a record by power index
func (k Keeper) DeleteValidatorByPowerIndex(ctx context.Context, validator types.Validator) error {
	store := k.storeService.OpenKVStore(ctx)
	return store.Delete(types.GetValidatorsByPowerIndexKey(validator, k.PowerReduction(ctx), k.validatorAddressCodec))
}

// SetNewValidatorByPowerIndex adds new entry by power index
func (k Keeper) SetNewValidatorByPowerIndex(ctx context.Context, validator types.Validator) error {
	store := k.storeService.OpenKVStore(ctx)
	str, err := k.validatorAddressCodec.StringToBytes(validator.GetOperator())
	if err != nil {
		return err
	}
	return store.Set(types.GetValidatorsByPowerIndexKey(validator, k.PowerReduction(ctx), k.validatorAddressCodec), str)
}

// AddValidatorTokensAndShares updates the tokens of an existing validator, updates the validators power index key
func (k Keeper) AddValidatorTokensAndShares(ctx context.Context, validator types.Validator,
	tokensToAdd math.Int,
) (valOut types.Validator, addedShares math.LegacyDec, err error) {
	err = k.DeleteValidatorByPowerIndex(ctx, validator)
	if err != nil {
		return valOut, addedShares, err
	}

	validator, addedShares = validator.AddTokensFromDel(tokensToAdd)
	err = k.SetValidator(ctx, validator)
	if err != nil {
		return validator, addedShares, err
	}

	err = k.SetValidatorByPowerIndex(ctx, validator)
	return validator, addedShares, err
}

// RemoveValidatorTokensAndShares updates the tokens of an existing validator, updates the validators power index key
func (k Keeper) RemoveValidatorTokensAndShares(ctx context.Context, validator types.Validator,
	sharesToRemove math.LegacyDec,
) (valOut types.Validator, removedTokens math.Int, err error) {
	err = k.DeleteValidatorByPowerIndex(ctx, validator)
	if err != nil {
		return valOut, removedTokens, err
	}
	validator, removedTokens = validator.RemoveDelShares(sharesToRemove)
	err = k.SetValidator(ctx, validator)
	if err != nil {
		return validator, removedTokens, err
	}

	err = k.SetValidatorByPowerIndex(ctx, validator)
	return validator, removedTokens, err
}

// RemoveValidatorTokens updates the tokens of an existing validator, updates the validators power index key
func (k Keeper) RemoveValidatorTokens(ctx context.Context,
	validator types.Validator, tokensToRemove math.Int,
) (types.Validator, error) {
	if err := k.DeleteValidatorByPowerIndex(ctx, validator); err != nil {
		return validator, err
	}

	validator = validator.RemoveTokens(tokensToRemove)
	if err := k.SetValidator(ctx, validator); err != nil {
		return validator, err
	}

	if err := k.SetValidatorByPowerIndex(ctx, validator); err != nil {
		return validator, err
	}

	return validator, nil
}

// UpdateValidatorCommission attempts to update a validator's commission rate.
// An error is returned if the new commission rate is invalid.
func (k Keeper) UpdateValidatorCommission(ctx context.Context,
	validator types.Validator, newRate math.LegacyDec,
) (types.Commission, error) {
	commission := validator.Commission
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	blockTime := sdkCtx.BlockHeader().Time

	if err := commission.ValidateNewRate(newRate, blockTime); err != nil {
		return commission, err
	}

	minCommissionRate, err := k.MinCommissionRate(ctx)
	if err != nil {
		return commission, err
	}

	if newRate.LT(minCommissionRate) {
		return commission, fmt.Errorf("cannot set validator commission to less than minimum rate of %s", minCommissionRate)
	}

	commission.Rate = newRate
	commission.UpdateTime = blockTime

	return commission, nil
}

// RemoveValidator removes the validator record and associated indexes
// except for the bonded validator index which is only handled in ApplyAndReturnTendermintUpdates
func (k Keeper) RemoveValidator(ctx context.Context, address sdk.ValAddress) error {
	// first retrieve the old validator record
	validator, err := k.GetValidator(ctx, address)
	if errors.Is(err, types.ErrNoValidatorFound) {
		return nil
	}

	if !validator.IsUnbonded() {
		return types.ErrBadRemoveValidator.Wrap("cannot call RemoveValidator on bonded or unbonding validators")
	}

	if validator.Tokens.IsPositive() {
		return types.ErrBadRemoveValidator.Wrap("attempting to remove a validator which still contains tokens")
	}

	valConsAddr, err := validator.GetConsAddr()
	if err != nil {
		return err
	}

	// delete the old validator record
	store := k.storeService.OpenKVStore(ctx)
	if err = store.Delete(types.GetValidatorKey(address)); err != nil {
		return err
	}

	if err = store.Delete(types.GetValidatorByConsAddrKey(valConsAddr)); err != nil {
		return err
	}

	if err = store.Delete(types.GetValidatorsByPowerIndexKey(validator, k.PowerReduction(ctx), k.validatorAddressCodec)); err != nil {
		return err
	}

	str, err := k.validatorAddressCodec.StringToBytes(validator.GetOperator())
	if err != nil {
		return err
	}

	if err := k.Hooks().AfterValidatorRemoved(ctx, valConsAddr, str); err != nil {
		k.Logger(ctx).Error("error in after validator removed hook", "error", err)
	}

	return nil
}

// get groups of validators

// GetAllValidators gets the set of all validators with no limits, used during genesis dump
func (k Keeper) GetAllValidators(ctx context.Context) (validators []types.Validator, err error) {
	store := k.storeService.OpenKVStore(ctx)

	iterator, err := store.Iterator(types.ValidatorsKey, storetypes.PrefixEndBytes(types.ValidatorsKey))
	if err != nil {
		return nil, err
	}
	defer iterator.Close()

	for ; iterator.Valid(); iterator.Next() {
		validator, err := types.UnmarshalValidator(k.cdc, iterator.Value())
		if err != nil {
			return nil, err
		}
		validators = append(validators, validator)
	}

	return validators, nil
}

// GetValidators returns a given amount of all the validators
func (k Keeper) GetValidators(ctx context.Context, maxRetrieve uint32) (validators []types.Validator, err error) {
	store := k.storeService.OpenKVStore(ctx)
	validators = make([]types.Validator, maxRetrieve)

	iterator, err := store.Iterator(types.ValidatorsKey, storetypes.PrefixEndBytes(types.ValidatorsKey))
	if err != nil {
		return nil, err
	}

	i := 0
	for ; iterator.Valid() && i < int(maxRetrieve); iterator.Next() {
		validator, err := types.UnmarshalValidator(k.cdc, iterator.Value())
		if err != nil {
			return nil, err
		}
		validators[i] = validator
		i++
	}

	return validators[:i], nil // trim if the array length < maxRetrieve
}

// GetBondedValidatorsByPower gets the current group of bonded validators sorted by power-rank
func (k Keeper) GetBondedValidatorsByPower(ctx context.Context) ([]types.Validator, error) {
	maxValidators, err := k.MaxValidators(ctx)
	if err != nil {
		return nil, err
	}
	validators := make([]types.Validator, maxValidators)

	iterator, err := k.ValidatorsPowerStoreIterator(ctx)
	if err != nil {
		return nil, err
	}
	defer iterator.Close()

	i := 0
	for ; iterator.Valid() && i < int(maxValidators); iterator.Next() {
		address := iterator.Value()
		validator := k.mustGetValidator(ctx, address)

		if validator.IsBonded() {
			validators[i] = validator
			i++
		}
	}

	return validators[:i], nil // trim
}

// ValidatorsPowerStoreIterator returns an iterator for the current validator power store
func (k Keeper) ValidatorsPowerStoreIterator(ctx context.Context) (corestore.Iterator, error) {
	store := k.storeService.OpenKVStore(ctx)
	return store.ReverseIterator(types.ValidatorsByPowerIndexKey, storetypes.PrefixEndBytes(types.ValidatorsByPowerIndexKey))
}

// Last Validator Index

// GetLastValidatorPower loads the last validator power.
// Returns zero if the operator was not a validator last block.
func (k Keeper) GetLastValidatorPower(ctx context.Context, operator sdk.ValAddress) (power int64, err error) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(types.GetLastValidatorPowerKey(operator))
	if err != nil {
		return 0, err
	}

	if bz == nil {
		return 0, nil
	}

	intV := gogotypes.Int64Value{}
	err = k.cdc.Unmarshal(bz, &intV)
	if err != nil {
		return 0, err
	}

	return intV.GetValue(), nil
}

// SetLastValidatorPower sets the last validator power.
func (k Keeper) SetLastValidatorPower(ctx context.Context, operator sdk.ValAddress, power int64) error {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := k.cdc.Marshal(&gogotypes.Int64Value{Value: power})
	if err != nil {
		return err
	}
	return store.Set(types.GetLastValidatorPowerKey(operator), bz)
}

// DeleteLastValidatorPower deletes the last validator power.
func (k Keeper) DeleteLastValidatorPower(ctx context.Context, operator sdk.ValAddress) error {
	store := k.storeService.OpenKVStore(ctx)
	return store.Delete(types.GetLastValidatorPowerKey(operator))
}

// lastValidatorsIterator returns an iterator for the consensus validators in the last block
func (k Keeper) LastValidatorsIterator(ctx context.Context) (corestore.Iterator, error) {
	store := k.storeService.OpenKVStore(ctx)
	return store.Iterator(types.LastValidatorPowerKey, storetypes.PrefixEndBytes(types.LastValidatorPowerKey))
}

// IterateLastValidatorPowers iterates over last validator powers.
func (k Keeper) IterateLastValidatorPowers(ctx context.Context, handler func(operator sdk.ValAddress, power int64) (stop bool)) error {
	iter, err := k.LastValidatorsIterator(ctx)
	if err != nil {
		return err
	}

	for ; iter.Valid(); iter.Next() {
		addr := sdk.ValAddress(types.AddressFromLastValidatorPowerKey(iter.Key()))
		intV := &gogotypes.Int64Value{}

		if err = k.cdc.Unmarshal(iter.Value(), intV); err != nil {
			return err
		}

		if handler(addr, intV.GetValue()) {
			break
		}
	}

	return nil
}

// GetLastValidators gets the group of the bonded validators
func (k Keeper) GetLastValidators(ctx context.Context) (validators []types.Validator, err error) {
	store := k.storeService.OpenKVStore(ctx)

	// add the actual validator power sorted store
	maxValidators, err := k.MaxValidators(ctx)
	if err != nil {
		return nil, err
	}
	validators = make([]types.Validator, maxValidators)

	iterator, err := store.Iterator(types.LastValidatorPowerKey, storetypes.PrefixEndBytes(types.LastValidatorPowerKey))
	if err != nil {
		return nil, err
	}
	defer iterator.Close()

	i := 0
	for ; iterator.Valid(); iterator.Next() {
		// sanity check
		if i >= int(maxValidators) {
			panic("more validators than maxValidators found")
		}

		address := types.AddressFromLastValidatorPowerKey(iterator.Key())
		validator, err := k.GetValidator(ctx, address)
		if err != nil {
			return nil, err
		}

		validators[i] = validator
		i++
	}

	return validators[:i], nil // trim
}

// GetUnbondingValidators returns a slice of mature validator addresses that
// complete their unbonding at a given time and height.
func (k Keeper) GetUnbondingValidators(ctx context.Context, endTime time.Time, endHeight int64) ([]string, error) {
	store := k.storeService.OpenKVStore(ctx)

	bz, err := store.Get(types.GetValidatorQueueKey(endTime, endHeight))
	if err != nil {
		return nil, err
	}

	if bz == nil {
		return []string{}, nil
	}

	addrs := types.ValAddresses{}
	if err = k.cdc.Unmarshal(bz, &addrs); err != nil {
		return nil, err
	}

	return addrs.Addresses, nil
}

// SetUnbondingValidatorsQueue sets a given slice of validator addresses into
// the unbonding validator queue by a given height and time.
func (k Keeper) SetUnbondingValidatorsQueue(ctx context.Context, endTime time.Time, endHeight int64, addrs []string) error {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := k.cdc.Marshal(&types.ValAddresses{Addresses: addrs})
	if err != nil {
		return err
	}
	return store.Set(types.GetValidatorQueueKey(endTime, endHeight), bz)
}

// InsertUnbondingValidatorQueue inserts a given unbonding validator address into
// the unbonding validator queue for a given height and time.
func (k Keeper) InsertUnbondingValidatorQueue(ctx context.Context, val types.Validator) error {
	addrs, err := k.GetUnbondingValidators(ctx, val.UnbondingTime, val.UnbondingHeight)
	if err != nil {
		return err
	}
	addrs = append(addrs, val.OperatorAddress)
	return k.SetUnbondingValidatorsQueue(ctx, val.UnbondingTime, val.UnbondingHeight, addrs)
}

// DeleteValidatorQueueTimeSlice deletes all entries in the queue indexed by a
// given height and time.
func (k Keeper) DeleteValidatorQueueTimeSlice(ctx context.Context, endTime time.Time, endHeight int64) error {
	store := k.storeService.OpenKVStore(ctx)
	return store.Delete(types.GetValidatorQueueKey(endTime, endHeight))
}

// DeleteValidatorQueue removes a validator by address from the unbonding queue
// indexed by a given height and time.
func (k Keeper) DeleteValidatorQueue(ctx context.Context, val types.Validator) error {
	addrs, err := k.GetUnbondingValidators(ctx, val.UnbondingTime, val.UnbondingHeight)
	if err != nil {
		return err
	}
	newAddrs := []string{}

	// since address string may change due to Bech32 prefix change, we parse the addresses into bytes
	// format for normalization
	deletingAddr, err := k.validatorAddressCodec.StringToBytes(val.OperatorAddress)
	if err != nil {
		return err
	}

	for _, addr := range addrs {
		storedAddr, err := k.validatorAddressCodec.StringToBytes(addr)
		if err != nil {
			// even if we don't error here, it will error in UnbondAllMatureValidators at unbond time
			return err
		}
		if !bytes.Equal(storedAddr, deletingAddr) {
			newAddrs = append(newAddrs, addr)
		}
	}

	if len(newAddrs) == 0 {
		return k.DeleteValidatorQueueTimeSlice(ctx, val.UnbondingTime, val.UnbondingHeight)
	}

	return k.SetUnbondingValidatorsQueue(ctx, val.UnbondingTime, val.UnbondingHeight, newAddrs)
}

// ValidatorQueueIterator returns an interator ranging over validators that are
// unbonding whose unbonding completion occurs at the given height and time.
func (k Keeper) ValidatorQueueIterator(ctx context.Context, endTime time.Time, endHeight int64) (corestore.Iterator, error) {
	store := k.storeService.OpenKVStore(ctx)
	return store.Iterator(types.ValidatorQueueKey, storetypes.InclusiveEndBytes(types.GetValidatorQueueKey(endTime, endHeight)))
}

// UnbondAllMatureValidators unbonds all the mature unbonding validators that
// have finished their unbonding period.
func (k Keeper) UnbondAllMatureValidators(ctx context.Context) error {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	blockTime := sdkCtx.BlockTime()
	blockHeight := sdkCtx.BlockHeight()

	// unbondingValIterator will contains all validator addresses indexed under
	// the ValidatorQueueKey prefix. Note, the entire index key is composed as
	// ValidatorQueueKey | timeBzLen (8-byte big endian) | timeBz | heightBz (8-byte big endian),
	// so it may be possible that certain validator addresses that are iterated
	// over are not ready to unbond, so an explicit check is required.
	unbondingValIterator, err := k.ValidatorQueueIterator(ctx, blockTime, blockHeight)
	if err != nil {
		return err
	}
	defer unbondingValIterator.Close()

	for ; unbondingValIterator.Valid(); unbondingValIterator.Next() {
		key := unbondingValIterator.Key()
		keyTime, keyHeight, err := types.ParseValidatorQueueKey(key)
		if err != nil {
			return fmt.Errorf("failed to parse unbonding key: %w", err)
		}

		// All addresses for the given key have the same unbonding height and time.
		// We only unbond if the height and time are less than the current height
		// and time.
		if keyHeight <= blockHeight && (keyTime.Before(blockTime) || keyTime.Equal(blockTime)) {
			addrs := types.ValAddresses{}
			if err = k.cdc.Unmarshal(unbondingValIterator.Value(), &addrs); err != nil {
				return err
			}

			for _, valAddr := range addrs.Addresses {
				addr, err := k.validatorAddressCodec.StringToBytes(valAddr)
				if err != nil {
					return err
				}
				val, err := k.GetValidator(ctx, addr)
				if err != nil {
					return errorsmod.Wrap(err, "validator in the unbonding queue was not found")
				}

				if !val.IsUnbonding() {
					return fmt.Errorf("unexpected validator in unbonding queue; status was not unbonding")
				}

				if val.UnbondingOnHoldRefCount == 0 {
					for _, id := range val.UnbondingIds {
						if err = k.DeleteUnbondingIndex(ctx, id); err != nil {
							return err
						}
					}

					val, err = k.UnbondingToUnbonded(ctx, val)
					if err != nil {
						return err
					}

					if val.GetDelegatorShares().IsZero() {
						str, err := k.validatorAddressCodec.StringToBytes(val.GetOperator())
						if err != nil {
							return err
						}
						if err = k.RemoveValidator(ctx, str); err != nil {
							return err
						}
					} else {
						// remove unbonding ids
						val.UnbondingIds = []uint64{}
					}

					// remove validator from queue
					if err = k.DeleteValidatorQueue(ctx, val); err != nil {
						return err
					}
				}
			}
		}
	}
	return nil
}

// IsValidatorJailed checks and returns boolean of a validator status jailed or not.
func (k Keeper) IsValidatorJailed(ctx context.Context, addr sdk.ConsAddress) (bool, error) {
	v, err := k.GetValidatorByConsAddr(ctx, addr)
	if err != nil {
		return false, err
	}

	return v.Jailed, nil
}

// GetPubKeyByConsAddr returns the consensus public key by consensus address.
func (k Keeper) GetPubKeyByConsAddr(ctx context.Context, addr sdk.ConsAddress) (cmtprotocrypto.PublicKey, error) {
	v, err := k.GetValidatorByConsAddr(ctx, addr)
	if err != nil {
		return cmtprotocrypto.PublicKey{}, err
	}

	pubkey, err := v.CmtConsPublicKey()
	if err != nil {
		return cmtprotocrypto.PublicKey{}, err
	}

	return pubkey, nil
}

func (k Keeper) SetNumberOfValidatorsInEpoch(ctx context.Context, n uint32) {
	store := k.storeService.OpenKVStore(ctx)

	bz := k.cdc.MustMarshal(&gogotypes.UInt32Value{Value: n})

	store.Set(types.NumberOfValidatorsInEpoch, bz)
}

func (k Keeper) GetNumberOfValidatorsInEpoch(ctx context.Context) uint32 {
	store := k.storeService.OpenKVStore(ctx)

	has, err := store.Has(types.NumberOfValidatorsInEpoch)
	if err != nil {
		// TODO(aeryz): check if its ok to panic here
		panic(err)
	}

	if has {
		bz, err := store.Get(types.NumberOfValidatorsInEpoch)
		if err != nil {
			// TODO(aeryz): check if its ok to panic here
			panic(err)
		}
		intV := gogotypes.UInt32Value{}
		k.cdc.MustUnmarshal(bz, &intV)
		return intV.Value
	} else {
		return 0
	}
}

func (k Keeper) GetNumberOfJailedValidators(ctx context.Context) uint32 {
	store := k.storeService.OpenKVStore(ctx)

	has, err := store.Has(types.NumberOfValidatorsInJail)
	if err != nil {
		// TODO(aeryz): check if its ok to panic here
		panic(err)
	}

	if has {
		bz, err := store.Get(types.NumberOfValidatorsInJail)
		if err != nil {
			// TODO(aeryz): check if its ok to panic here
			panic(err)
		}
		intV := gogotypes.UInt32Value{}
		k.cdc.MustUnmarshal(bz, &intV)
		return intV.Value
	} else {
		return 0
	}
}
