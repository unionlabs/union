package statesync

import (
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"

	abci "github.com/cometbft/cometbft/abci/types"
	ssproto "github.com/cometbft/cometbft/api/cometbft/statesync/v1"
	"github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/p2p"
	p2pmocks "github.com/cometbft/cometbft/p2p/mocks"
	proxymocks "github.com/cometbft/cometbft/proxy/mocks"
)

func TestReactor_Receive_ChunkRequest(t *testing.T) {
	testcases := map[string]struct {
		request        *ssproto.ChunkRequest
		chunk          []byte
		expectResponse *ssproto.ChunkResponse
	}{
		"chunk is returned": {
			&ssproto.ChunkRequest{Height: 1, Format: 1, Index: 1},
			[]byte{1, 2, 3},
			&ssproto.ChunkResponse{Height: 1, Format: 1, Index: 1, Chunk: []byte{1, 2, 3}},
		},
		"empty chunk is returned, as nil": {
			&ssproto.ChunkRequest{Height: 1, Format: 1, Index: 1},
			[]byte{},
			&ssproto.ChunkResponse{Height: 1, Format: 1, Index: 1, Chunk: nil},
		},
		"nil (missing) chunk is returned as missing": {
			&ssproto.ChunkRequest{Height: 1, Format: 1, Index: 1},
			nil,
			&ssproto.ChunkResponse{Height: 1, Format: 1, Index: 1, Missing: true},
		},
	}

	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			// Mock ABCI connection to return local snapshots
			conn := &proxymocks.AppConnSnapshot{}
			conn.On("LoadSnapshotChunk", mock.Anything, &abci.LoadSnapshotChunkRequest{
				Height: tc.request.Height,
				Format: tc.request.Format,
				Chunk:  tc.request.Index,
			}).Return(&abci.LoadSnapshotChunkResponse{Chunk: tc.chunk}, nil)

			// Mock peer to store response, if found
			peer := &p2pmocks.Peer{}
			peer.On("ID").Return(p2p.ID("id"))
			var response *ssproto.ChunkResponse
			if tc.expectResponse != nil {
				peer.On("Send", mock.MatchedBy(func(i any) bool {
					e, ok := i.(p2p.Envelope)
					return ok && e.ChannelID == ChunkChannel
				})).Run(func(args mock.Arguments) {
					e := args[0].(p2p.Envelope)

					// Marshal to simulate a wire roundtrip.
					bz, err := proto.Marshal(e.Message)
					require.NoError(t, err)
					err = proto.Unmarshal(bz, e.Message)
					require.NoError(t, err)
					response = e.Message.(*ssproto.ChunkResponse)
				}).Return(true)
			}

			// Start a reactor and send a ssproto.ChunkRequest, then wait for and check response
			cfg := config.DefaultStateSyncConfig()
			r := NewReactor(*cfg, conn, nil, NopMetrics())
			err := r.Start()
			require.NoError(t, err)
			t.Cleanup(func() {
				if err := r.Stop(); err != nil {
					t.Error(err)
				}
			})

			r.Receive(p2p.Envelope{
				ChannelID: ChunkChannel,
				Src:       peer,
				Message:   tc.request,
			})
			time.Sleep(100 * time.Millisecond)
			assert.Equal(t, tc.expectResponse, response)

			conn.AssertExpectations(t)
			peer.AssertExpectations(t)
		})
	}
}

func TestReactor_Receive_SnapshotsRequest(t *testing.T) {
	testcases := map[string]struct {
		snapshots       []*abci.Snapshot
		expectResponses []*ssproto.SnapshotsResponse
	}{
		"no snapshots": {nil, []*ssproto.SnapshotsResponse{}},
		">10 unordered snapshots": {
			[]*abci.Snapshot{
				{Height: 1, Format: 2, Chunks: 7, Hash: []byte{1, 2}, Metadata: []byte{1}},
				{Height: 2, Format: 2, Chunks: 7, Hash: []byte{2, 2}, Metadata: []byte{2}},
				{Height: 3, Format: 2, Chunks: 7, Hash: []byte{3, 2}, Metadata: []byte{3}},
				{Height: 1, Format: 1, Chunks: 7, Hash: []byte{1, 1}, Metadata: []byte{4}},
				{Height: 2, Format: 1, Chunks: 7, Hash: []byte{2, 1}, Metadata: []byte{5}},
				{Height: 3, Format: 1, Chunks: 7, Hash: []byte{3, 1}, Metadata: []byte{6}},
				{Height: 1, Format: 4, Chunks: 7, Hash: []byte{1, 4}, Metadata: []byte{7}},
				{Height: 2, Format: 4, Chunks: 7, Hash: []byte{2, 4}, Metadata: []byte{8}},
				{Height: 3, Format: 4, Chunks: 7, Hash: []byte{3, 4}, Metadata: []byte{9}},
				{Height: 1, Format: 3, Chunks: 7, Hash: []byte{1, 3}, Metadata: []byte{10}},
				{Height: 2, Format: 3, Chunks: 7, Hash: []byte{2, 3}, Metadata: []byte{11}},
				{Height: 3, Format: 3, Chunks: 7, Hash: []byte{3, 3}, Metadata: []byte{12}},
			},
			[]*ssproto.SnapshotsResponse{
				{Height: 3, Format: 4, Chunks: 7, Hash: []byte{3, 4}, Metadata: []byte{9}},
				{Height: 3, Format: 3, Chunks: 7, Hash: []byte{3, 3}, Metadata: []byte{12}},
				{Height: 3, Format: 2, Chunks: 7, Hash: []byte{3, 2}, Metadata: []byte{3}},
				{Height: 3, Format: 1, Chunks: 7, Hash: []byte{3, 1}, Metadata: []byte{6}},
				{Height: 2, Format: 4, Chunks: 7, Hash: []byte{2, 4}, Metadata: []byte{8}},
				{Height: 2, Format: 3, Chunks: 7, Hash: []byte{2, 3}, Metadata: []byte{11}},
				{Height: 2, Format: 2, Chunks: 7, Hash: []byte{2, 2}, Metadata: []byte{2}},
				{Height: 2, Format: 1, Chunks: 7, Hash: []byte{2, 1}, Metadata: []byte{5}},
				{Height: 1, Format: 4, Chunks: 7, Hash: []byte{1, 4}, Metadata: []byte{7}},
				{Height: 1, Format: 3, Chunks: 7, Hash: []byte{1, 3}, Metadata: []byte{10}},
			},
		},
	}

	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			// Mock ABCI connection to return local snapshots
			conn := &proxymocks.AppConnSnapshot{}
			conn.On("ListSnapshots", mock.Anything, &abci.ListSnapshotsRequest{}).Return(&abci.ListSnapshotsResponse{
				Snapshots: tc.snapshots,
			}, nil)

			// Mock peer to catch responses and store them in a slice
			responses := []*ssproto.SnapshotsResponse{}
			peer := &p2pmocks.Peer{}
			if len(tc.expectResponses) > 0 {
				peer.On("ID").Return(p2p.ID("id"))
				peer.On("Send", mock.MatchedBy(func(i any) bool {
					e, ok := i.(p2p.Envelope)
					return ok && e.ChannelID == SnapshotChannel
				})).Run(func(args mock.Arguments) {
					e := args[0].(p2p.Envelope)

					// Marshal to simulate a wire roundtrip.
					bz, err := proto.Marshal(e.Message)
					require.NoError(t, err)
					err = proto.Unmarshal(bz, e.Message)
					require.NoError(t, err)
					responses = append(responses, e.Message.(*ssproto.SnapshotsResponse))
				}).Return(true)
			}

			// Start a reactor and send a SnapshotsRequestMessage, then wait for and check responses
			cfg := config.DefaultStateSyncConfig()
			r := NewReactor(*cfg, conn, nil, NopMetrics())
			err := r.Start()
			require.NoError(t, err)
			t.Cleanup(func() {
				if err := r.Stop(); err != nil {
					t.Error(err)
				}
			})

			r.Receive(p2p.Envelope{
				ChannelID: SnapshotChannel,
				Src:       peer,
				Message:   &ssproto.SnapshotsRequest{},
			})
			time.Sleep(100 * time.Millisecond)
			assert.Equal(t, tc.expectResponses, responses)

			conn.AssertExpectations(t)
			peer.AssertExpectations(t)
		})
	}
}
