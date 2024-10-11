package synccommittee

import (
	"testing"

	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestSyncCommitteeContributionCache_Nil(t *testing.T) {
	store := NewStore()
	require.Equal(t, errNilContribution, store.SaveSyncCommitteeContribution(nil))
}

func TestSyncCommitteeContributionCache_RoundTrip(t *testing.T) {
	store := NewStore()

	conts := []*ethpb.SyncCommitteeContribution{
		{Slot: 1, SubcommitteeIndex: 0, Signature: []byte{'a'}},
		{Slot: 1, SubcommitteeIndex: 1, Signature: []byte{'b'}},
		{Slot: 2, SubcommitteeIndex: 0, Signature: []byte{'c'}},
		{Slot: 2, SubcommitteeIndex: 1, Signature: []byte{'d'}},
		{Slot: 3, SubcommitteeIndex: 0, Signature: []byte{'e'}},
		{Slot: 3, SubcommitteeIndex: 1, Signature: []byte{'f'}},
		{Slot: 4, SubcommitteeIndex: 0, Signature: []byte{'g'}},
		{Slot: 4, SubcommitteeIndex: 1, Signature: []byte{'h'}},
		{Slot: 5, SubcommitteeIndex: 0, Signature: []byte{'i'}},
		{Slot: 5, SubcommitteeIndex: 1, Signature: []byte{'j'}},
		{Slot: 6, SubcommitteeIndex: 0, Signature: []byte{'k'}},
		{Slot: 6, SubcommitteeIndex: 1, Signature: []byte{'l'}},
	}

	for _, sig := range conts {
		require.NoError(t, store.SaveSyncCommitteeContribution(sig))
	}

	conts, err := store.SyncCommitteeContributions(1)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeContribution{}, conts)

	conts, err = store.SyncCommitteeContributions(2)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeContribution{}, conts)

	conts, err = store.SyncCommitteeContributions(3)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeContribution{
		{Slot: 3, SubcommitteeIndex: 0, Signature: []byte{'e'}},
		{Slot: 3, SubcommitteeIndex: 1, Signature: []byte{'f'}},
	}, conts)

	conts, err = store.SyncCommitteeContributions(4)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeContribution{
		{Slot: 4, SubcommitteeIndex: 0, Signature: []byte{'g'}},
		{Slot: 4, SubcommitteeIndex: 1, Signature: []byte{'h'}},
	}, conts)

	conts, err = store.SyncCommitteeContributions(5)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeContribution{
		{Slot: 5, SubcommitteeIndex: 0, Signature: []byte{'i'}},
		{Slot: 5, SubcommitteeIndex: 1, Signature: []byte{'j'}},
	}, conts)

	conts, err = store.SyncCommitteeContributions(6)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeContribution{
		{Slot: 6, SubcommitteeIndex: 0, Signature: []byte{'k'}},
		{Slot: 6, SubcommitteeIndex: 1, Signature: []byte{'l'}},
	}, conts)

	// All the contributions should persist after get.
	conts, err = store.SyncCommitteeContributions(1)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeContribution{}, conts)
	conts, err = store.SyncCommitteeContributions(2)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeContribution{}, conts)

	conts, err = store.SyncCommitteeContributions(3)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeContribution{
		{Slot: 3, SubcommitteeIndex: 0, Signature: []byte{'e'}},
		{Slot: 3, SubcommitteeIndex: 1, Signature: []byte{'f'}},
	}, conts)

	conts, err = store.SyncCommitteeContributions(4)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeContribution{
		{Slot: 4, SubcommitteeIndex: 0, Signature: []byte{'g'}},
		{Slot: 4, SubcommitteeIndex: 1, Signature: []byte{'h'}},
	}, conts)

	conts, err = store.SyncCommitteeContributions(5)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeContribution{
		{Slot: 5, SubcommitteeIndex: 0, Signature: []byte{'i'}},
		{Slot: 5, SubcommitteeIndex: 1, Signature: []byte{'j'}},
	}, conts)

	conts, err = store.SyncCommitteeContributions(6)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeContribution{
		{Slot: 6, SubcommitteeIndex: 0, Signature: []byte{'k'}},
		{Slot: 6, SubcommitteeIndex: 1, Signature: []byte{'l'}},
	}, conts)
}
