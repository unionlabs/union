// Copyright 2014 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// No testdata on Android.

//go:build !android
// +build !android

package cha_test

import (
	"bytes"
	"fmt"
	"go/ast"
	"go/build"
	"go/parser"
	"go/token"
	"go/types"
	"os"
	"sort"
	"strings"
	"testing"

	"golang.org/x/tools/go/buildutil"
	"golang.org/x/tools/go/callgraph"
	"golang.org/x/tools/go/callgraph/cha"
	"golang.org/x/tools/go/loader"
	"golang.org/x/tools/go/ssa"
	"golang.org/x/tools/go/ssa/ssautil"
)

var inputs = []string{
	"testdata/func.go",
	"testdata/iface.go",
	"testdata/recv.go",
	"testdata/issue23925.go",
}

func expectation(f *ast.File) (string, token.Pos) {
	for _, c := range f.Comments {
		text := strings.TrimSpace(c.Text())
		if t := strings.TrimPrefix(text, "WANT:\n"); t != text {
			return t, c.Pos()
		}
	}
	return "", token.NoPos
}

// TestCHA runs CHA on each file in inputs, prints the dynamic edges of
// the call graph, and compares it with the golden results embedded in
// the WANT comment at the end of the file.
func TestCHA(t *testing.T) {
	for _, filename := range inputs {
		prog, f, mainPkg, err := loadProgInfo(filename, ssa.InstantiateGenerics)
		if err != nil {
			t.Error(err)
			continue
		}

		want, pos := expectation(f)
		if pos == token.NoPos {
			t.Error(fmt.Errorf("No WANT: comment in %s", filename))
			continue
		}

		cg := cha.CallGraph(prog)

		if got := printGraph(cg, mainPkg.Pkg, "dynamic", "Dynamic calls"); got != want {
			t.Errorf("%s: got:\n%s\nwant:\n%s",
				prog.Fset.Position(pos), got, want)
		}
	}
}

// TestCHAGenerics is TestCHA tailored for testing generics,
func TestCHAGenerics(t *testing.T) {
	filename := "testdata/generics.go"
	prog, f, mainPkg, err := loadProgInfo(filename, ssa.InstantiateGenerics)
	if err != nil {
		t.Fatal(err)
	}

	want, pos := expectation(f)
	if pos == token.NoPos {
		t.Fatal(fmt.Errorf("No WANT: comment in %s", filename))
	}

	cg := cha.CallGraph(prog)

	if got := printGraph(cg, mainPkg.Pkg, "", "All calls"); got != want {
		t.Errorf("%s: got:\n%s\nwant:\n%s",
			prog.Fset.Position(pos), got, want)
	}
}

// TestCHAUnexported tests call resolution for unexported methods.
func TestCHAUnexported(t *testing.T) {
	// The two packages below each have types with methods called "m".
	// Each of these methods should only be callable by functions in their
	// own package, because they are unexported.
	//
	// In particular:
	// - main.main can call    (main.S1).m
	// - p2.Foo    can call    (p2.S2).m
	// - main.main cannot call (p2.S2).m
	// - p2.Foo    cannot call (main.S1).m
	//
	// We use CHA to build a callgraph, then check that it has the
	// appropriate set of edges.

	main := `package main
		import "p2"
		type I1 interface { m() }
		type S1 struct { p2.I2 }
		func (s S1) m() { }
		func main() {
			var s S1
			var o I1 = s
			o.m()
			p2.Foo(s)
		}`

	p2 := `package p2
		type I2 interface { m() }
		type S2 struct { }
		func (s S2) m() { }
		func Foo(i I2) { i.m() }`

	want := `All calls
  main.init --> p2.init
  main.main --> (main.S1).m
  main.main --> p2.Foo
  p2.Foo --> (p2.S2).m`

	conf := loader.Config{
		Build: fakeContext(map[string]string{"main": main, "p2": p2}),
	}
	conf.Import("main")
	iprog, err := conf.Load()
	if err != nil {
		t.Fatalf("Load failed: %v", err)
	}
	prog := ssautil.CreateProgram(iprog, ssa.InstantiateGenerics)
	prog.Build()

	cg := cha.CallGraph(prog)

	// The graph is easier to read without synthetic nodes.
	cg.DeleteSyntheticNodes()

	if got := printGraph(cg, nil, "", "All calls"); got != want {
		t.Errorf("cha.CallGraph: got:\n%s\nwant:\n%s", got, want)
	}
}

// Simplifying wrapper around buildutil.FakeContext for single-file packages.
func fakeContext(pkgs map[string]string) *build.Context {
	pkgs2 := make(map[string]map[string]string)
	for path, content := range pkgs {
		pkgs2[path] = map[string]string{"x.go": content}
	}
	return buildutil.FakeContext(pkgs2)
}

func loadProgInfo(filename string, mode ssa.BuilderMode) (*ssa.Program, *ast.File, *ssa.Package, error) {
	content, err := os.ReadFile(filename)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("couldn't read file '%s': %s", filename, err)
	}

	conf := loader.Config{
		ParserMode: parser.ParseComments,
	}
	f, err := conf.ParseFile(filename, content)
	if err != nil {
		return nil, nil, nil, err
	}

	conf.CreateFromFiles("main", f)
	iprog, err := conf.Load()
	if err != nil {
		return nil, nil, nil, err
	}

	prog := ssautil.CreateProgram(iprog, mode)
	prog.Build()

	return prog, f, prog.Package(iprog.Created[0].Pkg), nil
}

// printGraph returns a string representation of cg involving only edges
// whose description contains edgeMatch. The string representation is
// prefixed with a desc line.
func printGraph(cg *callgraph.Graph, from *types.Package, edgeMatch string, desc string) string {
	var edges []string
	callgraph.GraphVisitEdges(cg, func(e *callgraph.Edge) error {
		if strings.Contains(e.Description(), edgeMatch) {
			edges = append(edges, fmt.Sprintf("%s --> %s",
				e.Caller.Func.RelString(from),
				e.Callee.Func.RelString(from)))
		}
		return nil
	})
	sort.Strings(edges)

	var buf bytes.Buffer
	buf.WriteString(desc + "\n")
	for _, edge := range edges {
		fmt.Fprintf(&buf, "  %s\n", edge)
	}
	return strings.TrimSpace(buf.String())
}
