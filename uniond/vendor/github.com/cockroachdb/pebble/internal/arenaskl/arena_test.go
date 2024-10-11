/*
 * Copyright 2017 Dgraph Labs, Inc. and Contributors
 * Modifications copyright (C) 2017 Andy Kimball and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package arenaskl

import (
	"math"
	"testing"

	"github.com/cockroachdb/pebble/internal/constants"
	"github.com/stretchr/testify/require"
)

func newArena(n uint32) *Arena {
	return NewArena(make([]byte, n))
}

// TestArenaSizeOverflow tests that large allocations do not cause Arena's
// internal size accounting to overflow and produce incorrect results.
func TestArenaSizeOverflow(t *testing.T) {
	a := newArena(constants.MaxUint32OrInt)

	// Allocating under the limit throws no error.
	offset, _, err := a.alloc(math.MaxUint16, 1, 0)
	require.Nil(t, err)
	require.Equal(t, uint32(1), offset)
	require.Equal(t, uint32(math.MaxUint16)+1, a.Size())

	// Allocating over the limit could cause an accounting
	// overflow if 32-bit arithmetic was used. It shouldn't.
	_, _, err = a.alloc(math.MaxUint32, 1, 0)
	require.Equal(t, ErrArenaFull, err)
	require.Equal(t, uint32(constants.MaxUint32OrInt), a.Size())

	// Continuing to allocate continues to throw an error.
	_, _, err = a.alloc(math.MaxUint16, 1, 0)
	require.Equal(t, ErrArenaFull, err)
	require.Equal(t, uint32(constants.MaxUint32OrInt), a.Size())
}
