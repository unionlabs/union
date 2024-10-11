package monitor

import (
	"bytes"
	"context"
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/sirupsen/logrus"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestGetAttestingIndices(t *testing.T) {
	ctx := context.Background()
	beaconState, _ := util.DeterministicGenesisState(t, 256)
	att := &ethpb.Attestation{
		Data: &ethpb.AttestationData{
			Slot:           1,
			CommitteeIndex: 0,
		},
		AggregationBits: bitfield.Bitlist{0b11, 0b1},
	}
	attestingIndices, err := attestingIndices(ctx, beaconState, att)
	require.NoError(t, err)
	require.DeepEqual(t, attestingIndices, []uint64{0xc, 0x2})

}

func TestProcessIncludedAttestationTwoTracked(t *testing.T) {
	hook := logTest.NewGlobal()
	s := setupService(t)
	state, _ := util.DeterministicGenesisStateAltair(t, 256)
	require.NoError(t, state.SetSlot(2))
	require.NoError(t, state.SetCurrentParticipationBits(bytes.Repeat([]byte{0xff}, 13)))

	att := &ethpb.Attestation{
		Data: &ethpb.AttestationData{
			Slot:            1,
			CommitteeIndex:  0,
			BeaconBlockRoot: bytesutil.PadTo([]byte("hello-world"), 32),
			Source: &ethpb.Checkpoint{
				Epoch: 0,
				Root:  bytesutil.PadTo([]byte("hello-world"), 32),
			},
			Target: &ethpb.Checkpoint{
				Epoch: 1,
				Root:  bytesutil.PadTo([]byte("hello-world"), 32),
			},
		},
		AggregationBits: bitfield.Bitlist{0b11, 0b1},
	}
	s.processIncludedAttestation(context.Background(), state, att)
	wanted1 := "\"Attestation included\" BalanceChange=0 CorrectHead=true CorrectSource=true CorrectTarget=true Head=0x68656c6c6f2d InclusionSlot=2 NewBalance=32000000000 Slot=1 Source=0x68656c6c6f2d Target=0x68656c6c6f2d ValidatorIndex=2 prefix=monitor"
	wanted2 := "\"Attestation included\" BalanceChange=100000000 CorrectHead=true CorrectSource=true CorrectTarget=true Head=0x68656c6c6f2d InclusionSlot=2 NewBalance=32000000000 Slot=1 Source=0x68656c6c6f2d Target=0x68656c6c6f2d ValidatorIndex=12 prefix=monitor"
	require.LogsContain(t, hook, wanted1)
	require.LogsContain(t, hook, wanted2)
}

func TestProcessUnaggregatedAttestationStateNotCached(t *testing.T) {
	logrus.SetLevel(logrus.DebugLevel)
	hook := logTest.NewGlobal()
	ctx := context.Background()

	s := setupService(t)
	state, _ := util.DeterministicGenesisStateAltair(t, 256)
	require.NoError(t, state.SetSlot(2))
	header := state.LatestBlockHeader()
	participation := []byte{0xff, 0xff, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}
	require.NoError(t, state.SetCurrentParticipationBits(participation))

	att := &ethpb.Attestation{
		Data: &ethpb.AttestationData{
			Slot:            1,
			CommitteeIndex:  0,
			BeaconBlockRoot: header.GetStateRoot(),
			Source: &ethpb.Checkpoint{
				Epoch: 0,
				Root:  bytesutil.PadTo([]byte("hello-world"), 32),
			},
			Target: &ethpb.Checkpoint{
				Epoch: 1,
				Root:  bytesutil.PadTo([]byte("hello-world"), 32),
			},
		},
		AggregationBits: bitfield.Bitlist{0b11, 0b1},
	}
	s.processUnaggregatedAttestation(ctx, att)
	require.LogsContain(t, hook, "Skipping unaggregated attestation due to state not found in cache")
	logrus.SetLevel(logrus.InfoLevel)
}

func TestProcessUnaggregatedAttestationStateCached(t *testing.T) {
	ctx := context.Background()
	hook := logTest.NewGlobal()

	s := setupService(t)
	state, _ := util.DeterministicGenesisStateAltair(t, 256)
	participation := []byte{0xff, 0xff, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}
	require.NoError(t, state.SetCurrentParticipationBits(participation))

	var root [32]byte
	copy(root[:], "hello-world")

	att := &ethpb.Attestation{
		Data: &ethpb.AttestationData{
			Slot:            1,
			CommitteeIndex:  0,
			BeaconBlockRoot: root[:],
			Source: &ethpb.Checkpoint{
				Epoch: 0,
				Root:  root[:],
			},
			Target: &ethpb.Checkpoint{
				Epoch: 1,
				Root:  root[:],
			},
		},
		AggregationBits: bitfield.Bitlist{0b11, 0b1},
	}
	require.NoError(t, s.config.StateGen.SaveState(ctx, root, state))
	s.processUnaggregatedAttestation(context.Background(), att)
	wanted1 := "\"Processed unaggregated attestation\" Head=0x68656c6c6f2d Slot=1 Source=0x68656c6c6f2d Target=0x68656c6c6f2d ValidatorIndex=2 prefix=monitor"
	wanted2 := "\"Processed unaggregated attestation\" Head=0x68656c6c6f2d Slot=1 Source=0x68656c6c6f2d Target=0x68656c6c6f2d ValidatorIndex=12 prefix=monitor"
	require.LogsContain(t, hook, wanted1)
	require.LogsContain(t, hook, wanted2)
}

func TestProcessAggregatedAttestationStateNotCached(t *testing.T) {
	logrus.SetLevel(logrus.DebugLevel)
	hook := logTest.NewGlobal()
	ctx := context.Background()

	s := setupService(t)
	state, _ := util.DeterministicGenesisStateAltair(t, 256)
	require.NoError(t, state.SetSlot(2))
	header := state.LatestBlockHeader()
	participation := []byte{0xff, 0xff, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}
	require.NoError(t, state.SetCurrentParticipationBits(participation))

	att := &ethpb.AggregateAttestationAndProof{
		AggregatorIndex: 2,
		Aggregate: &ethpb.Attestation{
			Data: &ethpb.AttestationData{
				Slot:            1,
				CommitteeIndex:  0,
				BeaconBlockRoot: header.GetStateRoot(),
				Source: &ethpb.Checkpoint{
					Epoch: 0,
					Root:  bytesutil.PadTo([]byte("hello-world"), 32),
				},
				Target: &ethpb.Checkpoint{
					Epoch: 1,
					Root:  bytesutil.PadTo([]byte("hello-world"), 32),
				},
			},
			AggregationBits: bitfield.Bitlist{0b11, 0b1},
		},
	}
	s.processAggregatedAttestation(ctx, att)
	require.LogsContain(t, hook, "\"Processed attestation aggregation\" AggregatorIndex=2 BeaconBlockRoot=0x000000000000 Slot=1 SourceRoot=0x68656c6c6f2d TargetRoot=0x68656c6c6f2d prefix=monitor")
	require.LogsContain(t, hook, "Skipping aggregated attestation due to state not found in cache")
	logrus.SetLevel(logrus.InfoLevel)
}

func TestProcessAggregatedAttestationStateCached(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	s := setupService(t)
	state, _ := util.DeterministicGenesisStateAltair(t, 256)
	participation := []byte{0xff, 0xff, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}
	require.NoError(t, state.SetCurrentParticipationBits(participation))

	var root [32]byte
	copy(root[:], "hello-world")

	att := &ethpb.AggregateAttestationAndProof{
		AggregatorIndex: 2,
		Aggregate: &ethpb.Attestation{
			Data: &ethpb.AttestationData{
				Slot:            1,
				CommitteeIndex:  0,
				BeaconBlockRoot: root[:],
				Source: &ethpb.Checkpoint{
					Epoch: 0,
					Root:  root[:],
				},
				Target: &ethpb.Checkpoint{
					Epoch: 1,
					Root:  root[:],
				},
			},
			AggregationBits: bitfield.Bitlist{0b10, 0b1},
		},
	}

	require.NoError(t, s.config.StateGen.SaveState(ctx, root, state))
	s.processAggregatedAttestation(ctx, att)
	require.LogsContain(t, hook, "\"Processed attestation aggregation\" AggregatorIndex=2 BeaconBlockRoot=0x68656c6c6f2d Slot=1 SourceRoot=0x68656c6c6f2d TargetRoot=0x68656c6c6f2d prefix=monitor")
	require.LogsContain(t, hook, "\"Processed aggregated attestation\" Head=0x68656c6c6f2d Slot=1 Source=0x68656c6c6f2d Target=0x68656c6c6f2d ValidatorIndex=2 prefix=monitor")
	require.LogsDoNotContain(t, hook, "\"Processed aggregated attestation\" Head=0x68656c6c6f2d Slot=1 Source=0x68656c6c6f2d Target=0x68656c6c6f2d ValidatorIndex=12 prefix=monitor")
}

func TestProcessAttestations(t *testing.T) {
	hook := logTest.NewGlobal()
	s := setupService(t)
	ctx := context.Background()
	state, _ := util.DeterministicGenesisStateAltair(t, 256)
	require.NoError(t, state.SetSlot(2))
	require.NoError(t, state.SetCurrentParticipationBits(bytes.Repeat([]byte{0xff}, 13)))

	att := &ethpb.Attestation{
		Data: &ethpb.AttestationData{
			Slot:            1,
			CommitteeIndex:  0,
			BeaconBlockRoot: bytesutil.PadTo([]byte("hello-world"), 32),
			Source: &ethpb.Checkpoint{
				Epoch: 0,
				Root:  bytesutil.PadTo([]byte("hello-world"), 32),
			},
			Target: &ethpb.Checkpoint{
				Epoch: 1,
				Root:  bytesutil.PadTo([]byte("hello-world"), 32),
			},
		},
		AggregationBits: bitfield.Bitlist{0b11, 0b1},
	}

	block := &ethpb.BeaconBlockAltair{
		Slot: 2,
		Body: &ethpb.BeaconBlockBodyAltair{
			Attestations: []*ethpb.Attestation{att},
		},
	}

	wrappedBlock, err := blocks.NewBeaconBlock(block)
	require.NoError(t, err)
	s.processAttestations(ctx, state, wrappedBlock)
	wanted1 := "\"Attestation included\" BalanceChange=0 CorrectHead=true CorrectSource=true CorrectTarget=true Head=0x68656c6c6f2d InclusionSlot=2 NewBalance=32000000000 Slot=1 Source=0x68656c6c6f2d Target=0x68656c6c6f2d ValidatorIndex=2 prefix=monitor"
	wanted2 := "\"Attestation included\" BalanceChange=100000000 CorrectHead=true CorrectSource=true CorrectTarget=true Head=0x68656c6c6f2d InclusionSlot=2 NewBalance=32000000000 Slot=1 Source=0x68656c6c6f2d Target=0x68656c6c6f2d ValidatorIndex=12 prefix=monitor"
	require.LogsContain(t, hook, wanted1)
	require.LogsContain(t, hook, wanted2)

}
