// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"encoding/binary"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/cache"
	"github.com/golang/snappy"
)

func decompressedLen(blockType blockType, b []byte) (int, int, error) {
	switch blockType {
	case noCompressionBlockType:
		return 0, 0, nil
	case snappyCompressionBlockType:
		l, err := snappy.DecodedLen(b)
		return l, 0, err
	case zstdCompressionBlockType:
		// This will also be used by zlib, bzip2 and lz4 to retrieve the decodedLen
		// if we implement these algorithms in the future.
		decodedLenU64, varIntLen := binary.Uvarint(b)
		if varIntLen <= 0 {
			return 0, 0, base.CorruptionErrorf("pebble/table: compression block has invalid length")
		}
		return int(decodedLenU64), varIntLen, nil
	default:
		return 0, 0, base.CorruptionErrorf("pebble/table: unknown block compression: %d", errors.Safe(blockType))
	}
}

func decompressInto(blockType blockType, compressed []byte, buf []byte) ([]byte, error) {
	var result []byte
	var err error
	switch blockType {
	case snappyCompressionBlockType:
		result, err = snappy.Decode(buf, compressed)
	case zstdCompressionBlockType:
		result, err = decodeZstd(buf, compressed)
	}
	if err != nil {
		return nil, base.MarkCorruptionError(err)
	}
	if len(result) != 0 && (len(result) != len(buf) || &result[0] != &buf[0]) {
		return nil, base.CorruptionErrorf("pebble/table: decompressed into unexpected buffer: %p != %p",
			errors.Safe(result), errors.Safe(buf))
	}
	return result, nil
}

// decompressBlock decompresses an SST block, with manually-allocated space.
// NB: If decompressBlock returns (nil, nil), no decompression was necessary and
// the caller may use `b` directly.
func decompressBlock(blockType blockType, b []byte) (*cache.Value, error) {
	if blockType == noCompressionBlockType {
		return nil, nil
	}
	// first obtain the decoded length.
	decodedLen, prefixLen, err := decompressedLen(blockType, b)
	if err != nil {
		return nil, err
	}
	b = b[prefixLen:]
	// Allocate sufficient space from the cache.
	decoded := cache.Alloc(decodedLen)
	decodedBuf := decoded.Buf()
	if _, err := decompressInto(blockType, b, decodedBuf); err != nil {
		cache.Free(decoded)
		return nil, err
	}
	return decoded, nil
}

// compressBlock compresses an SST block, using compressBuf as the desired destination.
func compressBlock(
	compression Compression, b []byte, compressedBuf []byte,
) (blockType blockType, compressed []byte) {
	switch compression {
	case SnappyCompression:
		return snappyCompressionBlockType, snappy.Encode(compressedBuf, b)
	case NoCompression:
		return noCompressionBlockType, b
	}

	if len(compressedBuf) < binary.MaxVarintLen64 {
		compressedBuf = append(compressedBuf, make([]byte, binary.MaxVarintLen64-len(compressedBuf))...)
	}
	varIntLen := binary.PutUvarint(compressedBuf, uint64(len(b)))
	switch compression {
	case ZstdCompression:
		return zstdCompressionBlockType, encodeZstd(compressedBuf, varIntLen, b)
	default:
		return noCompressionBlockType, b
	}
}
