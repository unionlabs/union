package keeper

import (
	"context"

	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

// Validator Set

// IterateValidators iterates through the validator set and perform the provided function
func (k Keeper) IterateValidators(ctx context.Context, fn func(index int64, validator types.ValidatorI) (stop bool)) error {
	store := k.storeService.OpenKVStore(ctx)
	iterator, err := store.Iterator(types.ValidatorsKey, storetypes.PrefixEndBytes(types.ValidatorsKey))
	if err != nil {
		return err
	}
	defer iterator.Close()

	i := int64(0)

	for ; iterator.Valid(); iterator.Next() {
		validator, err := types.UnmarshalValidator(k.cdc, iterator.Value())
		if err != nil {
			return err
		}
		stop := fn(i, validator) // XXX is this safe will the validator unexposed fields be able to get written to?

		if stop {
			break
		}
		i++
	}

	return nil
}

// IterateBondedValidatorsByPower iterates through the bonded validator set and perform the provided function
func (k Keeper) IterateBondedValidatorsByPower(ctx context.Context, fn func(index int64, validator types.ValidatorI) (stop bool)) error {
	store := k.storeService.OpenKVStore(ctx)
	maxValidators, err := k.MaxValidators(ctx)
	if err != nil {
		return err
	}

	iterator, err := store.ReverseIterator(types.ValidatorsByPowerIndexKey, storetypes.PrefixEndBytes(types.ValidatorsByPowerIndexKey))
	if err != nil {
		return err
	}
	defer iterator.Close()

	i := int64(0)
	for ; iterator.Valid() && i < int64(maxValidators); iterator.Next() {
		address := iterator.Value()
		validator := k.mustGetValidator(ctx, address)

		if validator.IsBonded() {
			stop := fn(i, validator) // XXX is this safe will the validator unexposed fields be able to get written to?
			if stop {
				break
			}
			i++
		}
	}

	return nil
}

// IterateLastValidators iterates through the active validator set and perform the provided function
func (k Keeper) IterateLastValidators(ctx context.Context, fn func(index int64, validator types.ValidatorI) (stop bool)) error {
	iterator, err := k.LastValidatorsIterator(ctx)
	if err != nil {
		return err
	}
	defer iterator.Close()

	i := int64(0)

	for ; iterator.Valid(); iterator.Next() {
		address := types.AddressFromLastValidatorPowerKey(iterator.Key())

		validator, err := k.GetValidator(ctx, address)
		if err != nil {
			return err
		}

		stop := fn(i, validator) // XXX is this safe will the validator unexposed fields be able to get written to?
		if stop {
			break
		}
		i++
	}
	return nil
}

// Validator gets the Validator interface for a particular address
func (k Keeper) Validator(ctx context.Context, address sdk.ValAddress) (types.ValidatorI, error) {
	return k.GetValidator(ctx, address)
}

// ValidatorByConsAddr gets the validator interface for a particular pubkey
func (k Keeper) ValidatorByConsAddr(ctx context.Context, addr sdk.ConsAddress) (types.ValidatorI, error) {
	return k.GetValidatorByConsAddr(ctx, addr)
}

// Delegation Set

// GetValidatorSet returns self as it is both a validatorset and delegationset
func (k Keeper) GetValidatorSet() types.ValidatorSet {
	return k
}

// Delegation gets the delegation interface for a particular set of delegator and validator addresses
func (k Keeper) Delegation(ctx context.Context, addrDel sdk.AccAddress, addrVal sdk.ValAddress) (types.DelegationI, error) {
	bond, err := k.GetDelegation(ctx, addrDel, addrVal)
	if err != nil {
		return nil, err
	}

	return bond, nil
}

// IterateDelegations iterates through all of the delegations from a delegator
func (k Keeper) IterateDelegations(ctx context.Context, delAddr sdk.AccAddress,
	fn func(index int64, del types.DelegationI) (stop bool),
) error {
	store := k.storeService.OpenKVStore(ctx)
	delegatorPrefixKey := types.GetDelegationsKey(delAddr)
	iterator, err := store.Iterator(delegatorPrefixKey, storetypes.PrefixEndBytes(delegatorPrefixKey))
	if err != nil {
		return err
	}
	defer iterator.Close()

	for i := int64(0); iterator.Valid(); iterator.Next() {
		del, err := types.UnmarshalDelegation(k.cdc, iterator.Value())
		if err != nil {
			return err
		}

		stop := fn(i, del)
		if stop {
			break
		}
		i++
	}

	return nil
}

// GetAllSDKDelegations returns all delegations used during genesis dump
// TODO: remove this func, change all usage for iterate functionality
func (k Keeper) GetAllSDKDelegations(ctx context.Context) (delegations []types.Delegation, err error) {
	store := k.storeService.OpenKVStore(ctx)
	iterator, err := store.Iterator(types.DelegationKey, storetypes.PrefixEndBytes(types.DelegationKey))
	if err != nil {
		return delegations, err
	}
	defer iterator.Close()

	for ; iterator.Valid(); iterator.Next() {
		delegation, err := types.UnmarshalDelegation(k.cdc, iterator.Value())
		if err != nil {
			return delegations, err
		}
		delegations = append(delegations, delegation)
	}

	return
}
