package kv

import (
	"bytes"
	"context"
	"testing"

	"github.com/golang/snappy"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	v1alpha1 "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"go.etcd.io/bbolt"
)

func Test_migrateStateValidators(t *testing.T) {
	tests := []struct {
		name  string
		setup func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator)
		eval  func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator)
	}{
		{
			name: "only runs once",
			setup: func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator) {
				// create some new buckets that should be present for this migration
				err := dbStore.db.Update(func(tx *bbolt.Tx) error {
					_, err := tx.CreateBucketIfNotExists(stateValidatorsBucket)
					assert.NoError(t, err)
					_, err = tx.CreateBucketIfNotExists(blockRootValidatorHashesBucket)
					assert.NoError(t, err)
					return nil
				})
				assert.NoError(t, err)
			},
			eval: func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator) {
				// check if the migration is completed, per migration table.
				err := dbStore.db.View(func(tx *bbolt.Tx) error {
					migrationCompleteOrNot := tx.Bucket(migrationsBucket).Get(migrationStateValidatorsKey)
					assert.DeepEqual(t, migrationCompleted, migrationCompleteOrNot, "migration is not complete")
					return nil
				})
				assert.NoError(t, err)
			},
		},
		{
			name: "once migrated, always enable flag",
			setup: func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator) {
				// create some new buckets that should be present for this migration
				err := dbStore.db.Update(func(tx *bbolt.Tx) error {
					_, err := tx.CreateBucketIfNotExists(stateValidatorsBucket)
					assert.NoError(t, err)
					_, err = tx.CreateBucketIfNotExists(blockRootValidatorHashesBucket)
					assert.NoError(t, err)
					return nil
				})
				assert.NoError(t, err)
			},
			eval: func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator) {
				// disable the flag and see if the code mandates that flag.
				resetCfg := features.InitWithReset(&features.Flags{
					EnableHistoricalSpaceRepresentation: false,
				})
				defer resetCfg()

				// check if the migration is completed, per migration table.
				err := dbStore.db.View(func(tx *bbolt.Tx) error {
					migrationCompleteOrNot := tx.Bucket(migrationsBucket).Get(migrationStateValidatorsKey)
					assert.DeepEqual(t, migrationCompleted, migrationCompleteOrNot, "migration is not complete")
					return nil
				})
				assert.NoError(t, err)

				// create a new state and save it
				blockRoot := [32]byte{'B'}
				st, err := util.NewBeaconState()
				newValidators := validators(10)
				assert.NoError(t, err)
				assert.NoError(t, st.SetSlot(101))
				assert.NoError(t, st.SetValidators(newValidators))
				assert.NoError(t, dbStore.SaveState(context.Background(), st, blockRoot))
				assert.NoError(t, err)

				// now check if this newly saved state followed the migrated code path
				// by checking if the new validators are saved in the validator bucket.
				var individualHashes [][]byte
				for _, val := range newValidators {
					hash, hashErr := val.HashTreeRoot()
					assert.NoError(t, hashErr)
					individualHashes = append(individualHashes, hash[:])
				}
				pbState, err := state_native.ProtobufBeaconStatePhase0(st.ToProtoUnsafe())
				assert.NoError(t, err)
				validatorsFoundCount := 0
				for _, val := range pbState.Validators {
					hash, hashErr := val.HashTreeRoot()
					assert.NoError(t, hashErr)
					found := false
					for _, h := range individualHashes {
						if bytes.Equal(hash[:], h) {
							found = true
						}
					}
					require.Equal(t, true, found)
					validatorsFoundCount++
				}
				require.Equal(t, len(vals), validatorsFoundCount)
			},
		},
		{
			name: "migrates validators and adds them to new buckets",
			setup: func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator) {
				// create some new buckets that should be present for this migration
				err := dbStore.db.Update(func(tx *bbolt.Tx) error {
					_, err := tx.CreateBucketIfNotExists(stateValidatorsBucket)
					assert.NoError(t, err)
					_, err = tx.CreateBucketIfNotExists(blockRootValidatorHashesBucket)
					assert.NoError(t, err)
					return nil
				})
				assert.NoError(t, err)
			},
			eval: func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator) {
				// check whether the new buckets are present
				err := dbStore.db.View(func(tx *bbolt.Tx) error {
					valBkt := tx.Bucket(stateValidatorsBucket)
					assert.NotNil(t, valBkt)
					idxBkt := tx.Bucket(blockRootValidatorHashesBucket)
					assert.NotNil(t, idxBkt)
					return nil
				})
				assert.NoError(t, err)

				// check if the migration worked
				blockRoot := [32]byte{'A'}
				rcvdState, err := dbStore.State(context.Background(), blockRoot)
				assert.NoError(t, err)
				require.DeepSSZEqual(t, rcvdState.ToProtoUnsafe(), state.ToProtoUnsafe(), "saved state with validators and retrieved state are not matching")

				// find hashes of the validators that are set as part of the state
				var hashes []byte
				var individualHashes [][]byte
				for _, val := range vals {
					hash, hashErr := val.HashTreeRoot()
					assert.NoError(t, hashErr)
					hashes = append(hashes, hash[:]...)
					individualHashes = append(individualHashes, hash[:])
				}

				// check if all the validators that were in the state, are stored properly in the validator bucket
				pbState, err := state_native.ProtobufBeaconStatePhase0(rcvdState.ToProtoUnsafe())
				assert.NoError(t, err)
				validatorsFoundCount := 0
				for _, val := range pbState.Validators {
					hash, hashErr := val.HashTreeRoot()
					assert.NoError(t, hashErr)
					found := false
					for _, h := range individualHashes {
						if bytes.Equal(hash[:], h) {
							found = true
						}
					}
					require.Equal(t, true, found)
					validatorsFoundCount++
				}
				require.Equal(t, len(vals), validatorsFoundCount)

				// check if the state validator indexes are stored properly
				err = dbStore.db.View(func(tx *bbolt.Tx) error {
					rcvdValhashBytes := tx.Bucket(blockRootValidatorHashesBucket).Get(blockRoot[:])
					rcvdValHashes, sErr := snappy.Decode(nil, rcvdValhashBytes)
					assert.NoError(t, sErr)
					require.DeepEqual(t, hashes, rcvdValHashes)
					return nil
				})
				assert.NoError(t, err)
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dbStore := setupDB(t)

			// add a state with the given validators
			vals := validators(10)
			blockRoot := [32]byte{'A'}
			st, err := util.NewBeaconState()
			assert.NoError(t, err)
			assert.NoError(t, st.SetSlot(100))
			assert.NoError(t, st.SetValidators(vals))
			assert.NoError(t, dbStore.SaveState(context.Background(), st, blockRoot))
			assert.NoError(t, err)

			// enable historical state representation flag to test this
			resetCfg := features.InitWithReset(&features.Flags{
				EnableHistoricalSpaceRepresentation: true,
			})
			defer resetCfg()

			tt.setup(t, dbStore, st, vals)
			assert.NoError(t, migrateStateValidators(context.Background(), dbStore.db), "migrateArchivedIndex(tx) error")
			tt.eval(t, dbStore, st, vals)
		})
	}
}

func Test_migrateAltairStateValidators(t *testing.T) {
	tests := []struct {
		name  string
		setup func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator)
		eval  func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator)
	}{
		{
			name: "migrates validators and adds them to new buckets",
			setup: func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator) {
				// create some new buckets that should be present for this migration
				err := dbStore.db.Update(func(tx *bbolt.Tx) error {
					_, err := tx.CreateBucketIfNotExists(stateValidatorsBucket)
					assert.NoError(t, err)
					_, err = tx.CreateBucketIfNotExists(blockRootValidatorHashesBucket)
					assert.NoError(t, err)
					return nil
				})
				assert.NoError(t, err)
			},
			eval: func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator) {
				// check whether the new buckets are present
				err := dbStore.db.View(func(tx *bbolt.Tx) error {
					valBkt := tx.Bucket(stateValidatorsBucket)
					assert.NotNil(t, valBkt)
					idxBkt := tx.Bucket(blockRootValidatorHashesBucket)
					assert.NotNil(t, idxBkt)
					return nil
				})
				assert.NoError(t, err)

				// check if the migration worked
				blockRoot := [32]byte{'A'}
				rcvdState, err := dbStore.State(context.Background(), blockRoot)
				assert.NoError(t, err)
				require.DeepSSZEqual(t, rcvdState.ToProtoUnsafe(), state.ToProtoUnsafe(), "saved state with validators and retrieved state are not matching")

				// find hashes of the validators that are set as part of the state
				var hashes []byte
				var individualHashes [][]byte
				for _, val := range vals {
					hash, hashErr := val.HashTreeRoot()
					assert.NoError(t, hashErr)
					hashes = append(hashes, hash[:]...)
					individualHashes = append(individualHashes, hash[:])
				}

				// check if all the validators that were in the state, are stored properly in the validator bucket
				pbState, err := state_native.ProtobufBeaconStateAltair(rcvdState.ToProtoUnsafe())
				assert.NoError(t, err)
				validatorsFoundCount := 0
				for _, val := range pbState.Validators {
					hash, hashErr := val.HashTreeRoot()
					assert.NoError(t, hashErr)
					found := false
					for _, h := range individualHashes {
						if bytes.Equal(hash[:], h) {
							found = true
						}
					}
					require.Equal(t, true, found)
					validatorsFoundCount++
				}
				require.Equal(t, len(vals), validatorsFoundCount)

				// check if the state validator indexes are stored properly
				err = dbStore.db.View(func(tx *bbolt.Tx) error {
					rcvdValhashBytes := tx.Bucket(blockRootValidatorHashesBucket).Get(blockRoot[:])
					rcvdValHashes, sErr := snappy.Decode(nil, rcvdValhashBytes)
					assert.NoError(t, sErr)
					require.DeepEqual(t, hashes, rcvdValHashes)
					return nil
				})
				assert.NoError(t, err)
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dbStore := setupDB(t)

			// add a state with the given validators
			vals := validators(10)
			blockRoot := [32]byte{'A'}
			st, _ := util.DeterministicGenesisStateAltair(t, 20)
			err := st.SetFork(&v1alpha1.Fork{
				PreviousVersion: params.BeaconConfig().GenesisForkVersion,
				CurrentVersion:  params.BeaconConfig().AltairForkVersion,
				Epoch:           0,
			})
			require.NoError(t, err)
			assert.NoError(t, st.SetSlot(100))
			assert.NoError(t, st.SetValidators(vals))
			assert.NoError(t, dbStore.SaveState(context.Background(), st, blockRoot))

			// enable historical state representation flag to test this
			resetCfg := features.InitWithReset(&features.Flags{
				EnableHistoricalSpaceRepresentation: true,
			})
			defer resetCfg()

			tt.setup(t, dbStore, st, vals)
			assert.NoError(t, migrateStateValidators(context.Background(), dbStore.db), "migrateArchivedIndex(tx) error")
			tt.eval(t, dbStore, st, vals)
		})
	}
}

func Test_migrateBellatrixStateValidators(t *testing.T) {
	tests := []struct {
		name  string
		setup func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator)
		eval  func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator)
	}{
		{
			name: "migrates validators and adds them to new buckets",
			setup: func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator) {
				// create some new buckets that should be present for this migration
				err := dbStore.db.Update(func(tx *bbolt.Tx) error {
					_, err := tx.CreateBucketIfNotExists(stateValidatorsBucket)
					assert.NoError(t, err)
					_, err = tx.CreateBucketIfNotExists(blockRootValidatorHashesBucket)
					assert.NoError(t, err)
					return nil
				})
				assert.NoError(t, err)
			},
			eval: func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator) {
				// check whether the new buckets are present
				err := dbStore.db.View(func(tx *bbolt.Tx) error {
					valBkt := tx.Bucket(stateValidatorsBucket)
					assert.NotNil(t, valBkt)
					idxBkt := tx.Bucket(blockRootValidatorHashesBucket)
					assert.NotNil(t, idxBkt)
					return nil
				})
				assert.NoError(t, err)

				// check if the migration worked
				blockRoot := [32]byte{'A'}
				rcvdState, err := dbStore.State(context.Background(), blockRoot)
				assert.NoError(t, err)
				require.DeepSSZEqual(t, rcvdState.ToProtoUnsafe(), state.ToProtoUnsafe(), "saved state with validators and retrieved state are not matching")

				// find hashes of the validators that are set as part of the state
				var hashes []byte
				var individualHashes [][]byte
				for _, val := range vals {
					hash, hashErr := val.HashTreeRoot()
					assert.NoError(t, hashErr)
					hashes = append(hashes, hash[:]...)
					individualHashes = append(individualHashes, hash[:])
				}

				// check if all the validators that were in the state, are stored properly in the validator bucket
				pbState, err := state_native.ProtobufBeaconStateBellatrix(rcvdState.ToProtoUnsafe())
				assert.NoError(t, err)
				validatorsFoundCount := 0
				for _, val := range pbState.Validators {
					hash, hashErr := val.HashTreeRoot()
					assert.NoError(t, hashErr)
					found := false
					for _, h := range individualHashes {
						if bytes.Equal(hash[:], h) {
							found = true
						}
					}
					require.Equal(t, true, found)
					validatorsFoundCount++
				}
				require.Equal(t, len(vals), validatorsFoundCount)

				// check if the state validator indexes are stored properly
				err = dbStore.db.View(func(tx *bbolt.Tx) error {
					rcvdValhashBytes := tx.Bucket(blockRootValidatorHashesBucket).Get(blockRoot[:])
					rcvdValHashes, sErr := snappy.Decode(nil, rcvdValhashBytes)
					assert.NoError(t, sErr)
					require.DeepEqual(t, hashes, rcvdValHashes)
					return nil
				})
				assert.NoError(t, err)
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dbStore := setupDB(t)

			// add a state with the given validators
			vals := validators(10)
			blockRoot := [32]byte{'A'}
			st, _ := util.DeterministicGenesisStateBellatrix(t, 20)
			err := st.SetFork(&v1alpha1.Fork{
				PreviousVersion: params.BeaconConfig().AltairForkVersion,
				CurrentVersion:  params.BeaconConfig().BellatrixForkVersion,
				Epoch:           0,
			})
			require.NoError(t, err)
			assert.NoError(t, st.SetSlot(100))
			assert.NoError(t, st.SetValidators(vals))
			assert.NoError(t, dbStore.SaveState(context.Background(), st, blockRoot))

			// enable historical state representation flag to test this
			resetCfg := features.InitWithReset(&features.Flags{
				EnableHistoricalSpaceRepresentation: true,
			})
			defer resetCfg()

			tt.setup(t, dbStore, st, vals)
			assert.NoError(t, migrateStateValidators(context.Background(), dbStore.db), "migrateArchivedIndex(tx) error")
			tt.eval(t, dbStore, st, vals)
		})
	}
}

func Test_migrateCapellaStateValidators(t *testing.T) {
	tests := []struct {
		name  string
		setup func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator)
		eval  func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator)
	}{
		{
			name: "migrates validators and adds them to new buckets",
			setup: func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator) {
				// create some new buckets that should be present for this migration
				err := dbStore.db.Update(func(tx *bbolt.Tx) error {
					_, err := tx.CreateBucketIfNotExists(stateValidatorsBucket)
					assert.NoError(t, err)
					_, err = tx.CreateBucketIfNotExists(blockRootValidatorHashesBucket)
					assert.NoError(t, err)
					return nil
				})
				assert.NoError(t, err)
			},
			eval: func(t *testing.T, dbStore *Store, state state.BeaconState, vals []*v1alpha1.Validator) {
				// check whether the new buckets are present
				err := dbStore.db.View(func(tx *bbolt.Tx) error {
					valBkt := tx.Bucket(stateValidatorsBucket)
					assert.NotNil(t, valBkt)
					idxBkt := tx.Bucket(blockRootValidatorHashesBucket)
					assert.NotNil(t, idxBkt)
					return nil
				})
				assert.NoError(t, err)

				// check if the migration worked
				blockRoot := [32]byte{'A'}
				rcvdState, err := dbStore.State(context.Background(), blockRoot)
				assert.NoError(t, err)
				require.DeepSSZEqual(t, rcvdState.ToProtoUnsafe(), state.ToProtoUnsafe(), "saved state with validators and retrieved state are not matching")

				// find hashes of the validators that are set as part of the state
				var hashes []byte
				var individualHashes [][]byte
				for _, val := range vals {
					hash, hashErr := val.HashTreeRoot()
					assert.NoError(t, hashErr)
					hashes = append(hashes, hash[:]...)
					individualHashes = append(individualHashes, hash[:])
				}

				// check if all the validators that were in the state, are stored properly in the validator bucket
				pbState, err := state_native.ProtobufBeaconStateCapella(rcvdState.ToProtoUnsafe())
				assert.NoError(t, err)
				validatorsFoundCount := 0
				for _, val := range pbState.Validators {
					hash, hashErr := val.HashTreeRoot()
					assert.NoError(t, hashErr)
					found := false
					for _, h := range individualHashes {
						if bytes.Equal(hash[:], h) {
							found = true
						}
					}
					require.Equal(t, true, found)
					validatorsFoundCount++
				}
				require.Equal(t, len(vals), validatorsFoundCount)

				// check if the state validator indexes are stored properly
				err = dbStore.db.View(func(tx *bbolt.Tx) error {
					rcvdValhashBytes := tx.Bucket(blockRootValidatorHashesBucket).Get(blockRoot[:])
					rcvdValHashes, sErr := snappy.Decode(nil, rcvdValhashBytes)
					assert.NoError(t, sErr)
					require.DeepEqual(t, hashes, rcvdValHashes)
					return nil
				})
				assert.NoError(t, err)
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dbStore := setupDB(t)

			// add a state with the given validators
			vals := validators(10)
			blockRoot := [32]byte{'A'}
			st, _ := util.DeterministicGenesisStateCapella(t, 20)
			err := st.SetFork(&v1alpha1.Fork{
				PreviousVersion: params.BeaconConfig().BellatrixForkVersion,
				CurrentVersion:  params.BeaconConfig().CapellaForkVersion,
				Epoch:           0,
			})
			require.NoError(t, err)
			assert.NoError(t, st.SetSlot(100))
			assert.NoError(t, st.SetValidators(vals))
			assert.NoError(t, dbStore.SaveState(context.Background(), st, blockRoot))

			// enable historical state representation flag to test this
			resetCfg := features.InitWithReset(&features.Flags{
				EnableHistoricalSpaceRepresentation: true,
			})
			defer resetCfg()

			tt.setup(t, dbStore, st, vals)
			assert.NoError(t, migrateStateValidators(context.Background(), dbStore.db), "migrateArchivedIndex(tx) error")
			tt.eval(t, dbStore, st, vals)
		})
	}
}
