package keeper

import (
	"encoding/binary"

	storetypes "cosmossdk.io/core/store"
	"cosmossdk.io/store/prefix"
	"cosmossdk.io/x/upgrade/types"

	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper *Keeper
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper *Keeper) Migrator {
	return Migrator{keeper: keeper}
}

// Migrate1to2 migrates from version 1 to 2.
func (m Migrator) Migrate1to2(ctx sdk.Context) error {
	return migrateDoneUpgradeKeys(ctx, m.keeper.storeService)
}

func migrateDoneUpgradeKeys(ctx sdk.Context, storeService storetypes.KVStoreService) error {
	store := storeService.OpenKVStore(ctx)
	oldDoneStore := prefix.NewStore(runtime.KVStoreAdapter(store), []byte{types.DoneByte})
	oldDoneStoreIter := oldDoneStore.Iterator(nil, nil)
	defer oldDoneStoreIter.Close()

	for ; oldDoneStoreIter.Valid(); oldDoneStoreIter.Next() {
		oldKey := oldDoneStoreIter.Key()
		upgradeName := string(oldKey)
		upgradeHeight := int64(binary.BigEndian.Uint64(oldDoneStoreIter.Value()))
		newKey := encodeDoneKey(upgradeName, upgradeHeight)

		err := store.Set(newKey, []byte{1})
		if err != nil {
			return err
		}

		oldDoneStore.Delete(oldKey)
	}
	return nil
}
