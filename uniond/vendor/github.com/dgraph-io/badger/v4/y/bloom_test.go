// Copyright 2013 The LevelDB-Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package y

import (
	"testing"
)

func (f Filter) String() string {
	s := make([]byte, 8*len(f))
	for i, x := range f {
		for j := 0; j < 8; j++ {
			if x&(1<<uint(j)) != 0 {
				s[8*i+j] = '1'
			} else {
				s[8*i+j] = '.'
			}
		}
	}
	return string(s)
}

func TestSmallBloomFilter(t *testing.T) {
	var hash []uint32
	for _, word := range [][]byte{
		[]byte("hello"),
		[]byte("world"),
	} {
		hash = append(hash, Hash(word))
	}

	f := NewFilter(hash, 10)
	got := f.String()
	// The magic want string comes from running the C++ leveldb code's bloom_test.cc.
	want := "1...1.........1.........1.....1...1...1.....1.........1.....1....11....."
	if got != want {
		t.Fatalf("bits:\ngot  %q\nwant %q", got, want)
	}

	m := map[string]bool{
		"hello": true,
		"world": true,
		"x":     false,
		"foo":   false,
	}
	for k, want := range m {
		got := f.MayContainKey([]byte(k))
		if got != want {
			t.Errorf("MayContain: k=%q: got %v, want %v", k, got, want)
		}
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
		var hashes []uint32
		for _, key := range keys {
			hashes = append(hashes, Hash(key))
		}
		f := NewFilter(hashes, 10)

		if len(f) > (length*10/8)+40 {
			t.Errorf("length=%d: len(f)=%d is too large", length, len(f))
			continue
		}

		// All added keys must match.
		for _, key := range keys {
			if !f.MayContainKey(key) {
				t.Errorf("length=%d: did not contain key %q", length, key)
				continue loop
			}
		}

		// Check false positive rate.
		nFalsePositive := 0
		for i := 0; i < 10000; i++ {
			if f.MayContainKey(le32(1e9 + i)) {
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
	// The magic want numbers come from running the C++ leveldb code in hash.cc.
	testCases := []struct {
		s    string
		want uint32
	}{
		{"", 0xbc9f1d34},
		{"g", 0xd04a8bda},
		{"go", 0x3e0b0745},
		{"gop", 0x0c326610},
		{"goph", 0x8c9d6390},
		{"gophe", 0x9bfd4b0a},
		{"gopher", 0xa78edc7c},
		{"I had a dream it would end this way.", 0xe14a9db9},
	}
	for _, tc := range testCases {
		if got := Hash([]byte(tc.s)); got != tc.want {
			t.Errorf("s=%q: got 0x%08x, want 0x%08x", tc.s, got, tc.want)
		}
	}
}
