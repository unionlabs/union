package kv

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"go.etcd.io/bbolt"
)

func Test_migrateBlockSlotIndex(t *testing.T) {
	tests := []struct {
		name  string
		setup func(t *testing.T, db *bbolt.DB)
		eval  func(t *testing.T, db *bbolt.DB)
	}{
		{
			name: "only runs once",
			setup: func(t *testing.T, db *bbolt.DB) {
				err := db.Update(func(tx *bbolt.Tx) error {
					if err := tx.Bucket(blockSlotIndicesBucket).Put([]byte("2048"), []byte("foo")); err != nil {
						return err
					}
					return tx.Bucket(migrationsBucket).Put(migrationBlockSlotIndex0Key, migrationCompleted)
				})
				assert.NoError(t, err)
			},
			eval: func(t *testing.T, db *bbolt.DB) {
				err := db.View(func(tx *bbolt.Tx) error {
					v := tx.Bucket(blockSlotIndicesBucket).Get([]byte("2048"))
					assert.DeepEqual(t, []byte("foo"), v, "Did not receive correct data for key 2048")
					return nil
				})
				assert.NoError(t, err)
			},
		},
		{
			name: "migrates and deletes entries",
			setup: func(t *testing.T, db *bbolt.DB) {
				err := db.Update(func(tx *bbolt.Tx) error {
					return tx.Bucket(blockSlotIndicesBucket).Put([]byte("2048"), []byte("foo"))
				})
				assert.NoError(t, err)
			},
			eval: func(t *testing.T, db *bbolt.DB) {
				err := db.View(func(tx *bbolt.Tx) error {
					k := uint64(2048)
					v := tx.Bucket(blockSlotIndicesBucket).Get(bytesutil.Uint64ToBytesBigEndian(k))
					assert.DeepEqual(t, []byte("foo"), v, "Did not receive correct data for key %d", k)
					return nil
				})
				assert.NoError(t, err)
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			db := setupDB(t).db
			tt.setup(t, db)
			assert.NoError(t, migrateBlockSlotIndex(context.Background(), db), "migrateBlockSlotIndex(tx) error")
			tt.eval(t, db)
		})
	}
}
