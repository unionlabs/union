package kv

import (
	"context"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestStore_GraffitiOrderedIndex_ReadAndWrite(t *testing.T) {
	ctx := context.Background()
	db := setupDB(t, [][fieldparams.BLSPubkeyLength]byte{})
	tests := []struct {
		name     string
		want     uint64
		write    uint64
		fileHash [32]byte
	}{
		{
			name:     "empty then write",
			want:     0,
			write:    15,
			fileHash: hash.Hash([]byte("one")),
		},
		{
			name:     "update with same file hash",
			want:     15,
			write:    20,
			fileHash: hash.Hash([]byte("one")),
		},
		{
			name:     "continued updates",
			want:     20,
			write:    21,
			fileHash: hash.Hash([]byte("one")),
		},
		{
			name:     "reset with new file hash",
			want:     0,
			write:    10,
			fileHash: hash.Hash([]byte("two")),
		},
		{
			name:     "read with new file hash",
			want:     10,
			fileHash: hash.Hash([]byte("two")),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := db.GraffitiOrderedIndex(ctx, tt.fileHash)
			require.NoError(t, err)
			require.DeepEqual(t, tt.want, got)
			err = db.SaveGraffitiOrderedIndex(ctx, tt.write)
			require.NoError(t, err)
		})
	}
}
