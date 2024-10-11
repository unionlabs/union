// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"os"
	"strings"
	"testing"
)

// TestFiles checks that every file in this package has a build constraint on Go 1.21.
//
// The QUIC implementation depends on crypto/tls features added in Go 1.21,
// so there's no point in trying to build on anything older.
//
// Drop this test when the x/net go.mod depends on 1.21 or newer.
func TestFiles(t *testing.T) {
	f, err := os.Open(".")
	if err != nil {
		t.Fatal(err)
	}
	names, err := f.Readdirnames(-1)
	if err != nil {
		t.Fatal(err)
	}
	for _, name := range names {
		if !strings.HasSuffix(name, ".go") {
			continue
		}
		b, err := os.ReadFile(name)
		if err != nil {
			t.Fatal(err)
		}
		// Check for copyright header while we're in here.
		if !bytes.Contains(b, []byte("The Go Authors.")) {
			t.Errorf("%v: missing copyright", name)
		}
		// doc.go doesn't need a build constraint.
		if name == "doc.go" {
			continue
		}
		if !bytes.Contains(b, []byte("//go:build go1.21")) {
			t.Errorf("%v: missing constraint on go1.21", name)
		}
	}
}
