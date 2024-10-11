// Copyright 2013 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package bloom

import (
	"crypto/rand"
	"strings"
	"testing"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/stretchr/testify/require"
)

func (f tableFilter) String() string {
	var buf strings.Builder
	for i, x := range f {
		if i > 0 {
			if i%8 == 0 {
				buf.WriteString("\n")
			} else {
				buf.WriteString("  ")
			}
		}

		for j := uint(0); j < 8; j++ {
			if x&(1<<(7-j)) != 0 {
				buf.WriteString("1")
			} else {
				buf.WriteString(".")
			}
		}
	}
	buf.WriteString("\n")
	return buf.String()
}

func newTableFilter(bitsPerKey int, keys ...[]byte) tableFilter {
	w := FilterPolicy(bitsPerKey).NewWriter(base.TableFilter)
	for _, key := range keys {
		w.AddKey(key)
	}
	return tableFilter(w.Finish(nil))
}

func TestSmallBloomFilter(t *testing.T) {
	f := newTableFilter(10, []byte("hello"), []byte("world"))

	// The magic expected string comes from running RocksDB's util/bloom_test.cc:FullBloomTest.FullSmall.
	want := `
........  ........  ........  .......1  ........  ........  ........  ........
........  .1......  ........  .1......  ........  ........  ........  ........
...1....  ........  ........  ........  ........  ........  ........  ........
........  ........  ........  ........  ........  ........  ........  ...1....
........  ........  ........  ........  .....1..  ........  ........  ........
.......1  ........  ........  ........  ........  ........  .1......  ........
........  ........  ........  ........  ........  ...1....  ........  ........
.......1  ........  ........  ........  .1...1..  ........  ........  ........
.....11.  .......1  ........  ........  ........
`
	want = strings.TrimLeft(want, "\n")
	require.EqualValues(t, want, f.String())

	m := map[string]bool{
		"hello": true,
		"world": true,
		"x":     false,
		"foo":   false,
	}
	for k, want := range m {
		require.EqualValues(t, want, f.MayContain([]byte(k)))
	}
}

func TestBloomFilter(t *testing.T) {
	nextLength := func(x int) int {
		if x < 10 {
			return x + 1
		}
		if x < 100 {
			return x + 10
		}
		if x < 1000 {
			return x + 100
		}
		return x + 1000
	}
	le32 := func(i int) []byte {
		b := make([]byte, 4)
		b[0] = uint8(uint32(i) >> 0)
		b[1] = uint8(uint32(i) >> 8)
		b[2] = uint8(uint32(i) >> 16)
		b[3] = uint8(uint32(i) >> 24)
		return b
	}

	nMediocreFilters, nGoodFilters := 0, 0
loop:
	for length := 1; length <= 10000; length = nextLength(length) {
		keys := make([][]byte, 0, length)
		for i := 0; i < length; i++ {
			keys = append(keys, le32(i))
		}
		f := newTableFilter(10, keys...)
		// The size of the table bloom filter is measured in multiples of the
		// cache line size. The '+2' contribution captures the rounding up in the
		// length division plus preferring an odd number of cache lines. As such,
		// this formula isn't exact, but the exact formula is hard to read.
		maxLen := 5 + ((length*10)/cacheLineBits+2)*cacheLineSize
		if len(f) > maxLen {
			t.Errorf("length=%d: len(f)=%d > max len %d", length, len(f), maxLen)
			continue
		}

		// All added keys must match.
		for _, key := range keys {
			if !f.MayContain(key) {
				t.Errorf("length=%d: did not contain key %q", length, key)
				continue loop
			}
		}

		// Check false positive rate.
		nFalsePositive := 0
		for i := 0; i < 10000; i++ {
			if f.MayContain(le32(1e9 + i)) {
				nFalsePositive++
			}
		}
		if nFalsePositive > 0.02*10000 {
			t.Errorf("length=%d: %d false positives in 10000", length, nFalsePositive)
			continue
		}
		if nFalsePositive > 0.0125*10000 {
			nMediocreFilters++
		} else {
			nGoodFilters++
		}
	}

	if nMediocreFilters > nGoodFilters/5 {
		t.Errorf("%d mediocre filters but only %d good filters", nMediocreFilters, nGoodFilters)
	}
}

func TestHash(t *testing.T) {
	testCases := []struct {
		s        string
		expected uint32
	}{
		// The magic expected numbers come from RocksDB's util/hash_test.cc:TestHash.
		{"", 3164544308},
		{"\x08", 422599524},
		{"\x17", 3168152998},
		{"\x9a", 3195034349},
		{"\x1c", 2651681383},
		{"\x4d\x76", 2447836956},
		{"\x52\xd5", 3854228105},
		{"\x91\xf7", 31066776},
		{"\xd6\x27", 1806091603},
		{"\x30\x46\x0b", 3808221797},
		{"\x56\xdc\xd6", 2157698265},
		{"\xd4\x52\x33", 1721992661},
		{"\x6a\xb5\xf4", 2469105222},
		{"\x67\x53\x81\x1c", 118283265},
		{"\x69\xb8\xc0\x88", 3416318611},
		{"\x1e\x84\xaf\x2d", 3315003572},
		{"\x46\xdc\x54\xbe", 447346355},
		{"\xd0\x7a\x6e\xea\x56", 4255445370},
		{"\x86\x83\xd5\xa4\xd8", 2390603402},
		{"\xb7\x46\xbb\x77\xce", 2048907743},
		{"\x6c\xa8\xbc\xe5\x99", 2177978500},
		{"\x5c\x5e\xe1\xa0\x73\x81", 1036846008},
		{"\x08\x5d\x73\x1c\xe5\x2e", 229980482},
		{"\x42\xfb\xf2\x52\xb4\x10", 3655585422},
		{"\x73\xe1\xff\x56\x9c\xce", 3502708029},
		{"\x5c\xbe\x97\x75\x54\x9a\x52", 815120748},
		{"\x16\x82\x39\x49\x88\x2b\x36", 3056033698},
		{"\x59\x77\xf0\xa7\x24\xf4\x78", 587205227},
		{"\xd3\xa5\x7c\x0e\xc0\x02\x07", 2030937252},
		{"\x31\x1b\x98\x75\x96\x22\xd3\x9a", 469635402},
		{"\x38\xd6\xf7\x28\x20\xb4\x8a\xe9", 3530274698},
		{"\xbb\x18\x5d\xf4\x12\x03\xf7\x99", 1974545809},
		{"\x80\xd4\x3b\x3b\xae\x22\xa2\x78", 3563570120},
		{"\x1a\xb5\xd0\xfe\xab\xc3\x61\xb2\x99", 2706087434},
		{"\x8e\x4a\xc3\x18\x20\x2f\x06\xe6\x3c", 1534654151},
		{"\xb6\xc0\xdd\x05\x3f\xc4\x86\x4c\xef", 2355554696},
		{"\x9a\x5f\x78\x0d\xaf\x50\xe1\x1f\x55", 1400800912},
		{"\x22\x6f\x39\x1f\xf8\xdd\x4f\x52\x17\x94", 3420325137},
		{"\x32\x89\x2a\x75\x48\x3a\x4a\x02\x69\xdd", 3427803584},
		{"\x06\x92\x5c\xf4\x88\x0e\x7e\x68\x38\x3e", 1152407945},
		{"\xbd\x2c\x63\x38\xbf\xe9\x78\xb7\xbf\x15", 3382479516},
	}
	for _, tc := range testCases {
		t.Run("", func(t *testing.T) {
			require.EqualValues(t, tc.expected, hash([]byte(tc.s)))
		})
	}
}

func BenchmarkBloomFilter(b *testing.B) {
	const keyLen = 128
	const numKeys = 1024
	keys := make([][]byte, numKeys)
	for i := range keys {
		keys[i] = make([]byte, keyLen)
		_, _ = rand.Read(keys[i])
	}
	b.ResetTimer()
	policy := FilterPolicy(10)
	for i := 0; i < b.N; i++ {
		w := policy.NewWriter(base.TableFilter)
		for _, key := range keys {
			w.AddKey(key)
		}
		w.Finish(nil)
	}
}
