// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"encoding/binary"
	"math/rand"
	"testing"
	"time"

	"github.com/cockroachdb/pebble/internal/cache"
	"github.com/stretchr/testify/require"
)

func TestCompressionRoundtrip(t *testing.T) {
	seed := time.Now().UnixNano()
	t.Logf("seed %d", seed)
	rng := rand.New(rand.NewSource(seed))

	for compression := DefaultCompression + 1; compression < NCompression; compression++ {
		t.Run(compression.String(), func(t *testing.T) {
			payload := make([]byte, rng.Intn(10<<10 /* 10 KiB */))
			rng.Read(payload)
			// Create a randomly-sized buffer to house the compressed output. If it's
			// not sufficient, compressBlock should allocate one that is.
			compressedBuf := make([]byte, rng.Intn(1<<10 /* 1 KiB */))

			btyp, compressed := compressBlock(compression, payload, compressedBuf)
			v, err := decompressBlock(btyp, compressed)
			require.NoError(t, err)
			got := payload
			if v != nil {
				got = v.Buf()
				require.Equal(t, payload, got)
				cache.Free(v)
			}
		})
	}
}

// TestDecompressionError tests that a decompressing a value that does not
// decompress returns an error.
func TestDecompressionError(t *testing.T) {
	rng := rand.New(rand.NewSource(1 /* fixed seed */))

	// Create a buffer to represent a faux zstd compressed block. It's prefixed
	// with a uvarint of the appropriate length, followed by garabge.
	fauxCompressed := make([]byte, rng.Intn(10<<10 /* 10 KiB */))
	compressedPayloadLen := len(fauxCompressed) - binary.MaxVarintLen64
	n := binary.PutUvarint(fauxCompressed, uint64(compressedPayloadLen))
	fauxCompressed = fauxCompressed[:n+compressedPayloadLen]
	rng.Read(fauxCompressed[n:])

	v, err := decompressBlock(zstdCompressionBlockType, fauxCompressed)
	t.Log(err)
	require.Error(t, err)
	require.Nil(t, v)
}
