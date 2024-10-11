package kv

import (
	"context"
	"fmt"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	bolt "go.etcd.io/bbolt"
)

func Test_migrateOptimalAttesterProtectionUp(t *testing.T) {
	tests := []struct {
		name  string
		setup func(t *testing.T, validatorDB *Store)
		eval  func(t *testing.T, validatorDB *Store)
	}{
		{
			name: "only runs once",
			setup: func(t *testing.T, validatorDB *Store) {
				err := validatorDB.update(func(tx *bolt.Tx) error {
					return tx.Bucket(migrationsBucket).Put(migrationOptimalAttesterProtectionKey, migrationCompleted)
				})
				require.NoError(t, err)
			},
			eval: func(t *testing.T, validatorDB *Store) {
				err := validatorDB.view(func(tx *bolt.Tx) error {
					data := tx.Bucket(migrationsBucket).Get(migrationOptimalAttesterProtectionKey)
					require.DeepEqual(t, data, migrationCompleted)
					return nil
				})
				require.NoError(t, err)
			},
		},
		{
			name: "populates optimized schema buckets",
			setup: func(t *testing.T, validatorDB *Store) {
				pubKey := [fieldparams.BLSPubkeyLength]byte{1}
				history := newDeprecatedAttestingHistory(0)
				// Attest all epochs from genesis to 50.
				numEpochs := primitives.Epoch(50)
				for i := primitives.Epoch(1); i <= numEpochs; i++ {
					var sr [32]byte
					copy(sr[:], fmt.Sprintf("%d", i))
					newHist, err := history.setTargetData(i, &deprecatedHistoryData{
						Source:      i - 1,
						SigningRoot: sr[:],
					})
					require.NoError(t, err)
					history = newHist
				}
				newHist, err := history.setLatestEpochWritten(numEpochs)
				require.NoError(t, err)

				err = validatorDB.update(func(tx *bolt.Tx) error {
					bucket := tx.Bucket(deprecatedAttestationHistoryBucket)
					return bucket.Put(pubKey[:], newHist)
				})
				require.NoError(t, err)
			},
			eval: func(t *testing.T, validatorDB *Store) {
				// Verify we indeed have the data for all epochs
				// since genesis to epoch 50 under the new schema.
				err := validatorDB.view(func(tx *bolt.Tx) error {
					pubKey := [fieldparams.BLSPubkeyLength]byte{1}
					bucket := tx.Bucket(pubKeysBucket)
					pkBucket := bucket.Bucket(pubKey[:])
					signingRootsBucket := pkBucket.Bucket(attestationSigningRootsBucket)
					sourceEpochsBucket := pkBucket.Bucket(attestationSourceEpochsBucket)
					numEpochs := uint64(50)

					// Verify we have signing roots for target epochs 1 to 50 correctly.
					for targetEpoch := uint64(1); targetEpoch <= numEpochs; targetEpoch++ {
						var sr [32]byte
						copy(sr[:], fmt.Sprintf("%d", targetEpoch))
						targetEpochBytes := bytesutil.Uint64ToBytesBigEndian(targetEpoch)
						migratedSigningRoot := signingRootsBucket.Get(targetEpochBytes)
						require.DeepEqual(t, sr[:], migratedSigningRoot)
					}

					// Verify we have (source epoch, target epoch) pairs for epochs 0 to 50 correctly.
					for sourceEpoch := uint64(0); sourceEpoch < numEpochs; sourceEpoch++ {
						sourceEpochBytes := bytesutil.Uint64ToBytesBigEndian(sourceEpoch)
						targetEpochBytes := sourceEpochsBucket.Get(sourceEpochBytes)
						targetEpoch := bytesutil.BytesToUint64BigEndian(targetEpochBytes)
						require.Equal(t, sourceEpoch+1, targetEpoch)
					}
					return nil
				})
				require.NoError(t, err)
			},
		},
		{
			name: "partial data saved for both types still completes the migration successfully",
			setup: func(t *testing.T, validatorDB *Store) {
				ctx := context.Background()
				pubKey := [fieldparams.BLSPubkeyLength]byte{1}
				history := newDeprecatedAttestingHistory(0)
				// Attest all epochs from genesis to 50.
				numEpochs := primitives.Epoch(50)
				for i := primitives.Epoch(1); i <= numEpochs; i++ {
					var sr [32]byte
					copy(sr[:], fmt.Sprintf("%d", i))
					newHist, err := history.setTargetData(i, &deprecatedHistoryData{
						Source:      i - 1,
						SigningRoot: sr[:],
					})
					require.NoError(t, err)
					history = newHist
				}
				newHist, err := history.setLatestEpochWritten(numEpochs)
				require.NoError(t, err)

				err = validatorDB.update(func(tx *bolt.Tx) error {
					bucket := tx.Bucket(deprecatedAttestationHistoryBucket)
					return bucket.Put(pubKey[:], newHist)
				})
				require.NoError(t, err)

				// Run the migration.
				require.NoError(t, validatorDB.migrateOptimalAttesterProtectionUp(ctx))

				// Then delete the migration completed key.
				err = validatorDB.update(func(tx *bolt.Tx) error {
					mb := tx.Bucket(migrationsBucket)
					return mb.Delete(migrationOptimalAttesterProtectionKey)
				})
				require.NoError(t, err)

				// Write one more entry to the DB with the old format.
				var sr [32]byte
				copy(sr[:], fmt.Sprintf("%d", numEpochs+1))
				newHist, err = newHist.setTargetData(numEpochs+1, &deprecatedHistoryData{
					Source:      numEpochs,
					SigningRoot: sr[:],
				})
				require.NoError(t, err)
				newHist, err = newHist.setLatestEpochWritten(numEpochs + 1)
				require.NoError(t, err)

				err = validatorDB.update(func(tx *bolt.Tx) error {
					bucket := tx.Bucket(deprecatedAttestationHistoryBucket)
					return bucket.Put(pubKey[:], newHist)
				})
				require.NoError(t, err)
			},
			eval: func(t *testing.T, validatorDB *Store) {
				// Verify we indeed have the data for all epochs
				// since genesis to epoch 50+1 under the new schema.
				err := validatorDB.view(func(tx *bolt.Tx) error {
					pubKey := [fieldparams.BLSPubkeyLength]byte{1}
					bucket := tx.Bucket(pubKeysBucket)
					pkBucket := bucket.Bucket(pubKey[:])
					signingRootsBucket := pkBucket.Bucket(attestationSigningRootsBucket)
					sourceEpochsBucket := pkBucket.Bucket(attestationSourceEpochsBucket)
					numEpochs := uint64(50)

					// Verify we have signing roots for target epochs 1 to 50 correctly.
					for targetEpoch := uint64(1); targetEpoch <= numEpochs+1; targetEpoch++ {
						var sr [32]byte
						copy(sr[:], fmt.Sprintf("%d", targetEpoch))
						targetEpochBytes := bytesutil.Uint64ToBytesBigEndian(targetEpoch)
						migratedSigningRoot := signingRootsBucket.Get(targetEpochBytes)
						require.DeepEqual(t, sr[:], migratedSigningRoot)
					}

					// Verify we have (source epoch, target epoch) pairs for epochs 0 to 50 correctly.
					for sourceEpoch := uint64(0); sourceEpoch < numEpochs+1; sourceEpoch++ {
						sourceEpochBytes := bytesutil.Uint64ToBytesBigEndian(sourceEpoch)
						targetEpochBytes := sourceEpochsBucket.Get(sourceEpochBytes)
						targetEpoch := bytesutil.BytesToUint64BigEndian(targetEpochBytes)
						require.Equal(t, sourceEpoch+1, targetEpoch)
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
			require.NoError(t, validatorDB.migrateOptimalAttesterProtectionUp(context.Background()))
			tt.eval(t, validatorDB)
		})
	}
}

func Test_migrateOptimalAttesterProtectionDown(t *testing.T) {
	tests := []struct {
		name  string
		setup func(t *testing.T, validatorDB *Store)
		eval  func(t *testing.T, validatorDB *Store)
	}{
		{
			name: "unsets the migration completed key upon completion",
			setup: func(t *testing.T, validatorDB *Store) {
				err := validatorDB.update(func(tx *bolt.Tx) error {
					return tx.Bucket(migrationsBucket).Put(migrationOptimalAttesterProtectionKey, migrationCompleted)
				})
				require.NoError(t, err)
			},
			eval: func(t *testing.T, validatorDB *Store) {
				err := validatorDB.view(func(tx *bolt.Tx) error {
					data := tx.Bucket(migrationsBucket).Get(migrationOptimalAttesterProtectionKey)
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
					data := tx.Bucket(migrationsBucket).Get(migrationOptimalAttesterProtectionKey)
					require.DeepNotEqual(t, data, migrationCompleted)
					return nil
				})
				require.NoError(t, err)
			},
		},
		{
			name: "populates old format from data using the new schema",
			setup: func(t *testing.T, validatorDB *Store) {
				pubKeys := [][fieldparams.BLSPubkeyLength]byte{{1}, {2}}
				// Create attesting history for two public keys
				err := validatorDB.update(func(tx *bolt.Tx) error {
					bkt := tx.Bucket(pubKeysBucket)
					for _, pubKey := range pubKeys {
						pkBucket, err := bkt.CreateBucketIfNotExists(pubKey[:])
						if err != nil {
							return err
						}
						sourceEpochsBucket, err := pkBucket.CreateBucketIfNotExists(attestationSourceEpochsBucket)
						if err != nil {
							return err
						}
						signingRootsBucket, err := pkBucket.CreateBucketIfNotExists(attestationSigningRootsBucket)
						if err != nil {
							return err
						}
						// The highest epoch we write is 50.
						highestEpoch := uint64(50)
						for i := uint64(1); i <= highestEpoch; i++ {
							source := bytesutil.Uint64ToBytesBigEndian(i - 1)
							target := bytesutil.Uint64ToBytesBigEndian(i)
							if err := sourceEpochsBucket.Put(source, target); err != nil {
								return err
							}
							var signingRoot [32]byte
							copy(signingRoot[:], fmt.Sprintf("%d", target))
							if err := signingRootsBucket.Put(target, signingRoot[:]); err != nil {
								return err
							}
						}
					}
					// Finally, we mark the migration as completed to show that we have the
					// new, optimized format for attester protection in the database.
					migrationBkt := tx.Bucket(migrationsBucket)
					return migrationBkt.Put(migrationOptimalAttesterProtectionKey, migrationCompleted)
				})
				require.NoError(t, err)
			},
			eval: func(t *testing.T, validatorDB *Store) {
				pubKeys := [][fieldparams.BLSPubkeyLength]byte{{1}, {2}}
				// Next up, we validate that we have indeed rolled back our data
				// into the old format for attesting history.
				err := validatorDB.view(func(tx *bolt.Tx) error {
					bkt := tx.Bucket(deprecatedAttestationHistoryBucket)
					for _, pubKey := range pubKeys {
						encodedHistoryBytes := bkt.Get(pubKey[:])
						require.NotNil(t, encodedHistoryBytes)
						attestingHistory := deprecatedEncodedAttestingHistory(encodedHistoryBytes)
						highestEpoch, err := attestingHistory.getLatestEpochWritten()
						require.NoError(t, err)
						// Verify the highest epoch written is 50 from the setup stage.
						require.Equal(t, primitives.Epoch(50), highestEpoch)
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
			require.NoError(t, validatorDB.migrateOptimalAttesterProtectionDown(context.Background()))
			tt.eval(t, validatorDB)
		})
	}
}
