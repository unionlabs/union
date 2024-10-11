package state_native

import (
	"testing"

	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestBeaconState_PreviousEpochAttestations(t *testing.T) {
	s, err := InitializeFromProtoPhase0(&ethpb.BeaconState{})
	require.NoError(t, err)
	atts, err := s.PreviousEpochAttestations()
	require.NoError(t, err)
	require.DeepEqual(t, []*ethpb.PendingAttestation(nil), atts)

	want := []*ethpb.PendingAttestation{{ProposerIndex: 100}}
	s, err = InitializeFromProtoPhase0(&ethpb.BeaconState{PreviousEpochAttestations: want})
	require.NoError(t, err)
	got, err := s.PreviousEpochAttestations()
	require.NoError(t, err)
	require.DeepEqual(t, want, got)

	// Test copy does not mutate.
	got[0].ProposerIndex = 101
	require.DeepNotEqual(t, want, got)
}

func TestBeaconState_CurrentEpochAttestations(t *testing.T) {
	s, err := InitializeFromProtoPhase0(&ethpb.BeaconState{})
	require.NoError(t, err)
	atts, err := s.CurrentEpochAttestations()
	require.NoError(t, err)
	require.DeepEqual(t, []*ethpb.PendingAttestation(nil), atts)

	want := []*ethpb.PendingAttestation{{ProposerIndex: 101}}
	s, err = InitializeFromProtoPhase0(&ethpb.BeaconState{CurrentEpochAttestations: want})
	require.NoError(t, err)
	got, err := s.CurrentEpochAttestations()
	require.NoError(t, err)
	require.DeepEqual(t, want, got)

	// Test copy does not mutate.
	got[0].ProposerIndex = 102
	require.DeepNotEqual(t, want, got)
}
