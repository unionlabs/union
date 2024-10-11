package kv

import (
	"context"
	"fmt"
	"path/filepath"
	"sync"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	logTest "github.com/sirupsen/logrus/hooks/test"
	bolt "go.etcd.io/bbolt"
)

func TestPendingAttestationRecords_Flush(t *testing.T) {
	queue := NewQueuedAttestationRecords()

	// Add 5 atts
	num := 5
	for i := 0; i < num; i++ {
		queue.Append(&AttestationRecord{
			Target: primitives.Epoch(i),
		})
	}

	res := queue.Flush()
	assert.Equal(t, len(res), num, "Wrong number of flushed attestations")
	assert.Equal(t, len(queue.records), 0, "Records were not cleared/flushed")
}

func TestPendingAttestationRecords_Len(t *testing.T) {
	queue := NewQueuedAttestationRecords()
	assert.Equal(t, queue.Len(), 0)
	queue.Append(&AttestationRecord{})
	assert.Equal(t, queue.Len(), 1)
	queue.Flush()
	assert.Equal(t, queue.Len(), 0)
}

func TestStore_CheckSlashableAttestation_DoubleVote(t *testing.T) {
	ctx := context.Background()
	numValidators := 1
	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, numValidators)
	validatorDB := setupDB(t, pubKeys)
	tests := []struct {
		name                string
		existingAttestation *ethpb.IndexedAttestation
		existingSigningRoot [32]byte
		incomingAttestation *ethpb.IndexedAttestation
		incomingSigningRoot [32]byte
		want                bool
	}{
		{
			name:                "different signing root at same target equals a double vote",
			existingAttestation: createAttestation(0, 1 /* Target */),
			existingSigningRoot: [32]byte{1},
			incomingAttestation: createAttestation(0, 1 /* Target */),
			incomingSigningRoot: [32]byte{2},
			want:                true,
		},
		{
			name:                "same signing root at same target is safe",
			existingAttestation: createAttestation(0, 1 /* Target */),
			existingSigningRoot: [32]byte{1},
			incomingAttestation: createAttestation(0, 1 /* Target */),
			incomingSigningRoot: [32]byte{1},
			want:                false,
		},
		{
			name:                "different signing root at different target is safe",
			existingAttestation: createAttestation(0, 1 /* Target */),
			existingSigningRoot: [32]byte{1},
			incomingAttestation: createAttestation(0, 2 /* Target */),
			incomingSigningRoot: [32]byte{2},
			want:                false,
		},
		{
			name:                "no data stored at target should not be considered a double vote",
			existingAttestation: createAttestation(0, 1 /* Target */),
			existingSigningRoot: [32]byte{1},
			incomingAttestation: createAttestation(0, 2 /* Target */),
			incomingSigningRoot: [32]byte{1},
			want:                false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := validatorDB.SaveAttestationForPubKey(
				ctx,
				pubKeys[0],
				tt.existingSigningRoot,
				tt.existingAttestation,
			)
			require.NoError(t, err)
			slashingKind, err := validatorDB.CheckSlashableAttestation(
				ctx,
				pubKeys[0],
				tt.incomingSigningRoot[:],
				tt.incomingAttestation,
			)
			if tt.want {
				require.NotNil(t, err)
				assert.Equal(t, DoubleVote, slashingKind)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestStore_CheckSlashableAttestation_SurroundVote_MultipleTargetsPerSource(t *testing.T) {
	ctx := context.Background()
	numValidators := 1
	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, numValidators)
	validatorDB := setupDB(t, pubKeys)

	// Create an attestation with source 1 and target 50, save it.
	firstAtt := createAttestation(1, 50)
	err := validatorDB.SaveAttestationForPubKey(ctx, pubKeys[0], [32]byte{0}, firstAtt)
	require.NoError(t, err)

	// Create an attestation with source 1 and target 100, save it.
	secondAtt := createAttestation(1, 100)
	err = validatorDB.SaveAttestationForPubKey(ctx, pubKeys[0], [32]byte{1}, secondAtt)
	require.NoError(t, err)

	// Create an attestation with source 0 and target 51, which should surround
	// our first attestation. Given there can be multiple attested target epochs per
	// source epoch, we expect our logic to be able to catch this slashable offense.
	evilAtt := createAttestation(firstAtt.Data.Source.Epoch-1, firstAtt.Data.Target.Epoch+1)
	slashable, err := validatorDB.CheckSlashableAttestation(ctx, pubKeys[0], []byte{2}, evilAtt)
	require.NotNil(t, err)
	assert.Equal(t, SurroundingVote, slashable)
}

func TestStore_CheckSlashableAttestation_SurroundVote_54kEpochs(t *testing.T) {
	ctx := context.Background()
	numValidators := 1
	numEpochs := primitives.Epoch(54000)
	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, numValidators)
	validatorDB := setupDB(t, pubKeys)

	// Attest to every (source = epoch, target = epoch + 1) sequential pair
	// since genesis up to and including the weak subjectivity period epoch (54,000).
	err := validatorDB.update(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(pubKeysBucket)
		pkBucket, err := bucket.CreateBucketIfNotExists(pubKeys[0][:])
		if err != nil {
			return err
		}
		sourceEpochsBucket, err := pkBucket.CreateBucketIfNotExists(attestationSourceEpochsBucket)
		if err != nil {
			return err
		}
		for epoch := primitives.Epoch(1); epoch < numEpochs; epoch++ {
			att := createAttestation(epoch-1, epoch)
			sourceEpoch := bytesutil.EpochToBytesBigEndian(att.Data.Source.Epoch)
			targetEpoch := bytesutil.EpochToBytesBigEndian(att.Data.Target.Epoch)
			if err := sourceEpochsBucket.Put(sourceEpoch, targetEpoch); err != nil {
				return err
			}
		}
		return nil
	})
	require.NoError(t, err)

	tests := []struct {
		name        string
		signingRoot []byte
		attestation *ethpb.IndexedAttestation
		want        SlashingKind
	}{
		{
			name:        "surround vote at half of the weak subjectivity period",
			signingRoot: []byte{},
			attestation: createAttestation(numEpochs/2, numEpochs),
			want:        SurroundingVote,
		},
		{
			name:        "spanning genesis to weak subjectivity period surround vote",
			signingRoot: []byte{},
			attestation: createAttestation(0, numEpochs),
			want:        SurroundingVote,
		},
		{
			name:        "simple surround vote at end of weak subjectivity period",
			signingRoot: []byte{},
			attestation: createAttestation(numEpochs-3, numEpochs),
			want:        SurroundingVote,
		},
		{
			name:        "non-slashable vote",
			signingRoot: []byte{},
			attestation: createAttestation(numEpochs, numEpochs+1),
			want:        NotSlashable,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			slashingKind, err := validatorDB.CheckSlashableAttestation(ctx, pubKeys[0], tt.signingRoot, tt.attestation)
			if tt.want != NotSlashable {
				require.NotNil(t, err)
			}
			assert.Equal(t, tt.want, slashingKind)
		})
	}
}

func TestLowestSignedSourceEpoch_SaveRetrieve(t *testing.T) {
	ctx := context.Background()
	validatorDB, err := NewKVStore(ctx, t.TempDir(), &Config{})
	require.NoError(t, err, "Failed to instantiate DB")
	t.Cleanup(func() {
		require.NoError(t, validatorDB.Close(), "Failed to close database")
		require.NoError(t, validatorDB.ClearDB(), "Failed to clear database")
	})
	p0 := [fieldparams.BLSPubkeyLength]byte{0}
	p1 := [fieldparams.BLSPubkeyLength]byte{1}
	// Can save.
	require.NoError(
		t,
		validatorDB.SaveAttestationForPubKey(ctx, p0, [32]byte{}, createAttestation(100, 101)),
	)
	require.NoError(
		t,
		validatorDB.SaveAttestationForPubKey(ctx, p1, [32]byte{}, createAttestation(200, 201)),
	)
	got, _, err := validatorDB.LowestSignedSourceEpoch(ctx, p0)
	require.NoError(t, err)
	require.Equal(t, primitives.Epoch(100), got)
	got, _, err = validatorDB.LowestSignedSourceEpoch(ctx, p1)
	require.NoError(t, err)
	require.Equal(t, primitives.Epoch(200), got)

	// Can replace.
	require.NoError(
		t,
		validatorDB.SaveAttestationForPubKey(ctx, p0, [32]byte{}, createAttestation(99, 100)),
	)
	require.NoError(
		t,
		validatorDB.SaveAttestationForPubKey(ctx, p1, [32]byte{}, createAttestation(199, 200)),
	)
	got, _, err = validatorDB.LowestSignedSourceEpoch(ctx, p0)
	require.NoError(t, err)
	require.Equal(t, primitives.Epoch(99), got)
	got, _, err = validatorDB.LowestSignedSourceEpoch(ctx, p1)
	require.NoError(t, err)
	require.Equal(t, primitives.Epoch(199), got)

	// Can not replace.
	require.NoError(
		t,
		validatorDB.SaveAttestationForPubKey(ctx, p0, [32]byte{}, createAttestation(100, 101)),
	)
	require.NoError(
		t,
		validatorDB.SaveAttestationForPubKey(ctx, p1, [32]byte{}, createAttestation(200, 201)),
	)
	got, _, err = validatorDB.LowestSignedSourceEpoch(ctx, p0)
	require.NoError(t, err)
	require.Equal(t, primitives.Epoch(99), got)
	got, _, err = validatorDB.LowestSignedSourceEpoch(ctx, p1)
	require.NoError(t, err)
	require.Equal(t, primitives.Epoch(199), got)
}

func TestLowestSignedTargetEpoch_SaveRetrieveReplace(t *testing.T) {
	ctx := context.Background()
	validatorDB, err := NewKVStore(ctx, t.TempDir(), &Config{})
	require.NoError(t, err, "Failed to instantiate DB")
	t.Cleanup(func() {
		require.NoError(t, validatorDB.Close(), "Failed to close database")
		require.NoError(t, validatorDB.ClearDB(), "Failed to clear database")
	})
	p0 := [fieldparams.BLSPubkeyLength]byte{0}
	p1 := [fieldparams.BLSPubkeyLength]byte{1}
	// Can save.
	require.NoError(
		t,
		validatorDB.SaveAttestationForPubKey(ctx, p0, [32]byte{}, createAttestation(99, 100)),
	)
	require.NoError(
		t,
		validatorDB.SaveAttestationForPubKey(ctx, p1, [32]byte{}, createAttestation(199, 200)),
	)
	got, _, err := validatorDB.LowestSignedTargetEpoch(ctx, p0)
	require.NoError(t, err)
	require.Equal(t, primitives.Epoch(100), got)
	got, _, err = validatorDB.LowestSignedTargetEpoch(ctx, p1)
	require.NoError(t, err)
	require.Equal(t, primitives.Epoch(200), got)

	// Can replace.
	require.NoError(
		t,
		validatorDB.SaveAttestationForPubKey(ctx, p0, [32]byte{}, createAttestation(98, 99)),
	)
	require.NoError(
		t,
		validatorDB.SaveAttestationForPubKey(ctx, p1, [32]byte{}, createAttestation(198, 199)),
	)
	got, _, err = validatorDB.LowestSignedTargetEpoch(ctx, p0)
	require.NoError(t, err)
	require.Equal(t, primitives.Epoch(99), got)
	got, _, err = validatorDB.LowestSignedTargetEpoch(ctx, p1)
	require.NoError(t, err)
	require.Equal(t, primitives.Epoch(199), got)

	// Can not replace.
	require.NoError(
		t,
		validatorDB.SaveAttestationForPubKey(ctx, p0, [32]byte{}, createAttestation(99, 100)),
	)
	require.NoError(
		t,
		validatorDB.SaveAttestationForPubKey(ctx, p1, [32]byte{}, createAttestation(199, 200)),
	)
	got, _, err = validatorDB.LowestSignedTargetEpoch(ctx, p0)
	require.NoError(t, err)
	require.Equal(t, primitives.Epoch(99), got)
	got, _, err = validatorDB.LowestSignedTargetEpoch(ctx, p1)
	require.NoError(t, err)
	require.Equal(t, primitives.Epoch(199), got)
}

func TestStore_SaveAttestationsForPubKey(t *testing.T) {
	ctx := context.Background()
	numValidators := 1
	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, numValidators)
	validatorDB := setupDB(t, pubKeys)
	atts := make([]*ethpb.IndexedAttestation, 0)
	signingRoots := make([][]byte, 0)
	for i := primitives.Epoch(1); i < 10; i++ {
		atts = append(atts, createAttestation(i-1, i))
		var sr []byte
		copy(sr, fmt.Sprintf("%d", i))
		signingRoots = append(signingRoots, sr)
	}
	err := validatorDB.SaveAttestationsForPubKey(
		ctx,
		pubKeys[0],
		signingRoots[:1],
		atts,
	)
	require.ErrorContains(t, "does not match number of attestations", err)
	err = validatorDB.SaveAttestationsForPubKey(
		ctx,
		pubKeys[0],
		signingRoots,
		atts,
	)
	require.NoError(t, err)
	for _, att := range atts {
		// Ensure the same attestations but different signing root lead to double votes.
		slashingKind, err := validatorDB.CheckSlashableAttestation(
			ctx,
			pubKeys[0],
			[]byte{},
			att,
		)
		require.NotNil(t, err)
		require.Equal(t, DoubleVote, slashingKind)
	}
}

func TestSaveAttestationForPubKey_BatchWrites_FullCapacity(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	numValidators := attestationBatchCapacity
	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, numValidators)
	validatorDB := setupDB(t, pubKeys)

	// For each public key, we attempt to save an attestation with signing root.
	var wg sync.WaitGroup
	for i, pubKey := range pubKeys {
		wg.Add(1)
		go func(j primitives.Epoch, pk [fieldparams.BLSPubkeyLength]byte, w *sync.WaitGroup) {
			defer w.Done()
			var signingRoot [32]byte
			copy(signingRoot[:], fmt.Sprintf("%d", j))
			att := createAttestation(j, j+1)
			err := validatorDB.SaveAttestationForPubKey(ctx, pk, signingRoot, att)
			require.NoError(t, err)
		}(primitives.Epoch(i), pubKey, &wg)
	}
	wg.Wait()

	// We verify that we reached the max capacity of batched attestations
	// before we are required to force flush them to the DB.
	require.LogsContain(t, hook, "Reached max capacity of batched attestation records")
	require.LogsDoNotContain(t, hook, "Batched attestation records write interval reached")
	require.LogsContain(t, hook, "Successfully flushed batched attestations to DB")
	require.Equal(t, 0, validatorDB.batchedAttestations.Len())

	// We then verify all the data we wanted to save is indeed saved to disk.
	err := validatorDB.view(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(pubKeysBucket)
		for i, pubKey := range pubKeys {
			var signingRoot [32]byte
			copy(signingRoot[:], fmt.Sprintf("%d", i))
			pkBucket := bucket.Bucket(pubKey[:])
			signingRootsBucket := pkBucket.Bucket(attestationSigningRootsBucket)
			sourceEpochsBucket := pkBucket.Bucket(attestationSourceEpochsBucket)

			source := bytesutil.Uint64ToBytesBigEndian(uint64(i))
			target := bytesutil.Uint64ToBytesBigEndian(uint64(i) + 1)
			savedSigningRoot := signingRootsBucket.Get(target)
			require.DeepEqual(t, signingRoot[:], savedSigningRoot)
			savedTarget := sourceEpochsBucket.Get(source)
			require.DeepEqual(t, signingRoot[:], savedSigningRoot)
			require.DeepEqual(t, target, savedTarget)
		}
		return nil
	})
	require.NoError(t, err)
}

func TestSaveAttestationForPubKey_BatchWrites_LowCapacity_TimerReached(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	// Number of validators equal to half the total capacity
	// of batch attestation processing. This will allow us to
	// test force flushing to the DB based on a timer instead
	// of the max capacity being reached.
	numValidators := attestationBatchCapacity / 2
	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, numValidators)
	validatorDB := setupDB(t, pubKeys)

	// For each public key, we attempt to save an attestation with signing root.
	var wg sync.WaitGroup
	for i, pubKey := range pubKeys {
		wg.Add(1)
		go func(j primitives.Epoch, pk [fieldparams.BLSPubkeyLength]byte, w *sync.WaitGroup) {
			defer w.Done()
			var signingRoot [32]byte
			copy(signingRoot[:], fmt.Sprintf("%d", j))
			att := createAttestation(j, j+1)
			err := validatorDB.SaveAttestationForPubKey(ctx, pk, signingRoot, att)
			require.NoError(t, err)
		}(primitives.Epoch(i), pubKey, &wg)
	}
	wg.Wait()

	// We verify that we reached a timer interval for force flushing records
	// before we are required to force flush them to the DB.
	require.LogsDoNotContain(t, hook, "Reached max capacity of batched attestation records")
	require.LogsContain(t, hook, "Batched attestation records write interval reached")
	require.LogsContain(t, hook, "Successfully flushed batched attestations to DB")
	require.Equal(t, 0, validatorDB.batchedAttestations.Len())

	// We then verify all the data we wanted to save is indeed saved to disk.
	err := validatorDB.view(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(pubKeysBucket)
		for i, pubKey := range pubKeys {
			var signingRoot [32]byte
			copy(signingRoot[:], fmt.Sprintf("%d", i))
			pkBucket := bucket.Bucket(pubKey[:])
			signingRootsBucket := pkBucket.Bucket(attestationSigningRootsBucket)
			sourceEpochsBucket := pkBucket.Bucket(attestationSourceEpochsBucket)

			source := bytesutil.Uint64ToBytesBigEndian(uint64(i))
			target := bytesutil.Uint64ToBytesBigEndian(uint64(i) + 1)
			savedSigningRoot := signingRootsBucket.Get(target)
			require.DeepEqual(t, signingRoot[:], savedSigningRoot)
			savedTarget := sourceEpochsBucket.Get(source)
			require.DeepEqual(t, signingRoot[:], savedSigningRoot)
			require.DeepEqual(t, target, savedTarget)
		}
		return nil
	})
	require.NoError(t, err)
}

func BenchmarkStore_CheckSlashableAttestation_Surround_SafeAttestation_54kEpochs(b *testing.B) {
	numValidators := 1
	numEpochs := primitives.Epoch(54000)
	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, numValidators)
	benchCheckSurroundVote(b, pubKeys, numEpochs, false /* surround */)
}

func BenchmarkStore_CheckSurroundVote_Surround_Slashable_54kEpochs(b *testing.B) {
	numValidators := 1
	numEpochs := primitives.Epoch(54000)
	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, numValidators)
	benchCheckSurroundVote(b, pubKeys, numEpochs, true /* surround */)
}

func benchCheckSurroundVote(
	b *testing.B,
	pubKeys [][fieldparams.BLSPubkeyLength]byte,
	numEpochs primitives.Epoch,
	shouldSurround bool,
) {
	ctx := context.Background()
	validatorDB, err := NewKVStore(ctx, filepath.Join(b.TempDir(), "benchsurroundvote"), &Config{
		PubKeys: pubKeys,
	})
	require.NoError(b, err, "Failed to instantiate DB")
	defer func() {
		require.NoError(b, validatorDB.Close(), "Failed to close database")
		require.NoError(b, validatorDB.ClearDB(), "Failed to clear database")
	}()
	// Every validator will have attested every (source, target) sequential pair
	// since genesis up to and including the weak subjectivity period epoch (54,000).
	err = validatorDB.update(func(tx *bolt.Tx) error {
		for _, pubKey := range pubKeys {
			bucket := tx.Bucket(pubKeysBucket)
			pkBucket, err := bucket.CreateBucketIfNotExists(pubKey[:])
			if err != nil {
				return err
			}
			sourceEpochsBucket, err := pkBucket.CreateBucketIfNotExists(attestationSourceEpochsBucket)
			if err != nil {
				return err
			}
			for epoch := primitives.Epoch(1); epoch < numEpochs; epoch++ {
				att := createAttestation(epoch-1, epoch)
				sourceEpoch := bytesutil.EpochToBytesBigEndian(att.Data.Source.Epoch)
				targetEpoch := bytesutil.EpochToBytesBigEndian(att.Data.Target.Epoch)
				if err := sourceEpochsBucket.Put(sourceEpoch, targetEpoch); err != nil {
					return err
				}
			}
		}
		return nil
	})
	require.NoError(b, err)

	// Will surround many attestations.
	var surroundingVote *ethpb.IndexedAttestation
	if shouldSurround {
		surroundingVote = createAttestation(numEpochs/2, numEpochs)
	} else {
		surroundingVote = createAttestation(numEpochs+1, numEpochs+2)
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		for _, pubKey := range pubKeys {
			slashingKind, err := validatorDB.CheckSlashableAttestation(ctx, pubKey, []byte{}, surroundingVote)
			if shouldSurround {
				require.NotNil(b, err)
				assert.Equal(b, SurroundingVote, slashingKind)
			} else {
				require.NoError(b, err)
			}
		}
	}
}

func createAttestation(source, target primitives.Epoch) *ethpb.IndexedAttestation {
	return &ethpb.IndexedAttestation{
		Data: &ethpb.AttestationData{
			Source: &ethpb.Checkpoint{
				Epoch: source,
			},
			Target: &ethpb.Checkpoint{
				Epoch: target,
			},
		},
	}
}

func TestStore_flushAttestationRecords_InProgress(t *testing.T) {
	s := &Store{}
	s.batchedAttestationsFlushInProgress.Set()

	hook := logTest.NewGlobal()
	s.flushAttestationRecords(context.Background(), nil)
	assert.LogsContain(t, hook, "Attempted to flush attestation records when already in progress")
}
