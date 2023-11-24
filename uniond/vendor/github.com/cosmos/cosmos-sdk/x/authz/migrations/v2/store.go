package v2

import (
	"context"

	corestoretypes "cosmossdk.io/core/store"
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/internal/conv"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/authz"
)

// MigrateStore performs in-place store migrations from v0.45 to v0.46. The
// migration includes:
//
// - pruning expired authorizations
// - create secondary index for pruning expired authorizations
func MigrateStore(ctx context.Context, storeService corestoretypes.KVStoreService, cdc codec.BinaryCodec) error {
	store := storeService.OpenKVStore(ctx)
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	err := addExpiredGrantsIndex(sdkCtx, runtime.KVStoreAdapter(store), cdc)
	if err != nil {
		return err
	}

	return nil
}

func addExpiredGrantsIndex(ctx sdk.Context, store storetypes.KVStore, cdc codec.BinaryCodec) error {
	grantsStore := prefix.NewStore(store, GrantPrefix)

	grantsIter := grantsStore.Iterator(nil, nil)
	defer grantsIter.Close()

	queueItems := make(map[string][]string)
	now := ctx.BlockTime()
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
