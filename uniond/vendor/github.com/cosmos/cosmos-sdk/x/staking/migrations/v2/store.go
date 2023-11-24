package v2

import (
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	v1auth "github.com/cosmos/cosmos-sdk/x/auth/migrations/v1"
	v2distribution "github.com/cosmos/cosmos-sdk/x/distribution/migrations/v2"
	v1 "github.com/cosmos/cosmos-sdk/x/staking/migrations/v1"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

// migratePrefixAddressAddressAddress is a helper function that migrates all keys of format:
// prefix_bytes | address_1_bytes | address_2_bytes | address_3_bytes
// into format:
// prefix_bytes | address_1_len (1 byte) | address_1_bytes | address_2_len (1 byte) | address_2_bytes | address_3_len (1 byte) | address_3_bytes
func migratePrefixAddressAddressAddress(store storetypes.KVStore, prefixBz []byte) {
	oldStore := prefix.NewStore(store, prefixBz)

	oldStoreIter := oldStore.Iterator(nil, nil)
	defer oldStoreIter.Close()

	for ; oldStoreIter.Valid(); oldStoreIter.Next() {
		addr1 := oldStoreIter.Key()[:v1auth.AddrLen]
		addr2 := oldStoreIter.Key()[v1auth.AddrLen : 2*v1auth.AddrLen]
		addr3 := oldStoreIter.Key()[2*v1auth.AddrLen:]
		newStoreKey := append(append(append(
			prefixBz,
			address.MustLengthPrefix(addr1)...), address.MustLengthPrefix(addr2)...), address.MustLengthPrefix(addr3)...,
		)

		// Set new key on store. Values don't change.
		store.Set(newStoreKey, oldStoreIter.Value())
		oldStore.Delete(oldStoreIter.Key())
	}
}

const powerBytesLen = 8

func migrateValidatorsByPowerIndexKey(store storetypes.KVStore) {
	oldStore := prefix.NewStore(store, v1.ValidatorsByPowerIndexKey)

	oldStoreIter := oldStore.Iterator(nil, nil)
	defer oldStoreIter.Close()

	for ; oldStoreIter.Valid(); oldStoreIter.Next() {
		powerBytes := oldStoreIter.Key()[:powerBytesLen]
		valAddr := oldStoreIter.Key()[powerBytesLen:]
		newStoreKey := append(append(types.ValidatorsByPowerIndexKey, powerBytes...), address.MustLengthPrefix(valAddr)...)

		// Set new key on store. Values don't change.
		store.Set(newStoreKey, oldStoreIter.Value())
		oldStore.Delete(oldStoreIter.Key())
	}
}

// MigrateStore performs in-place store migrations from v0.40 to v0.43. The
// migration includes:
//
// - Setting the Power Reduction param in the paramstore
func MigrateStore(ctx sdk.Context, store storetypes.KVStore) error {
	v2distribution.MigratePrefixAddress(store, v1.LastValidatorPowerKey)
	v2distribution.MigratePrefixAddress(store, v1.ValidatorsKey)
	v2distribution.MigratePrefixAddress(store, v1.ValidatorsByConsAddrKey)
	migrateValidatorsByPowerIndexKey(store)

	v2distribution.MigratePrefixAddressAddress(store, v1.DelegationKey)
	v2distribution.MigratePrefixAddressAddress(store, v1.UnbondingDelegationKey)
	v2distribution.MigratePrefixAddressAddress(store, v1.UnbondingDelegationByValIndexKey)
	migratePrefixAddressAddressAddress(store, v1.RedelegationKey)
	migratePrefixAddressAddressAddress(store, v1.RedelegationByValSrcIndexKey)
	migratePrefixAddressAddressAddress(store, v1.RedelegationByValDstIndexKey)

	return nil
}
