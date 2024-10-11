// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package cpu

import (
	"runtime"
	"testing"
)

func TestAuxvFromRuntime(t *testing.T) {
	got := getAuxv()
	t.Logf("got: %v", got) // notably: we didn't panic
	if runtime.GOOS == "linux" && len(got) == 0 {
		t.Errorf("expected auxv on linux; got zero entries")
	}
}
