package kv

import (
	"bytes"
	"context"
	"fmt"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/snappy"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz/detect"
	"github.com/prysmaticlabs/prysm/v4/monitoring/progress"
	v1alpha1 "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/schollz/progressbar/v3"
	bolt "go.etcd.io/bbolt"
)

const batchSize = 10

var migrationStateValidatorsKey = []byte("migration_state_validator")

func shouldMigrateValidators(db *bolt.DB) (bool, error) {
	migrateDB := false
	if updateErr := db.View(func(tx *bolt.Tx) error {
		mb := tx.Bucket(migrationsBucket)
		// feature flag is not enabled
		// - migration is complete, don't migrate the DB but warn that this will work as if the flag is enabled.
		// - migration is not complete, don't migrate the DB.
		if !features.Get().EnableHistoricalSpaceRepresentation {
			b := mb.Get(migrationStateValidatorsKey)
			if bytes.Equal(b, migrationCompleted) {
				log.Warning("migration of historical states already completed. The node will work as if --enable-historical-state-representation=true.")
				return nil
			} else {
				return nil
			}
		}

		// if the migration flag is enabled (checked in the above condition)
		//  and if migration is complete, don't migrate again.
		if b := mb.Get(migrationStateValidatorsKey); bytes.Equal(b, migrationCompleted) {
			return nil
		}

		// migrate flag is enabled and DB is not migrated yet
		migrateDB = true
		return nil
	}); updateErr != nil {
		log.WithError(updateErr).Errorf("could not migrate bucket: %s", stateBucket)
		return false, updateErr
	}

	return migrateDB, nil
}

func migrateStateValidators(ctx context.Context, db *bolt.DB) error {
	if ok, err := shouldMigrateValidators(db); err != nil {
		return err
	} else if !ok {
		// A migration is not required.
		return nil
	}

	log.Infof("Performing a one-time migration to a more efficient database schema for %s. It will take few minutes", stateBucket)

	// get all the keys to migrate
	var keys [][]byte
	if err := db.Update(func(tx *bolt.Tx) error {
		stateBkt := tx.Bucket(stateBucket)
		if stateBkt == nil {
			return nil
		}
		k, err := stateBucketKeys(stateBkt)
		if err != nil {
			return err
		}
		keys = k
		return nil
	}); err != nil {
		return err
	}
	log.Infof("total keys = %d", len(keys))

	// prepare the progress bar with the total count of the keys to migrate
	bar := progress.InitializeProgressBar(len(keys), "Migrating state validators to new schema.")

	for batchIndex := 0; batchIndex < len(keys); batchIndex += batchSize {
		if err := db.Update(performValidatorStateMigration(ctx, bar, batchIndex, keys)); err != nil {
			return err
		}
	}

	// set the migration entry to done
	if err := db.Update(func(tx *bolt.Tx) error {
		mb := tx.Bucket(migrationsBucket)
		if mb == nil {
			return nil
		}
		return mb.Put(migrationStateValidatorsKey, migrationCompleted)
	}); err != nil {
		return err
	}
	log.Infof("migration done for bucket %s.", stateBucket)
	return nil
}

func performValidatorStateMigration(ctx context.Context, bar *progressbar.ProgressBar, batchIndex int, keys [][]byte) func(tx *bolt.Tx) error {
	return func(tx *bolt.Tx) error {
		//create the source and destination buckets
		stateBkt := tx.Bucket(stateBucket)
		if stateBkt == nil {
			return nil
		}
		valBkt := tx.Bucket(stateValidatorsBucket)
		if valBkt == nil {
			return nil
		}
		indexBkt := tx.Bucket(blockRootValidatorHashesBucket)
		if indexBkt == nil {
			return nil
		}

		// migrate the key values for this batch
		cursor := stateBkt.Cursor()
		count := 0
		index := batchIndex
		for _, v := cursor.Seek(keys[index]); count < batchSize && index < len(keys); _, v = cursor.Next() {
			enc, err := snappy.Decode(nil, v)
			if err != nil {
				return err
			}
			item := enc
			switch {
			case hasAltairKey(enc):
				item = item[len(altairKey):]
			case hasBellatrixKey(enc):
				item = item[len(bellatrixKey):]
			case hasCapellaKey(enc):
				item = item[len(capellaKey):]
			}

			detector, err := detect.FromState(item)
			if err != nil {
				return err
			}
			beaconState, err := detector.UnmarshalBeaconState(item)
			if err != nil {
				return err
			}
			// no validators in state to migrate
			if beaconState.NumValidators() == 0 {
				return fmt.Errorf("no validator entries in state key 0x%s", hexutil.Encode(keys[index]))
			}
			vals := beaconState.Validators()
			validatorKeys, insertErr := insertValidatorHashes(ctx, vals, valBkt)
			if insertErr != nil {
				return insertErr
			}
			// add the validator entry keys for a given block root.
			compValidatorKeys := snappy.Encode(nil, validatorKeys)
			idxErr := indexBkt.Put(keys[index], compValidatorKeys)
			if idxErr != nil {
				return idxErr
			}
			// zero the validator entries in BeaconState object .
			if err := beaconState.SetValidators(make([]*v1alpha1.Validator, 0)); err != nil {
				return err
			}
			rawObj, err := beaconState.MarshalSSZ()
			if err != nil {
				return err
			}
			var stateBytes []byte
			switch {
			case hasAltairKey(enc):
				stateBytes = snappy.Encode(nil, append(altairKey, rawObj...))
			case hasBellatrixKey(enc):
				stateBytes = snappy.Encode(nil, append(bellatrixKey, rawObj...))
			case hasCapellaKey(enc):
				stateBytes = snappy.Encode(nil, append(capellaKey, rawObj...))
			default:
				stateBytes = snappy.Encode(nil, rawObj)
			}
			if stateErr := stateBkt.Put(keys[index], stateBytes); stateErr != nil {
				return stateErr
			}
			count++
			index++

			if barErr := bar.Add(1); barErr != nil {
				return barErr
			}
		}

		return nil
	}
}

func stateBucketKeys(stateBucket *bolt.Bucket) ([][]byte, error) {
	var keys [][]byte
	if err := stateBucket.ForEach(func(pubKey, v []byte) error {
		keys = append(keys, pubKey)
		return nil
	}); err != nil {
		return nil, err
	}
	return keys, nil
}

func insertValidatorHashes(ctx context.Context, validators []*v1alpha1.Validator, valBkt *bolt.Bucket) ([]byte, error) {
	// move all the validators in this state registry out to a new bucket.
	var validatorKeys []byte
	for _, val := range validators {
		valBytes, encodeErr := encode(ctx, val)
		if encodeErr != nil {
			return nil, encodeErr
		}

		// create the unique hash for that validator entry.
		hash, hashErr := val.HashTreeRoot()
		if hashErr != nil {
			return nil, hashErr
		}

		// add the validator in the stateValidatorsBucket, if it is not present.
		if valEntry := valBkt.Get(hash[:]); valEntry == nil {
			if putErr := valBkt.Put(hash[:], valBytes); putErr != nil {
				return nil, putErr
			}
		}

		// note down the pointer of the stateValidatorsBucket.
		validatorKeys = append(validatorKeys, hash[:]...)
	}
	return validatorKeys, nil
}
