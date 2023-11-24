package v3

import (
	"cosmossdk.io/core/store"
	"cosmossdk.io/log"
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	v2 "github.com/cosmos/cosmos-sdk/x/bank/migrations/v2"
	"github.com/cosmos/cosmos-sdk/x/bank/types"
)

// MigrateStore performs in-place store migrations from v0.43 to v0.45. The
// migration includes:
//
// - Migrate coin storage to save only amount.
// - Add an additional reverse index from denomination to address.
// - Remove duplicate denom from denom metadata store key.
func MigrateStore(ctx sdk.Context, storeService store.KVStoreService, cdc codec.BinaryCodec) error {
	store := runtime.KVStoreAdapter(storeService.OpenKVStore(ctx))
	err := addDenomReverseIndex(store, cdc, ctx.Logger())
	if err != nil {
		return err
	}

	return migrateDenomMetadata(store, ctx.Logger())
}

func addDenomReverseIndex(store storetypes.KVStore, cdc codec.BinaryCodec, logger log.Logger) error {
	oldBalancesStore := prefix.NewStore(store, v2.BalancesPrefix)

	oldBalancesIter := oldBalancesStore.Iterator(nil, nil)
	defer sdk.LogDeferred(logger, func() error { return oldBalancesIter.Close() })

	denomPrefixStores := make(map[string]prefix.Store) // memoize prefix stores

	for ; oldBalancesIter.Valid(); oldBalancesIter.Next() {
		var balance sdk.Coin
		if err := cdc.Unmarshal(oldBalancesIter.Value(), &balance); err != nil {
			return err
		}

		addr, err := v2.AddressFromBalancesStore(oldBalancesIter.Key())
		if err != nil {
			return err
		}

		var coin sdk.DecCoin
		if err := cdc.Unmarshal(oldBalancesIter.Value(), &coin); err != nil {
			return err
		}

		bz, err := coin.Amount.Marshal()
		if err != nil {
			return err
		}

		newStore := prefix.NewStore(store, CreateAccountBalancesPrefix(addr))
		newStore.Set([]byte(coin.Denom), bz)

		denomPrefixStore, ok := denomPrefixStores[balance.Denom]
		if !ok {
			denomPrefixStore = prefix.NewStore(store, CreateDenomAddressPrefix(balance.Denom))
			denomPrefixStores[balance.Denom] = denomPrefixStore
		}

		// Store a reverse index from denomination to account address with a
		// sentinel value.
		denomPrefixStore.Set(address.MustLengthPrefix(addr), []byte{0})
	}

	return nil
}

func migrateDenomMetadata(store storetypes.KVStore, logger log.Logger) error {
	oldDenomMetaDataStore := prefix.NewStore(store, v2.DenomMetadataPrefix)

	oldDenomMetaDataIter := oldDenomMetaDataStore.Iterator(nil, nil)
	defer sdk.LogDeferred(logger, func() error { return oldDenomMetaDataIter.Close() })

	for ; oldDenomMetaDataIter.Valid(); oldDenomMetaDataIter.Next() {
		oldKey := oldDenomMetaDataIter.Key()
		l := len(oldKey) / 2

		newKey := make([]byte, len(types.DenomMetadataPrefix)+l)
		// old key: prefix_bytes | denom_bytes | denom_bytes
		copy(newKey, types.DenomMetadataPrefix)
		copy(newKey[len(types.DenomMetadataPrefix):], oldKey[:l])
		store.Set(newKey, oldDenomMetaDataIter.Value())
		oldDenomMetaDataStore.Delete(oldKey)
	}

	return nil
}

// CreateAccountBalancesPrefix creates the prefix for an account's balances.
func CreateAccountBalancesPrefix(addr []byte) []byte {
	return append(types.BalancesPrefix.Bytes(), address.MustLengthPrefix(addr)...)
}
