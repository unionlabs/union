package kv

import (
	"context"
	"crypto/rand"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	bolt "go.etcd.io/bbolt"
)

func Test_deleteValueForIndices(t *testing.T) {
	db := setupDB(t)
	blocks := make([]byte, 128)
	_, err := rand.Read(blocks)
	require.NoError(t, err)
	tests := []struct {
		name          string
		inputIndices  map[string][]byte
		root          []byte
		outputIndices map[string][]byte
		wantedErr     string
	}{
		{
			name:          "empty input, no root",
			inputIndices:  map[string][]byte{},
			root:          []byte{},
			outputIndices: map[string][]byte{},
		},
		{
			name:          "empty input, root does not exist",
			inputIndices:  map[string][]byte{},
			root:          bytesutil.PadTo([]byte("not found"), 32),
			outputIndices: map[string][]byte{},
		},
		{
			name: "non empty input, root does not exist",
			inputIndices: map[string][]byte{
				"blocks": bytesutil.PadTo([]byte{0xde, 0xad, 0xbe, 0xef}, 64),
			},
			root: bytesutil.PadTo([]byte("not found"), 32),
			outputIndices: map[string][]byte{
				"blocks": bytesutil.PadTo([]byte{0xde, 0xad, 0xbe, 0xef}, 64),
			},
		},
		{
			name: "removes value for a single bucket",
			inputIndices: map[string][]byte{
				"blocks": {0xde, 0xad, 0xbe, 0xef},
			},
			root: []byte{0xde},
			outputIndices: map[string][]byte{
				"blocks": {0xad, 0xbe, 0xef},
			},
		},
		{
			name: "removes multi-byte value for a single bucket (non-aligned)",
			inputIndices: map[string][]byte{
				"blocks": {0xde, 0xad, 0xbe, 0xef},
			},
			root: []byte{0xad, 0xbe},
			outputIndices: map[string][]byte{
				"blocks": {0xde, 0xad, 0xbe, 0xef},
			},
		},
		{
			name: "removes multi-byte value for a single bucket (non-aligned)",
			inputIndices: map[string][]byte{
				"blocks": {0xde, 0xad, 0xbe, 0xef, 0xff, 0x01},
			},
			root: []byte{0xbe, 0xef},
			outputIndices: map[string][]byte{
				"blocks": {0xde, 0xad, 0xff, 0x01},
			},
		},
		{
			name: "removes value from multiple buckets",
			inputIndices: map[string][]byte{
				"blocks":      {0xff, 0x32, 0x45, 0x25, 0xde, 0xad, 0xbe, 0xef, 0x24},
				"state":       {0x01, 0x02, 0x03, 0x04},
				"check-point": {0xde, 0xad, 0xbe, 0xef},
				"powchain":    {0xba, 0xad, 0xb0, 0x00, 0xde, 0xad, 0xbe, 0xef, 0xff},
			},
			root: []byte{0xde, 0xad, 0xbe, 0xef},
			outputIndices: map[string][]byte{
				"blocks":      {0xff, 0x32, 0x45, 0x25, 0x24},
				"state":       {0x01, 0x02, 0x03, 0x04},
				"check-point": nil,
				"powchain":    {0xba, 0xad, 0xb0, 0x00, 0xff},
			},
		},
		{
			name: "root as subsequence of two values (preserve)",
			inputIndices: map[string][]byte{
				"blocks": blocks,
			},
			outputIndices: map[string][]byte{
				"blocks": blocks,
			},
			root: blocks[48:80],
		},
		{
			name: "root as subsequence of two values (remove)",
			inputIndices: map[string][]byte{
				"blocks": blocks,
			},
			outputIndices: map[string][]byte{
				"blocks": append(blocks[0:64], blocks[96:]...),
			},
			root: blocks[64:96],
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := db.db.Update(func(tx *bolt.Tx) error {
				for k, idx := range tt.inputIndices {
					bkt := tx.Bucket([]byte(k))
					require.NoError(t, bkt.Put(idx, tt.inputIndices[k]))
				}
				err := deleteValueForIndices(context.Background(), tt.inputIndices, tt.root, tx)
				if tt.wantedErr != "" {
					assert.ErrorContains(t, tt.wantedErr, err)
					return nil
				}
				assert.NoError(t, err)
				// Check updated indices.
				for k, idx := range tt.inputIndices {
					bkt := tx.Bucket([]byte(k))
					valuesAtIndex := bkt.Get(idx)
					assert.DeepEqual(t, tt.outputIndices[k], valuesAtIndex)
				}
				return nil
			})
			require.NoError(t, err)
		})
	}
}

func testPack(bs [][32]byte) []byte {
	r := make([]byte, 0)
	for _, b := range bs {
		r = append(r, b[:]...)
	}
	return r
}

func TestSplitRoots(t *testing.T) {
	bt := make([][32]byte, 0)
	for _, x := range []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9} {
		var b [32]byte
		for i := 0; i < 32; i++ {
			b[i] = x
		}
		bt = append(bt, b)
	}
	cases := []struct {
		name   string
		b      []byte
		expect [][32]byte
		err    error
	}{
		{
			name: "misaligned",
			b:    make([]byte, 61),
			err:  errMisalignedRootList,
		},
		{
			name:   "happy",
			b:      testPack(bt[0:5]),
			expect: bt[0:5],
		},
		{
			name:   "single",
			b:      testPack([][32]byte{bt[0]}),
			expect: [][32]byte{bt[0]},
		},
		{
			name:   "empty",
			b:      []byte{},
			expect: [][32]byte{},
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			r, err := splitRoots(c.b)
			if c.err != nil {
				require.ErrorIs(t, err, c.err)
				return
			}
			require.NoError(t, err)
			require.DeepEqual(t, c.expect, r)
		})
	}
}
