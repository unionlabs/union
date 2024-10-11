// Copyright 2013 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

// Package bloom implements Bloom filters.
package bloom // import "github.com/cockroachdb/pebble/bloom"

import (
	"encoding/binary"
	"fmt"
	"sync"

	"github.com/cockroachdb/pebble/internal/base"
)

const (
	cacheLineSize = 64
	cacheLineBits = cacheLineSize * 8
)

type tableFilter []byte

func (f tableFilter) MayContain(key []byte) bool {
	if len(f) <= 5 {
		return false
	}
	n := len(f) - 5
	nProbes := f[n]
	nLines := binary.LittleEndian.Uint32(f[n+1:])
	cacheLineBits := 8 * (uint32(n) / nLines)

	h := hash(key)
	delta := h>>17 | h<<15
	b := (h % nLines) * cacheLineBits

	for j := uint8(0); j < nProbes; j++ {
		bitPos := b + (h % cacheLineBits)
		if f[bitPos/8]&(1<<(bitPos%8)) == 0 {
			return false
		}
		h += delta
	}
	return true
}

func calculateProbes(bitsPerKey int) uint32 {
	// We intentionally round down to reduce probing cost a little bit
	n := uint32(float64(bitsPerKey) * 0.69) // 0.69 =~ ln(2)
	if n < 1 {
		n = 1
	}
	if n > 30 {
		n = 30
	}
	return n
}

// extend appends n zero bytes to b. It returns the overall slice (of length
// n+len(originalB)) and the slice of n trailing zeroes.
func extend(b []byte, n int) (overall, trailer []byte) {
	want := n + len(b)
	if want <= cap(b) {
		overall = b[:want]
		trailer = overall[len(b):]
		for i := range trailer {
			trailer[i] = 0
		}
	} else {
		// Grow the capacity exponentially, with a 1KiB minimum.
		c := 1024
		for c < want {
			c += c / 4
		}
		overall = make([]byte, want, c)
		trailer = overall[len(b):]
		copy(overall, b)
	}
	return overall, trailer
}

// hash implements a hashing algorithm similar to the Murmur hash.
func hash(b []byte) uint32 {
	const (
		seed = 0xbc9f1d34
		m    = 0xc6a4a793
	)
	h := uint32(seed) ^ uint32(uint64(uint32(len(b))*m))
	for ; len(b) >= 4; b = b[4:] {
		h += uint32(b[0]) | uint32(b[1])<<8 | uint32(b[2])<<16 | uint32(b[3])<<24
		h *= m
		h ^= h >> 16
	}

	// The code below first casts each byte to a signed 8-bit integer. This is
	// necessary to match RocksDB's behavior. Note that the `byte` type in Go is
	// unsigned. What is the difference between casting a signed 8-bit value vs
	// unsigned 8-bit value into an unsigned 32-bit value?
	// Sign-extension. Consider the value 250 which has the bit pattern 11111010:
	//
	//   uint32(250)        = 00000000000000000000000011111010
	//   uint32(int8(250))  = 11111111111111111111111111111010
	//
	// Note that the original LevelDB code did not explicitly cast to a signed
	// 8-bit value which left the behavior dependent on whether C characters were
	// signed or unsigned which is a compiler flag for gcc (-funsigned-char).
	switch len(b) {
	case 3:
		h += uint32(int8(b[2])) << 16
		fallthrough
	case 2:
		h += uint32(int8(b[1])) << 8
		fallthrough
	case 1:
		h += uint32(int8(b[0]))
		h *= m
		h ^= h >> 24
	}
	return h
}

const hashBlockLen = 16384

type hashBlock [hashBlockLen]uint32

var hashBlockPool = sync.Pool{
	New: func() interface{} {
		return &hashBlock{}
	},
}

type tableFilterWriter struct {
	bitsPerKey int

	numHashes int
	// We store the hashes in blocks.
	blocks   []*hashBlock
	lastHash uint32

	// Initial "in-line" storage for the blocks slice (to avoid some small
	// allocations).
	blocksBuf [16]*hashBlock
}

func newTableFilterWriter(bitsPerKey int) *tableFilterWriter {
	w := &tableFilterWriter{
		bitsPerKey: bitsPerKey,
	}
	w.blocks = w.blocksBuf[:0]
	return w
}

// AddKey implements the base.FilterWriter interface.
func (w *tableFilterWriter) AddKey(key []byte) {
	h := hash(key)
	if w.numHashes != 0 && h == w.lastHash {
		return
	}
	ofs := w.numHashes % hashBlockLen
	if ofs == 0 {
		// Time for a new block.
		w.blocks = append(w.blocks, hashBlockPool.Get().(*hashBlock))
	}
	w.blocks[len(w.blocks)-1][ofs] = h
	w.numHashes++
	w.lastHash = h
}

// Finish implements the base.FilterWriter interface.
func (w *tableFilterWriter) Finish(buf []byte) []byte {
	// The table filter format matches the RocksDB full-file filter format.
	var nLines int
	if w.numHashes != 0 {
		nLines = (w.numHashes*w.bitsPerKey + cacheLineBits - 1) / (cacheLineBits)
		// Make nLines an odd number to make sure more bits are involved when
		// determining which block.
		if nLines%2 == 0 {
			nLines++
		}
	}

	nBytes := nLines * cacheLineSize
	// +5: 4 bytes for num-lines, 1 byte for num-probes
	buf, filter := extend(buf, nBytes+5)

	if nLines != 0 {
		nProbes := calculateProbes(w.bitsPerKey)
		for bIdx, b := range w.blocks {
			length := hashBlockLen
			if bIdx == len(w.blocks)-1 && w.numHashes%hashBlockLen != 0 {
				length = w.numHashes % hashBlockLen
			}
			for _, h := range b[:length] {
				delta := h>>17 | h<<15 // rotate right 17 bits
				b := (h % uint32(nLines)) * (cacheLineBits)
				for i := uint32(0); i < nProbes; i++ {
					bitPos := b + (h % cacheLineBits)
					filter[bitPos/8] |= (1 << (bitPos % 8))
					h += delta
				}
			}
		}
		filter[nBytes] = byte(nProbes)
		binary.LittleEndian.PutUint32(filter[nBytes+1:], uint32(nLines))
	}

	// Release the hash blocks.
	for i, b := range w.blocks {
		hashBlockPool.Put(b)
		w.blocks[i] = nil
	}
	w.blocks = w.blocks[:0]
	w.numHashes = 0
	return buf
}

// FilterPolicy implements the FilterPolicy interface from the pebble package.
//
// The integer value is the approximate number of bits used per key. A good
// value is 10, which yields a filter with ~ 1% false positive rate.
type FilterPolicy int

var _ base.FilterPolicy = FilterPolicy(0)

// Name implements the pebble.FilterPolicy interface.
func (p FilterPolicy) Name() string {
	// This string looks arbitrary, but its value is written to LevelDB .sst
	// files, and should be this exact value to be compatible with those files
	// and with the C++ LevelDB code.
	return "rocksdb.BuiltinBloomFilter"
}

// MayContain implements the pebble.FilterPolicy interface.
func (p FilterPolicy) MayContain(ftype base.FilterType, f, key []byte) bool {
	switch ftype {
	case base.TableFilter:
		return tableFilter(f).MayContain(key)
	default:
		panic(fmt.Sprintf("unknown filter type: %v", ftype))
	}
}

// NewWriter implements the pebble.FilterPolicy interface.
func (p FilterPolicy) NewWriter(ftype base.FilterType) base.FilterWriter {
	switch ftype {
	case base.TableFilter:
		return newTableFilterWriter(int(p))
	default:
		panic(fmt.Sprintf("unknown filter type: %v", ftype))
	}
}
