package slasherkv

import (
	"context"
	"fmt"
	"testing"

	slashertypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/slasher/types"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	logTest "github.com/sirupsen/logrus/hooks/test"
	bolt "go.etcd.io/bbolt"
)

func TestStore_PruneProposalsAtEpoch(t *testing.T) {
	ctx := context.Background()

	// If the lowest stored epoch in the database is >= the end epoch of the pruning process,
	// there is nothing to prune, so we also expect exiting early.
	t.Run("lowest_stored_epoch_greater_than_pruning_limit_epoch", func(t *testing.T) {
		hook := logTest.NewGlobal()
		beaconDB := setupDB(t)

		// With a current epoch of 20 and a history length of 10, we should be pruning
		// everything before epoch (20 - 10) = 10.
		currentEpoch := primitives.Epoch(20)
		historyLength := primitives.Epoch(10)

		pruningLimitEpoch := currentEpoch - historyLength
		lowestStoredSlot, err := slots.EpochEnd(pruningLimitEpoch)
		require.NoError(t, err)

		err = beaconDB.db.Update(func(tx *bolt.Tx) error {
			bkt := tx.Bucket(proposalRecordsBucket)
			key, err := keyForValidatorProposal(lowestStoredSlot+1, 0 /* proposer index */)
			if err != nil {
				return err
			}
			return bkt.Put(key, []byte("hi"))
		})
		require.NoError(t, err)

		_, err = beaconDB.PruneProposalsAtEpoch(ctx, pruningLimitEpoch)
		require.NoError(t, err)
		expectedLog := fmt.Sprintf(
			"Lowest slot %d is > pruning slot %d, nothing to prune", lowestStoredSlot+1, lowestStoredSlot,
		)
		require.LogsContain(t, hook, expectedLog)
	})

	t.Run("prune_and_verify_deletions", func(t *testing.T) {
		beaconDB := setupDB(t)

		params.SetupTestConfigCleanup(t)
		config := params.BeaconConfig()
		config.SlotsPerEpoch = 2
		params.OverrideBeaconConfig(config)

		historyLength := primitives.Epoch(10)
		currentEpoch := primitives.Epoch(20)
		pruningLimitEpoch := currentEpoch - historyLength

		// We create proposals from genesis to the current epoch, with 2 proposals
		// at each slot to ensure the entire pruning logic works correctly.
		slotsPerEpoch := params.BeaconConfig().SlotsPerEpoch
		proposals := make([]*slashertypes.SignedBlockHeaderWrapper, 0, uint64(currentEpoch)*uint64(slotsPerEpoch)*2)
		for i := primitives.Epoch(0); i < currentEpoch; i++ {
			startSlot, err := slots.EpochStart(i)
			require.NoError(t, err)
			endSlot, err := slots.EpochStart(i + 1)
			require.NoError(t, err)
			for j := startSlot; j < endSlot; j++ {
				prop1 := createProposalWrapper(t, j, 0 /* proposer index */, []byte{0})
				prop2 := createProposalWrapper(t, j, 1 /* proposer index */, []byte{1})
				proposals = append(proposals, prop1, prop2)
			}
		}

		require.NoError(t, beaconDB.SaveBlockProposals(ctx, proposals))

		// We expect pruning completes without an issue and properly logs progress.
		_, err := beaconDB.PruneProposalsAtEpoch(ctx, pruningLimitEpoch)
		require.NoError(t, err)

		// Everything before epoch 10 should be deleted.
		for i := primitives.Epoch(0); i < pruningLimitEpoch; i++ {
			err = beaconDB.db.View(func(tx *bolt.Tx) error {
				bkt := tx.Bucket(proposalRecordsBucket)
				startSlot, err := slots.EpochStart(i)
				require.NoError(t, err)
				endSlot, err := slots.EpochStart(i + 1)
				require.NoError(t, err)
				for j := startSlot; j < endSlot; j++ {
					prop1Key, err := keyForValidatorProposal(j, 0)
					if err != nil {
						return err
					}
					prop2Key, err := keyForValidatorProposal(j, 1)
					if err != nil {
						return err
					}
					if bkt.Get(prop1Key) != nil {
						return fmt.Errorf("proposal still exists for epoch %d, validator 0", j)
					}
					if bkt.Get(prop2Key) != nil {
						return fmt.Errorf("proposal still exists for slot %d, validator 1", j)
					}
				}
				return nil
			})
			require.NoError(t, err)
		}
	})
}

func TestStore_PruneAttestations_OK(t *testing.T) {
	ctx := context.Background()

	// If the lowest stored epoch in the database is >= the end epoch of the pruning process,
	// there is nothing to prune, so we also expect exiting early.
	t.Run("lowest_stored_epoch_greater_than_pruning_limit_epoch", func(t *testing.T) {
		hook := logTest.NewGlobal()
		beaconDB := setupDB(t)

		// With a current epoch of 20 and a history length of 10, we should be pruning
		// everything before epoch (20 - 10) = 10.
		currentEpoch := primitives.Epoch(20)
		historyLength := primitives.Epoch(10)

		pruningLimitEpoch := currentEpoch - historyLength
		lowestStoredEpoch := pruningLimitEpoch

		err := beaconDB.db.Update(func(tx *bolt.Tx) error {
			bkt := tx.Bucket(attestationDataRootsBucket)
			encIdx := encodeValidatorIndex(primitives.ValidatorIndex(0))
			encodedTargetEpoch := encodeTargetEpoch(lowestStoredEpoch + 1)
			key := append(encodedTargetEpoch, encIdx...)
			return bkt.Put(key, []byte("hi"))
		})
		require.NoError(t, err)

		_, err = beaconDB.PruneAttestationsAtEpoch(ctx, pruningLimitEpoch)
		require.NoError(t, err)
		expectedLog := fmt.Sprintf(
			"Lowest epoch %d is > pruning epoch %d, nothing to prune", lowestStoredEpoch+1, lowestStoredEpoch,
		)
		require.LogsContain(t, hook, expectedLog)
	})

	t.Run("prune_and_verify_deletions", func(t *testing.T) {
		beaconDB := setupDB(t)

		params.SetupTestConfigCleanup(t)
		config := params.BeaconConfig()
		config.SlotsPerEpoch = 2
		params.OverrideBeaconConfig(config)

		historyLength := primitives.Epoch(10)
		currentEpoch := primitives.Epoch(20)
		pruningLimitEpoch := currentEpoch - historyLength

		// We create attestations from genesis to the current epoch, with 2 attestations
		// at each slot to ensure the entire pruning logic works correctly.
		slotsPerEpoch := params.BeaconConfig().SlotsPerEpoch
		attestations := make([]*slashertypes.IndexedAttestationWrapper, 0, uint64(currentEpoch)*uint64(slotsPerEpoch)*2)
		for i := primitives.Epoch(0); i < currentEpoch; i++ {
			startSlot, err := slots.EpochStart(i)
			require.NoError(t, err)
			endSlot, err := slots.EpochStart(i + 1)
			require.NoError(t, err)
			for j := startSlot; j < endSlot; j++ {
				attester1 := uint64(j + 10)
				attester2 := uint64(j + 11)
				target := i
				var source primitives.Epoch
				if i > 0 {
					source = target - 1
				}
				att1 := createAttestationWrapper(source, target, []uint64{attester1}, []byte{0})
				att2 := createAttestationWrapper(source, target, []uint64{attester2}, []byte{1})
				attestations = append(attestations, att1, att2)
			}
		}

		require.NoError(t, beaconDB.SaveAttestationRecordsForValidators(ctx, attestations))

		// We expect pruning completes without an issue.
		_, err := beaconDB.PruneAttestationsAtEpoch(ctx, pruningLimitEpoch)
		require.NoError(t, err)

		// Everything before epoch 10 should be deleted.
		for i := primitives.Epoch(0); i < pruningLimitEpoch; i++ {
			err = beaconDB.db.View(func(tx *bolt.Tx) error {
				bkt := tx.Bucket(attestationDataRootsBucket)
				startSlot, err := slots.EpochStart(i)
				require.NoError(t, err)
				endSlot, err := slots.EpochStart(i + 1)
				require.NoError(t, err)
				for j := startSlot; j < endSlot; j++ {
					attester1 := primitives.ValidatorIndex(j + 10)
					attester2 := primitives.ValidatorIndex(j + 11)
					key1 := append(encodeTargetEpoch(i), encodeValidatorIndex(attester1)...)
					key2 := append(encodeTargetEpoch(i), encodeValidatorIndex(attester2)...)
					if bkt.Get(key1) != nil {
						return fmt.Errorf("still exists for epoch %d, validator %d", i, attester1)
					}
					if bkt.Get(key2) != nil {
						return fmt.Errorf("still exists for slot %d, validator %d", i, attester2)
					}
				}
				return nil
			})
			require.NoError(t, err)
		}
	})
}
