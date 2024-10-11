package slasher

import (
	"context"
	"fmt"
	"testing"
	"time"

	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	dbtest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	slashingsmock "github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/slashings/mock"
	slashertypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/slasher/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func Test_processQueuedAttestations(t *testing.T) {
	type args struct {
		attestationQueue []*slashertypes.IndexedAttestationWrapper
		currentEpoch     primitives.Epoch
	}
	tests := []struct {
		name                 string
		args                 args
		shouldNotBeSlashable bool
	}{
		{
			name: "Detects surrounding vote (source 1, target 2), (source 0, target 3)",
			args: args{
				attestationQueue: []*slashertypes.IndexedAttestationWrapper{
					createAttestationWrapper(t, 1, 2, []uint64{0, 1}, nil),
					createAttestationWrapper(t, 0, 3, []uint64{0, 1}, nil),
				},
				currentEpoch: 4,
			},
		},
		{
			name: "Detects surrounding vote (source 50, target 51), (source 0, target 1000)",
			args: args{
				attestationQueue: []*slashertypes.IndexedAttestationWrapper{
					createAttestationWrapper(t, 50, 51, []uint64{0}, nil),
					createAttestationWrapper(t, 0, 1000, []uint64{0}, nil),
				},
				currentEpoch: 1000,
			},
		},
		{
			name: "Detects surrounded vote (source 0, target 3), (source 1, target 2)",
			args: args{
				attestationQueue: []*slashertypes.IndexedAttestationWrapper{
					createAttestationWrapper(t, 0, 3, []uint64{0, 1}, nil),
					createAttestationWrapper(t, 1, 2, []uint64{0, 1}, nil),
				},
				currentEpoch: 4,
			},
		},
		{
			name: "Detects double vote, (source 1, target 2), (source 0, target 2)",
			args: args{
				attestationQueue: []*slashertypes.IndexedAttestationWrapper{
					createAttestationWrapper(t, 1, 2, []uint64{0, 1}, nil),
					createAttestationWrapper(t, 0, 2, []uint64{0, 1}, nil),
				},
				currentEpoch: 4,
			},
		},
		{
			name: "Not slashable, surrounding but non-overlapping attesting indices within same validator chunk index",
			args: args{
				attestationQueue: []*slashertypes.IndexedAttestationWrapper{
					createAttestationWrapper(t, 1, 2, []uint64{0}, nil),
					createAttestationWrapper(t, 0, 3, []uint64{1}, nil),
				},
				currentEpoch: 4,
			},
			shouldNotBeSlashable: true,
		},
		{
			name: "Not slashable, surrounded but non-overlapping attesting indices within same validator chunk index",
			args: args{
				attestationQueue: []*slashertypes.IndexedAttestationWrapper{
					createAttestationWrapper(t, 0, 3, []uint64{0, 1}, nil),
					createAttestationWrapper(t, 1, 2, []uint64{2, 3}, nil),
				},
				currentEpoch: 4,
			},
			shouldNotBeSlashable: true,
		},
		{
			name: "Not slashable, surrounding but non-overlapping attesting indices in different validator chunk index",
			args: args{
				attestationQueue: []*slashertypes.IndexedAttestationWrapper{
					createAttestationWrapper(t, 0, 3, []uint64{0}, nil),
					createAttestationWrapper(
						t,
						1,
						2,
						[]uint64{params.BeaconConfig().MinGenesisActiveValidatorCount - 1},
						nil,
					),
				},
				currentEpoch: 4,
			},
			shouldNotBeSlashable: true,
		},
		{
			name: "Not slashable, surrounded but non-overlapping attesting indices in different validator chunk index",
			args: args{
				attestationQueue: []*slashertypes.IndexedAttestationWrapper{
					createAttestationWrapper(t, 0, 3, []uint64{0}, nil),
					createAttestationWrapper(
						t,
						1,
						2,
						[]uint64{params.BeaconConfig().MinGenesisActiveValidatorCount - 1},
						nil,
					),
				},
				currentEpoch: 4,
			},
			shouldNotBeSlashable: true,
		},
		{
			name: "Not slashable, (source 1, target 2), (source 2, target 3)",
			args: args{
				attestationQueue: []*slashertypes.IndexedAttestationWrapper{
					createAttestationWrapper(t, 1, 2, []uint64{0, 1}, nil),
					createAttestationWrapper(t, 2, 3, []uint64{0, 1}, nil),
				},
				currentEpoch: 4,
			},
			shouldNotBeSlashable: true,
		},
		{
			name: "Not slashable, (source 0, target 3), (source 2, target 4)",
			args: args{
				attestationQueue: []*slashertypes.IndexedAttestationWrapper{
					createAttestationWrapper(t, 0, 3, []uint64{0, 1}, nil),
					createAttestationWrapper(t, 2, 4, []uint64{0, 1}, nil),
				},
				currentEpoch: 4,
			},
			shouldNotBeSlashable: true,
		},
		{
			name: "Not slashable, (source 0, target 2), (source 0, target 3)",
			args: args{
				attestationQueue: []*slashertypes.IndexedAttestationWrapper{
					createAttestationWrapper(t, 0, 2, []uint64{0, 1}, nil),
					createAttestationWrapper(t, 0, 3, []uint64{0, 1}, nil),
				},
				currentEpoch: 4,
			},
			shouldNotBeSlashable: true,
		},
		{
			name: "Not slashable, (source 0, target 3), (source 0, target 2)",
			args: args{
				attestationQueue: []*slashertypes.IndexedAttestationWrapper{
					createAttestationWrapper(t, 0, 3, []uint64{0, 1}, nil),
					createAttestationWrapper(t, 0, 2, []uint64{0, 1}, nil),
				},
				currentEpoch: 4,
			},
			shouldNotBeSlashable: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			hook := logTest.NewGlobal()
			defer hook.Reset()
			slasherDB := dbtest.SetupSlasherDB(t)
			ctx, cancel := context.WithCancel(context.Background())

			currentTime := time.Now()
			totalSlots := uint64(tt.args.currentEpoch) * uint64(params.BeaconConfig().SlotsPerEpoch)
			secondsSinceGenesis := time.Duration(totalSlots * params.BeaconConfig().SecondsPerSlot)
			genesisTime := currentTime.Add(-secondsSinceGenesis * time.Second)

			beaconState, err := util.NewBeaconState()
			require.NoError(t, err)
			slot, err := slots.EpochStart(tt.args.currentEpoch)
			require.NoError(t, err)
			require.NoError(t, beaconState.SetSlot(slot))
			mockChain := &mock.ChainService{
				State: beaconState,
				Slot:  &slot,
			}

			// Initialize validators in the state.
			numVals := params.BeaconConfig().MinGenesisActiveValidatorCount
			validators := make([]*ethpb.Validator, numVals)
			privKeys := make([]bls.SecretKey, numVals)
			for i := range validators {
				privKey, err := bls.RandKey()
				require.NoError(t, err)
				privKeys[i] = privKey
				validators[i] = &ethpb.Validator{
					PublicKey:             privKey.PublicKey().Marshal(),
					WithdrawalCredentials: make([]byte, 32),
				}
			}
			err = beaconState.SetValidators(validators)
			require.NoError(t, err)
			domain, err := signing.Domain(
				beaconState.Fork(),
				0,
				params.BeaconConfig().DomainBeaconAttester,
				beaconState.GenesisValidatorsRoot(),
			)
			require.NoError(t, err)

			// Create valid signatures for all input attestations in the test.
			for _, attestationWrapper := range tt.args.attestationQueue {
				signingRoot, err := signing.ComputeSigningRoot(attestationWrapper.IndexedAttestation.Data, domain)
				require.NoError(t, err)
				attestingIndices := attestationWrapper.IndexedAttestation.AttestingIndices
				sigs := make([]bls.Signature, len(attestingIndices))
				for i, validatorIndex := range attestingIndices {
					privKey := privKeys[validatorIndex]
					sigs[i] = privKey.Sign(signingRoot[:])
				}
				attestationWrapper.IndexedAttestation.Signature = bls.AggregateSignatures(sigs).Marshal()
			}

			s, err := New(context.Background(),
				&ServiceConfig{
					Database:                slasherDB,
					StateNotifier:           &mock.MockStateNotifier{},
					HeadStateFetcher:        mockChain,
					AttestationStateFetcher: mockChain,
					SlashingPoolInserter:    &slashingsmock.PoolMock{},
					ClockWaiter:             startup.NewClockSynchronizer(),
				})
			require.NoError(t, err)
			s.genesisTime = genesisTime

			currentSlotChan := make(chan primitives.Slot)
			exitChan := make(chan struct{})
			go func() {
				s.processQueuedAttestations(ctx, currentSlotChan)
				exitChan <- struct{}{}
			}()
			s.attsQueue.extend(tt.args.attestationQueue)
			currentSlotChan <- slot
			time.Sleep(time.Millisecond * 200)
			cancel()
			<-exitChan
			if tt.shouldNotBeSlashable {
				require.LogsDoNotContain(t, hook, "Attester slashing detected")
			} else {
				require.LogsContain(t, hook, "Attester slashing detected")
			}
		})
	}
}

func Test_processQueuedAttestations_MultipleChunkIndices(t *testing.T) {
	hook := logTest.NewGlobal()
	defer hook.Reset()

	slasherDB := dbtest.SetupSlasherDB(t)
	ctx, cancel := context.WithCancel(context.Background())
	slasherParams := DefaultParams()

	// We process submit attestations from chunk index 0 to chunk index 1.
	// What we want to test here is if we can proceed
	// with processing queued attestations once the chunk index changes.
	// For example, epochs 0 - 15 are chunk 0, epochs 16 - 31 are chunk 1, etc.
	startEpoch := primitives.Epoch(slasherParams.chunkSize)
	endEpoch := primitives.Epoch(slasherParams.chunkSize + 1)

	currentTime := time.Now()
	totalSlots := uint64(startEpoch) * uint64(params.BeaconConfig().SlotsPerEpoch)
	secondsSinceGenesis := time.Duration(totalSlots * params.BeaconConfig().SecondsPerSlot)
	genesisTime := currentTime.Add(-secondsSinceGenesis * time.Second)

	beaconState, err := util.NewBeaconState()
	require.NoError(t, err)
	mockChain := &mock.ChainService{
		State: beaconState,
	}

	s, err := New(context.Background(),
		&ServiceConfig{
			Database:                slasherDB,
			StateNotifier:           &mock.MockStateNotifier{},
			HeadStateFetcher:        mockChain,
			AttestationStateFetcher: mockChain,
			SlashingPoolInserter:    &slashingsmock.PoolMock{},
			ClockWaiter:             startup.NewClockSynchronizer(),
		})
	require.NoError(t, err)
	s.genesisTime = genesisTime

	currentSlotChan := make(chan primitives.Slot)
	exitChan := make(chan struct{})
	go func() {
		s.processQueuedAttestations(ctx, currentSlotChan)
		exitChan <- struct{}{}
	}()

	for i := startEpoch; i <= endEpoch; i++ {
		source := primitives.Epoch(0)
		target := primitives.Epoch(0)
		if i != 0 {
			source = i - 1
			target = i
		}
		var sr [32]byte
		copy(sr[:], fmt.Sprintf("%d", i))
		att := createAttestationWrapper(t, source, target, []uint64{0}, sr[:])
		s.attsQueue = newAttestationsQueue()
		s.attsQueue.push(att)
		slot, err := slots.EpochStart(i)
		require.NoError(t, err)
		require.NoError(t, mockChain.State.SetSlot(slot))
		s.serviceCfg.HeadStateFetcher = mockChain
		currentSlotChan <- slot
	}

	time.Sleep(time.Millisecond * 200)
	cancel()
	<-exitChan
	require.LogsDoNotContain(t, hook, "Slashable offenses found")
	require.LogsDoNotContain(t, hook, "Could not detect")
}

func Test_processQueuedAttestations_OverlappingChunkIndices(t *testing.T) {
	hook := logTest.NewGlobal()
	defer hook.Reset()

	slasherDB := dbtest.SetupSlasherDB(t)
	ctx, cancel := context.WithCancel(context.Background())
	slasherParams := DefaultParams()

	startEpoch := primitives.Epoch(slasherParams.chunkSize)

	currentTime := time.Now()
	totalSlots := uint64(startEpoch) * uint64(params.BeaconConfig().SlotsPerEpoch)
	secondsSinceGenesis := time.Duration(totalSlots * params.BeaconConfig().SecondsPerSlot)
	genesisTime := currentTime.Add(-secondsSinceGenesis * time.Second)

	beaconState, err := util.NewBeaconState()
	require.NoError(t, err)
	mockChain := &mock.ChainService{
		State: beaconState,
	}

	s, err := New(context.Background(),
		&ServiceConfig{
			Database:                slasherDB,
			StateNotifier:           &mock.MockStateNotifier{},
			HeadStateFetcher:        mockChain,
			AttestationStateFetcher: mockChain,
			SlashingPoolInserter:    &slashingsmock.PoolMock{},
			ClockWaiter:             startup.NewClockSynchronizer(),
		})
	require.NoError(t, err)
	s.genesisTime = genesisTime

	currentSlotChan := make(chan primitives.Slot)
	exitChan := make(chan struct{})
	go func() {
		s.processQueuedAttestations(ctx, currentSlotChan)
		exitChan <- struct{}{}
	}()

	// We create two attestations fully spanning chunk indices 0 and chunk 1
	att1 := createAttestationWrapper(t, primitives.Epoch(slasherParams.chunkSize-2), primitives.Epoch(slasherParams.chunkSize), []uint64{0, 1}, nil)
	att2 := createAttestationWrapper(t, primitives.Epoch(slasherParams.chunkSize-1), primitives.Epoch(slasherParams.chunkSize+1), []uint64{0, 1}, nil)

	// We attempt to process the batch.
	s.attsQueue = newAttestationsQueue()
	s.attsQueue.push(att1)
	s.attsQueue.push(att2)
	slot, err := slots.EpochStart(att2.IndexedAttestation.Data.Target.Epoch)
	require.NoError(t, err)
	mockChain.Slot = &slot
	s.serviceCfg.HeadStateFetcher = mockChain
	currentSlotChan <- slot

	time.Sleep(time.Millisecond * 200)
	cancel()
	<-exitChan
	require.LogsDoNotContain(t, hook, "Slashable offenses found")
	require.LogsDoNotContain(t, hook, "Could not detect")
}

func Test_epochUpdateForValidators(t *testing.T) {
	ctx := context.Background()
	slasherDB := dbtest.SetupSlasherDB(t)

	// Check if the chunk at chunk index already exists in-memory.
	s := &Service{
		params: &Parameters{
			chunkSize:          2, // 2 epochs in a chunk.
			validatorChunkSize: 2, // 2 validators in a chunk.
			historyLength:      4,
		},
		serviceCfg:                     &ServiceConfig{Database: slasherDB},
		latestEpochWrittenForValidator: map[primitives.ValidatorIndex]primitives.Epoch{},
	}

	t.Run("no update if no latest written epoch", func(t *testing.T) {
		validators := []primitives.ValidatorIndex{
			1, 2,
		}
		currentEpoch := primitives.Epoch(3)
		// No last written epoch for both validators.
		s.latestEpochWrittenForValidator = map[primitives.ValidatorIndex]primitives.Epoch{}

		// Because the validators have no recorded latest epoch written, we expect
		// no chunks to be loaded nor updated to.
		updatedChunks := make(map[uint64]Chunker)
		for _, valIdx := range validators {
			err := s.epochUpdateForValidator(
				ctx,
				&chunkUpdateArgs{
					currentEpoch: currentEpoch,
				},
				updatedChunks,
				valIdx,
			)
			require.NoError(t, err)
		}
		require.Equal(t, 0, len(updatedChunks))
	})

	t.Run("update from latest written epoch", func(t *testing.T) {
		validators := []primitives.ValidatorIndex{
			1, 2,
		}
		currentEpoch := primitives.Epoch(3)

		// Set the latest written epoch for validators to current epoch - 1.
		latestWrittenEpoch := currentEpoch - 1
		s.latestEpochWrittenForValidator = map[primitives.ValidatorIndex]primitives.Epoch{
			1: latestWrittenEpoch,
			2: latestWrittenEpoch,
		}

		// Because the latest written epoch for the input validators is == 2, we expect
		// that we will update all epochs from 2 up to 3 (the current epoch). This is all
		// safe contained in chunk index 1.
		updatedChunks := make(map[uint64]Chunker)
		for _, valIdx := range validators {
			err := s.epochUpdateForValidator(
				ctx,
				&chunkUpdateArgs{
					currentEpoch: currentEpoch,
				},
				updatedChunks,
				valIdx,
			)
			require.NoError(t, err)
		}
		require.Equal(t, 1, len(updatedChunks))
		_, ok := updatedChunks[1]
		require.Equal(t, true, ok)
	})
}

func Test_applyAttestationForValidator_MinSpanChunk(t *testing.T) {
	ctx := context.Background()
	slasherDB := dbtest.SetupSlasherDB(t)
	defaultParams := DefaultParams()
	srv, err := New(context.Background(),
		&ServiceConfig{
			Database:      slasherDB,
			StateNotifier: &mock.MockStateNotifier{},
			ClockWaiter:   startup.NewClockSynchronizer(),
		})
	require.NoError(t, err)

	// We initialize an empty chunks slice.
	chunk := EmptyMinSpanChunksSlice(defaultParams)
	chunkIdx := uint64(0)
	currentEpoch := primitives.Epoch(3)
	validatorIdx := primitives.ValidatorIndex(0)
	args := &chunkUpdateArgs{
		chunkIndex:   chunkIdx,
		currentEpoch: currentEpoch,
	}
	chunksByChunkIdx := map[uint64]Chunker{
		chunkIdx: chunk,
	}

	// We apply attestation with (source 1, target 2) for our validator.
	source := primitives.Epoch(1)
	target := primitives.Epoch(2)
	att := createAttestationWrapper(t, source, target, nil, nil)
	slashing, err := srv.applyAttestationForValidator(
		ctx,
		args,
		validatorIdx,
		chunksByChunkIdx,
		att,
	)
	require.NoError(t, err)
	require.Equal(t, true, slashing == nil)
	att.IndexedAttestation.AttestingIndices = []uint64{uint64(validatorIdx)}
	err = slasherDB.SaveAttestationRecordsForValidators(
		ctx,
		[]*slashertypes.IndexedAttestationWrapper{att},
	)
	require.NoError(t, err)

	// Next, we apply an attestation with (source 0, target 3) and
	// expect a slashable offense to be returned.
	source = primitives.Epoch(0)
	target = primitives.Epoch(3)
	slashableAtt := createAttestationWrapper(t, source, target, nil, nil)
	slashing, err = srv.applyAttestationForValidator(
		ctx,
		args,
		validatorIdx,
		chunksByChunkIdx,
		slashableAtt,
	)
	require.NoError(t, err)
	require.NotNil(t, slashing)
}

func Test_applyAttestationForValidator_MaxSpanChunk(t *testing.T) {
	ctx := context.Background()
	slasherDB := dbtest.SetupSlasherDB(t)
	defaultParams := DefaultParams()
	srv, err := New(context.Background(),
		&ServiceConfig{
			Database:      slasherDB,
			StateNotifier: &mock.MockStateNotifier{},
			ClockWaiter:   startup.NewClockSynchronizer(),
		})
	require.NoError(t, err)

	// We initialize an empty chunks slice.
	chunk := EmptyMaxSpanChunksSlice(defaultParams)
	chunkIdx := uint64(0)
	currentEpoch := primitives.Epoch(3)
	validatorIdx := primitives.ValidatorIndex(0)
	args := &chunkUpdateArgs{
		chunkIndex:   chunkIdx,
		currentEpoch: currentEpoch,
	}
	chunksByChunkIdx := map[uint64]Chunker{
		chunkIdx: chunk,
	}

	// We apply attestation with (source 0, target 3) for our validator.
	source := primitives.Epoch(0)
	target := primitives.Epoch(3)
	att := createAttestationWrapper(t, source, target, nil, nil)
	slashing, err := srv.applyAttestationForValidator(
		ctx,
		args,
		validatorIdx,
		chunksByChunkIdx,
		att,
	)
	require.NoError(t, err)
	require.Equal(t, true, slashing == nil)
	att.IndexedAttestation.AttestingIndices = []uint64{uint64(validatorIdx)}
	err = slasherDB.SaveAttestationRecordsForValidators(
		ctx,
		[]*slashertypes.IndexedAttestationWrapper{att},
	)
	require.NoError(t, err)

	// Next, we apply an attestation with (source 1, target 2) and
	// expect a slashable offense to be returned.
	source = primitives.Epoch(1)
	target = primitives.Epoch(2)
	slashableAtt := createAttestationWrapper(t, source, target, nil, nil)
	slashing, err = srv.applyAttestationForValidator(
		ctx,
		args,
		validatorIdx,
		chunksByChunkIdx,
		slashableAtt,
	)
	require.NoError(t, err)
	require.NotNil(t, slashing)
}

func Test_checkDoubleVotes_SlashableInputAttestations(t *testing.T) {
	slasherDB := dbtest.SetupSlasherDB(t)
	ctx := context.Background()
	// For a list of input attestations, check that we can
	// indeed check there could exist a double vote offense
	// within the list with respect to other entries in the list.
	atts := []*slashertypes.IndexedAttestationWrapper{
		createAttestationWrapper(t, 0, 1, []uint64{1, 2}, []byte{1}),
		createAttestationWrapper(t, 0, 2, []uint64{1, 2}, []byte{1}),
		createAttestationWrapper(t, 0, 2, []uint64{1, 2}, []byte{2}), // Different signing root.
	}
	srv, err := New(context.Background(),
		&ServiceConfig{
			Database:      slasherDB,
			StateNotifier: &mock.MockStateNotifier{},
			ClockWaiter:   startup.NewClockSynchronizer(),
		})
	require.NoError(t, err)

	prev1 := createAttestationWrapper(t, 0, 2, []uint64{1, 2}, []byte{1})
	cur1 := createAttestationWrapper(t, 0, 2, []uint64{1, 2}, []byte{2})
	prev2 := createAttestationWrapper(t, 0, 2, []uint64{1, 2}, []byte{1})
	cur2 := createAttestationWrapper(t, 0, 2, []uint64{1, 2}, []byte{2})
	wanted := []*ethpb.AttesterSlashing{
		{
			Attestation_1: prev1.IndexedAttestation,
			Attestation_2: cur1.IndexedAttestation,
		},
		{
			Attestation_1: prev2.IndexedAttestation,
			Attestation_2: cur2.IndexedAttestation,
		},
	}
	slashings, err := srv.checkDoubleVotes(ctx, atts)
	require.NoError(t, err)
	require.DeepEqual(t, wanted, slashings)
}

func Test_checkDoubleVotes_SlashableAttestationsOnDisk(t *testing.T) {
	slasherDB := dbtest.SetupSlasherDB(t)
	ctx := context.Background()
	// For a list of input attestations, check that we can
	// indeed check there could exist a double vote offense
	// within the list with respect to previous entries in the db.
	prevAtts := []*slashertypes.IndexedAttestationWrapper{
		createAttestationWrapper(t, 0, 1, []uint64{1, 2}, []byte{1}),
		createAttestationWrapper(t, 0, 2, []uint64{1, 2}, []byte{1}),
	}
	srv, err := New(context.Background(),
		&ServiceConfig{
			Database:      slasherDB,
			StateNotifier: &mock.MockStateNotifier{},
			ClockWaiter:   startup.NewClockSynchronizer(),
		})
	require.NoError(t, err)

	err = slasherDB.SaveAttestationRecordsForValidators(ctx, prevAtts)
	require.NoError(t, err)

	prev1 := createAttestationWrapper(t, 0, 2, []uint64{1, 2}, []byte{1})
	cur1 := createAttestationWrapper(t, 0, 2, []uint64{1, 2}, []byte{2})
	prev2 := createAttestationWrapper(t, 0, 2, []uint64{1, 2}, []byte{1})
	cur2 := createAttestationWrapper(t, 0, 2, []uint64{1, 2}, []byte{2})
	wanted := []*ethpb.AttesterSlashing{
		{
			Attestation_1: prev1.IndexedAttestation,
			Attestation_2: cur1.IndexedAttestation,
		},
		{
			Attestation_1: prev2.IndexedAttestation,
			Attestation_2: cur2.IndexedAttestation,
		},
	}
	newAtts := []*slashertypes.IndexedAttestationWrapper{
		createAttestationWrapper(t, 0, 2, []uint64{1, 2}, []byte{2}), // Different signing root.
	}
	slashings, err := srv.checkDoubleVotes(ctx, newAtts)
	require.NoError(t, err)
	require.DeepEqual(t, wanted, slashings)
}

func Test_loadChunks_MinSpans(t *testing.T) {
	testLoadChunks(t, slashertypes.MinSpan)
}

func Test_loadChunks_MaxSpans(t *testing.T) {
	testLoadChunks(t, slashertypes.MaxSpan)
}

func testLoadChunks(t *testing.T, kind slashertypes.ChunkKind) {
	slasherDB := dbtest.SetupSlasherDB(t)
	ctx := context.Background()

	// Check if the chunk at chunk index already exists in-memory.
	s, err := New(context.Background(),
		&ServiceConfig{
			Database:      slasherDB,
			StateNotifier: &mock.MockStateNotifier{},
			ClockWaiter:   startup.NewClockSynchronizer(),
		})
	require.NoError(t, err)

	defaultParams := s.params

	// If a chunk at a chunk index does not exist, ensure it
	// is initialized as an empty chunk.
	var emptyChunk Chunker
	if kind == slashertypes.MinSpan {
		emptyChunk = EmptyMinSpanChunksSlice(defaultParams)
	} else {
		emptyChunk = EmptyMaxSpanChunksSlice(defaultParams)
	}
	chunkIdx := uint64(2)
	received, err := s.loadChunks(ctx, &chunkUpdateArgs{
		validatorChunkIndex: 0,
		kind:                kind,
	}, []uint64{chunkIdx})
	require.NoError(t, err)
	wanted := map[uint64]Chunker{
		chunkIdx: emptyChunk,
	}
	require.DeepEqual(t, wanted, received)

	// Save chunks to disk, then load them properly from disk.
	var existingChunk Chunker
	if kind == slashertypes.MinSpan {
		existingChunk = EmptyMinSpanChunksSlice(defaultParams)
	} else {
		existingChunk = EmptyMaxSpanChunksSlice(defaultParams)
	}
	validatorIdx := primitives.ValidatorIndex(0)
	epochInChunk := primitives.Epoch(0)
	targetEpoch := primitives.Epoch(2)
	err = setChunkDataAtEpoch(
		defaultParams,
		existingChunk.Chunk(),
		validatorIdx,
		epochInChunk,
		targetEpoch,
	)
	require.NoError(t, err)
	require.DeepNotEqual(t, existingChunk, emptyChunk)

	updatedChunks := map[uint64]Chunker{
		2: existingChunk,
		4: existingChunk,
		6: existingChunk,
	}
	err = s.saveUpdatedChunks(
		ctx,
		&chunkUpdateArgs{
			validatorChunkIndex: 0,
			kind:                kind,
		},
		updatedChunks,
	)
	require.NoError(t, err)
	// Check if the retrieved chunks match what we just saved to disk.
	received, err = s.loadChunks(ctx, &chunkUpdateArgs{
		validatorChunkIndex: 0,
		kind:                kind,
	}, []uint64{2, 4, 6})
	require.NoError(t, err)
	require.DeepEqual(t, updatedChunks, received)
}

func TestService_processQueuedAttestations(t *testing.T) {
	hook := logTest.NewGlobal()
	slasherDB := dbtest.SetupSlasherDB(t)

	beaconState, err := util.NewBeaconState()
	require.NoError(t, err)
	slot, err := slots.EpochStart(1)
	require.NoError(t, err)
	require.NoError(t, beaconState.SetSlot(slot))
	mockChain := &mock.ChainService{
		State: beaconState,
		Slot:  &slot,
	}

	s, err := New(context.Background(),
		&ServiceConfig{
			Database:         slasherDB,
			StateNotifier:    &mock.MockStateNotifier{},
			HeadStateFetcher: mockChain,
			ClockWaiter:      startup.NewClockSynchronizer(),
		})
	require.NoError(t, err)

	s.attsQueue.extend([]*slashertypes.IndexedAttestationWrapper{
		createAttestationWrapper(t, 0, 1, []uint64{0, 1} /* indices */, nil /* signingRoot */),
	})
	ctx, cancel := context.WithCancel(context.Background())
	tickerChan := make(chan primitives.Slot)
	exitChan := make(chan struct{})
	go func() {
		s.processQueuedAttestations(ctx, tickerChan)
		exitChan <- struct{}{}
	}()

	// Send a value over the ticker.
	tickerChan <- 1
	cancel()
	<-exitChan
	assert.LogsContain(t, hook, "Processing queued")
}

func BenchmarkCheckSlashableAttestations(b *testing.B) {
	slasherDB := dbtest.SetupSlasherDB(b)

	beaconState, err := util.NewBeaconState()
	require.NoError(b, err)
	slot := primitives.Slot(0)
	mockChain := &mock.ChainService{
		State: beaconState,
		Slot:  &slot,
	}

	s, err := New(context.Background(), &ServiceConfig{
		Database:         slasherDB,
		StateNotifier:    &mock.MockStateNotifier{},
		HeadStateFetcher: mockChain,
		ClockWaiter:      startup.NewClockSynchronizer(),
	})
	require.NoError(b, err)

	b.Run("1 attestation 1 validator", func(b *testing.B) {
		b.ResetTimer()
		runAttestationsBenchmark(b, s, 1, 1 /* validator */)
	})
	b.Run("1 attestation 100 validators", func(b *testing.B) {
		b.ResetTimer()
		runAttestationsBenchmark(b, s, 1, 100 /* validator */)
	})
	b.Run("1 attestation 1000 validators", func(b *testing.B) {
		b.ResetTimer()
		runAttestationsBenchmark(b, s, 1, 1000 /* validator */)
	})

	b.Run("100 attestations 1 validator", func(b *testing.B) {
		b.ResetTimer()
		runAttestationsBenchmark(b, s, 100, 1 /* validator */)
	})
	b.Run("100 attestations 100 validators", func(b *testing.B) {
		b.ResetTimer()
		runAttestationsBenchmark(b, s, 100, 100 /* validator */)
	})
	b.Run("100 attestations 1000 validators", func(b *testing.B) {
		b.ResetTimer()
		runAttestationsBenchmark(b, s, 100, 1000 /* validator */)
	})

	b.Run("1000 attestations 1 validator", func(b *testing.B) {
		b.ResetTimer()
		runAttestationsBenchmark(b, s, 1000, 1 /* validator */)
	})
	b.Run("1000 attestations 100 validators", func(b *testing.B) {
		b.ResetTimer()
		runAttestationsBenchmark(b, s, 1000, 100 /* validator */)
	})
	b.Run("1000 attestations 1000 validators", func(b *testing.B) {
		b.ResetTimer()
		runAttestationsBenchmark(b, s, 1000, 1000 /* validator */)
	})
}

func runAttestationsBenchmark(b *testing.B, s *Service, numAtts, numValidators uint64) {
	indices := make([]uint64, numValidators)
	for i := uint64(0); i < numValidators; i++ {
		indices[i] = i
	}
	atts := make([]*slashertypes.IndexedAttestationWrapper, numAtts)
	for i := uint64(0); i < numAtts; i++ {
		source := primitives.Epoch(i)
		target := primitives.Epoch(i + 1)
		var signingRoot [32]byte
		copy(signingRoot[:], fmt.Sprintf("%d", i))
		atts[i] = createAttestationWrapper(
			b,
			source,
			target,         /* target */
			indices,        /* indices */
			signingRoot[:], /* signingRoot */
		)
	}
	for i := 0; i < b.N; i++ {
		numEpochs := numAtts
		totalSeconds := numEpochs * uint64(params.BeaconConfig().SlotsPerEpoch) * params.BeaconConfig().SecondsPerSlot
		genesisTime := time.Now().Add(-time.Second * time.Duration(totalSeconds))
		s.genesisTime = genesisTime

		epoch := slots.EpochsSinceGenesis(genesisTime)
		_, err := s.checkSlashableAttestations(context.Background(), epoch, atts)
		require.NoError(b, err)
	}
}

func createAttestationWrapper(t testing.TB, source, target primitives.Epoch, indices []uint64, signingRoot []byte) *slashertypes.IndexedAttestationWrapper {
	data := &ethpb.AttestationData{
		BeaconBlockRoot: bytesutil.PadTo(signingRoot, 32),
		Source: &ethpb.Checkpoint{
			Epoch: source,
			Root:  params.BeaconConfig().ZeroHash[:],
		},
		Target: &ethpb.Checkpoint{
			Epoch: target,
			Root:  params.BeaconConfig().ZeroHash[:],
		},
	}
	signRoot, err := data.HashTreeRoot()
	if err != nil {
		t.Fatal(err)
	}
	return &slashertypes.IndexedAttestationWrapper{
		IndexedAttestation: &ethpb.IndexedAttestation{
			AttestingIndices: indices,
			Data:             data,
			Signature:        params.BeaconConfig().EmptySignature[:],
		},
		SigningRoot: signRoot,
	}
}
