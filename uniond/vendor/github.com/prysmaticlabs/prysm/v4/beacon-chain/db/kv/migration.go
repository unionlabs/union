package kv

import (
	"context"

	bolt "go.etcd.io/bbolt"
)

var migrationCompleted = []byte("done")

type migration func(context.Context, *bolt.DB) error

var migrations = []migration{
	migrateArchivedIndex,
	migrateBlockSlotIndex,
	migrateStateValidators,
}

// RunMigrations defined in the migrations array.
func (s *Store) RunMigrations(ctx context.Context) error {
	for _, m := range migrations {
		if err := m(ctx, s.db); err != nil {
			return err
		}
	}
	return nil
}
