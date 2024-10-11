package v4

import (
	"context"
	"errors"

	gogotypes "github.com/cosmos/gogoproto/types"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/store"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var OldProposerKey = []byte{0x01}

// MigrateStore removes the last proposer from store.
func MigrateStore(ctx context.Context, env appmodule.Environment, _ codec.BinaryCodec) error {
	kvStore := env.KVStoreService.OpenKVStore(ctx)
	return kvStore.Delete(OldProposerKey)
}

// GetPreviousProposerConsAddr returns the proposer consensus address for the
// current block.
func GetPreviousProposerConsAddr(ctx context.Context, storeService store.KVStoreService, cdc codec.BinaryCodec) (sdk.ConsAddress, error) {
	kvStore := storeService.OpenKVStore(ctx)
	bz, err := kvStore.Get(OldProposerKey)
	if err != nil {
		return nil, err
	}

	if bz == nil {
		return nil, errors.New("previous proposer not set")
	}

	addrValue := gogotypes.BytesValue{}
	err = cdc.Unmarshal(bz, &addrValue)
	if err != nil {
		return nil, err
	}

	return addrValue.GetValue(), nil
}

// SetPreviousProposerConsAddr set the proposer public key for this block.
func SetPreviousProposerConsAddr(ctx context.Context, storeService store.KVStoreService, cdc codec.BinaryCodec, consAddr sdk.ConsAddress) error {
	kvStore := storeService.OpenKVStore(ctx)
	bz := cdc.MustMarshal(&gogotypes.BytesValue{Value: consAddr})
	return kvStore.Set(OldProposerKey, bz)
}
