// Copyright 2023 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package bitutil

import (
	"bytes"
	"testing"

	"github.com/ethereum/go-ethereum/common/bitutil"
)

func FuzzEncoder(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzzEncode(data)
	})
}
func FuzzDecoder(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzzDecode(data)
	})
}

// fuzzEncode implements a go-fuzz fuzzer method to test the bitset encoding and
// decoding algorithm.
func fuzzEncode(data []byte) {
	proc, _ := bitutil.DecompressBytes(bitutil.CompressBytes(data), len(data))
	if !bytes.Equal(data, proc) {
		panic("content mismatch")
	}
}

// fuzzDecode implements a go-fuzz fuzzer method to test the bit decoding and
// reencoding algorithm.
func fuzzDecode(data []byte) {
	blob, err := bitutil.DecompressBytes(data, 1024)
	if err != nil {
		return
	}
	// re-compress it (it's OK if the re-compressed differs from the
	// original - the first input may not have been compressed at all)
	comp := bitutil.CompressBytes(blob)
	if len(comp) > len(blob) {
		// After compression, it must be smaller or equal
		panic("bad compression")
	}
	// But decompressing it once again should work
	decomp, err := bitutil.DecompressBytes(data, 1024)
	if err != nil {
		panic(err)
	}
	if !bytes.Equal(decomp, blob) {
		panic("content mismatch")
	}
}
