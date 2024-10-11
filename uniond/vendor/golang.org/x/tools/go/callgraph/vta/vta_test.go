// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:debug gotypesalias=1

package vta

import (
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/analysistest"
	"golang.org/x/tools/go/analysis/passes/buildssa"
	"golang.org/x/tools/go/callgraph/cha"
	"golang.org/x/tools/go/ssa"
	"golang.org/x/tools/go/ssa/ssautil"
	"golang.org/x/tools/internal/testenv"
)

func TestVTACallGraph(t *testing.T) {
	errDiff := func(want, got, missing []string) {
		t.Errorf("got:\n%s\n\nwant:\n%s\n\nmissing:\n%s\n\ndiff:\n%s",
			strings.Join(got, "\n"),
			strings.Join(want, "\n"),
			strings.Join(missing, "\n"),
			cmp.Diff(got, want)) // to aid debugging
	}

	files := []string{
		"testdata/src/callgraph_static.go",
		"testdata/src/callgraph_ho.go",
		"testdata/src/callgraph_interfaces.go",
		"testdata/src/callgraph_pointers.go",
		"testdata/src/callgraph_collections.go",
		"testdata/src/callgraph_fields.go",
		"testdata/src/callgraph_field_funcs.go",
		"testdata/src/callgraph_recursive_types.go",
		"testdata/src/callgraph_issue_57756.go",
		"testdata/src/callgraph_comma_maps.go",
		"testdata/src/callgraph_type_aliases.go", // https://github.com/golang/go/issues/68799
	}
	if testenv.Go1Point() >= 23 {
		files = append(files, "testdata/src/callgraph_range_over_func.go")
	}

	for _, file := range files {
		t.Run(file, func(t *testing.T) {
			prog, want, err := testProg(file, ssa.BuilderMode(0))
			if err != nil {
				t.Fatalf("couldn't load test file '%s': %s", file, err)
			}
			if len(want) == 0 {
				t.Fatalf("couldn't find want in `%s`", file)
			}

			// First test VTA with lazy-CHA initial call graph.
			g := CallGraph(ssautil.AllFunctions(prog), nil)
			got := callGraphStr(g)
			if missing := setdiff(want, got); len(missing) > 0 {
				errDiff(want, got, missing)
			}

			// Repeat the test with explicit CHA initial call graph.
			g = CallGraph(ssautil.AllFunctions(prog), cha.CallGraph(prog))
			got = callGraphStr(g)
			if missing := setdiff(want, got); len(missing) > 0 {
				errDiff(want, got, missing)
			}
		})
	}
}

// TestVTAProgVsFuncSet exemplifies and tests different possibilities
// enabled by having an arbitrary function set as input to CallGraph
// instead of the whole program (i.e., ssautil.AllFunctions(prog)).
func TestVTAProgVsFuncSet(t *testing.T) {
	prog, want, err := testProg("testdata/src/callgraph_nested_ptr.go", ssa.BuilderMode(0))
	if err != nil {
		t.Fatalf("couldn't load test `testdata/src/callgraph_nested_ptr.go`: %s", err)
	}
	if len(want) == 0 {
		t.Fatal("couldn't find want in `testdata/src/callgraph_nested_ptr.go`")
	}

	allFuncs := ssautil.AllFunctions(prog)
	g := CallGraph(allFuncs, cha.CallGraph(prog))
	// VTA over the whole program will produce a call graph that
	// includes Baz:(**i).Foo -> A.Foo, B.Foo.
	got := callGraphStr(g)
	if diff := setdiff(want, got); len(diff) > 0 {
		t.Errorf("computed callgraph %v should contain %v (diff: %v)", got, want, diff)
	}

	// Prune the set of program functions to exclude Bar(). This should
	// yield a call graph that includes different set of callees for Baz
	// Baz:(**i).Foo -> A.Foo
	//
	// Note that the exclusion of Bar can happen, for instance, if Baz is
	// considered an entry point of some data flow analysis and Bar is
	// provably (e.g., using CHA forward reachability) unreachable from Baz.
	noBarFuncs := make(map[*ssa.Function]bool)
	for f, in := range allFuncs {
		noBarFuncs[f] = in && (funcName(f) != "Bar")
	}
	want = []string{"Baz: Do(i) -> Do; invoke t2.Foo() -> A.Foo"}
	g = CallGraph(noBarFuncs, cha.CallGraph(prog))
	got = callGraphStr(g)
	if diff := setdiff(want, got); len(diff) > 0 {
		t.Errorf("pruned callgraph %v should contain %v (diff: %v)", got, want, diff)
	}
}

// TestVTAPanicMissingDefinitions tests if VTA gracefully handles the case
// where VTA panics when a definition of a function or method is not
// available, which can happen when using analysis package. A successful
// test simply does not panic.
func TestVTAPanicMissingDefinitions(t *testing.T) {
	run := func(pass *analysis.Pass) (interface{}, error) {
		s := pass.ResultOf[buildssa.Analyzer].(*buildssa.SSA)
		CallGraph(ssautil.AllFunctions(s.Pkg.Prog), cha.CallGraph(s.Pkg.Prog))
		return nil, nil
	}

	analyzer := &analysis.Analyzer{
		Name: "test",
		Doc:  "test",
		Run:  run,
		Requires: []*analysis.Analyzer{
			buildssa.Analyzer,
		},
	}

	testdata := analysistest.TestData()
	res := analysistest.Run(t, testdata, analyzer, "t", "d")
	if len(res) != 2 {
		t.Errorf("want analysis results for 2 packages; got %v", len(res))
	}
	for _, r := range res {
		if r.Err != nil {
			t.Errorf("want no error for package %v; got %v", r.Pass.Pkg.Path(), r.Err)
		}
	}
}

func TestVTACallGraphGenerics(t *testing.T) {
	// TODO(zpavlinovic): add more tests
	files := []string{
		"testdata/src/arrays_generics.go",
		"testdata/src/callgraph_generics.go",
		"testdata/src/issue63146.go",
	}
	for _, file := range files {
		t.Run(file, func(t *testing.T) {
			prog, want, err := testProg(file, ssa.InstantiateGenerics)
			if err != nil {
				t.Fatalf("couldn't load test file '%s': %s", file, err)
			}
			if len(want) == 0 {
				t.Fatalf("couldn't find want in `%s`", file)
			}

			g := CallGraph(ssautil.AllFunctions(prog), cha.CallGraph(prog))
			got := callGraphStr(g)
			if diff := setdiff(want, got); len(diff) != 0 {
				t.Errorf("computed callgraph %v should contain %v (diff: %v)", got, want, diff)
				logFns(t, prog)
			}
		})
	}
}

func TestVTACallGraphGo117(t *testing.T) {
	file := "testdata/src/go117.go"
	prog, want, err := testProg(file, ssa.BuilderMode(0))
	if err != nil {
		t.Fatalf("couldn't load test file '%s': %s", file, err)
	}
	if len(want) == 0 {
		t.Fatalf("couldn't find want in `%s`", file)
	}

	g, _ := typePropGraph(ssautil.AllFunctions(prog), makeCalleesFunc(nil, cha.CallGraph(prog)))
	got := vtaGraphStr(g)
	if diff := setdiff(want, got); len(diff) != 0 {
		t.Errorf("`%s`: want superset of %v;\n got %v", file, want, got)
	}
}
