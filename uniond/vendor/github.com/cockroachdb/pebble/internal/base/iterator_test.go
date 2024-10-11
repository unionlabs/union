// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package base

import (
	"fmt"
	"testing"
)

func TestFlags(t *testing.T) {
	t.Run("SeekGEFlags", func(t *testing.T) {
		f := SeekGEFlagsNone
		flags := []flag{
			{
				"TrySeekUsingNext",
				func() bool { return f.TrySeekUsingNext() },
				func() { f = f.EnableTrySeekUsingNext() },
				func() { f = f.DisableTrySeekUsingNext() },
			},
			{
				"RelativeSeek",
				func() bool { return f.RelativeSeek() },
				func() { f = f.EnableRelativeSeek() },
				func() { f = f.DisableRelativeSeek() },
			},
			{
				"BatchJustRefreshed",
				func() bool { return f.BatchJustRefreshed() },
				func() { f = f.EnableBatchJustRefreshed() },
				func() { f = f.DisableBatchJustRefreshed() },
			},
		}
		ref := make([]bool, len(flags))
		checkCombination(t, 0, flags, ref)
	})
	t.Run("SeekLTFlags", func(t *testing.T) {
		f := SeekLTFlagsNone
		flags := []flag{
			{
				"RelativeSeek",
				func() bool { return f.RelativeSeek() },
				func() { f = f.EnableRelativeSeek() },
				func() { f = f.DisableRelativeSeek() },
			},
		}
		ref := make([]bool, len(flags))
		checkCombination(t, 0, flags, ref)
	})
}

type flag struct {
	label string
	pred  func() bool
	set   func()
	unset func()
}

func checkCombination(t *testing.T, i int, flags []flag, ref []bool) {
	if i >= len(ref) {
		// Verify that ref matches the flag predicates.
		for j := 0; j < i; j++ {
			if got := flags[j].pred(); ref[j] != got {
				t.Errorf("%s() = %t, want %t", flags[j].label, got, ref[j])
			}
		}
		return
	}

	// flag i remains unset.
	t.Run(fmt.Sprintf("%s begin unset", flags[i].label), func(t *testing.T) {
		checkCombination(t, i+1, flags, ref)
	})

	// set flag i
	ref[i] = true
	flags[i].set()
	t.Run(fmt.Sprintf("%s set", flags[i].label), func(t *testing.T) {
		checkCombination(t, i+1, flags, ref)
	})

	// unset flag i
	ref[i] = false
	flags[i].unset()
	t.Run(fmt.Sprintf("%s unset", flags[i].label), func(t *testing.T) {
		checkCombination(t, i+1, flags, ref)
	})
}
