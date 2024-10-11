package v2

import (
	"context"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/store"
	"cosmossdk.io/store/prefix"
	"cosmossdk.io/x/feegrant"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
)

func addAllowancesByExpTimeQueue(ctx context.Context, env appmodule.Environment, store store.KVStore, cdc codec.BinaryCodec) error {
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(store), FeeAllowanceKeyPrefix)
	iterator := prefixStore.Iterator(nil, nil)
	defer iterator.Close()

	for ; iterator.Valid(); iterator.Next() {
		var grant feegrant.Grant
		if err := cdc.Unmarshal(iterator.Value(), &grant); err != nil {
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
			if exp.Before(env.HeaderService.HeaderInfo(ctx).Time) {
				prefixStore.Delete(key)
			} else {
				grantByExpTimeQueueKey := FeeAllowancePrefixQueue(exp, key)
				if err := store.Set(grantByExpTimeQueueKey, []byte{}); err != nil {
					return err
				}
			}
		}
	}

	return nil
}

func MigrateStore(ctx context.Context, env appmodule.Environment, cdc codec.BinaryCodec) error {
	store := env.KVStoreService.OpenKVStore(ctx)
	return addAllowancesByExpTimeQueue(ctx, env, store, cdc)
}
