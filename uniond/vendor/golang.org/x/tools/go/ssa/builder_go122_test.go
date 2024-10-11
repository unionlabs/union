// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.22
// +build go1.22

package ssa_test

import (
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"go/types"
	"testing"

	"golang.org/x/tools/go/expect"
	"golang.org/x/tools/go/ssa"
	"golang.org/x/tools/go/ssa/ssautil"
	"golang.org/x/tools/internal/testenv"
)

// TestMultipleGoversions tests that globals initialized to equivalent
// function literals are compiled based on the different GoVersion in each file.
func TestMultipleGoversions(t *testing.T) {
	var contents = map[string]string{
		"post.go": `
	//go:build go1.22
	package p

	var distinct = func(l []int) {
		for i := range l {
			print(&i)
		}
	}
	`,
		"pre.go": `
	package p

	var same = func(l []int) {
		for i := range l {
			print(&i)
		}
	}
	`,
	}

	fset := token.NewFileSet()
	var files []*ast.File
	for _, fname := range []string{"post.go", "pre.go"} {
		file, err := parser.ParseFile(fset, fname, contents[fname], 0)
		if err != nil {
			t.Fatal(err)
		}
		files = append(files, file)
	}

	pkg := types.NewPackage("p", "")
	conf := &types.Config{Importer: nil, GoVersion: "go1.21"}
	p, _, err := ssautil.BuildPackage(conf, fset, pkg, files, ssa.SanityCheckFunctions)
	if err != nil {
		t.Fatal(err)
	}

	// Test that global is initialized to a function literal that was
	// compiled to have the expected for loop range variable lifetime for i.
	for _, test := range []struct {
		global *ssa.Global
		want   string // basic block to []*ssa.Alloc.
	}{
		{p.Var("same"), "map[entry:[new int (i)]]"},               // i is allocated in the entry block.
		{p.Var("distinct"), "map[rangeindex.body:[new int (i)]]"}, // i is allocated in the body block.
	} {
		// Find the function the test.name global is initialized to.
		var fn *ssa.Function
		for _, b := range p.Func("init").Blocks {
			for _, instr := range b.Instrs {
				if s, ok := instr.(*ssa.Store); ok && s.Addr == test.global {
					fn, _ = s.Val.(*ssa.Function)
				}
			}
		}
		if fn == nil {
			t.Fatalf("Failed to find *ssa.Function for initial value of global %s", test.global)
		}

		allocs := make(map[string][]string) // block comments -> []Alloc
		for _, b := range fn.Blocks {
			for _, instr := range b.Instrs {
				if a, ok := instr.(*ssa.Alloc); ok {
					allocs[b.Comment] = append(allocs[b.Comment], a.String())
				}
			}
		}
		if got := fmt.Sprint(allocs); got != test.want {
			t.Errorf("[%s:=%s] expected the allocations to be in the basic blocks %q, got %q", test.global, fn, test.want, got)
		}
	}
}

const rangeOverIntSrc = `
package p

type I uint8

func noKey(x int) {
	for range x {
		// does not crash
	}
}

func untypedConstantOperand() {
	for i := range 10 {
		print(i) /*@ types("int")*/
	}
}

func unsignedOperand(x uint64) {
	for i := range x {
		print(i) /*@ types("uint64")*/
	}
}

func namedOperand(x I) {
	for i := range x {
		print(i)  /*@ types("p.I")*/
	}
}

func typeparamOperand[T int](x T) {
	for i := range x {
		print(i)  /*@ types("T")*/
	}
}

func assignment(x I) {
	var k I
	for k = range x {
		print(k) /*@ types("p.I")*/
	}
}
`

// TestRangeOverInt tests that, in a range-over-int (#61405),
// the type of each range var v (identified by print(v) calls)
// has the expected type.
func TestRangeOverInt(t *testing.T) {
	testenv.NeedsGoExperiment(t, "range")

	fset := token.NewFileSet()
	f, err := parser.ParseFile(fset, "p.go", rangeOverIntSrc, parser.ParseComments)
	if err != nil {
		t.Fatal(err)
	}

	pkg := types.NewPackage("p", "")
	conf := &types.Config{}
	p, _, err := ssautil.BuildPackage(conf, fset, pkg, []*ast.File{f}, ssa.SanityCheckFunctions)
	if err != nil {
		t.Fatal(err)
	}

	// Collect all notes in f, i.e. comments starting with "//@ types".
	notes, err := expect.ExtractGo(fset, f)
	if err != nil {
		t.Fatal(err)
	}

	// Collect calls to the built-in print function.
	fns := make(map[*ssa.Function]bool)
	for _, mem := range p.Members {
		if fn, ok := mem.(*ssa.Function); ok {
			fns[fn] = true
		}
	}
	probes := callsTo(fns, "print")
	expectations := matchNotes(fset, notes, probes)

	for call := range probes {
		if expectations[call] == nil {
			t.Errorf("Unmatched call: %v @ %s", call, fset.Position(call.Pos()))
		}
	}

	// Check each expectation.
	for call, note := range expectations {
		var args []string
		for _, a := range call.Args {
			args = append(args, a.Type().String())
		}
		if got, want := fmt.Sprint(args), fmt.Sprint(note.Args); got != want {
			at := fset.Position(call.Pos())
			t.Errorf("%s: arguments to print had types %s, want %s", at, got, want)
			logFunction(t, probes[call])
		}
	}
}
