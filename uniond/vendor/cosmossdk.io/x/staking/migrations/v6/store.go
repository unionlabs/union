package v6

import (
	"context"

	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
)

// MigrateStore performs in-place store migrations from v5 to v6.
// It deletes the ValidatorUpdatesKey from the store.
func MigrateStore(ctx context.Context, store storetypes.KVStore, cdc codec.BinaryCodec) error {
	store.Delete(ValidatorUpdatesKey)
	store.Delete(HistoricalInfoKey)
	return nil
}
