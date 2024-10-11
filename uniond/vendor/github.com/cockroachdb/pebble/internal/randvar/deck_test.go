// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package randvar

import "testing"

func TestDeck(t *testing.T) {
	d := NewDeck(nil, 10, 20, 20, 0, 30)

	x := make([]int, 10000)
	for i := range x {
		x[i] = d.Int()
	}

	if testing.Verbose() {
		dumpSamples(x)
	}
}
