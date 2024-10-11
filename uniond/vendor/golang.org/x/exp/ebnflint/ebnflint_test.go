// Copyright 2012 The Go Authors.  All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package main

import (
	"os"
	"runtime"
	"testing"
)

func TestSpec(t *testing.T) {
	if err := verify(runtime.GOROOT()+"/doc/go_spec.html", "SourceFile", nil); err != nil {
		if os.IsNotExist(err) {
			// Couldn't find/open the file - skip test rather than
			// complain since not all builders copy the spec.
			t.Skip("spec file not found")
		}
		t.Fatal(err)
	}
}
