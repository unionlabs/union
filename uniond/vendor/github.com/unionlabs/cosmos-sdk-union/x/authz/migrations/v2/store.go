package v2

import (
	"context"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/store/prefix"
	"cosmossdk.io/x/authz"
	"cosmossdk.io/x/authz/internal/conv"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
)

// MigrateStore performs in-place store migrations from v0.45 to v0.46. The
// migration includes:
//
// - pruning expired authorizations
// - create secondary index for pruning expired authorizations
func MigrateStore(ctx context.Context, env appmodule.Environment, cdc codec.BinaryCodec) error {
	err := addExpiredGrantsIndex(ctx, env, cdc)
	if err != nil {
		return err
	}

	return nil
}

func addExpiredGrantsIndex(ctx context.Context, env appmodule.Environment, cdc codec.BinaryCodec) error {
	store := runtime.KVStoreAdapter(env.KVStoreService.OpenKVStore(ctx))
	grantsStore := prefix.NewStore(store, GrantPrefix)

	grantsIter := grantsStore.Iterator(nil, nil)
	defer grantsIter.Close()

	queueItems := make(map[string][]string)
	now := env.HeaderService.HeaderInfo(ctx).Time
	for ; grantsIter.Valid(); grantsIter.Next() {
		var grant authz.Grant
		bz := grantsIter.Value()
		if err := cdc.Unmarshal(bz, &grant); err != nil {
			return err
		}

		// delete expired authorization
		// before 0.46 Expiration was required so it's safe to dereference
		if grant.Expiration.Before(now) {
			grantsStore.Delete(grantsIter.Key())
		} else {
			granter, grantee, msgType := ParseGrantKey(grantsIter.Key())
			// before 0.46 expiration was not a pointer, so now it's safe to dereference
			key := GrantQueueKey(*grant.Expiration, granter, grantee)

			queueItem, ok := queueItems[conv.UnsafeBytesToStr(key)]
			if !ok {
				queueItems[string(key)] = []string{msgType}
			} else {
				queueItem = append(queueItem, msgType)
				queueItems[string(key)] = queueItem
			}
		}
	}

	for key, v := range queueItems {
		bz, err := cdc.Marshal(&authz.GrantQueueItem{
			MsgTypeUrls: v,
		})
		if err != nil {
			return err
		}
		store.Set(conv.UnsafeStrToBytes(key), bz)
	}

	return nil
}
