// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package randvar

import "testing"

func TestWeighted(t *testing.T) {
	w := NewWeighted(nil, 1, 2, 2, 0, 3)

	x := make([]int, 10000)
	for i := range x {
		x[i] = w.Int()
	}

	if testing.Verbose() {
		dumpSamples(x)
	}
}
