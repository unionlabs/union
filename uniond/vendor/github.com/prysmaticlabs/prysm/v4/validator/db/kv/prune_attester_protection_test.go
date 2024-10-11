package kv

import (
	"context"
	"fmt"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	bolt "go.etcd.io/bbolt"
)

func TestPruneAttestations_NoPruning(t *testing.T) {
	pubKey := [fieldparams.BLSPubkeyLength]byte{1}
	validatorDB := setupDB(t, [][fieldparams.BLSPubkeyLength]byte{pubKey})

	// Write attesting history for every single epoch
	// since genesis to a specified number of epochs.
	numEpochs := params.BeaconConfig().SlashingProtectionPruningEpochs - 1
	err := setupAttestationsForEveryEpoch(validatorDB, pubKey, numEpochs)
	require.NoError(t, err)

	// Next, attempt to prune and realize that we still have all epochs intact
	err = validatorDB.PruneAttestations(context.Background())
	require.NoError(t, err)

	startEpoch := primitives.Epoch(0)
	err = checkAttestingHistoryAfterPruning(
		t,
		validatorDB,
		pubKey,
		startEpoch,
		numEpochs,
		false, /* should be pruned */
	)
	require.NoError(t, err)
}

func TestPruneAttestations_OK(t *testing.T) {
	numKeys := uint64(64)
	pks := make([][fieldparams.BLSPubkeyLength]byte, 0, numKeys)
	for i := uint64(0); i < numKeys; i++ {
		pks = append(pks, bytesutil.ToBytes48(bytesutil.ToBytes(i, 48)))
	}
	validatorDB := setupDB(t, pks)

	// Write attesting history for every single epoch
	// since genesis to SLASHING_PROTECTION_PRUNING_EPOCHS * 2.
	numEpochs := params.BeaconConfig().SlashingProtectionPruningEpochs * 2
	for _, pk := range pks {
		require.NoError(t, setupAttestationsForEveryEpoch(validatorDB, pk, numEpochs))
	}

	require.NoError(t, validatorDB.PruneAttestations(context.Background()))

	// Next, verify that we pruned every epoch
	// from genesis to SLASHING_PROTECTION_PRUNING_EPOCHS - 1.
	startEpoch := primitives.Epoch(0)
	for _, pk := range pks {
		err := checkAttestingHistoryAfterPruning(
			t,
			validatorDB,
			pk,
			startEpoch,
			params.BeaconConfig().SlashingProtectionPruningEpochs-1,
			true, /* should be pruned */
		)
		require.NoError(t, err)
	}

	// Next, verify that we pruned every epoch
	// from N = SLASHING_PROTECTION_PRUNING_EPOCHS to N * 2.
	startEpoch = params.BeaconConfig().SlashingProtectionPruningEpochs
	endEpoch := startEpoch * 2
	for _, pk := range pks {
		err := checkAttestingHistoryAfterPruning(
			t,
			validatorDB,
			pk,
			startEpoch,
			endEpoch,
			false, /* should not be pruned */
		)
		require.NoError(t, err)
	}
}

func BenchmarkPruneAttestations(b *testing.B) {
	numKeys := uint64(8)
	pks := make([][fieldparams.BLSPubkeyLength]byte, 0, numKeys)
	for i := uint64(0); i < numKeys; i++ {
		pks = append(pks, bytesutil.ToBytes48(bytesutil.ToBytes(i, 48)))
	}
	validatorDB := setupDB(b, pks)

	// Write attesting history for every single epoch
	// since genesis to SLASHING_PROTECTION_PRUNING_EPOCHS * 20.
	numEpochs := params.BeaconConfig().SlashingProtectionPruningEpochs * 20

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		b.StopTimer()
		for _, pk := range pks {
			require.NoError(b, setupAttestationsForEveryEpoch(validatorDB, pk, numEpochs))
		}
		b.StartTimer()

		require.NoError(b, validatorDB.PruneAttestations(context.Background()))
	}
}

// Saves attesting history for every (source, target = source + 1) pairs since genesis
// up to a given number of epochs for a validator public key.
func setupAttestationsForEveryEpoch(validatorDB *Store, pubKey [48]byte, numEpochs primitives.Epoch) error {
	return validatorDB.update(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(pubKeysBucket)
		pkBucket, err := bucket.CreateBucketIfNotExists(pubKey[:])
		if err != nil {
			return err
		}
		signingRootsBucket, err := pkBucket.CreateBucketIfNotExists(attestationSigningRootsBucket)
		if err != nil {
			return err
		}
		sourceEpochsBucket, err := pkBucket.CreateBucketIfNotExists(attestationSourceEpochsBucket)
		if err != nil {
			return err
		}
		for sourceEpoch := primitives.Epoch(0); sourceEpoch < numEpochs; sourceEpoch++ {
			targetEpoch := sourceEpoch + 1
			targetEpochBytes := bytesutil.EpochToBytesBigEndian(targetEpoch)
			sourceEpochBytes := bytesutil.EpochToBytesBigEndian(sourceEpoch)
			// Save (source epoch, target epoch) pairs.
			if err := sourceEpochsBucket.Put(sourceEpochBytes, targetEpochBytes); err != nil {
				return err
			}
			// Save signing root for target epoch.
			var signingRoot [32]byte
			copy(signingRoot[:], fmt.Sprintf("%d", targetEpochBytes))
			if err := signingRootsBucket.Put(targetEpochBytes, signingRoot[:]); err != nil {
				return err
			}
		}
		return nil
	})
}

// Verifies, based on a boolean input argument, whether or not we should have
// pruned all attesting history since genesis up to a specified number of epochs.
func checkAttestingHistoryAfterPruning(
	t testing.TB,
	validatorDB *Store,
	pubKey [fieldparams.BLSPubkeyLength]byte,
	startEpoch,
	numEpochs primitives.Epoch,
	shouldBePruned bool,
) error {
	return validatorDB.view(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(pubKeysBucket)
		pkBkt := bucket.Bucket(pubKey[:])
		signingRootsBkt := pkBkt.Bucket(attestationSigningRootsBucket)
		sourceEpochsBkt := pkBkt.Bucket(attestationSourceEpochsBucket)
		for sourceEpoch := startEpoch; sourceEpoch < numEpochs; sourceEpoch++ {
			targetEpoch := sourceEpoch + 1
			targetEpochBytes := bytesutil.EpochToBytesBigEndian(targetEpoch)
			sourceEpochBytes := bytesutil.EpochToBytesBigEndian(sourceEpoch)

			storedTargetEpoch := sourceEpochsBkt.Get(sourceEpochBytes)
			signingRoot := signingRootsBkt.Get(targetEpochBytes)
			if shouldBePruned {
				// Expect to have no data if we have pruned.
				require.Equal(t, true, signingRoot == nil)
				require.Equal(t, true, storedTargetEpoch == nil)
			} else {
				// Expect the correct signing root.
				var expectedSigningRoot [32]byte
				copy(expectedSigningRoot[:], fmt.Sprintf("%d", targetEpochBytes))
				require.DeepEqual(t, expectedSigningRoot[:], signingRoot)
				// Expect the correct target epoch.
				require.DeepEqual(t, targetEpochBytes, storedTargetEpoch)
			}
		}
		return nil
	})
}
