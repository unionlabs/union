package kv

import (
	"bytes"
	"context"

	"github.com/prysmaticlabs/prysm/v4/monitoring/progress"
	bolt "go.etcd.io/bbolt"
)

var (
	migrationSourceTargetEpochsBucketKey = []byte("source_target_epochs_bucket_0")
)

const (
	publicKeyMigrationBatchSize = 100 // Batch update 100 public keys at a time.
)

func (s *Store) migrateSourceTargetEpochsBucketUp(_ context.Context) error {
	// First, we extract the public keys we need to migrate data for.
	publicKeyBytes := make([][]byte, 0)
	err := s.db.View(func(tx *bolt.Tx) error {
		mb := tx.Bucket(migrationsBucket)
		if b := mb.Get(migrationSourceTargetEpochsBucketKey); bytes.Equal(b, migrationCompleted) {
			return nil // Migration already completed.
		}
		bkt := tx.Bucket(pubKeysBucket)
		return bkt.ForEach(func(k, _ []byte) error {
			if k == nil {
				return nil
			}
			nk := make([]byte, len(k))
			copy(nk, k)
			publicKeyBytes = append(publicKeyBytes, nk)
			return nil
		})
	})
	if err != nil {
		return err
	}

	// Next up, we initiate a bolt transaction for batches of public keys for efficiency.
	// If we did a single transaction for all public keys, resource use might be too high,
	// and if we do a single one per key, the migration will take too long.
	batchedKeys := batchPublicKeys(publicKeyBytes, publicKeyMigrationBatchSize)
	bar := progress.InitializeProgressBar(
		len(batchedKeys), "Adding optimizations for validator slashing protection",
	)
	for _, batch := range batchedKeys {
		err = s.db.Update(func(tx *bolt.Tx) error {
			bkt := tx.Bucket(pubKeysBucket)
			for _, pubKey := range batch {
				pkb := bkt.Bucket(pubKey)
				if pkb == nil {
					continue
				}
				sourceBucket := pkb.Bucket(attestationSourceEpochsBucket)
				if sourceBucket == nil {
					continue
				}
				targetBucket, err := pkb.CreateBucketIfNotExists(attestationTargetEpochsBucket)
				if err != nil {
					return err
				}
				err = sourceBucket.ForEach(func(sourceEpochBytes, targetEpochsBytes []byte) error {
					for i := 0; i < len(targetEpochsBytes); i += 8 {
						if err := insertTargetSource(
							targetBucket,
							targetEpochsBytes[i:i+8],
							sourceEpochBytes,
						); err != nil {
							return err
						}
					}
					return nil
				})
				if err != nil {
					return err
				}
			}
			return nil
		})
		if err != nil {
			return err
		}
		if err := bar.Add(1); err != nil {
			return err
		}
	}

	// Finally we mark the migration as completed.
	return s.db.Update(func(tx *bolt.Tx) error {
		mb := tx.Bucket(migrationsBucket)
		return mb.Put(migrationSourceTargetEpochsBucketKey, migrationCompleted)
	})
}

func (s *Store) migrateSourceTargetEpochsBucketDown(_ context.Context) error {
	return s.db.Update(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(pubKeysBucket)
		err := bkt.ForEach(func(k, _ []byte) error {
			if k == nil {
				return nil
			}
			pkBucket := bkt.Bucket(k)
			if pkBucket == nil {
				return nil
			}
			return pkBucket.DeleteBucket(attestationTargetEpochsBucket)
		})
		if err != nil {
			return err
		}
		migrationsBkt := tx.Bucket(migrationsBucket)
		return migrationsBkt.Delete(migrationSourceTargetEpochsBucketKey)
	})
}

func insertTargetSource(bkt *bolt.Bucket, targetEpochBytes, sourceEpochBytes []byte) error {
	var existingAttestedSourceBytes []byte
	if existing := bkt.Get(targetEpochBytes); existing != nil {
		existingAttestedSourceBytes = append(existing, sourceEpochBytes...)
	} else {
		existingAttestedSourceBytes = sourceEpochBytes
	}
	return bkt.Put(targetEpochBytes, existingAttestedSourceBytes)
}

func batchPublicKeys(publicKeys [][]byte, batchSize int) [][][]byte {
	if len(publicKeys) < batchSize {
		return [][][]byte{publicKeys}
	}
	batch := make([][][]byte, 0)
	for i := 0; i < len(publicKeys); i += batchSize {
		if i+batchSize >= len(publicKeys)+1 {
			batch = append(batch, publicKeys[i:])
		} else {
			batch = append(batch, publicKeys[i:i+batchSize])
		}
	}
	return batch
}
