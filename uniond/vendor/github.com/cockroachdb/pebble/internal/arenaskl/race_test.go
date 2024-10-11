//go:build race
// +build race

// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package arenaskl

import (
	"testing"

	"github.com/stretchr/testify/require"
)

// TestNodeArenaEnd tests allocating a node at the boundary of an arena. In Go
// 1.14 when the race detector is running, Go will also perform some pointer
// alignment checks. It will detect alignment issues, for example #667 where a
// node's memory would straddle the arena boundary, with unused regions of the
// node struct dipping into unallocated memory. This test is only run when the
// race build tag is provided.
func TestNodeArenaEnd(t *testing.T) {
	ikey := makeIkey("a")
	val := []byte("b")

	// Rather than hardcode an arena size at just the right size, try
	// allocating using successively larger arena sizes until we allocate
	// successfully. The prior attempt will have exercised the right code
	// path.
	for i := uint32(1); i < 256; i++ {
		a := newArena(i)
		_, err := newNode(a, 1, ikey, val)
		if err == nil {
			// We reached an arena size big enough to allocate a node.
			// If there's an issue at the boundary, the race detector would
			// have found it by now.
			t.Log(i)
			break
		}
		require.Equal(t, ErrArenaFull, err)
	}
}
