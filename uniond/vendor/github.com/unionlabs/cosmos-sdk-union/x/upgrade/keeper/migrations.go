package keeper

import (
	"context"
	"encoding/binary"
	"errors"
	"fmt"

	storetypes "cosmossdk.io/core/store"
	"cosmossdk.io/store/prefix"
	"cosmossdk.io/x/upgrade/types"

	"github.com/cosmos/cosmos-sdk/runtime"
)

const (
	// LegacyProtocolVersionByte was the prefix to look up Protocol Version (AppVersion)
	LegacyProtocolVersionByte = 0x3
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
func (m Migrator) Migrate1to2(ctx context.Context) error {
	return migrateDoneUpgradeKeys(ctx, m.keeper.KVStoreService)
}

func migrateDoneUpgradeKeys(ctx context.Context, storeService storetypes.KVStoreService) error {
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

// Migrate2to3 migrates from version 2 to 3.
// It takes the legacy protocol version and if it exists, uses it to set
// the app version (of the baseapp)
func (m Migrator) Migrate2to3(ctx context.Context) error {
	return migrateAppVersion(ctx, m.keeper)
}

func migrateAppVersion(ctx context.Context, keeper *Keeper) error {
	if keeper.versionModifier == nil {
		return errors.New("version modifier is not set")
	}

	store := keeper.KVStoreService.OpenKVStore(ctx)
	// if the key was never set then we don't need to migrate anything
	exists, err := store.Has([]byte{LegacyProtocolVersionByte})
	if err != nil {
		return fmt.Errorf("error checking if legacy protocol version key exists: %w", err)
	}
	if !exists {
		return nil
	}

	versionBytes, err := store.Get([]byte{LegacyProtocolVersionByte})
	if err != nil {
		return fmt.Errorf("error getting legacy protocol version: %w", err)
	}
	appVersion := binary.BigEndian.Uint64(versionBytes)

	if err := keeper.versionModifier.SetAppVersion(ctx, appVersion); err != nil {
		return fmt.Errorf("error migration app version: %w", err)
	}

	return store.Delete([]byte{LegacyProtocolVersionByte})
}
