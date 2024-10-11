package synccommittee

import (
	"testing"

	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestSyncCommitteeSignatureCache_Nil(t *testing.T) {
	store := NewStore()
	require.Equal(t, errNilMessage, store.SaveSyncCommitteeMessage(nil))
}

func TestSyncCommitteeSignatureCache_RoundTrip(t *testing.T) {
	store := NewStore()

	msgs := []*ethpb.SyncCommitteeMessage{
		{Slot: 1, ValidatorIndex: 0, Signature: []byte{'a'}},
		{Slot: 1, ValidatorIndex: 1, Signature: []byte{'b'}},
		{Slot: 2, ValidatorIndex: 0, Signature: []byte{'c'}},
		{Slot: 2, ValidatorIndex: 1, Signature: []byte{'d'}},
		{Slot: 3, ValidatorIndex: 0, Signature: []byte{'e'}},
		{Slot: 3, ValidatorIndex: 1, Signature: []byte{'f'}},
		{Slot: 4, ValidatorIndex: 0, Signature: []byte{'g'}},
		{Slot: 4, ValidatorIndex: 1, Signature: []byte{'h'}},
		{Slot: 5, ValidatorIndex: 0, Signature: []byte{'i'}},
		{Slot: 5, ValidatorIndex: 1, Signature: []byte{'j'}},
		{Slot: 6, ValidatorIndex: 0, Signature: []byte{'k'}},
		{Slot: 6, ValidatorIndex: 1, Signature: []byte{'l'}},
	}

	for _, msg := range msgs {
		require.NoError(t, store.SaveSyncCommitteeMessage(msg))
	}

	msgs, err := store.SyncCommitteeMessages(1)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeMessage(nil), msgs)

	msgs, err = store.SyncCommitteeMessages(2)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeMessage(nil), msgs)

	msgs, err = store.SyncCommitteeMessages(3)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeMessage{
		{Slot: 3, ValidatorIndex: 0, Signature: []byte{'e'}},
		{Slot: 3, ValidatorIndex: 1, Signature: []byte{'f'}},
	}, msgs)

	msgs, err = store.SyncCommitteeMessages(4)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeMessage{
		{Slot: 4, ValidatorIndex: 0, Signature: []byte{'g'}},
		{Slot: 4, ValidatorIndex: 1, Signature: []byte{'h'}},
	}, msgs)

	msgs, err = store.SyncCommitteeMessages(5)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeMessage{
		{Slot: 5, ValidatorIndex: 0, Signature: []byte{'i'}},
		{Slot: 5, ValidatorIndex: 1, Signature: []byte{'j'}},
	}, msgs)

	msgs, err = store.SyncCommitteeMessages(6)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeMessage{
		{Slot: 6, ValidatorIndex: 0, Signature: []byte{'k'}},
		{Slot: 6, ValidatorIndex: 1, Signature: []byte{'l'}},
	}, msgs)

	// Messages should persist after retrieval.
	msgs, err = store.SyncCommitteeMessages(1)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeMessage(nil), msgs)

	msgs, err = store.SyncCommitteeMessages(2)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeMessage(nil), msgs)

	msgs, err = store.SyncCommitteeMessages(3)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeMessage{
		{Slot: 3, ValidatorIndex: 0, Signature: []byte{'e'}},
		{Slot: 3, ValidatorIndex: 1, Signature: []byte{'f'}},
	}, msgs)

	msgs, err = store.SyncCommitteeMessages(4)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeMessage{
		{Slot: 4, ValidatorIndex: 0, Signature: []byte{'g'}},
		{Slot: 4, ValidatorIndex: 1, Signature: []byte{'h'}},
	}, msgs)

	msgs, err = store.SyncCommitteeMessages(5)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeMessage{
		{Slot: 5, ValidatorIndex: 0, Signature: []byte{'i'}},
		{Slot: 5, ValidatorIndex: 1, Signature: []byte{'j'}},
	}, msgs)

	msgs, err = store.SyncCommitteeMessages(6)
	require.NoError(t, err)
	require.DeepSSZEqual(t, []*ethpb.SyncCommitteeMessage{
		{Slot: 6, ValidatorIndex: 0, Signature: []byte{'k'}},
		{Slot: 6, ValidatorIndex: 1, Signature: []byte{'l'}},
	}, msgs)
}
