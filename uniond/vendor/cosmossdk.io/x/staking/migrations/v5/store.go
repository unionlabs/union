package v5

import (
	"context"
	"fmt"
	"strconv"

	"cosmossdk.io/core/log"
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
)

func migrateDelegationsByValidatorIndex(store storetypes.KVStore) error {
	iterator := storetypes.KVStorePrefixIterator(store, DelegationKey)

	for ; iterator.Valid(); iterator.Next() {
		key := iterator.Key()
		del, val, err := ParseDelegationKey(key)
		if err != nil {
			return err
		}

		store.Set(GetDelegationsByValKey(val, del), []byte{})
	}

	return nil
}

// MigrateStore performs in-place store migrations from v4 to v5.
func MigrateStore(ctx context.Context, store storetypes.KVStore, cdc codec.BinaryCodec, logger log.Logger) error {
	if err := migrateDelegationsByValidatorIndex(store); err != nil {
		return err
	}
	return migrateHistoricalInfoKeys(store, logger)
}

// migrateHistoricalInfoKeys migrate HistoricalInfo keys to binary format
func migrateHistoricalInfoKeys(store storetypes.KVStore, logger log.Logger) error {
	// old key is of format:
	// prefix (0x50) || heightBytes (string representation of height in 10 base)
	// new key is of format:
	// prefix (0x50) || heightBytes (byte array representation using big-endian byte order)
	oldStore := prefix.NewStore(store, HistoricalInfoKey)

	oldStoreIter := oldStore.Iterator(nil, nil)
	defer logDeferred(logger, func() error { return oldStoreIter.Close() })

	for ; oldStoreIter.Valid(); oldStoreIter.Next() {
		strHeight := oldStoreIter.Key()

		intHeight, err := strconv.ParseInt(string(strHeight), 10, 64)
		if err != nil {
			return fmt.Errorf("can't parse height from key %q to int64: %w", strHeight, err)
		}

		newStoreKey := GetHistoricalInfoKey(intHeight)

		// Set new key on store. Values don't change.
		store.Set(newStoreKey, oldStoreIter.Value())
		oldStore.Delete(oldStoreIter.Key())
	}

	return nil
}

// logDeferred logs an error in a deferred function call if the returned error is non-nil.
func logDeferred(logger log.Logger, f func() error) {
	if err := f(); err != nil {
		logger.Error(err.Error())
	}
}
