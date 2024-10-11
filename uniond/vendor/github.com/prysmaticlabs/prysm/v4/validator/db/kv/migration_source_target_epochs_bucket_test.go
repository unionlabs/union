package kv

import (
	"context"
	"errors"
	"fmt"
	"reflect"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	bolt "go.etcd.io/bbolt"
)

func TestStore_migrateSourceTargetEpochsBucketUp(t *testing.T) {
	numEpochs := uint64(100)
	// numKeys should be more than batch size for testing.
	// See: https://github.com/prysmaticlabs/prysm/issues/8509
	numKeys := 2*publicKeyMigrationBatchSize + 1
	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, numKeys)
	for i := 0; i < numKeys; i++ {
		var pk [fieldparams.BLSPubkeyLength]byte
		copy(pk[:], fmt.Sprintf("%d", i))
		pubKeys[i] = pk
	}
	tests := []struct {
		name  string
		setup func(t *testing.T, validatorDB *Store)
		eval  func(t *testing.T, validatorDB *Store)
	}{
		{
			name: "only runs once",
			setup: func(t *testing.T, validatorDB *Store) {
				err := validatorDB.update(func(tx *bolt.Tx) error {
					return tx.Bucket(migrationsBucket).Put(migrationSourceTargetEpochsBucketKey, migrationCompleted)
				})
				require.NoError(t, err)
			},
			eval: func(t *testing.T, validatorDB *Store) {
				err := validatorDB.view(func(tx *bolt.Tx) error {
					data := tx.Bucket(migrationsBucket).Get(migrationSourceTargetEpochsBucketKey)
					require.DeepEqual(t, data, migrationCompleted)
					return nil
				})
				require.NoError(t, err)
			},
		},
		{
			name: "populates new target epochs bucket",
			setup: func(t *testing.T, validatorDB *Store) {
				err := validatorDB.update(func(tx *bolt.Tx) error {
					bucket := tx.Bucket(pubKeysBucket)
					for _, pubKey := range pubKeys {
						pkBucket, err := bucket.CreateBucketIfNotExists(pubKey[:])
						if err != nil {
							return err
						}
						sourceEpochsBucket, err := pkBucket.CreateBucketIfNotExists(attestationSourceEpochsBucket)
						if err != nil {
							return err
						}
						for epoch := uint64(1); epoch < numEpochs; epoch++ {
							source := epoch - 1
							target := epoch
							sourceEpoch := bytesutil.Uint64ToBytesBigEndian(source)
							targetEpoch := bytesutil.Uint64ToBytesBigEndian(target)
							if err := sourceEpochsBucket.Put(sourceEpoch, targetEpoch); err != nil {
								return err
							}
						}
					}
					return nil
				})
				require.NoError(t, err)
			},
			eval: func(t *testing.T, validatorDB *Store) {
				// Verify we indeed have the data for all epochs
				// since genesis to epoch 50 under the new schema.
				err := validatorDB.view(func(tx *bolt.Tx) error {
					bucket := tx.Bucket(pubKeysBucket)
					for _, pubKey := range pubKeys {
						pkBucket := bucket.Bucket(pubKey[:])
						sourceEpochsBucket := pkBucket.Bucket(attestationSourceEpochsBucket)
						targetEpochsBucket := pkBucket.Bucket(attestationTargetEpochsBucket)

						// Verify we have (source epoch, target epoch) pairs.
						for sourceEpoch := uint64(0); sourceEpoch < numEpochs-1; sourceEpoch++ {
							sourceEpochBytes := bytesutil.Uint64ToBytesBigEndian(sourceEpoch)
							targetEpochBytes := sourceEpochsBucket.Get(sourceEpochBytes)
							targetEpoch := bytesutil.BytesToUint64BigEndian(targetEpochBytes)
							require.Equal(t, sourceEpoch+1, targetEpoch)
						}
						// Verify we have (target epoch, source epoch) pairs.
						for targetEpoch := uint64(1); targetEpoch < numEpochs; targetEpoch++ {
							targetEpochBytes := bytesutil.Uint64ToBytesBigEndian(targetEpoch)
							sourceEpochBytes := targetEpochsBucket.Get(targetEpochBytes)
							sourceEpoch := bytesutil.BytesToUint64BigEndian(sourceEpochBytes)
							require.Equal(t, targetEpoch-1, sourceEpoch)
						}
					}
					return nil
				})
				require.NoError(t, err)
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			validatorDB := setupDB(t, pubKeys)
			tt.setup(t, validatorDB)
			require.NoError(t, validatorDB.migrateSourceTargetEpochsBucketUp(context.Background()))
			tt.eval(t, validatorDB)
		})
	}
}

func TestStore_migrateSourceTargetEpochsBucketDown(t *testing.T) {
	// numKeys should be more than batch size for testing.
	// See: https://github.com/prysmaticlabs/prysm/issues/8509
	numKeys := 2*publicKeyMigrationBatchSize + 1
	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, numKeys)
	for i := 0; i < numKeys; i++ {
		var pk [fieldparams.BLSPubkeyLength]byte
		copy(pk[:], fmt.Sprintf("%d", i))
		pubKeys[i] = pk
	}
	tests := []struct {
		name  string
		setup func(t *testing.T, validatorDB *Store)
		eval  func(t *testing.T, validatorDB *Store)
	}{
		{
			name: "unsets the migration completed key upon completion",
			setup: func(t *testing.T, validatorDB *Store) {
				err := validatorDB.update(func(tx *bolt.Tx) error {
					return tx.Bucket(migrationsBucket).Put(migrationSourceTargetEpochsBucketKey, migrationCompleted)
				})
				require.NoError(t, err)
			},
			eval: func(t *testing.T, validatorDB *Store) {
				err := validatorDB.view(func(tx *bolt.Tx) error {
					data := tx.Bucket(migrationsBucket).Get(migrationSourceTargetEpochsBucketKey)
					require.DeepEqual(t, true, data == nil)
					return nil
				})
				require.NoError(t, err)
			},
		},
		{
			name:  "unsets the migration, even if unset already (no panic)",
			setup: func(t *testing.T, validatorDB *Store) {},
			eval: func(t *testing.T, validatorDB *Store) {
				// Ensure the migration is not marked as complete.
				err := validatorDB.view(func(tx *bolt.Tx) error {
					data := tx.Bucket(migrationsBucket).Get(migrationSourceTargetEpochsBucketKey)
					require.DeepNotEqual(t, data, migrationCompleted)
					return nil
				})
				require.NoError(t, err)
			},
		},
		{
			name: "deletes the new bucket that was created in the up migration",
			setup: func(t *testing.T, validatorDB *Store) {
				err := validatorDB.update(func(tx *bolt.Tx) error {
					bucket := tx.Bucket(pubKeysBucket)
					for _, pubKey := range pubKeys {
						pkBucket, err := bucket.CreateBucketIfNotExists(pubKey[:])
						if err != nil {
							return err
						}
						if _, err := pkBucket.CreateBucketIfNotExists(attestationSourceEpochsBucket); err != nil {
							return err
						}
						if _, err := pkBucket.CreateBucketIfNotExists(attestationTargetEpochsBucket); err != nil {
							return err
						}
					}
					return nil
				})
				require.NoError(t, err)
			},
			eval: func(t *testing.T, validatorDB *Store) {
				err := validatorDB.view(func(tx *bolt.Tx) error {
					bucket := tx.Bucket(pubKeysBucket)
					for _, pubKey := range pubKeys {
						pkBucket := bucket.Bucket(pubKey[:])
						if pkBucket == nil {
							return errors.New("expected pubkey bucket to exist")
						}
						targetEpochsBucket := pkBucket.Bucket(attestationTargetEpochsBucket)
						if targetEpochsBucket != nil {
							return errors.New("expected target epochs bucket to have been deleted")
						}
					}
					return nil
				})
				require.NoError(t, err)
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			validatorDB := setupDB(t, nil)
			tt.setup(t, validatorDB)
			require.NoError(t, validatorDB.migrateSourceTargetEpochsBucketDown(context.Background()))
			tt.eval(t, validatorDB)
		})
	}
}

func Test_batchPublicKeys(t *testing.T) {
	tests := []struct {
		name       string
		batchSize  int
		publicKeys [][]byte
		want       [][][]byte
	}{
		{
			name:       "less than batch size returns all keys",
			batchSize:  100,
			publicKeys: [][]byte{{1}, {2}, {3}},
			want:       [][][]byte{{{1}, {2}, {3}}},
		},
		{
			name:       "equals batch size returns all keys",
			batchSize:  3,
			publicKeys: [][]byte{{1}, {2}, {3}},
			want:       [][][]byte{{{1}, {2}, {3}}},
		},
		{
			name:       "> batch size returns proper batches",
			batchSize:  5,
			publicKeys: [][]byte{{1}, {2}, {3}, {4}, {5}, {6}, {7}, {8}},
			want:       [][][]byte{{{1}, {2}, {3}, {4}, {5}}, {{6}, {7}, {8}}},
		},
		{
			name:       "equal size batches returns proper batches",
			batchSize:  2,
			publicKeys: [][]byte{{1}, {2}, {3}, {4}},
			want:       [][][]byte{{{1}, {2}}, {{3}, {4}}},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := batchPublicKeys(tt.publicKeys, tt.batchSize); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("batchPublicKeys() = %v, want %v", got, tt.want)
			}
		})
	}
}
