// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package modfile

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// TODO(#45713): Update these tests once AddUse sets the module path.
var workAddUseTests = []struct {
	desc       string
	in         string
	path       string
	modulePath string
	out        string
}{
	{
		`empty`,
		``,
		`foo`, `bar`,
		`use foo`,
	},
	{
		`go_stmt_only`,
		`go 1.17
		`,
		`foo`, `bar`,
		`go 1.17
		use foo
		`,
	},
	{
		`use_line_present`,
		`go 1.17
		use baz`,
		`foo`, `bar`,
		`go 1.17
		use (
			baz
		  foo
		)
		`,
	},
	{
		`use_block_present`,
		`go 1.17
		use (
			baz
			quux
		)
		`,
		`foo`, `bar`,
		`go 1.17
		use (
			baz
		  quux
			foo
		)
		`,
	},
	{
		`use_and_replace_present`,
		`go 1.17
		use baz
		replace a => ./b
		`,
		`foo`, `bar`,
		`go 1.17
		use (
			baz
			foo
		)
		replace a => ./b
		`,
	},
}

var workDropUseTests = []struct {
	desc string
	in   string
	path string
	out  string
}{
	{
		`empty`,
		``,
		`foo`,
		``,
	},
	{
		`go_stmt_only`,
		`go 1.17
		`,
		`foo`,
		`go 1.17
		`,
	},
	{
		`single_use`,
		`go 1.17
		use foo`,
		`foo`,
		`go 1.17
		`,
	},
	{
		`use_block`,
		`go 1.17
		use (
			foo
			bar
			baz
		)`,
		`bar`,
		`go 1.17
		use (
			foo
			baz
		)`,
	},
	{
		`use_multi`,
		`go 1.17
		use (
			foo
			bar
			baz
		)
		use foo
		use quux
		use foo`,
		`foo`,
		`go 1.17
		use (
			bar
			baz
		)
		use quux`,
	},
}

var workAddGoTests = []struct {
	desc    string
	in      string
	version string
	out     string
}{
	{
		`empty`,
		``,
		`1.17`,
		`go 1.17
		`,
	},
	{
		`comment`,
		`// this is a comment`,
		`1.17`,
		`// this is a comment

		go 1.17`,
	},
	{
		`use_after_replace`,
		`
		replace example.com/foo => ../bar
		use foo
		`,
		`1.17`,
		`
		go 1.17
		replace example.com/foo => ../bar
		use foo
		`,
	},
	{
		`use_before_replace`,
		`use foo
		replace example.com/foo => ../bar
		`,
		`1.17`,
		`
		go 1.17
		use foo
		replace example.com/foo => ../bar
		`,
	},
	{
		`use_only`,
		`use foo
		`,
		`1.17`,
		`
		go 1.17
		use foo
		`,
	},
	{
		`already_have_go`,
		`go 1.17
		`,
		`1.18`,
		`
		go 1.18
		`,
	},
}

var workAddToolchainTests = []struct {
	desc    string
	in      string
	version string
	out     string
}{
	{
		`empty`,
		``,
		`go1.17`,
		`toolchain go1.17
		`,
	},
	{
		`aftergo`,
		`// this is a comment
		use foo

		go 1.17

		use bar
		`,
		`go1.17`,
		`// this is a comment
		use foo

		go 1.17

		toolchain go1.17

		use bar
		`,
	},
	{
		`already_have_toolchain`,
		`go 1.17

		toolchain go1.18
		`,
		`go1.19`,
		`go 1.17

		toolchain go1.19
		`,
	},
}

var workSortBlocksTests = []struct {
	desc, in, out string
}{
	{
		`use_duplicates_not_removed`,
		`go 1.17
		use foo
		use bar
		use (
			foo
		)`,
		`go 1.17
		use foo
		use bar
		use (
			foo
		)`,
	},
	{
		`replace_duplicates_removed`,
		`go 1.17
		use foo
		replace x.y/z v1.0.0 => ./a
		replace x.y/z v1.1.0 => ./b
		replace (
			x.y/z v1.0.0 => ./c
		)
		`,
		`go 1.17
		use foo
		replace x.y/z v1.1.0 => ./b
		replace (
			x.y/z v1.0.0 => ./c
		)
		`,
	},
}

func TestAddUse(t *testing.T) {
	for _, tt := range workAddUseTests {
		t.Run(tt.desc, func(t *testing.T) {
			testWorkEdit(t, tt.in, tt.out, func(f *WorkFile) error {
				return f.AddUse(tt.path, tt.modulePath)
			})
		})
	}
}

func TestDropUse(t *testing.T) {
	for _, tt := range workDropUseTests {
		t.Run(tt.desc, func(t *testing.T) {
			testWorkEdit(t, tt.in, tt.out, func(f *WorkFile) error {
				if err := f.DropUse(tt.path); err != nil {
					return err
				}
				f.Cleanup()
				return nil
			})
		})
	}
}

func TestWorkAddGo(t *testing.T) {
	for _, tt := range workAddGoTests {
		t.Run(tt.desc, func(t *testing.T) {
			testWorkEdit(t, tt.in, tt.out, func(f *WorkFile) error {
				return f.AddGoStmt(tt.version)
			})
		})
	}
}

func TestWorkAddToolchain(t *testing.T) {
	for _, tt := range workAddToolchainTests {
		t.Run(tt.desc, func(t *testing.T) {
			testWorkEdit(t, tt.in, tt.out, func(f *WorkFile) error {
				return f.AddToolchainStmt(tt.version)
			})
		})
	}
}

func TestWorkSortBlocks(t *testing.T) {
	for _, tt := range workSortBlocksTests {
		t.Run(tt.desc, func(t *testing.T) {
			testWorkEdit(t, tt.in, tt.out, func(f *WorkFile) error {
				f.SortBlocks()
				return nil
			})
		})
	}
}

func TestWorkAddGodebug(t *testing.T) {
	for _, tt := range addGodebugTests {
		t.Run(tt.desc, func(t *testing.T) {
			in := strings.ReplaceAll(tt.in, "module m", "use foo")
			out := strings.ReplaceAll(tt.out, "module m", "use foo")
			testWorkEdit(t, in, out, func(f *WorkFile) error {
				err := f.AddGodebug(tt.key, tt.value)
				f.Cleanup()
				return err
			})
		})
	}
}

func TestWorkDropGodebug(t *testing.T) {
	for _, tt := range dropGodebugTests {
		t.Run(tt.desc, func(t *testing.T) {
			in := strings.ReplaceAll(tt.in, "module m", "use foo")
			out := strings.ReplaceAll(tt.out, "module m", "use foo")
			testWorkEdit(t, in, out, func(f *WorkFile) error {
				f.DropGodebug(tt.key)
				f.Cleanup()
				return nil
			})
		})
	}
}

// Test that when files in the testdata directory are parsed
// and printed and parsed again, we get the same parse tree
// both times.
func TestWorkPrintParse(t *testing.T) {
	outs, err := filepath.Glob("testdata/work/*")
	if err != nil {
		t.Fatal(err)
	}
	for _, out := range outs {
		out := out
		name := filepath.Base(out)
		t.Run(name, func(t *testing.T) {
			t.Parallel()
			data, err := os.ReadFile(out)
			if err != nil {
				t.Fatal(err)
			}

			base := "testdata/work/" + filepath.Base(out)
			f, err := parse(base, data)
			if err != nil {
				t.Fatalf("parsing original: %v", err)
			}

			ndata := Format(f)
			f2, err := parse(base, ndata)
			if err != nil {
				t.Fatalf("parsing reformatted: %v", err)
			}

			eq := eqchecker{file: base}
			if err := eq.check(f, f2); err != nil {
				t.Errorf("not equal (parse/Format/parse): %v", err)
			}

			pf1, err := ParseWork(base, data, nil)
			if err != nil {
				t.Errorf("should parse %v: %v", base, err)
			}
			if err == nil {
				pf2, err := ParseWork(base, ndata, nil)
				if err != nil {
					t.Fatalf("Parsing reformatted: %v", err)
				}
				eq := eqchecker{file: base}
				if err := eq.check(pf1, pf2); err != nil {
					t.Errorf("not equal (parse/Format/Parse): %v", err)
				}

				ndata2 := Format(pf1.Syntax)
				pf3, err := ParseWork(base, ndata2, nil)
				if err != nil {
					t.Fatalf("Parsing reformatted2: %v", err)
				}
				eq = eqchecker{file: base}
				if err := eq.check(pf1, pf3); err != nil {
					t.Errorf("not equal (Parse/Format/Parse): %v", err)
				}
				ndata = ndata2
			}

			if strings.HasSuffix(out, ".in") {
				golden, err := os.ReadFile(strings.TrimSuffix(out, ".in") + ".golden")
				if err != nil {
					t.Fatal(err)
				}
				if !bytes.Equal(ndata, golden) {
					t.Errorf("formatted %s incorrectly: diff shows -golden, +ours", base)
					tdiff(t, string(golden), string(ndata))
					return
				}
			}
		})
	}
}

func testWorkEdit(t *testing.T, in, want string, transform func(f *WorkFile) error) *WorkFile {
	t.Helper()
	parse := ParseWork
	f, err := parse("in", []byte(in), nil)
	if err != nil {
		t.Fatal(err)
	}
	g, err := parse("out", []byte(want), nil)
	if err != nil {
		t.Fatal(err)
	}
	golden := Format(g.Syntax)

	if err := transform(f); err != nil {
		t.Fatal(err)
	}
	out := Format(f.Syntax)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(out, golden) {
		t.Errorf("have:\n%s\nwant:\n%s", out, golden)
	}

	return f
}
