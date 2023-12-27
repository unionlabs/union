// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build cgo
// +build cgo

package sstable

import (
	"bytes"

	"github.com/DataDog/zstd"
)

// decodeZstd decompresses b with the Zstandard algorithm.
// It reuses the preallocated capacity of decodedBuf if it is sufficient.
// On success, it returns the decoded byte slice.
func decodeZstd(decodedBuf, b []byte) ([]byte, error) {
	return zstd.Decompress(decodedBuf, b)
}

// encodeZstd compresses b with the Zstandard algorithm at default compression
// level (level 3). It reuses the preallocated capacity of compressedBuf if it
// is sufficient. The subslice `compressedBuf[:varIntLen]` should already encode
// the length of `b` before calling encodeZstd. It returns the encoded byte
// slice, including the `compressedBuf[:varIntLen]` prefix.
func encodeZstd(compressedBuf []byte, varIntLen int, b []byte) []byte {
	buf := bytes.NewBuffer(compressedBuf[:varIntLen])
	writer := zstd.NewWriterLevel(buf, 3)
	writer.Write(b)
	writer.Close()
	return buf.Bytes()
}
