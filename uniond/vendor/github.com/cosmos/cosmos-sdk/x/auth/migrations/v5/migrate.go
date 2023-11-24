package v5

import (
	"context"

	"github.com/cosmos/gogoproto/types"

	"cosmossdk.io/collections"
	storetypes "cosmossdk.io/core/store"
)

var LegacyGlobalAccountNumberKey = []byte("globalAccountNumber")

func Migrate(ctx context.Context, storeService storetypes.KVStoreService, sequence collections.Sequence) error {
	store := storeService.OpenKVStore(ctx)
	b, err := store.Get(LegacyGlobalAccountNumberKey)
	if err != nil {
		return err
	}
	if b == nil {
		// this would mean no account was ever created in this chain which is being migrated?
		// we're doing nothing as the collections.Sequence already handles the non-existing value.
		return nil
	}

	// get old value
	v := new(types.UInt64Value)
	err = v.Unmarshal(b)
	if err != nil {
		return err
	}

	// set the old value in the collection
	err = sequence.Set(ctx, v.Value)
	if err != nil {
		return err
	}

	// remove the value from the old prefix.
	err = store.Delete(LegacyGlobalAccountNumberKey)
	if err != nil {
		return err
	}

	return nil
}
