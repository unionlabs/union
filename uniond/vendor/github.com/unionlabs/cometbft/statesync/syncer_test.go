package statesync

import (
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"

	abci "github.com/cometbft/cometbft/abci/types"
	cmtstate "github.com/cometbft/cometbft/api/cometbft/state/v1"
	ssproto "github.com/cometbft/cometbft/api/cometbft/statesync/v1"
	cmtversion "github.com/cometbft/cometbft/api/cometbft/version/v1"
	"github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/libs/log"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	"github.com/cometbft/cometbft/p2p"
	p2pmocks "github.com/cometbft/cometbft/p2p/mocks"
	"github.com/cometbft/cometbft/proxy"
	proxymocks "github.com/cometbft/cometbft/proxy/mocks"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/statesync/mocks"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
	"github.com/cometbft/cometbft/version"
)

const testAppVersion = 9

// Sets up a basic syncer that can be used to test OfferSnapshot requests.
func setupOfferSyncer() (*syncer, *proxymocks.AppConnSnapshot) {
	connQuery := &proxymocks.AppConnQuery{}
	connSnapshot := &proxymocks.AppConnSnapshot{}
	stateProvider := &mocks.StateProvider{}
	stateProvider.On("AppHash", mock.Anything, mock.Anything).Return([]byte("app_hash"), nil)
	cfg := config.DefaultStateSyncConfig()
	syncer := newSyncer(*cfg, log.NewNopLogger(), connSnapshot, connQuery, stateProvider, "")

	return syncer, connSnapshot
}

// Sets up a simple peer mock with an ID.
func simplePeer(id string) *p2pmocks.Peer {
	peer := &p2pmocks.Peer{}
	peer.On("ID").Return(p2p.ID(id))
	return peer
}

func TestSyncer_SyncAny(t *testing.T) {
	state := sm.State{
		ChainID: "chain",
		Version: cmtstate.Version{
			Consensus: cmtversion.Consensus{
				Block: version.BlockProtocol,
				App:   testAppVersion,
			},
			Software: version.CMTSemVer,
		},

		LastBlockHeight: 1,
		LastBlockID:     types.BlockID{Hash: []byte("blockhash")},
		LastBlockTime:   cmttime.Now(),
		LastResultsHash: []byte("last_results_hash"),
		AppHash:         []byte("app_hash"),

		LastValidators: &types.ValidatorSet{Proposer: &types.Validator{Address: []byte("val1")}},
		Validators:     &types.ValidatorSet{Proposer: &types.Validator{Address: []byte("val2")}},
		NextValidators: &types.ValidatorSet{Proposer: &types.Validator{Address: []byte("val3")}},

		ConsensusParams:                  *types.DefaultConsensusParams(),
		LastHeightConsensusParamsChanged: 1,
	}
	commit := &types.Commit{BlockID: types.BlockID{Hash: []byte("blockhash")}}

	chunks := []*chunk{
		{Height: 1, Format: 1, Index: 0, Chunk: []byte{1, 1, 0}},
		{Height: 1, Format: 1, Index: 1, Chunk: []byte{1, 1, 1}},
		{Height: 1, Format: 1, Index: 2, Chunk: []byte{1, 1, 2}},
	}
	s := &snapshot{Height: 1, Format: 1, Chunks: 3, Hash: []byte{1, 2, 3}}

	stateProvider := &mocks.StateProvider{}
	stateProvider.On("AppHash", mock.Anything, uint64(1)).Return(state.AppHash, nil)
	stateProvider.On("AppHash", mock.Anything, uint64(2)).Return([]byte("app_hash_2"), nil)
	stateProvider.On("Commit", mock.Anything, uint64(1)).Return(commit, nil)
	stateProvider.On("State", mock.Anything, uint64(1)).Return(state, nil)
	connSnapshot := &proxymocks.AppConnSnapshot{}
	connQuery := &proxymocks.AppConnQuery{}

	cfg := config.DefaultStateSyncConfig()
	syncer := newSyncer(*cfg, log.NewNopLogger(), connSnapshot, connQuery, stateProvider, "")

	// Adding a chunk should error when no sync is in progress
	_, err := syncer.AddChunk(&chunk{Height: 1, Format: 1, Index: 0, Chunk: []byte{1}})
	require.Error(t, err)

	// Adding a couple of peers should trigger snapshot discovery messages
	peerA := &p2pmocks.Peer{}
	peerA.On("ID").Return(p2p.ID("a"))
	peerA.On("Send", mock.MatchedBy(func(i any) bool {
		e, ok := i.(p2p.Envelope)
		if !ok {
			return false
		}
		req, ok := e.Message.(*ssproto.SnapshotsRequest)
		return ok && e.ChannelID == SnapshotChannel && req != nil
	})).Return(true)
	syncer.AddPeer(peerA)
	peerA.AssertExpectations(t)

	peerB := &p2pmocks.Peer{}
	peerB.On("ID").Return(p2p.ID("b"))
	peerB.On("Send", mock.MatchedBy(func(i any) bool {
		e, ok := i.(p2p.Envelope)
		if !ok {
			return false
		}
		req, ok := e.Message.(*ssproto.SnapshotsRequest)
		return ok && e.ChannelID == SnapshotChannel && req != nil
	})).Return(true)
	syncer.AddPeer(peerB)
	peerB.AssertExpectations(t)

	// Both peers report back with snapshots. One of them also returns a snapshot we don't want, in
	// format 2, which will be rejected by the ABCI application.
	isNew, err := syncer.AddSnapshot(peerA, s)
	require.NoError(t, err)
	assert.True(t, isNew)

	isNew, err = syncer.AddSnapshot(peerB, s)
	require.NoError(t, err)
	assert.False(t, isNew)

	isNew, err = syncer.AddSnapshot(peerB, &snapshot{Height: 2, Format: 2, Chunks: 3, Hash: []byte{1}})
	require.NoError(t, err)
	assert.True(t, isNew)

	// We start a sync, with peers sending back chunks when requested. We first reject the snapshot
	// with height 2 format 2, and accept the snapshot at height 1.
	connSnapshot.On("OfferSnapshot", mock.Anything, &abci.OfferSnapshotRequest{
		Snapshot: &abci.Snapshot{
			Height: 2,
			Format: 2,
			Chunks: 3,
			Hash:   []byte{1},
		},
		AppHash: []byte("app_hash_2"),
	}).Return(&abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_REJECT_FORMAT}, nil)
	connSnapshot.On("OfferSnapshot", mock.Anything, &abci.OfferSnapshotRequest{
		Snapshot: &abci.Snapshot{
			Height:   s.Height,
			Format:   s.Format,
			Chunks:   s.Chunks,
			Hash:     s.Hash,
			Metadata: s.Metadata,
		},
		AppHash: []byte("app_hash"),
	}).Times(2).Return(&abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_ACCEPT}, nil)

	chunkRequests := make(map[uint32]int)
	chunkRequestsMtx := cmtsync.Mutex{}
	onChunkRequest := func(args mock.Arguments) {
		e, ok := args[0].(p2p.Envelope)
		require.True(t, ok)
		msg := e.Message.(*ssproto.ChunkRequest)
		require.EqualValues(t, 1, msg.Height)
		require.EqualValues(t, 1, msg.Format)
		require.LessOrEqual(t, msg.Index, uint32(len(chunks)))

		added, err := syncer.AddChunk(chunks[msg.Index])
		require.NoError(t, err)
		assert.True(t, added)

		chunkRequestsMtx.Lock()
		chunkRequests[msg.Index]++
		chunkRequestsMtx.Unlock()
	}
	peerA.On("Send", mock.MatchedBy(func(i any) bool {
		e, ok := i.(p2p.Envelope)
		return ok && e.ChannelID == ChunkChannel
	})).Maybe().Run(onChunkRequest).Return(true)
	peerB.On("Send", mock.MatchedBy(func(i any) bool {
		e, ok := i.(p2p.Envelope)
		return ok && e.ChannelID == ChunkChannel
	})).Maybe().Run(onChunkRequest).Return(true)

	// The first time we're applying chunk 2 we tell it to retry the snapshot and discard chunk 1,
	// which should cause it to keep the existing chunk 0 and 2, and restart restoration from
	// beginning. We also wait for a little while, to exercise the retry logic in fetchChunks().
	connSnapshot.On("ApplySnapshotChunk", mock.Anything, &abci.ApplySnapshotChunkRequest{
		Index: 2, Chunk: []byte{1, 1, 2},
	}).Once().Run(func(_ mock.Arguments) { time.Sleep(2 * time.Second) }).Return(
		&abci.ApplySnapshotChunkResponse{
			Result:        abci.APPLY_SNAPSHOT_CHUNK_RESULT_RETRY_SNAPSHOT,
			RefetchChunks: []uint32{1},
		}, nil)

	connSnapshot.On("ApplySnapshotChunk", mock.Anything, &abci.ApplySnapshotChunkRequest{
		Index: 0, Chunk: []byte{1, 1, 0},
	}).Times(2).Return(&abci.ApplySnapshotChunkResponse{Result: abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT}, nil)
	connSnapshot.On("ApplySnapshotChunk", mock.Anything, &abci.ApplySnapshotChunkRequest{
		Index: 1, Chunk: []byte{1, 1, 1},
	}).Times(2).Return(&abci.ApplySnapshotChunkResponse{Result: abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT}, nil)
	connSnapshot.On("ApplySnapshotChunk", mock.Anything, &abci.ApplySnapshotChunkRequest{
		Index: 2, Chunk: []byte{1, 1, 2},
	}).Once().Return(&abci.ApplySnapshotChunkResponse{Result: abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT}, nil)
	connQuery.On("Info", mock.Anything, proxy.InfoRequest).Return(&abci.InfoResponse{
		AppVersion:       testAppVersion,
		LastBlockHeight:  1,
		LastBlockAppHash: []byte("app_hash"),
	}, nil)

	newState, lastCommit, err := syncer.SyncAny(0, func() {})
	require.NoError(t, err)

	time.Sleep(50 * time.Millisecond) // wait for peers to receive requests

	chunkRequestsMtx.Lock()
	assert.Equal(t, map[uint32]int{0: 1, 1: 2, 2: 1}, chunkRequests)
	chunkRequestsMtx.Unlock()

	expectState := state

	assert.Equal(t, expectState, newState)
	assert.Equal(t, commit, lastCommit)

	connSnapshot.AssertExpectations(t)
	connQuery.AssertExpectations(t)
	peerA.AssertExpectations(t)
	peerB.AssertExpectations(t)
}

func TestSyncer_SyncAny_noSnapshots(t *testing.T) {
	syncer, _ := setupOfferSyncer()
	_, _, err := syncer.SyncAny(0, func() {})
	assert.Equal(t, errNoSnapshots, err)
}

func TestSyncer_SyncAny_abort(t *testing.T) {
	syncer, connSnapshot := setupOfferSyncer()

	s := &snapshot{Height: 1, Format: 1, Chunks: 3, Hash: []byte{1, 2, 3}}
	_, err := syncer.AddSnapshot(simplePeer("id"), s)
	require.NoError(t, err)
	connSnapshot.On("OfferSnapshot", mock.Anything, &abci.OfferSnapshotRequest{
		Snapshot: toABCI(s), AppHash: []byte("app_hash"),
	}).Once().Return(&abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_ABORT}, nil)

	_, _, err = syncer.SyncAny(0, func() {})
	assert.Equal(t, errAbort, err)
	connSnapshot.AssertExpectations(t)
}

func TestSyncer_SyncAny_reject(t *testing.T) {
	syncer, connSnapshot := setupOfferSyncer()

	// s22 is tried first, then s12, then s11, then errNoSnapshots
	s22 := &snapshot{Height: 2, Format: 2, Chunks: 3, Hash: []byte{1, 2, 3}}
	s12 := &snapshot{Height: 1, Format: 2, Chunks: 3, Hash: []byte{1, 2, 3}}
	s11 := &snapshot{Height: 1, Format: 1, Chunks: 3, Hash: []byte{1, 2, 3}}
	_, err := syncer.AddSnapshot(simplePeer("id"), s22)
	require.NoError(t, err)
	_, err = syncer.AddSnapshot(simplePeer("id"), s12)
	require.NoError(t, err)
	_, err = syncer.AddSnapshot(simplePeer("id"), s11)
	require.NoError(t, err)

	connSnapshot.On("OfferSnapshot", mock.Anything, &abci.OfferSnapshotRequest{
		Snapshot: toABCI(s22), AppHash: []byte("app_hash"),
	}).Once().Return(&abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_REJECT}, nil)

	connSnapshot.On("OfferSnapshot", mock.Anything, &abci.OfferSnapshotRequest{
		Snapshot: toABCI(s12), AppHash: []byte("app_hash"),
	}).Once().Return(&abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_REJECT}, nil)

	connSnapshot.On("OfferSnapshot", mock.Anything, &abci.OfferSnapshotRequest{
		Snapshot: toABCI(s11), AppHash: []byte("app_hash"),
	}).Once().Return(&abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_REJECT}, nil)

	_, _, err = syncer.SyncAny(0, func() {})
	assert.Equal(t, errNoSnapshots, err)
	connSnapshot.AssertExpectations(t)
}

func TestSyncer_SyncAny_reject_format(t *testing.T) {
	syncer, connSnapshot := setupOfferSyncer()

	// s22 is tried first, which reject s22 and s12, then s11 will abort.
	s22 := &snapshot{Height: 2, Format: 2, Chunks: 3, Hash: []byte{1, 2, 3}}
	s12 := &snapshot{Height: 1, Format: 2, Chunks: 3, Hash: []byte{1, 2, 3}}
	s11 := &snapshot{Height: 1, Format: 1, Chunks: 3, Hash: []byte{1, 2, 3}}
	_, err := syncer.AddSnapshot(simplePeer("id"), s22)
	require.NoError(t, err)
	_, err = syncer.AddSnapshot(simplePeer("id"), s12)
	require.NoError(t, err)
	_, err = syncer.AddSnapshot(simplePeer("id"), s11)
	require.NoError(t, err)

	connSnapshot.On("OfferSnapshot", mock.Anything, &abci.OfferSnapshotRequest{
		Snapshot: toABCI(s22), AppHash: []byte("app_hash"),
	}).Once().Return(&abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_REJECT_FORMAT}, nil)

	connSnapshot.On("OfferSnapshot", mock.Anything, &abci.OfferSnapshotRequest{
		Snapshot: toABCI(s11), AppHash: []byte("app_hash"),
	}).Once().Return(&abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_ABORT}, nil)

	_, _, err = syncer.SyncAny(0, func() {})
	assert.Equal(t, errAbort, err)
	connSnapshot.AssertExpectations(t)
}

func TestSyncer_SyncAny_reject_sender(t *testing.T) {
	syncer, connSnapshot := setupOfferSyncer()

	peerA := simplePeer("a")
	peerB := simplePeer("b")
	peerC := simplePeer("c")

	// sbc will be offered first, which will be rejected with reject_sender, causing all snapshots
	// submitted by both b and c (i.e. sb, sc, sbc) to be rejected. Finally, sa will reject and
	// errNoSnapshots is returned.
	sa := &snapshot{Height: 1, Format: 1, Chunks: 3, Hash: []byte{1, 2, 3}}
	sb := &snapshot{Height: 2, Format: 1, Chunks: 3, Hash: []byte{1, 2, 3}}
	sc := &snapshot{Height: 3, Format: 1, Chunks: 3, Hash: []byte{1, 2, 3}}
	sbc := &snapshot{Height: 4, Format: 1, Chunks: 3, Hash: []byte{1, 2, 3}}
	_, err := syncer.AddSnapshot(peerA, sa)
	require.NoError(t, err)
	_, err = syncer.AddSnapshot(peerB, sb)
	require.NoError(t, err)
	_, err = syncer.AddSnapshot(peerC, sc)
	require.NoError(t, err)
	_, err = syncer.AddSnapshot(peerB, sbc)
	require.NoError(t, err)
	_, err = syncer.AddSnapshot(peerC, sbc)
	require.NoError(t, err)

	connSnapshot.On("OfferSnapshot", mock.Anything, &abci.OfferSnapshotRequest{
		Snapshot: toABCI(sbc), AppHash: []byte("app_hash"),
	}).Once().Return(&abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_REJECT_SENDER}, nil)

	connSnapshot.On("OfferSnapshot", mock.Anything, &abci.OfferSnapshotRequest{
		Snapshot: toABCI(sa), AppHash: []byte("app_hash"),
	}).Once().Return(&abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_REJECT}, nil)

	_, _, err = syncer.SyncAny(0, func() {})
	assert.Equal(t, errNoSnapshots, err)
	connSnapshot.AssertExpectations(t)
}

func TestSyncer_SyncAny_abciError(t *testing.T) {
	syncer, connSnapshot := setupOfferSyncer()

	errBoom := errors.New("boom")
	s := &snapshot{Height: 1, Format: 1, Chunks: 3, Hash: []byte{1, 2, 3}}
	_, err := syncer.AddSnapshot(simplePeer("id"), s)
	require.NoError(t, err)
	connSnapshot.On("OfferSnapshot", mock.Anything, &abci.OfferSnapshotRequest{
		Snapshot: toABCI(s), AppHash: []byte("app_hash"),
	}).Once().Return(nil, errBoom)

	_, _, err = syncer.SyncAny(0, func() {})
	require.ErrorIs(t, err, errBoom)
	connSnapshot.AssertExpectations(t)
}

func TestSyncer_offerSnapshot(t *testing.T) {
	unknownErr := errors.New("unknown error")
	boom := errors.New("boom")

	testcases := map[string]struct {
		result    abci.OfferSnapshotResult
		err       error
		expectErr error
	}{
		"accept":           {abci.OFFER_SNAPSHOT_RESULT_ACCEPT, nil, nil},
		"abort":            {abci.OFFER_SNAPSHOT_RESULT_ABORT, nil, errAbort},
		"reject":           {abci.OFFER_SNAPSHOT_RESULT_REJECT, nil, errRejectSnapshot},
		"reject_format":    {abci.OFFER_SNAPSHOT_RESULT_REJECT_FORMAT, nil, errRejectFormat},
		"reject_sender":    {abci.OFFER_SNAPSHOT_RESULT_REJECT_SENDER, nil, errRejectSender},
		"unknown":          {abci.OFFER_SNAPSHOT_RESULT_UNKNOWN, nil, unknownErr},
		"error":            {0, boom, boom},
		"unknown non-zero": {9, nil, unknownErr},
	}
	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			syncer, connSnapshot := setupOfferSyncer()
			s := &snapshot{Height: 1, Format: 1, Chunks: 3, Hash: []byte{1, 2, 3}, trustedAppHash: []byte("app_hash")}
			connSnapshot.On("OfferSnapshot", mock.Anything, &abci.OfferSnapshotRequest{
				Snapshot: toABCI(s),
				AppHash:  []byte("app_hash"),
			}).Return(&abci.OfferSnapshotResponse{Result: tc.result}, tc.err)
			err := syncer.offerSnapshot(s)
			if tc.expectErr == unknownErr {
				require.Error(t, err)
			} else {
				unwrapped := errors.Unwrap(err)
				if unwrapped != nil {
					err = unwrapped
				}
				assert.Equal(t, tc.expectErr, err)
			}
		})
	}
}

func TestSyncer_applyChunks_Results(t *testing.T) {
	unknownErr := errors.New("unknown error")
	boom := errors.New("boom")

	testcases := map[string]struct {
		result    abci.ApplySnapshotChunkResult
		err       error
		expectErr error
	}{
		"accept":           {abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT, nil, nil},
		"abort":            {abci.APPLY_SNAPSHOT_CHUNK_RESULT_ABORT, nil, errAbort},
		"retry":            {abci.APPLY_SNAPSHOT_CHUNK_RESULT_RETRY, nil, nil},
		"retry_snapshot":   {abci.APPLY_SNAPSHOT_CHUNK_RESULT_RETRY_SNAPSHOT, nil, errRetrySnapshot},
		"reject_snapshot":  {abci.APPLY_SNAPSHOT_CHUNK_RESULT_REJECT_SNAPSHOT, nil, errRejectSnapshot},
		"unknown":          {abci.APPLY_SNAPSHOT_CHUNK_RESULT_UNKNOWN, nil, unknownErr},
		"error":            {0, boom, boom},
		"unknown non-zero": {9, nil, unknownErr},
	}
	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			connQuery := &proxymocks.AppConnQuery{}
			connSnapshot := &proxymocks.AppConnSnapshot{}
			stateProvider := &mocks.StateProvider{}
			stateProvider.On("AppHash", mock.Anything, mock.Anything).Return([]byte("app_hash"), nil)

			cfg := config.DefaultStateSyncConfig()
			syncer := newSyncer(*cfg, log.NewNopLogger(), connSnapshot, connQuery, stateProvider, "")

			body := []byte{1, 2, 3}
			chunks, err := newChunkQueue(&snapshot{Height: 1, Format: 1, Chunks: 1}, "")
			require.NoError(t, err)
			_, err = chunks.Add(&chunk{Height: 1, Format: 1, Index: 0, Chunk: body})
			require.NoError(t, err)

			connSnapshot.On("ApplySnapshotChunk", mock.Anything, &abci.ApplySnapshotChunkRequest{
				Index: 0, Chunk: body,
			}).Once().Return(&abci.ApplySnapshotChunkResponse{Result: tc.result}, tc.err)
			if tc.result == abci.APPLY_SNAPSHOT_CHUNK_RESULT_RETRY {
				connSnapshot.On("ApplySnapshotChunk", mock.Anything, &abci.ApplySnapshotChunkRequest{
					Index: 0, Chunk: body,
				}).Once().Return(&abci.ApplySnapshotChunkResponse{
					Result: abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT,
				}, nil)
			}

			err = syncer.applyChunks(chunks)
			if tc.expectErr == unknownErr {
				require.Error(t, err)
			} else {
				unwrapped := errors.Unwrap(err)
				if unwrapped != nil {
					err = unwrapped
				}
				assert.Equal(t, tc.expectErr, err)
			}
			connSnapshot.AssertExpectations(t)
		})
	}
}

func TestSyncer_applyChunks_RefetchChunks(t *testing.T) {
	// Discarding chunks via refetch_chunks should work the same for all results
	testcases := map[string]struct {
		result abci.ApplySnapshotChunkResult
	}{
		"accept":          {abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT},
		"abort":           {abci.APPLY_SNAPSHOT_CHUNK_RESULT_ABORT},
		"retry":           {abci.APPLY_SNAPSHOT_CHUNK_RESULT_RETRY},
		"retry_snapshot":  {abci.APPLY_SNAPSHOT_CHUNK_RESULT_RETRY_SNAPSHOT},
		"reject_snapshot": {abci.APPLY_SNAPSHOT_CHUNK_RESULT_REJECT_SNAPSHOT},
	}
	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			connQuery := &proxymocks.AppConnQuery{}
			connSnapshot := &proxymocks.AppConnSnapshot{}
			stateProvider := &mocks.StateProvider{}
			stateProvider.On("AppHash", mock.Anything, mock.Anything).Return([]byte("app_hash"), nil)

			cfg := config.DefaultStateSyncConfig()
			syncer := newSyncer(*cfg, log.NewNopLogger(), connSnapshot, connQuery, stateProvider, "")

			chunks, err := newChunkQueue(&snapshot{Height: 1, Format: 1, Chunks: 3}, "")
			require.NoError(t, err)
			added, err := chunks.Add(&chunk{Height: 1, Format: 1, Index: 0, Chunk: []byte{0}})
			require.True(t, added)
			require.NoError(t, err)
			added, err = chunks.Add(&chunk{Height: 1, Format: 1, Index: 1, Chunk: []byte{1}})
			require.True(t, added)
			require.NoError(t, err)
			added, err = chunks.Add(&chunk{Height: 1, Format: 1, Index: 2, Chunk: []byte{2}})
			require.True(t, added)
			require.NoError(t, err)

			// The first two chunks are accepted, before the last one asks for 1 to be refetched
			connSnapshot.On("ApplySnapshotChunk", mock.Anything, &abci.ApplySnapshotChunkRequest{
				Index: 0, Chunk: []byte{0},
			}).Once().Return(&abci.ApplySnapshotChunkResponse{Result: abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT}, nil)
			connSnapshot.On("ApplySnapshotChunk", mock.Anything, &abci.ApplySnapshotChunkRequest{
				Index: 1, Chunk: []byte{1},
			}).Once().Return(&abci.ApplySnapshotChunkResponse{Result: abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT}, nil)
			connSnapshot.On("ApplySnapshotChunk", mock.Anything, &abci.ApplySnapshotChunkRequest{
				Index: 2, Chunk: []byte{2},
			}).Once().Return(&abci.ApplySnapshotChunkResponse{
				Result:        tc.result,
				RefetchChunks: []uint32{1},
			}, nil)

			// Since removing the chunk will cause Next() to block, we spawn a goroutine, then
			// check the queue contents, and finally close the queue to end the goroutine.
			// We don't really care about the result of applyChunks, since it has separate test.
			go func() {
				syncer.applyChunks(chunks) //nolint:errcheck // purposefully ignore error
			}()

			time.Sleep(50 * time.Millisecond)
			assert.True(t, chunks.Has(0))
			assert.False(t, chunks.Has(1))
			assert.True(t, chunks.Has(2))
			err = chunks.Close()
			require.NoError(t, err)
		})
	}
}

func TestSyncer_applyChunks_RejectSenders(t *testing.T) {
	// Banning chunks senders via ban_chunk_senders should work the same for all results
	testcases := map[string]struct {
		result abci.ApplySnapshotChunkResult
	}{
		"accept":          {abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT},
		"abort":           {abci.APPLY_SNAPSHOT_CHUNK_RESULT_ABORT},
		"retry":           {abci.APPLY_SNAPSHOT_CHUNK_RESULT_RETRY},
		"retry_snapshot":  {abci.APPLY_SNAPSHOT_CHUNK_RESULT_RETRY_SNAPSHOT},
		"reject_snapshot": {abci.APPLY_SNAPSHOT_CHUNK_RESULT_REJECT_SNAPSHOT},
	}
	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			connQuery := &proxymocks.AppConnQuery{}
			connSnapshot := &proxymocks.AppConnSnapshot{}
			stateProvider := &mocks.StateProvider{}
			stateProvider.On("AppHash", mock.Anything, mock.Anything).Return([]byte("app_hash"), nil)

			cfg := config.DefaultStateSyncConfig()
			syncer := newSyncer(*cfg, log.NewNopLogger(), connSnapshot, connQuery, stateProvider, "")

			// Set up three peers across two snapshots, and ask for one of them to be banned.
			// It should be banned from all snapshots.
			peerA := simplePeer("a")
			peerB := simplePeer("b")
			peerC := simplePeer("c")

			s1 := &snapshot{Height: 1, Format: 1, Chunks: 3}
			s2 := &snapshot{Height: 2, Format: 1, Chunks: 3}
			_, err := syncer.AddSnapshot(peerA, s1)
			require.NoError(t, err)
			_, err = syncer.AddSnapshot(peerA, s2)
			require.NoError(t, err)
			_, err = syncer.AddSnapshot(peerB, s1)
			require.NoError(t, err)
			_, err = syncer.AddSnapshot(peerB, s2)
			require.NoError(t, err)
			_, err = syncer.AddSnapshot(peerC, s1)
			require.NoError(t, err)
			_, err = syncer.AddSnapshot(peerC, s2)
			require.NoError(t, err)

			chunks, err := newChunkQueue(s1, "")
			require.NoError(t, err)
			added, err := chunks.Add(&chunk{Height: 1, Format: 1, Index: 0, Chunk: []byte{0}, Sender: peerA.ID()})
			require.True(t, added)
			require.NoError(t, err)
			added, err = chunks.Add(&chunk{Height: 1, Format: 1, Index: 1, Chunk: []byte{1}, Sender: peerB.ID()})
			require.True(t, added)
			require.NoError(t, err)
			added, err = chunks.Add(&chunk{Height: 1, Format: 1, Index: 2, Chunk: []byte{2}, Sender: peerC.ID()})
			require.True(t, added)
			require.NoError(t, err)

			// The first two chunks are accepted, before the last one asks for b sender to be rejected
			connSnapshot.On("ApplySnapshotChunk", mock.Anything, &abci.ApplySnapshotChunkRequest{
				Index: 0, Chunk: []byte{0}, Sender: "a",
			}).Once().Return(&abci.ApplySnapshotChunkResponse{Result: abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT}, nil)
			connSnapshot.On("ApplySnapshotChunk", mock.Anything, &abci.ApplySnapshotChunkRequest{
				Index: 1, Chunk: []byte{1}, Sender: "b",
			}).Once().Return(&abci.ApplySnapshotChunkResponse{Result: abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT}, nil)
			connSnapshot.On("ApplySnapshotChunk", mock.Anything, &abci.ApplySnapshotChunkRequest{
				Index: 2, Chunk: []byte{2}, Sender: "c",
			}).Once().Return(&abci.ApplySnapshotChunkResponse{
				Result:        tc.result,
				RejectSenders: []string{string(peerB.ID())},
			}, nil)

			// On retry, the last chunk will be tried again, so we just accept it then.
			if tc.result == abci.APPLY_SNAPSHOT_CHUNK_RESULT_RETRY {
				connSnapshot.On("ApplySnapshotChunk", mock.Anything, &abci.ApplySnapshotChunkRequest{
					Index: 2, Chunk: []byte{2}, Sender: "c",
				}).Once().Return(&abci.ApplySnapshotChunkResponse{Result: abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT}, nil)
			}

			// We don't really care about the result of applyChunks, since it has separate test.
			// However, it will block on e.g. retry result, so we spawn a goroutine that will
			// be shut down when the chunk queue closes.
			go func() {
				syncer.applyChunks(chunks) //nolint:errcheck // purposefully ignore error
			}()

			time.Sleep(50 * time.Millisecond)

			s1peers := syncer.snapshots.GetPeers(s1)
			assert.Len(t, s1peers, 2)
			assert.EqualValues(t, "a", s1peers[0].ID())
			assert.EqualValues(t, "c", s1peers[1].ID())

			syncer.snapshots.GetPeers(s1)
			assert.Len(t, s1peers, 2)
			assert.EqualValues(t, "a", s1peers[0].ID())
			assert.EqualValues(t, "c", s1peers[1].ID())

			err = chunks.Close()
			require.NoError(t, err)
		})
	}
}

func TestSyncer_verifyApp(t *testing.T) {
	boom := errors.New("boom")
	const appVersion = 9
	appVersionMismatchErr := errors.New("app version mismatch. Expected: 9, got: 2")
	s := &snapshot{Height: 3, Format: 1, Chunks: 5, Hash: []byte{1, 2, 3}, trustedAppHash: []byte("app_hash")}

	testcases := map[string]struct {
		response  *abci.InfoResponse
		err       error
		expectErr error
	}{
		"verified": {&abci.InfoResponse{
			LastBlockHeight:  3,
			LastBlockAppHash: []byte("app_hash"),
			AppVersion:       appVersion,
		}, nil, nil},
		"invalid app version": {&abci.InfoResponse{
			LastBlockHeight:  3,
			LastBlockAppHash: []byte("app_hash"),
			AppVersion:       2,
		}, nil, appVersionMismatchErr},
		"invalid height": {&abci.InfoResponse{
			LastBlockHeight:  5,
			LastBlockAppHash: []byte("app_hash"),
			AppVersion:       appVersion,
		}, nil, errVerifyFailed},
		"invalid hash": {&abci.InfoResponse{
			LastBlockHeight:  3,
			LastBlockAppHash: []byte("xxx"),
			AppVersion:       appVersion,
		}, nil, errVerifyFailed},
		"error": {nil, boom, boom},
	}
	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			connQuery := &proxymocks.AppConnQuery{}
			connSnapshot := &proxymocks.AppConnSnapshot{}
			stateProvider := &mocks.StateProvider{}

			cfg := config.DefaultStateSyncConfig()
			syncer := newSyncer(*cfg, log.NewNopLogger(), connSnapshot, connQuery, stateProvider, "")

			connQuery.On("Info", mock.Anything, proxy.InfoRequest).Return(tc.response, tc.err)
			err := syncer.verifyApp(s, appVersion)
			unwrapped := errors.Unwrap(err)
			if unwrapped != nil {
				err = unwrapped
			}
			require.Equal(t, tc.expectErr, err)
		})
	}
}

func toABCI(s *snapshot) *abci.Snapshot {
	return &abci.Snapshot{
		Height:   s.Height,
		Format:   s.Format,
		Chunks:   s.Chunks,
		Hash:     s.Hash,
		Metadata: s.Metadata,
	}
}
