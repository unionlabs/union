package statesync

import (
	"encoding/hex"
	"testing"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/require"

	ssproto "github.com/cometbft/cometbft/api/cometbft/statesync/v1"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/cometbft/cometbft/types"
)

func TestValidateMsg(t *testing.T) {
	testcases := map[string]struct {
		msg   proto.Message
		valid bool
	}{
		"nil":       {nil, false},
		"unrelated": {&cmtproto.Block{}, false},

		"ChunkRequest valid":    {&ssproto.ChunkRequest{Height: 1, Format: 1, Index: 1}, true},
		"ChunkRequest 0 height": {&ssproto.ChunkRequest{Height: 0, Format: 1, Index: 1}, false},
		"ChunkRequest 0 format": {&ssproto.ChunkRequest{Height: 1, Format: 0, Index: 1}, true},
		"ChunkRequest 0 chunk":  {&ssproto.ChunkRequest{Height: 1, Format: 1, Index: 0}, true},

		"ChunkResponse valid": {
			&ssproto.ChunkResponse{Height: 1, Format: 1, Index: 1, Chunk: []byte{1}},
			true,
		},
		"ChunkResponse 0 height": {
			&ssproto.ChunkResponse{Height: 0, Format: 1, Index: 1, Chunk: []byte{1}},
			false,
		},
		"ChunkResponse 0 format": {
			&ssproto.ChunkResponse{Height: 1, Format: 0, Index: 1, Chunk: []byte{1}},
			true,
		},
		"ChunkResponse 0 chunk": {
			&ssproto.ChunkResponse{Height: 1, Format: 1, Index: 0, Chunk: []byte{1}},
			true,
		},
		"ChunkResponse empty body": {
			&ssproto.ChunkResponse{Height: 1, Format: 1, Index: 1, Chunk: []byte{}},
			true,
		},
		"ChunkResponse nil body": {
			&ssproto.ChunkResponse{Height: 1, Format: 1, Index: 1, Chunk: nil},
			false,
		},
		"ChunkResponse missing": {
			&ssproto.ChunkResponse{Height: 1, Format: 1, Index: 1, Missing: true},
			true,
		},
		"ChunkResponse missing with empty": {
			&ssproto.ChunkResponse{Height: 1, Format: 1, Index: 1, Missing: true, Chunk: []byte{}},
			true,
		},
		"ChunkResponse missing with body": {
			&ssproto.ChunkResponse{Height: 1, Format: 1, Index: 1, Missing: true, Chunk: []byte{1}},
			false,
		},

		"SnapshotsRequest valid": {&ssproto.SnapshotsRequest{}, true},

		"SnapshotsResponse valid": {
			&ssproto.SnapshotsResponse{Height: 1, Format: 1, Chunks: 2, Hash: []byte{1}},
			true,
		},
		"SnapshotsResponse 0 height": {
			&ssproto.SnapshotsResponse{Height: 0, Format: 1, Chunks: 2, Hash: []byte{1}},
			false,
		},
		"SnapshotsResponse 0 format": {
			&ssproto.SnapshotsResponse{Height: 1, Format: 0, Chunks: 2, Hash: []byte{1}},
			true,
		},
		"SnapshotsResponse 0 chunks": {
			&ssproto.SnapshotsResponse{Height: 1, Format: 1, Hash: []byte{1}},
			false,
		},
		"SnapshotsResponse no hash": {
			&ssproto.SnapshotsResponse{Height: 1, Format: 1, Chunks: 2, Hash: []byte{}},
			false,
		},
	}
	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			err := validateMsg(tc.msg)
			if tc.valid {
				require.NoError(t, err)
			} else {
				require.Error(t, err)
			}
		})
	}
}

//nolint:lll // ignore line length
func TestStateSyncVectors(t *testing.T) {
	testCases := []struct {
		testName string
		msg      proto.Message
		expBytes string
	}{
		{"SnapshotsRequest", &ssproto.SnapshotsRequest{}, "0a00"},
		{"SnapshotsResponse", &ssproto.SnapshotsResponse{Height: 1, Format: 2, Chunks: 3, Hash: []byte("chuck hash"), Metadata: []byte("snapshot metadata")}, "1225080110021803220a636875636b20686173682a11736e617073686f74206d65746164617461"},
		{"ChunkRequest", &ssproto.ChunkRequest{Height: 1, Format: 2, Index: 3}, "1a06080110021803"},
		{"ChunkResponse", &ssproto.ChunkResponse{Height: 1, Format: 2, Index: 3, Chunk: []byte("it's a chunk")}, "2214080110021803220c697427732061206368756e6b"},
	}

	for _, tc := range testCases {
		w := tc.msg.(types.Wrapper).Wrap()
		bz, err := proto.Marshal(w)
		require.NoError(t, err)

		require.Equal(t, tc.expBytes, hex.EncodeToString(bz), tc.testName)
	}
}
