package v2

import (
	"context"

	"cosmossdk.io/core/store"
	"cosmossdk.io/store/prefix"
	"cosmossdk.io/x/feegrant"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/types"
)

func addAllowancesByExpTimeQueue(ctx context.Context, store store.KVStore, cdc codec.BinaryCodec) error {
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(store), FeeAllowanceKeyPrefix)
	iterator := prefixStore.Iterator(nil, nil)
	defer iterator.Close()

	for ; iterator.Valid(); iterator.Next() {
		var grant feegrant.Grant
		bz := iterator.Value()
		if err := cdc.Unmarshal(bz, &grant); err != nil {
			return err
		}

		grantInfo, err := grant.GetGrant()
		if err != nil {
			return err
		}

		exp, err := grantInfo.ExpiresAt()
		if err != nil {
			return err
		}

		if exp != nil {
			// store key is not changed in 0.46
			key := iterator.Key()
			if exp.Before(types.UnwrapSDKContext(ctx).BlockTime()) {
				prefixStore.Delete(key)
			} else {
				grantByExpTimeQueueKey := FeeAllowancePrefixQueue(exp, key)
				store.Set(grantByExpTimeQueueKey, []byte{})
			}
		}
	}

	return nil
}

func MigrateStore(ctx context.Context, storeService store.KVStoreService, cdc codec.BinaryCodec) error {
	store := storeService.OpenKVStore(ctx)
	return addAllowancesByExpTimeQueue(ctx, store, cdc)
}
