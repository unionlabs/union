// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package intern

import (
	"bytes"
	"testing"

	"github.com/cockroachdb/pebble/internal/invariants"
)

func TestBytes(t *testing.T) {
	if invariants.RaceEnabled {
		// sync.Pool is a no-op under -race, making this test fail.
		t.Skip("not supported under -race")
	}

	const abc = "abc"
	s := bytes.Repeat([]byte(abc), 100)
	n := testing.AllocsPerRun(100, func() {
		for i := 0; i < 100; i++ {
			_ = Bytes(s[i*len(abc) : (i+1)*len(abc)])
		}
	})
	if n > 0 {
		t.Fatalf("Bytes allocated %d, want 0", int(n))
	}
}
