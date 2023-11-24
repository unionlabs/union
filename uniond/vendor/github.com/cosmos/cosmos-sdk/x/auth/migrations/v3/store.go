package v3

import (
	corestore "cosmossdk.io/core/store"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

func mapAccountAddressToAccountID(ctx sdk.Context, storeService corestore.KVStoreService, cdc codec.BinaryCodec) error {
	store := storeService.OpenKVStore(ctx)
	iterator, err := store.Iterator(types.AddressStoreKeyPrefix, storetypes.PrefixEndBytes(types.AddressStoreKeyPrefix))
	if err != nil {
		return err
	}

	defer iterator.Close()
	for ; iterator.Valid(); iterator.Next() {
		var acc sdk.AccountI
		if err := cdc.UnmarshalInterface(iterator.Value(), &acc); err != nil {
			return err
		}
		store.Set(accountNumberStoreKey(acc.GetAccountNumber()), acc.GetAddress().Bytes())
	}

	return nil
}

// MigrateStore performs in-place store migrations from v0.45 to v0.46. The
// migration includes:
// - Add an Account number as an index to get the account address
func MigrateStore(ctx sdk.Context, storeService corestore.KVStoreService, cdc codec.BinaryCodec) error {
	return mapAccountAddressToAccountID(ctx, storeService, cdc)
}

// accountNumberStoreKey turn an account number to key used to get the account address from account store
// NOTE(tip): exists for legacy compatibility
func accountNumberStoreKey(accountNumber uint64) []byte {
	return append(types.AccountNumberStoreKeyPrefix, sdk.Uint64ToBigEndian(accountNumber)...)
}
