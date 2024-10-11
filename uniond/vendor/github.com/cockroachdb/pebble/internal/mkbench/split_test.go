// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestFindOptimalSplit(t *testing.T) {
	testCases := []struct {
		passes, fails []int
		want          int
	}{
		{
			// Not enough data.
			passes: []int{},
			fails:  []int{},
			want:   -1,
		},
		{
			// Not enough data.
			passes: []int{1, 2, 3},
			fails:  []int{},
			want:   -1,
		},
		{
			// Not enough data.
			passes: []int{},
			fails:  []int{1, 2, 3},
			want:   -1,
		},
		{
			// Trivial example.
			passes: []int{100},
			fails:  []int{200},
			want:   150,
		},
		{
			// Example given in the doc comment for the function.
			passes: []int{100, 210, 300, 380, 450, 470, 490, 510, 520},
			fails:  []int{310, 450, 560, 610, 640, 700, 720, 810},
			want:   550,
		},
		{
			// Empirical data from an actual test run (~1hr).
			passes: []int{
				1000, 1100, 1300, 1700, 2500, 4100, 7300, 13700, 26500, 52100,
				52100, 52100, 26500, 26600, 26800, 27200, 28000, 29600, 32800,
				32800, 32900, 32900, 33000, 33000, 33100, 33100, 33100, 33100,
				33100, 33100, 33000, 33100, 33000, 32900, 33000, 33200, 33600,
				34400, 36000,
			},
			fails: []int{
				103300, 52200, 52200, 52100, 39200, 33100, 33200, 33300, 33200,
				33200, 33200, 33200, 33200, 33100, 33300, 33100, 33100, 33000,
				39200, 36100,
			},
			want: 33100,
		},
	}

	for _, tc := range testCases {
		t.Run("", func(t *testing.T) {
			split := findOptimalSplit(tc.passes, tc.fails)
			require.Equal(t, tc.want, split)
		})
	}
}
