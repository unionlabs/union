// Copyright 2022 The go-ethereum Authors
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

package lru

import (
	"encoding/binary"
	"fmt"
	"testing"
)

type testKey [8]byte

func mkKey(i int) (key testKey) {
	binary.LittleEndian.PutUint64(key[:], uint64(i))
	return key
}

func TestSizeConstrainedCache(t *testing.T) {
	lru := NewSizeConstrainedCache[testKey, []byte](100)
	var want uint64
	// Add 11 items of 10 byte each. First item should be swapped out
	for i := 0; i < 11; i++ {
		k := mkKey(i)
		v := fmt.Sprintf("value-%04d", i)
		lru.Add(k, []byte(v))
		want += uint64(len(v))
		if want > 100 {
			want = 100
		}
		if have := lru.size; have != want {
			t.Fatalf("size wrong, have %d want %d", have, want)
		}
	}
	// Zero:th should be evicted
	{
		k := mkKey(0)
		if _, ok := lru.Get(k); ok {
			t.Fatalf("should be evicted: %v", k)
		}
	}
	// Elems 1-11 should be present
	for i := 1; i < 11; i++ {
		k := mkKey(i)
		want := fmt.Sprintf("value-%04d", i)
		have, ok := lru.Get(k)
		if !ok {
			t.Fatalf("missing key %v", k)
		}
		if string(have) != want {
			t.Fatalf("wrong value, have %v want %v", have, want)
		}
	}
}

// This test adds inserting an element exceeding the max size.
func TestSizeConstrainedCacheOverflow(t *testing.T) {
	lru := NewSizeConstrainedCache[testKey, []byte](100)

	// Add 10 items of 10 byte each, filling the cache
	for i := 0; i < 10; i++ {
		k := mkKey(i)
		v := fmt.Sprintf("value-%04d", i)
		lru.Add(k, []byte(v))
	}
	// Add one single large elem. We expect it to swap out all entries.
	{
		k := mkKey(1337)
		v := make([]byte, 200)
		lru.Add(k, v)
	}
	// Elems 0-9 should be missing
	for i := 1; i < 10; i++ {
		k := mkKey(i)
		if _, ok := lru.Get(k); ok {
			t.Fatalf("should be evicted: %v", k)
		}
	}
	// The size should be accurate
	if have, want := lru.size, uint64(200); have != want {
		t.Fatalf("size wrong, have %d want %d", have, want)
	}
	// Adding one small item should swap out the large one
	{
		i := 0
		k := mkKey(i)
		v := fmt.Sprintf("value-%04d", i)
		lru.Add(k, []byte(v))
		if have, want := lru.size, uint64(10); have != want {
			t.Fatalf("size wrong, have %d want %d", have, want)
		}
	}
}

// This checks what happens when inserting the same k/v multiple times.
func TestSizeConstrainedCacheSameItem(t *testing.T) {
	lru := NewSizeConstrainedCache[testKey, []byte](100)

	// Add one 10 byte-item 10 times.
	k := mkKey(0)
	v := fmt.Sprintf("value-%04d", 0)
	for i := 0; i < 10; i++ {
		lru.Add(k, []byte(v))
	}

	// The size should be accurate.
	if have, want := lru.size, uint64(10); have != want {
		t.Fatalf("size wrong, have %d want %d", have, want)
	}
}

// This tests that empty/nil values are handled correctly.
func TestSizeConstrainedCacheEmpties(t *testing.T) {
	lru := NewSizeConstrainedCache[testKey, []byte](100)

	// This test abuses the lru a bit, using different keys for identical value(s).
	for i := 0; i < 10; i++ {
		lru.Add(testKey{byte(i)}, []byte{})
		lru.Add(testKey{byte(255 - i)}, nil)
	}

	// The size should not count, only the values count. So this could be a DoS
	// since it basically has no cap, and it is intentionally overloaded with
	// different-keyed 0-length values.
	if have, want := lru.size, uint64(0); have != want {
		t.Fatalf("size wrong, have %d want %d", have, want)
	}

	for i := 0; i < 10; i++ {
		if v, ok := lru.Get(testKey{byte(i)}); !ok {
			t.Fatalf("test %d: expected presence", i)
		} else if v == nil {
			t.Fatalf("test %d, v is nil", i)
		}

		if v, ok := lru.Get(testKey{byte(255 - i)}); !ok {
			t.Fatalf("test %d: expected presence", i)
		} else if v != nil {
			t.Fatalf("test %d, v is not nil", i)
		}
	}
}
