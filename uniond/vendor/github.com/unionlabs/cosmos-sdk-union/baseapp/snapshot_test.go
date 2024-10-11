package baseapp_test

import (
	"context"
	"fmt"
	"testing"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	"github.com/stretchr/testify/require"

	pruningtypes "cosmossdk.io/store/pruning/types"
	snapshottypes "cosmossdk.io/store/snapshots/types"
)

func TestABCI_ListSnapshots(t *testing.T) {
	ssCfg := SnapshotsConfig{
		blocks:             5,
		blockTxs:           4,
		snapshotInterval:   2,
		snapshotKeepRecent: 2,
		pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
	}

	suite := NewBaseAppSuiteWithSnapshots(t, ssCfg)

	resp, err := suite.baseApp.ListSnapshots(&abci.ListSnapshotsRequest{})
	require.NoError(t, err)
	for _, s := range resp.Snapshots {
		require.NotEmpty(t, s.Hash)
		require.NotEmpty(t, s.Metadata)

		s.Hash = nil
		s.Metadata = nil
	}

	require.Equal(t, &abci.ListSnapshotsResponse{Snapshots: []*abci.Snapshot{
		{Height: 4, Format: snapshottypes.CurrentFormat, Chunks: 2},
		{Height: 2, Format: snapshottypes.CurrentFormat, Chunks: 1},
	}}, resp)
}

func TestABCI_SnapshotWithPruning(t *testing.T) {
	testCases := map[string]struct {
		ssCfg             SnapshotsConfig
		expectedSnapshots []*abci.Snapshot
	}{
		"prune nothing with snapshot": {
			ssCfg: SnapshotsConfig{
				blocks:             20,
				blockTxs:           2,
				snapshotInterval:   5,
				snapshotKeepRecent: 1,
				pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
			},
			expectedSnapshots: []*abci.Snapshot{
				{Height: 20, Format: snapshottypes.CurrentFormat, Chunks: 5},
			},
		},
		"prune everything with snapshot": {
			ssCfg: SnapshotsConfig{
				blocks:             20,
				blockTxs:           2,
				snapshotInterval:   5,
				snapshotKeepRecent: 1,
				pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningEverything),
			},
			expectedSnapshots: []*abci.Snapshot{
				{Height: 20, Format: snapshottypes.CurrentFormat, Chunks: 5},
			},
		},
		"default pruning with snapshot": {
			ssCfg: SnapshotsConfig{
				blocks:             20,
				blockTxs:           2,
				snapshotInterval:   5,
				snapshotKeepRecent: 1,
				pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningDefault),
			},
			expectedSnapshots: []*abci.Snapshot{
				{Height: 20, Format: snapshottypes.CurrentFormat, Chunks: 5},
			},
		},
		"custom": {
			ssCfg: SnapshotsConfig{
				blocks:             25,
				blockTxs:           2,
				snapshotInterval:   5,
				snapshotKeepRecent: 2,
				pruningOpts:        pruningtypes.NewCustomPruningOptions(12, 12),
			},
			expectedSnapshots: []*abci.Snapshot{
				{Height: 25, Format: snapshottypes.CurrentFormat, Chunks: 6},
				{Height: 20, Format: snapshottypes.CurrentFormat, Chunks: 5},
			},
		},
		"no snapshots": {
			ssCfg: SnapshotsConfig{
				blocks:           10,
				blockTxs:         2,
				snapshotInterval: 0, // 0 implies disable snapshots
				pruningOpts:      pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
			},
			expectedSnapshots: []*abci.Snapshot{},
		},
		"keep all snapshots": {
			ssCfg: SnapshotsConfig{
				blocks:             10,
				blockTxs:           2,
				snapshotInterval:   3,
				snapshotKeepRecent: 0, // 0 implies keep all snapshots
				pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
			},
			expectedSnapshots: []*abci.Snapshot{
				{Height: 9, Format: snapshottypes.CurrentFormat, Chunks: 2},
				{Height: 6, Format: snapshottypes.CurrentFormat, Chunks: 2},
				{Height: 3, Format: snapshottypes.CurrentFormat, Chunks: 1},
			},
		},
	}

	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			suite := NewBaseAppSuiteWithSnapshots(t, tc.ssCfg)

			resp, err := suite.baseApp.ListSnapshots(&abci.ListSnapshotsRequest{})
			require.NoError(t, err)
			for _, s := range resp.Snapshots {
				require.NotEmpty(t, s.Hash)
				require.NotEmpty(t, s.Metadata)

				s.Hash = nil
				s.Metadata = nil
			}

			require.Equal(t, &abci.ListSnapshotsResponse{Snapshots: tc.expectedSnapshots}, resp)

			// Validate that heights were pruned correctly by querying the state at the last height that should be present relative to latest
			// and the first height that should be pruned.
			//
			// Exceptions:
			//   * Prune nothing: should be able to query all heights (we only test first and latest)
			//   * Prune default: should be able to query all heights (we only test first and latest)
			//      * The reason for default behaving this way is that we only commit 20 heights but default has 100_000 keep-recent
			var lastExistingHeight int64
			if tc.ssCfg.pruningOpts.GetPruningStrategy() == pruningtypes.PruningNothing || tc.ssCfg.pruningOpts.GetPruningStrategy() == pruningtypes.PruningDefault {
				lastExistingHeight = 1
			} else {
				// Integer division rounds down so by multiplying back we get the last height at which we pruned
				lastExistingHeight = int64((tc.ssCfg.blocks/tc.ssCfg.pruningOpts.Interval)*tc.ssCfg.pruningOpts.Interval - tc.ssCfg.pruningOpts.KeepRecent)
			}

			// Query 1
			res, err := suite.baseApp.Query(context.TODO(), &abci.QueryRequest{Path: fmt.Sprintf("/store/%s/key", capKey2.Name()), Data: []byte("0"), Height: lastExistingHeight})
			require.NoError(t, err)
			require.NotNil(t, res, "height: %d", lastExistingHeight)
			require.NotNil(t, res.Value, "height: %d", lastExistingHeight)

			// Query 2
			res, err = suite.baseApp.Query(context.TODO(), &abci.QueryRequest{Path: fmt.Sprintf("/store/%s/key", capKey2.Name()), Data: []byte("0"), Height: lastExistingHeight - 1})
			require.NoError(t, err)
			require.NotNil(t, res, "height: %d", lastExistingHeight-1)

			if tc.ssCfg.pruningOpts.GetPruningStrategy() == pruningtypes.PruningNothing || tc.ssCfg.pruningOpts.GetPruningStrategy() == pruningtypes.PruningDefault {
				// With prune nothing or default, we query height 0 which translates to the latest height.
				require.NotNil(t, res.Value, "height: %d", lastExistingHeight-1)
			}
		})
	}
}

func TestABCI_LoadSnapshotChunk(t *testing.T) {
	ssCfg := SnapshotsConfig{
		blocks:             2,
		blockTxs:           5,
		snapshotInterval:   2,
		snapshotKeepRecent: snapshottypes.CurrentFormat,
		pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
	}
	suite := NewBaseAppSuiteWithSnapshots(t, ssCfg)

	testCases := map[string]struct {
		height      uint64
		format      uint32
		chunk       uint32
		expectEmpty bool
	}{
		"Existing snapshot": {2, snapshottypes.CurrentFormat, 1, false},
		"Missing height":    {100, snapshottypes.CurrentFormat, 1, true},
		"Missing format":    {2, snapshottypes.CurrentFormat + 1, 1, true},
		"Missing chunk":     {2, snapshottypes.CurrentFormat, 9, true},
		"Zero height":       {0, snapshottypes.CurrentFormat, 1, true},
		"Zero format":       {2, 0, 1, true},
		"Zero chunk":        {2, snapshottypes.CurrentFormat, 0, false},
	}

	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			resp, _ := suite.baseApp.LoadSnapshotChunk(&abci.LoadSnapshotChunkRequest{
				Height: tc.height,
				Format: tc.format,
				Chunk:  tc.chunk,
			})
			if tc.expectEmpty {
				require.Equal(t, &abci.LoadSnapshotChunkResponse{}, resp)
				return
			}

			require.NotEmpty(t, resp.Chunk)
		})
	}
}

func TestABCI_OfferSnapshot_Errors(t *testing.T) {
	ssCfg := SnapshotsConfig{
		blocks:             0,
		blockTxs:           0,
		snapshotInterval:   2,
		snapshotKeepRecent: 2,
		pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
	}
	suite := NewBaseAppSuiteWithSnapshots(t, ssCfg)

	m := snapshottypes.Metadata{ChunkHashes: [][]byte{{1}, {2}, {3}}}
	metadata, err := m.Marshal()
	require.NoError(t, err)

	hash := []byte{1, 2, 3}

	testCases := map[string]struct {
		snapshot *abci.Snapshot
		result   abci.OfferSnapshotResult
	}{
		"nil snapshot": {nil, abci.OFFER_SNAPSHOT_RESULT_REJECT},
		"invalid format": {&abci.Snapshot{
			Height: 1, Format: 9, Chunks: 3, Hash: hash, Metadata: metadata,
		}, abci.OFFER_SNAPSHOT_RESULT_REJECT_FORMAT},
		"incorrect chunk count": {&abci.Snapshot{
			Height: 1, Format: snapshottypes.CurrentFormat, Chunks: 2, Hash: hash, Metadata: metadata,
		}, abci.OFFER_SNAPSHOT_RESULT_REJECT},
		"no chunks": {&abci.Snapshot{
			Height: 1, Format: snapshottypes.CurrentFormat, Chunks: 0, Hash: hash, Metadata: metadata,
		}, abci.OFFER_SNAPSHOT_RESULT_REJECT},
		"invalid metadata serialization": {&abci.Snapshot{
			Height: 1, Format: snapshottypes.CurrentFormat, Chunks: 0, Hash: hash, Metadata: []byte{3, 1, 4},
		}, abci.OFFER_SNAPSHOT_RESULT_REJECT},
	}
	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			resp, err := suite.baseApp.OfferSnapshot(&abci.OfferSnapshotRequest{Snapshot: tc.snapshot})
			require.NoError(t, err)
			require.Equal(t, tc.result, resp.Result)
		})
	}

	// Offering a snapshot after one has been accepted should error
	resp, err := suite.baseApp.OfferSnapshot(&abci.OfferSnapshotRequest{Snapshot: &abci.Snapshot{
		Height:   1,
		Format:   snapshottypes.CurrentFormat,
		Chunks:   3,
		Hash:     []byte{1, 2, 3},
		Metadata: metadata,
	}})
	require.NoError(t, err)
	require.Equal(t, &abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_ACCEPT}, resp)

	resp, err = suite.baseApp.OfferSnapshot(&abci.OfferSnapshotRequest{Snapshot: &abci.Snapshot{
		Height:   2,
		Format:   snapshottypes.CurrentFormat,
		Chunks:   3,
		Hash:     []byte{1, 2, 3},
		Metadata: metadata,
	}})
	require.NoError(t, err)
	require.Equal(t, &abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_ABORT}, resp)
}

func TestABCI_ApplySnapshotChunk(t *testing.T) {
	srcCfg := SnapshotsConfig{
		blocks:             4,
		blockTxs:           10,
		snapshotInterval:   2,
		snapshotKeepRecent: 2,
		pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
	}
	srcSuite := NewBaseAppSuiteWithSnapshots(t, srcCfg)

	targetCfg := SnapshotsConfig{
		blocks:             0,
		blockTxs:           0,
		snapshotInterval:   2,
		snapshotKeepRecent: 2,
		pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
	}
	targetSuite := NewBaseAppSuiteWithSnapshots(t, targetCfg)

	// fetch latest snapshot to restore
	respList, err := srcSuite.baseApp.ListSnapshots(&abci.ListSnapshotsRequest{})
	require.NoError(t, err)
	require.NotEmpty(t, respList.Snapshots)
	snapshot := respList.Snapshots[0]

	// make sure the snapshot has at least 3 chunks
	require.GreaterOrEqual(t, snapshot.Chunks, uint32(3), "Not enough snapshot chunks")

	// begin a snapshot restoration in the target
	respOffer, err := targetSuite.baseApp.OfferSnapshot(&abci.OfferSnapshotRequest{Snapshot: snapshot})
	require.NoError(t, err)
	require.Equal(t, &abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_ACCEPT}, respOffer)

	// We should be able to pass an invalid chunk and get a verify failure, before
	// reapplying it.
	respApply, err := targetSuite.baseApp.ApplySnapshotChunk(&abci.ApplySnapshotChunkRequest{
		Index:  0,
		Chunk:  []byte{9},
		Sender: "sender",
	})
	require.NoError(t, err)
	require.Equal(t, &abci.ApplySnapshotChunkResponse{
		Result:        abci.APPLY_SNAPSHOT_CHUNK_RESULT_RETRY,
		RefetchChunks: []uint32{0},
		RejectSenders: []string{"sender"},
	}, respApply)

	// fetch each chunk from the source and apply it to the target
	for index := uint32(0); index < snapshot.Chunks; index++ {
		respChunk, err := srcSuite.baseApp.LoadSnapshotChunk(&abci.LoadSnapshotChunkRequest{
			Height: snapshot.Height,
			Format: snapshot.Format,
			Chunk:  index,
		})
		require.NoError(t, err)
		require.NotNil(t, respChunk.Chunk)

		respApply, err := targetSuite.baseApp.ApplySnapshotChunk(&abci.ApplySnapshotChunkRequest{
			Index: index,
			Chunk: respChunk.Chunk,
		})
		require.NoError(t, err)
		require.Equal(t, &abci.ApplySnapshotChunkResponse{
			Result: abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT,
		}, respApply)
	}

	// the target should now have the same hash as the source
	require.Equal(t, srcSuite.baseApp.LastCommitID(), targetSuite.baseApp.LastCommitID())
}
