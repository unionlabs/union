package kv

import (
	"context"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	bolt "go.etcd.io/bbolt"
	"go.opencensus.io/trace"
)

// PruneAttestations loops through every public key in the public keys bucket
// and prunes all attestation data that has target epochs older the highest
// target epoch minus some constant of how many epochs we keep track of for slashing
// protection. This routine is meant to run on startup.
func (s *Store) PruneAttestations(ctx context.Context) error {
	_, span := trace.StartSpan(ctx, "Validator.PruneAttestations")
	defer span.End()
	var pubkeys [][]byte
	err := s.view(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(pubKeysBucket)
		return bucket.ForEach(func(pubKey []byte, _ []byte) error {
			key := make([]byte, len(pubKey))
			copy(key, pubKey)
			pubkeys = append(pubkeys, pubKey)
			return nil
		})
	})
	if err != nil {
		return err
	}
	for _, k := range pubkeys {
		err = s.update(func(tx *bolt.Tx) error {
			bucket := tx.Bucket(pubKeysBucket)
			pkBucket := bucket.Bucket(k)
			if pkBucket == nil {
				return nil
			}
			if err := pruneSourceEpochsBucket(pkBucket); err != nil {
				return err
			}
			if err := pruneTargetEpochsBucket(pkBucket); err != nil {
				return err
			}
			return pruneSigningRootsBucket(pkBucket)
		})
		if err != nil {
			return err
		}
	}
	return nil
}

func pruneSourceEpochsBucket(bucket *bolt.Bucket) error {
	sourceEpochsBucket := bucket.Bucket(attestationSourceEpochsBucket)
	if sourceEpochsBucket == nil {
		return nil
	}

	return pruneBucket(sourceEpochsBucket)
}

func pruneTargetEpochsBucket(bucket *bolt.Bucket) error {
	targetEpochsBucket := bucket.Bucket(attestationTargetEpochsBucket)
	if targetEpochsBucket == nil {
		return nil
	}

	return pruneBucket(targetEpochsBucket)
}

func pruneSigningRootsBucket(bucket *bolt.Bucket) error {
	signingRootsBucket := bucket.Bucket(attestationSigningRootsBucket)
	if signingRootsBucket == nil {
		return nil
	}

	return pruneBucket(signingRootsBucket)
}

// pruneBucket iterates through epoch keys and deletes any key/value lower than
// the pruning cut off epoch as determined by the highest key in the bucket.
func pruneBucket(bkt *bolt.Bucket) error {
	if bkt == nil {
		return nil
	}

	// We obtain the highest target epoch from the signing roots bucket.
	highestEpochBytes, _ := bkt.Cursor().Last()
	highestEpoch := bytesutil.BytesToEpochBigEndian(highestEpochBytes)
	upperBounds := pruningEpochCutoff(highestEpoch)

	c := bkt.Cursor()
	for k, _ := c.First(); k != nil; k, _ = c.Next() {
		targetEpoch := bytesutil.BytesToEpochBigEndian(k)
		if targetEpoch >= upperBounds {
			return nil
		}
		if err := bkt.Delete(k); err != nil {
			return err
		}
	}

	return nil
}

// This helper function determines the cutoff epoch where, for all epochs before it, we should prune
// the slashing protection database. This is computed by taking in an epoch and subtracting
// SLASHING_PROTECTION_PRUNING_EPOCHS from the value. For example, if we are keeping track of 512 epochs
// in the database, if we pass in epoch 612, then we want to prune all epochs before epoch 100.
func pruningEpochCutoff(epoch primitives.Epoch) primitives.Epoch {
	minEpoch := primitives.Epoch(0)
	if epoch > params.BeaconConfig().SlashingProtectionPruningEpochs {
		minEpoch = epoch - params.BeaconConfig().SlashingProtectionPruningEpochs
	}
	return minEpoch
}
