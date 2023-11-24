package v2

import (
	"cosmossdk.io/core/store"
	"cosmossdk.io/log"
	"cosmossdk.io/math"
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
	v1auth "github.com/cosmos/cosmos-sdk/x/auth/migrations/v1"
	v1 "github.com/cosmos/cosmos-sdk/x/bank/migrations/v1"
	"github.com/cosmos/cosmos-sdk/x/bank/types"
)

// migrateSupply migrates the supply to be stored by denom key instead in a
// single blob.
// ref: https://github.com/cosmos/cosmos-sdk/issues/7092
func migrateSupply(store storetypes.KVStore, cdc codec.BinaryCodec) error {
	// Old supply was stored as a single blob under the SupplyKey.
	var oldSupplyI v1.SupplyI
	err := cdc.UnmarshalInterface(store.Get(v1.SupplyKey), &oldSupplyI)
	if err != nil {
		return err
	}

	// We delete the single key holding the whole blob.
	store.Delete(v1.SupplyKey)

	if oldSupplyI == nil {
		return nil
	}

	// We add a new key for each denom
	supplyStore := prefix.NewStore(store, SupplyKey)

	// We're sure that SupplyI is a Supply struct, there's no other
	// implementation.
	oldSupply := oldSupplyI.(*types.Supply)
	for i := range oldSupply.Total {
		coin := oldSupply.Total[i]
		coinBz, err := coin.Amount.Marshal()
		if err != nil {
			return err
		}

		supplyStore.Set([]byte(coin.Denom), coinBz)
	}

	return nil
}

// migrateBalanceKeys migrate the balances keys to cater for variable-length
// addresses.
func migrateBalanceKeys(store storetypes.KVStore, logger log.Logger) {
	// old key is of format:
	// prefix ("balances") || addrBytes (20 bytes) || denomBytes
	// new key is of format
	// prefix (0x02) || addrLen (1 byte) || addrBytes || denomBytes
	oldStore := prefix.NewStore(store, v1.BalancesPrefix)

	oldStoreIter := oldStore.Iterator(nil, nil)
	defer sdk.LogDeferred(logger, func() error { return oldStoreIter.Close() })

	for ; oldStoreIter.Valid(); oldStoreIter.Next() {
		addr := v1.AddressFromBalancesStore(oldStoreIter.Key())
		denom := oldStoreIter.Key()[v1auth.AddrLen:]
		newStoreKey := CreatePrefixedAccountStoreKey(addr, denom)

		// Set new key on store. Values don't change.
		store.Set(newStoreKey, oldStoreIter.Value())
		oldStore.Delete(oldStoreIter.Key())
	}
}

// MigrateStore performs in-place store migrations from v0.40 to v0.43. The
// migration includes:
//
// - Change addresses to be length-prefixed.
// - Change balances prefix to 1 byte
// - Change supply to be indexed by denom
// - Prune balances & supply with zero coins (ref: https://github.com/cosmos/cosmos-sdk/pull/9229)
func MigrateStore(ctx sdk.Context, storeService store.KVStoreService, cdc codec.BinaryCodec) error {
	store := runtime.KVStoreAdapter(storeService.OpenKVStore(ctx))
	migrateBalanceKeys(store, ctx.Logger())

	if err := pruneZeroBalances(store, cdc); err != nil {
		return err
	}

	if err := migrateSupply(store, cdc); err != nil {
		return err
	}

	return pruneZeroSupply(store)
}

// pruneZeroBalances removes the zero balance addresses from balances store.
func pruneZeroBalances(store storetypes.KVStore, cdc codec.BinaryCodec) error {
	balancesStore := prefix.NewStore(store, BalancesPrefix)
	iterator := balancesStore.Iterator(nil, nil)
	defer iterator.Close()

	for ; iterator.Valid(); iterator.Next() {
		var balance sdk.Coin
		if err := cdc.Unmarshal(iterator.Value(), &balance); err != nil {
			return err
		}

		if balance.IsZero() {
			balancesStore.Delete(iterator.Key())
		}
	}
	return nil
}

// pruneZeroSupply removes zero balance denom from supply store.
func pruneZeroSupply(store storetypes.KVStore) error {
	supplyStore := prefix.NewStore(store, SupplyKey)
	iterator := supplyStore.Iterator(nil, nil)
	defer iterator.Close()

	for ; iterator.Valid(); iterator.Next() {
		var amount math.Int
		if err := amount.Unmarshal(iterator.Value()); err != nil {
			return err
		}

		if amount.IsZero() {
			supplyStore.Delete(iterator.Key())
		}
	}

	return nil
}

// CreatePrefixedAccountStoreKey returns the key for the given account and denomination.
// This method can be used when performing an ABCI query for the balance of an account.
func CreatePrefixedAccountStoreKey(addr, denom []byte) []byte {
	return append(CreateAccountBalancesPrefix(addr), denom...)
}
