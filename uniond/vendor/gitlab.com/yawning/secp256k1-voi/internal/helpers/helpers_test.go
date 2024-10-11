// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package helpers

import (
	"math"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestUint64IsZero(t *testing.T) {
	for _, v := range []uint64{
		0,
		1,
		math.MaxUint64,
	} {
		var expected uint64
		if v == 0 {
			expected = 1
		}

		require.Equal(t, expected, Uint64IsZero(v), "Uint64IsZero(%d)", v)
	}
}

func TestUint64IsNonzero(t *testing.T) {
	for _, v := range []uint64{
		0,
		1,
		math.MaxUint64,
	} {
		var expected uint64
		if v != 0 {
			expected = 1
		}

		require.Equal(t, expected, Uint64IsNonzero(v), "Uint64IsNonzero(%d)", v)
	}
}

func TestMustBytesFromHex(t *testing.T) {
	require.Panics(t, func() { MustBytesFromHex("The Light - Hex-Sealed Fusion") })
}

func TestMust256BitsFromHex(t *testing.T) {
	require.Panics(t, func() { Must256BitsFromHex("The Dark - Hex-Sealed Fusion") })
	require.Panics(t, func() { Must256BitsFromHex(strings.Repeat("0", 65)) })
}
