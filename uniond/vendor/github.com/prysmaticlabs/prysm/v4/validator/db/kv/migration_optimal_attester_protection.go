package kv

import (
	"bytes"
	"context"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/progress"
	bolt "go.etcd.io/bbolt"
)

var migrationOptimalAttesterProtectionKey = []byte("optimal_attester_protection_0")

// Migrate attester protection to a more optimal format in the DB. Given we
// stored attesting history as large, 2Mb arrays per validator, we need to perform
// this migration differently than the rest, ensuring we perform each expensive bolt
// update in its own transaction to prevent having everything on the heap.
func (s *Store) migrateOptimalAttesterProtectionUp(_ context.Context) error {
	publicKeyBytes := make([][]byte, 0)
	attestingHistoryBytes := make([][]byte, 0)
	numKeys := 0
	err := s.db.Update(func(tx *bolt.Tx) error {
		mb := tx.Bucket(migrationsBucket)
		if b := mb.Get(migrationOptimalAttesterProtectionKey); bytes.Equal(b, migrationCompleted) {
			return nil // Migration already completed.
		}

		bkt := tx.Bucket(deprecatedAttestationHistoryBucket)
		numKeys = bkt.Stats().KeyN
		return bkt.ForEach(func(k, v []byte) error {
			if v == nil {
				return nil
			}
			bucket := tx.Bucket(pubKeysBucket)
			pkBucket, err := bucket.CreateBucketIfNotExists(k)
			if err != nil {
				return err
			}
			_, err = pkBucket.CreateBucketIfNotExists(attestationSourceEpochsBucket)
			if err != nil {
				return err
			}
			_, err = pkBucket.CreateBucketIfNotExists(attestationSigningRootsBucket)
			if err != nil {
				return err
			}
			nk := make([]byte, len(k))
			copy(nk, k)
			nv := make([]byte, len(v))
			copy(nv, v)
			publicKeyBytes = append(publicKeyBytes, nk)
			attestingHistoryBytes = append(attestingHistoryBytes, nv)
			return nil
		})
	})
	if err != nil {
		return err
	}

	bar := progress.InitializeProgressBar(numKeys, "Migrating attesting history to more efficient format")
	for i, publicKey := range publicKeyBytes {
		attestingHistory := deprecatedEncodedAttestingHistory(attestingHistoryBytes[i])
		err = s.db.Update(func(tx *bolt.Tx) error {
			if attestingHistory == nil {
				return nil
			}
			bucket := tx.Bucket(pubKeysBucket)
			pkBucket := bucket.Bucket(publicKey)
			sourceEpochsBucket := pkBucket.Bucket(attestationSourceEpochsBucket)

			signingRootsBucket := pkBucket.Bucket(attestationSigningRootsBucket)

			// Extract every single source, target, signing root
			// from the attesting history then insert them into the
			// respective buckets under the new db schema.
			latestEpochWritten, err := attestingHistory.getLatestEpochWritten()
			if err != nil {
				return err
			}
			// For every epoch since genesis up to the highest epoch written, we then
			// extract historical data and insert it into the new schema.
			for targetEpoch := primitives.Epoch(0); targetEpoch <= latestEpochWritten; targetEpoch++ {
				historicalAtt, err := attestingHistory.getTargetData(targetEpoch)
				if err != nil {
					return err
				}
				if historicalAtt.isEmpty() {
					continue
				}
				targetEpochBytes := bytesutil.EpochToBytesBigEndian(targetEpoch)
				sourceEpochBytes := bytesutil.EpochToBytesBigEndian(historicalAtt.Source)
				if err := sourceEpochsBucket.Put(sourceEpochBytes, targetEpochBytes); err != nil {
					return err
				}
				if err := signingRootsBucket.Put(targetEpochBytes, historicalAtt.SigningRoot); err != nil {
					return err
				}
			}
			return bar.Add(1)
		})
		if err != nil {
			return err
		}
	}

	return s.db.Update(func(tx *bolt.Tx) error {
		mb := tx.Bucket(migrationsBucket)
		return mb.Put(migrationOptimalAttesterProtectionKey, migrationCompleted)
	})
}

// Migrate attester protection from the more optimal format to the old format in the DB.
func (s *Store) migrateOptimalAttesterProtectionDown(_ context.Context) error {
	// First we extract the public keys we are migrating down for.
	pubKeys, err := s.extractPubKeysForMigratingDown()
	if err != nil {
		return err
	}

	// Next up, we extract the data for attested epochs and signing roots
	// from the optimized db schema into maps we can use later.
	signingRootsByTarget := make(map[primitives.Epoch][]byte)
	targetEpochsBySource := make(map[primitives.Epoch][]primitives.Epoch)
	err = s.view(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(pubKeysBucket)
		if bkt == nil {
			return nil
		}
		for _, pubKey := range pubKeys {
			pubKeyBkt := bkt.Bucket(pubKey[:])
			if pubKeyBkt == nil {
				continue
			}
			sourceEpochsBucket := pubKeyBkt.Bucket(attestationSourceEpochsBucket)
			signingRootsBucket := pubKeyBkt.Bucket(attestationSigningRootsBucket)
			// Extract signing roots.
			if err := signingRootsBucket.ForEach(func(targetBytes, signingRoot []byte) error {
				var sr [32]byte
				copy(sr[:], signingRoot)
				signingRootsByTarget[bytesutil.BytesToEpochBigEndian(targetBytes)] = sr[:]
				return nil
			}); err != nil {
				return err
			}
			// Next up, extract the target epochs by source.
			if err := sourceEpochsBucket.ForEach(func(sourceBytes, targetEpochsBytes []byte) error {
				targetEpochs := make([]primitives.Epoch, 0)
				for i := 0; i < len(targetEpochsBytes); i += 8 {
					targetEpochs = append(targetEpochs, bytesutil.BytesToEpochBigEndian(targetEpochsBytes[i:i+8]))
				}
				targetEpochsBySource[bytesutil.BytesToEpochBigEndian(sourceBytes)] = targetEpochs
				return nil
			}); err != nil {
				return err
			}
		}
		return nil
	})
	if err != nil {
		return err
	}

	// Then, we use the data we extracted to recreate the old
	// attesting history format and for each public key, we save it
	// to the appropriate bucket.
	err = s.update(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(pubKeysBucket)
		if bkt == nil {
			return nil
		}
		bar := progress.InitializeProgressBar(len(pubKeys), "Migrating attesting history to old format")
		for _, pubKey := range pubKeys {
			// Now we write the attesting history using the data we extracted
			// from the buckets accordingly.
			history := newDeprecatedAttestingHistory(0)
			var maxTargetWritten primitives.Epoch
			for source, targetEpochs := range targetEpochsBySource {
				for _, target := range targetEpochs {
					signingRoot := params.BeaconConfig().ZeroHash[:]
					if sr, ok := signingRootsByTarget[target]; ok {
						signingRoot = sr
					}
					newHist, err := history.setTargetData(target, &deprecatedHistoryData{
						Source:      source,
						SigningRoot: signingRoot,
					})
					if err != nil {
						return err
					}
					history = newHist
					if target > maxTargetWritten {
						maxTargetWritten = target
					}
				}
			}
			newHist, err := history.setLatestEpochWritten(maxTargetWritten)
			if err != nil {
				return err
			}
			history = newHist
			deprecatedBkt, err := tx.CreateBucketIfNotExists(deprecatedAttestationHistoryBucket)
			if err != nil {
				return err
			}
			if err := deprecatedBkt.Put(pubKey[:], history); err != nil {
				return err
			}
			if err := bar.Add(1); err != nil {
				return err
			}
		}
		return nil
	})
	if err != nil {
		return err
	}

	// Finally, we clear the migration key.
	return s.update(func(tx *bolt.Tx) error {
		migrationsBkt := tx.Bucket(migrationsBucket)
		return migrationsBkt.Delete(migrationOptimalAttesterProtectionKey)
	})
}

func (s *Store) extractPubKeysForMigratingDown() ([][fieldparams.BLSPubkeyLength]byte, error) {
	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, 0)
	err := s.view(func(tx *bolt.Tx) error {
		mb := tx.Bucket(migrationsBucket)
		if b := mb.Get(migrationOptimalAttesterProtectionKey); b == nil {
			// Migration has not occurred, meaning data is already in old format
			// so no need to perform a down migration.
			return nil
		}
		bkt := tx.Bucket(pubKeysBucket)
		if bkt == nil {
			return nil
		}
		return bkt.ForEach(func(pubKey, v []byte) error {
			if pubKey == nil {
				return nil
			}
			pkBucket := bkt.Bucket(pubKey)
			if pkBucket == nil {
				return nil
			}
			pubKeys = append(pubKeys, bytesutil.ToBytes48(pubKey))
			return nil
		})
	})
	if err != nil {
		return nil, err
	}
	return pubKeys, nil
}
