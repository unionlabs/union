// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sharedcache

import (
	"reflect"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestSharedCacheLruList(t *testing.T) {
	var s shard
	s.mu.blocks = make([]cacheBlockState, 100)
	expect := func(vals ...int) {
		t.Helper()
		if s.mu.lruHead == invalidBlockIndex {
			if len(vals) != 0 {
				t.Fatalf("expected non-empty list")
			}
			return
		}
		var list []int
		prev := s.lruPrev(s.mu.lruHead)
		b := s.mu.lruHead
		for {
			list = append(list, int(b))
			if s.lruPrev(b) != prev {
				t.Fatalf("back link broken: %d:next=%d,prev=%d  %d:next=%d,prev=%d",
					prev, s.lruNext(prev), s.lruPrev(prev),
					b, s.lruNext(b), s.lruPrev(b),
				)
			}
			prev = b
			b = s.lruNext(b)
			if b == s.mu.lruHead {
				break
			}
		}
		if !reflect.DeepEqual(vals, list) {
			t.Fatalf("expected %v, got %v", vals, list)
		}
	}

	s.mu.lruHead = invalidBlockIndex
	expect()
	s.lruInsertFront(1)
	expect(1)
	s.lruInsertFront(10)
	expect(10, 1)
	s.lruInsertFront(5)
	expect(5, 10, 1)
	s.lruUnlink(5)
	expect(10, 1)
	s.lruUnlink(1)
	expect(10)
	s.lruUnlink(10)
	expect()
}

func TestSharedCacheFreeList(t *testing.T) {
	var s shard
	s.mu.blocks = make([]cacheBlockState, 100)
	expect := func(vals ...int) {
		t.Helper()
		var list []int
		for b := s.mu.freeHead; b != invalidBlockIndex; b = s.mu.blocks[b].next {
			list = append(list, int(b))
		}
		if !reflect.DeepEqual(vals, list) {
			t.Fatalf("expected %v, got %v", vals, list)
		}
	}
	s.mu.freeHead = invalidBlockIndex
	expect()
	s.freePush(1)
	expect(1)
	s.freePush(10)
	expect(10, 1)
	s.freePush(20)
	expect(20, 10, 1)
	require.Equal(t, cacheBlockIndex(20), s.freePop())
	expect(10, 1)
	require.Equal(t, cacheBlockIndex(10), s.freePop())
	expect(1)
	require.Equal(t, cacheBlockIndex(1), s.freePop())
	expect()
}
