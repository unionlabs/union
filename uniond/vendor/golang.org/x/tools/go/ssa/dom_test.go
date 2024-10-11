// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ssa_test

import (
	"fmt"
	"path/filepath"
	"testing"

	"golang.org/x/tools/go/packages"
	"golang.org/x/tools/go/ssa/ssautil"
	"golang.org/x/tools/internal/testenv"
)

func TestDominatorOrder(t *testing.T) {
	testenv.NeedsGoBuild(t) // for go/packages

	const src = `package p

func f(cond bool) {
	// (Print operands match BasicBlock IDs.)
	print(0)
	if cond {
		print(1)
	} else {
		print(2)
	}
	print(3)
}
`
	dir := t.TempDir()
	cfg := &packages.Config{
		Dir:  dir,
		Mode: packages.LoadSyntax,
		Overlay: map[string][]byte{
			filepath.Join(dir, "p.go"): []byte(src),
		},
	}
	initial, err := packages.Load(cfg, "./p.go")
	if err != nil {
		t.Fatal(err)
	}
	if packages.PrintErrors(initial) > 0 {
		t.Fatal("packages contain errors")
	}
	_, pkgs := ssautil.Packages(initial, 0)
	p := pkgs[0]
	p.Build()
	f := p.Func("f")

	if got, want := fmt.Sprint(f.DomPreorder()), "[0 1 2 3]"; got != want {
		t.Errorf("DomPreorder: got %v, want %s", got, want)
	}
	if got, want := fmt.Sprint(f.DomPostorder()), "[1 2 3 0]"; got != want {
		t.Errorf("DomPostorder: got %v, want %s", got, want)
	}
}
