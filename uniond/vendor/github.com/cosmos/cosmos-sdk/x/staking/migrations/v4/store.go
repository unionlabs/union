package v4

import (
	"sort"

	sdkmath "cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/exported"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

// MigrateStore performs in-place store migrations from v3 to v4.
func MigrateStore(ctx sdk.Context, store storetypes.KVStore, cdc codec.BinaryCodec, legacySubspace exported.Subspace) error {
	// migrate params
	if err := migrateParams(ctx, store, cdc, legacySubspace); err != nil {
		return err
	}

	// migrate unbonding delegations
	if err := migrateUBDEntries(ctx, store, cdc, legacySubspace); err != nil {
		return err
	}

	return nil
}

// migrateParams will set the params to store from legacySubspace
func migrateParams(ctx sdk.Context, store storetypes.KVStore, cdc codec.BinaryCodec, legacySubspace exported.Subspace) error {
	var legacyParams types.Params
	legacySubspace.GetParamSet(ctx, &legacyParams)

	if err := legacyParams.Validate(); err != nil {
		return err
	}

	bz := cdc.MustMarshal(&legacyParams)
	store.Set(types.ParamsKey, bz)
	return nil
}

// migrateUBDEntries will remove the ubdEntries with same creation_height
// and create a new ubdEntry with updated balance and initial_balance
func migrateUBDEntries(ctx sdk.Context, store storetypes.KVStore, cdc codec.BinaryCodec, legacySubspace exported.Subspace) error {
	iterator := storetypes.KVStorePrefixIterator(store, types.UnbondingDelegationKey)
	defer iterator.Close()

	for ; iterator.Valid(); iterator.Next() {
		ubd := types.MustUnmarshalUBD(cdc, iterator.Value())

		entriesAtSameCreationHeight := make(map[int64][]types.UnbondingDelegationEntry)
		for _, ubdEntry := range ubd.Entries {
			entriesAtSameCreationHeight[ubdEntry.CreationHeight] = append(entriesAtSameCreationHeight[ubdEntry.CreationHeight], ubdEntry)
		}

		creationHeights := make([]int64, 0, len(entriesAtSameCreationHeight))
		for k := range entriesAtSameCreationHeight {
			creationHeights = append(creationHeights, k)
		}

		sort.Slice(creationHeights, func(i, j int) bool { return creationHeights[i] < creationHeights[j] })

		ubd.Entries = make([]types.UnbondingDelegationEntry, 0, len(creationHeights))

		for _, h := range creationHeights {
			ubdEntry := types.UnbondingDelegationEntry{
				Balance:        sdkmath.ZeroInt(),
				InitialBalance: sdkmath.ZeroInt(),
			}
			for _, entry := range entriesAtSameCreationHeight[h] {
				ubdEntry.Balance = ubdEntry.Balance.Add(entry.Balance)
				ubdEntry.InitialBalance = ubdEntry.InitialBalance.Add(entry.InitialBalance)
				ubdEntry.CreationHeight = entry.CreationHeight
				ubdEntry.CompletionTime = entry.CompletionTime
			}
			ubd.Entries = append(ubd.Entries, ubdEntry)
		}

		// set the new ubd to the store
		setUBDToStore(ctx, store, cdc, ubd)
	}
	return nil
}

func setUBDToStore(ctx sdk.Context, store storetypes.KVStore, cdc codec.BinaryCodec, ubd types.UnbondingDelegation) {
	delegatorAddress := sdk.MustAccAddressFromBech32(ubd.DelegatorAddress)

	bz := types.MustMarshalUBD(cdc, ubd)

	addr, err := sdk.ValAddressFromBech32(ubd.ValidatorAddress)
	if err != nil {
		panic(err)
	}

	key := types.GetUBDKey(delegatorAddress, addr)

	store.Set(key, bz)
}
