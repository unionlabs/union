package kv

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestStateSummary_CanSaveRetrieve(t *testing.T) {
	db := setupDB(t)
	ctx := context.Background()
	r1 := bytesutil.ToBytes32([]byte{'A'})
	r2 := bytesutil.ToBytes32([]byte{'B'})
	s1 := &ethpb.StateSummary{Slot: 1, Root: r1[:]}

	// State summary should not exist yet.
	require.Equal(t, false, db.HasStateSummary(ctx, r1), "State summary should not be saved")
	require.NoError(t, db.SaveStateSummary(ctx, s1))
	require.Equal(t, true, db.HasStateSummary(ctx, r1), "State summary should be saved")

	saved, err := db.StateSummary(ctx, r1)
	require.NoError(t, err)
	assert.DeepEqual(t, s1, saved, "State summary does not equal")

	// Save a new state summary.
	s2 := &ethpb.StateSummary{Slot: 2, Root: r2[:]}

	// State summary should not exist yet.
	require.Equal(t, false, db.HasStateSummary(ctx, r2), "State summary should not be saved")
	require.NoError(t, db.SaveStateSummary(ctx, s2))
	require.Equal(t, true, db.HasStateSummary(ctx, r2), "State summary should be saved")

	saved, err = db.StateSummary(ctx, r2)
	require.NoError(t, err)
	assert.DeepEqual(t, s2, saved, "State summary does not equal")
}

func TestStateSummary_CacheToDB(t *testing.T) {
	db := setupDB(t)

	summaries := make([]*ethpb.StateSummary, stateSummaryCachePruneCount-1)
	for i := range summaries {
		summaries[i] = &ethpb.StateSummary{Slot: primitives.Slot(i), Root: bytesutil.PadTo(bytesutil.Uint64ToBytesLittleEndian(uint64(i)), 32)}
	}

	require.NoError(t, db.SaveStateSummaries(context.Background(), summaries))
	require.Equal(t, db.stateSummaryCache.len(), stateSummaryCachePruneCount-1)

	require.NoError(t, db.SaveStateSummary(context.Background(), &ethpb.StateSummary{Slot: 1000, Root: []byte{'a', 'b'}}))
	require.Equal(t, db.stateSummaryCache.len(), stateSummaryCachePruneCount)

	require.NoError(t, db.SaveStateSummary(context.Background(), &ethpb.StateSummary{Slot: 1001, Root: []byte{'c', 'd'}}))
	require.Equal(t, db.stateSummaryCache.len(), 1)

	for i := range summaries {
		r := bytesutil.Uint64ToBytesLittleEndian(uint64(i))
		require.Equal(t, true, db.HasStateSummary(context.Background(), bytesutil.ToBytes32(r)))
	}
}

func TestStateSummary_CanDelete(t *testing.T) {
	db := setupDB(t)
	ctx := context.Background()
	r1 := bytesutil.ToBytes32([]byte{'A'})
	s1 := &ethpb.StateSummary{Slot: 1, Root: r1[:]}

	require.Equal(t, false, db.HasStateSummary(ctx, r1), "State summary should not be saved")
	require.NoError(t, db.SaveStateSummary(ctx, s1))
	require.Equal(t, true, db.HasStateSummary(ctx, r1), "State summary should be saved")

	require.NoError(t, db.deleteStateSummary(r1))
	require.Equal(t, false, db.HasStateSummary(ctx, r1), "State summary should be deleted")
}
