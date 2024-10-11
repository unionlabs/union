package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper Keeper
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper Keeper) Migrator {
	return Migrator{
		keeper: keeper,
	}
}

// MigrateChecksums migrates the wasm store from using a single key to
// store a list of checksums to using a collections.KeySet to store the checksums.
//
// It grabs the checksums stored previously under the old key and stores
// them in the global KeySet collection. It then deletes the old key and
// the checksums stored under it.
func (m Migrator) MigrateChecksums(ctx sdk.Context) error {
	checksums, err := m.getStoredChecksums(ctx)
	if err != nil {
		return err
	}

	for _, hash := range checksums {
		if err := ibcwasm.Checksums.Set(ctx, hash); err != nil {
			return err
		}
	}

	// delete the previously stored checksums
	if err := m.deleteChecksums(ctx); err != nil {
		return err
	}

	types.Logger(ctx).Info("successfully migrated Checksums to collections")
	return nil
}

// getStoredChecksums returns the checksums stored under the KeyChecksums key.
func (m Migrator) getStoredChecksums(ctx sdk.Context) ([][]byte, error) {
	store := m.keeper.storeService.OpenKVStore(ctx)

	bz, err := store.Get([]byte(types.KeyChecksums))
	if err != nil {
		return [][]byte{}, err
	}

	var hashes types.Checksums
	err = m.keeper.cdc.Unmarshal(bz, &hashes)
	if err != nil {
		return [][]byte{}, err
	}

	return hashes.Checksums, nil
}

// deleteChecksums deletes the checksums stored under the KeyChecksums key.
func (m Migrator) deleteChecksums(ctx sdk.Context) error {
	store := m.keeper.storeService.OpenKVStore(ctx)
	err := store.Delete([]byte(types.KeyChecksums))

	return err
}
