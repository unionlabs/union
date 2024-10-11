// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build race

package slog

import "testing"

func wantAllocs(t *testing.T, want int, f func()) {
	t.Log("skipping allocation tests with race detector")
}
