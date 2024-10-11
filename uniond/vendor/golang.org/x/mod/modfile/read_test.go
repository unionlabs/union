// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package modfile

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
)

// exists reports whether the named file exists.
func exists(name string) bool {
	_, err := os.Stat(name)
	return err == nil
}

// Test that reading and then writing the golden files
// does not change their output.
func TestPrintGolden(t *testing.T) {
	outs, err := filepath.Glob("testdata/*.golden")
	if err != nil {
		t.Fatal(err)
	}
	for _, out := range outs {
		out := out
		name := strings.TrimSuffix(filepath.Base(out), ".golden")
		t.Run(name, func(t *testing.T) {
			t.Parallel()
			testPrint(t, out, out)
		})
	}
}

// testPrint is a helper for testing the printer.
// It reads the file named in, reformats it, and compares
// the result to the file named out.
func testPrint(t *testing.T, in, out string) {
	data, err := os.ReadFile(in)
	if err != nil {
		t.Error(err)
		return
	}

	golden, err := os.ReadFile(out)
	if err != nil {
		t.Error(err)
		return
	}

	base := "testdata/" + filepath.Base(in)
	f, err := parse(in, data)
	if err != nil {
		t.Error(err)
		return
	}

	ndata := Format(f)

	if !bytes.Equal(ndata, golden) {
		t.Errorf("formatted %s incorrectly: diff shows -golden, +ours", base)
		tdiff(t, string(golden), string(ndata))
		return
	}
}

// TestParsePunctuation verifies that certain ASCII punctuation characters
// (brackets, commas) are lexed as separate tokens, even when they're
// surrounded by identifier characters.
func TestParsePunctuation(t *testing.T) {
	for _, test := range []struct {
		desc, src, want string
	}{
		{"paren", "require ()", "require ( )"},
		{"brackets", "require []{},", "require [ ] { } ,"},
		{"mix", "require a[b]c{d}e,", "require a [ b ] c { d } e ,"},
		{"block_mix", "require (\n\ta[b]\n)", "require ( a [ b ] )"},
		{"interval", "require [v1.0.0, v1.1.0)", "require [ v1.0.0 , v1.1.0 )"},
	} {
		t.Run(test.desc, func(t *testing.T) {
			f, err := parse("go.mod", []byte(test.src))
			if err != nil {
				t.Fatalf("parsing %q: %v", test.src, err)
			}
			var tokens []string
			for _, stmt := range f.Stmt {
				switch stmt := stmt.(type) {
				case *Line:
					tokens = append(tokens, stmt.Token...)
				case *LineBlock:
					tokens = append(tokens, stmt.Token...)
					tokens = append(tokens, "(")
					for _, line := range stmt.Line {
						tokens = append(tokens, line.Token...)
					}
					tokens = append(tokens, ")")
				default:
					t.Fatalf("parsing %q: unexpected statement of type %T", test.src, stmt)
				}
			}
			got := strings.Join(tokens, " ")
			if got != test.want {
				t.Errorf("parsing %q: got %q, want %q", test.src, got, test.want)
			}
		})
	}
}

func TestParseLax(t *testing.T) {
	badFile := []byte(`module m
		surprise attack
		x y (
			z
		)
		exclude v1.2.3
		replace <-!!!
		retract v1.2.3 v1.2.4
		retract (v1.2.3, v1.2.4]
		retract v1.2.3 (
			key1 value1
			key2 value2
		)
		require good v1.0.0
	`)
	f, err := ParseLax("file", badFile, nil)
	if err != nil {
		t.Fatalf("ParseLax did not ignore irrelevant errors: %v", err)
	}
	if f.Module == nil || f.Module.Mod.Path != "m" {
		t.Errorf("module directive was not parsed")
	}
	if len(f.Require) != 1 || f.Require[0].Mod.Path != "good" {
		t.Errorf("require directive at end of file was not parsed")
	}
}

// Test that when files in the testdata directory are parsed
// and printed and parsed again, we get the same parse tree
// both times.
func TestPrintParse(t *testing.T) {
	outs, err := filepath.Glob("testdata/*")
	if err != nil {
		t.Fatal(err)
	}
	for _, out := range outs {
		out := out
		name := filepath.Base(out)
		if !strings.HasSuffix(out, ".in") && !strings.HasSuffix(out, ".golden") {
			continue
		}
		t.Run(name, func(t *testing.T) {
			t.Parallel()
			data, err := os.ReadFile(out)
			if err != nil {
				t.Fatal(err)
			}

			base := "testdata/" + filepath.Base(out)
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

			pf1, err := Parse(base, data, nil)
			if err != nil {
				switch base {
				case "testdata/block.golden",
					"testdata/block.in",
					"testdata/comment.golden",
					"testdata/comment.in",
					"testdata/rule1.golden":
					// ignore
				default:
					t.Errorf("should parse %v: %v", base, err)
				}
			}
			if err == nil {
				pf2, err := Parse(base, ndata, nil)
				if err != nil {
					t.Fatalf("Parsing reformatted: %v", err)
				}
				eq := eqchecker{file: base}
				if err := eq.check(pf1, pf2); err != nil {
					t.Errorf("not equal (parse/Format/Parse): %v", err)
				}

				ndata2, err := pf1.Format()
				if err != nil {
					t.Errorf("reformat: %v", err)
				}
				pf3, err := Parse(base, ndata2, nil)
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

// An eqchecker holds state for checking the equality of two parse trees.
type eqchecker struct {
	file string
	pos  Position
}

// errorf returns an error described by the printf-style format and arguments,
// inserting the current file position before the error text.
func (eq *eqchecker) errorf(format string, args ...interface{}) error {
	return fmt.Errorf("%s:%d: %s", eq.file, eq.pos.Line,
		fmt.Sprintf(format, args...))
}

// check checks that v and w represent the same parse tree.
// If not, it returns an error describing the first difference.
func (eq *eqchecker) check(v, w interface{}) error {
	return eq.checkValue(reflect.ValueOf(v), reflect.ValueOf(w))
}

var (
	posType      = reflect.TypeOf(Position{})
	commentsType = reflect.TypeOf(Comments{})
)

// checkValue checks that v and w represent the same parse tree.
// If not, it returns an error describing the first difference.
func (eq *eqchecker) checkValue(v, w reflect.Value) error {
	// inner returns the innermost expression for v.
	// if v is a non-nil interface value, it returns the concrete
	// value in the interface.
	inner := func(v reflect.Value) reflect.Value {
		for {
			if v.Kind() == reflect.Interface && !v.IsNil() {
				v = v.Elem()
				continue
			}
			break
		}
		return v
	}

	v = inner(v)
	w = inner(w)
	if v.Kind() == reflect.Invalid && w.Kind() == reflect.Invalid {
		return nil
	}
	if v.Kind() == reflect.Invalid {
		return eq.errorf("nil interface became %s", w.Type())
	}
	if w.Kind() == reflect.Invalid {
		return eq.errorf("%s became nil interface", v.Type())
	}

	if v.Type() != w.Type() {
		return eq.errorf("%s became %s", v.Type(), w.Type())
	}

	if p, ok := v.Interface().(Expr); ok {
		eq.pos, _ = p.Span()
	}

	switch v.Kind() {
	default:
		return eq.errorf("unexpected type %s", v.Type())

	case reflect.Bool, reflect.Int, reflect.String:
		vi := v.Interface()
		wi := w.Interface()
		if vi != wi {
			return eq.errorf("%v became %v", vi, wi)
		}

	case reflect.Slice:
		vl := v.Len()
		wl := w.Len()
		for i := 0; i < vl || i < wl; i++ {
			if i >= vl {
				return eq.errorf("unexpected %s", w.Index(i).Type())
			}
			if i >= wl {
				return eq.errorf("missing %s", v.Index(i).Type())
			}
			if err := eq.checkValue(v.Index(i), w.Index(i)); err != nil {
				return err
			}
		}

	case reflect.Struct:
		// Fields in struct must match.
		t := v.Type()
		n := t.NumField()
		for i := 0; i < n; i++ {
			tf := t.Field(i)
			switch {
			default:
				if err := eq.checkValue(v.Field(i), w.Field(i)); err != nil {
					return err
				}

			case tf.Type == posType: // ignore positions
			case tf.Type == commentsType: // ignore comment assignment
			}
		}

	case reflect.Ptr, reflect.Interface:
		if v.IsNil() != w.IsNil() {
			if v.IsNil() {
				return eq.errorf("unexpected %s", w.Elem().Type())
			}
			return eq.errorf("missing %s", v.Elem().Type())
		}
		if err := eq.checkValue(v.Elem(), w.Elem()); err != nil {
			return err
		}
	}
	return nil
}

// diff returns the output of running diff on b1 and b2.
func diff(b1, b2 []byte) (data []byte, err error) {
	f1, err := os.CreateTemp("", "testdiff")
	if err != nil {
		return nil, err
	}
	defer os.Remove(f1.Name())
	defer f1.Close()

	f2, err := os.CreateTemp("", "testdiff")
	if err != nil {
		return nil, err
	}
	defer os.Remove(f2.Name())
	defer f2.Close()

	f1.Write(b1)
	f2.Write(b2)

	data, err = exec.Command("diff", "-u", f1.Name(), f2.Name()).CombinedOutput()
	if len(data) > 0 {
		// diff exits with a non-zero status when the files don't match.
		// Ignore that failure as long as we get output.
		err = nil
	}
	return
}

// tdiff logs the diff output to t.Error.
func tdiff(t *testing.T, a, b string) {
	data, err := diff([]byte(a), []byte(b))
	if err != nil {
		t.Error(err)
		return
	}
	t.Error(string(data))
}

var modulePathTests = []struct {
	input    []byte
	expected string
}{
	{input: []byte("module \"github.com/rsc/vgotest\""), expected: "github.com/rsc/vgotest"},
	{input: []byte("module github.com/rsc/vgotest"), expected: "github.com/rsc/vgotest"},
	{input: []byte("module  \"github.com/rsc/vgotest\""), expected: "github.com/rsc/vgotest"},
	{input: []byte("module  github.com/rsc/vgotest"), expected: "github.com/rsc/vgotest"},
	{input: []byte("module `github.com/rsc/vgotest`"), expected: "github.com/rsc/vgotest"},
	{input: []byte("module \"github.com/rsc/vgotest/v2\""), expected: "github.com/rsc/vgotest/v2"},
	{input: []byte("module github.com/rsc/vgotest/v2"), expected: "github.com/rsc/vgotest/v2"},
	{input: []byte("module \"gopkg.in/yaml.v2\""), expected: "gopkg.in/yaml.v2"},
	{input: []byte("module gopkg.in/yaml.v2"), expected: "gopkg.in/yaml.v2"},
	{input: []byte("module \"gopkg.in/check.v1\"\n"), expected: "gopkg.in/check.v1"},
	{input: []byte("module \"gopkg.in/check.v1\n\""), expected: ""},
	{input: []byte("module gopkg.in/check.v1\n"), expected: "gopkg.in/check.v1"},
	{input: []byte("module \"gopkg.in/check.v1\"\r\n"), expected: "gopkg.in/check.v1"},
	{input: []byte("module gopkg.in/check.v1\r\n"), expected: "gopkg.in/check.v1"},
	{input: []byte("module \"gopkg.in/check.v1\"\n\n"), expected: "gopkg.in/check.v1"},
	{input: []byte("module gopkg.in/check.v1\n\n"), expected: "gopkg.in/check.v1"},
	{input: []byte("module \n\"gopkg.in/check.v1\"\n\n"), expected: ""},
	{input: []byte("module \ngopkg.in/check.v1\n\n"), expected: ""},
	{input: []byte("module \"gopkg.in/check.v1\"asd"), expected: ""},
	{input: []byte("module \n\"gopkg.in/check.v1\"\n\n"), expected: ""},
	{input: []byte("module \ngopkg.in/check.v1\n\n"), expected: ""},
	{input: []byte("module \"gopkg.in/check.v1\"asd"), expected: ""},
	{input: []byte("module  \nmodule a/b/c "), expected: "a/b/c"},
	{input: []byte("module \"   \""), expected: "   "},
	{input: []byte("module   "), expected: ""},
	{input: []byte("module \"  a/b/c  \""), expected: "  a/b/c  "},
	{input: []byte("module \"github.com/rsc/vgotest1\" // with a comment"), expected: "github.com/rsc/vgotest1"},
}

func TestModulePath(t *testing.T) {
	for _, test := range modulePathTests {
		t.Run(string(test.input), func(t *testing.T) {
			result := ModulePath(test.input)
			if result != test.expected {
				t.Fatalf("ModulePath(%q): %s, want %s", string(test.input), result, test.expected)
			}
		})
	}
}

func TestParseVersions(t *testing.T) {
	tests := []struct {
		desc, input string
		ok          bool
		laxOK       bool // ok=true implies laxOK=true; only set if ok=false
	}{
		// go lines
		{desc: "empty", input: "module m\ngo \n", ok: false},
		{desc: "one", input: "module m\ngo 1\n", ok: false},
		{desc: "two", input: "module m\ngo 1.22\n", ok: true},
		{desc: "three", input: "module m\ngo 1.22.333", ok: true},
		{desc: "before", input: "module m\ngo v1.2\n", ok: false},
		{desc: "after", input: "module m\ngo 1.2rc1\n", ok: true},
		{desc: "space", input: "module m\ngo 1.2 3.4\n", ok: false},
		{desc: "alt1", input: "module m\ngo 1.2.3\n", ok: true},
		{desc: "alt2", input: "module m\ngo 1.2rc1\n", ok: true},
		{desc: "alt3", input: "module m\ngo 1.2beta1\n", ok: true},
		{desc: "alt4", input: "module m\ngo 1.2.beta1\n", ok: false, laxOK: true},
		{desc: "alt1", input: "module m\ngo v1.2.3\n", ok: false, laxOK: true},
		{desc: "alt2", input: "module m\ngo v1.2rc1\n", ok: false, laxOK: true},
		{desc: "alt3", input: "module m\ngo v1.2beta1\n", ok: false, laxOK: true},
		{desc: "alt4", input: "module m\ngo v1.2.beta1\n", ok: false, laxOK: true},
		{desc: "alt1", input: "module m\ngo v1.2\n", ok: false, laxOK: true},

		// toolchain lines
		{desc: "tool", input: "module m\ntoolchain go1.2\n", ok: true},
		{desc: "tool1", input: "module m\ntoolchain go1.2.3\n", ok: true},
		{desc: "tool2", input: "module m\ntoolchain go1.2rc1\n", ok: true},
		{desc: "tool3", input: "module m\ntoolchain go1.2rc1-gccgo\n", ok: true},
		{desc: "tool4", input: "module m\ntoolchain default\n", ok: true},
		{desc: "tool5", input: "module m\ntoolchain inconceivable!\n", ok: false, laxOK: true},
	}
	t.Run("Strict", func(t *testing.T) {
		for _, test := range tests {
			t.Run(test.desc, func(t *testing.T) {
				if _, err := Parse("go.mod", []byte(test.input), nil); err == nil && !test.ok {
					t.Error("unexpected success")
				} else if err != nil && test.ok {
					t.Errorf("unexpected error: %v", err)
				}
			})
		}
	})
	t.Run("Lax", func(t *testing.T) {
		for _, test := range tests {
			t.Run(test.desc, func(t *testing.T) {
				if _, err := ParseLax("go.mod", []byte(test.input), nil); err == nil && !(test.ok || test.laxOK) {
					t.Error("unexpected success")
				} else if err != nil && test.ok {
					t.Errorf("unexpected error: %v", err)
				}
			})
		}
	})
}

func TestComments(t *testing.T) {
	for _, test := range []struct {
		desc, input, want string
	}{
		{
			desc: "comment_only",
			input: `
// a
// b
`,
			want: `
comments before "// a"
comments before "// b"
`,
		}, {
			desc: "line",
			input: `
// a

// b
module m // c
// d

// e
`,
			want: `
comments before "// a"
line before "// b"
line suffix "// c"
comments before "// d"
comments before "// e"
`,
		}, {
			desc: "block",
			input: `
// a

// b
block ( // c
	// d

	// e
	x // f
	// g

	// h
) // i
// j

// k
`,
			want: `
comments before "// a"
block before "// b"
lparen suffix "// c"
blockline before "// d"
blockline before ""
blockline before "// e"
blockline suffix "// f"
rparen before "// g"
rparen before ""
rparen before "// h"
rparen suffix "// i"
comments before "// j"
comments before "// k"
`,
		}, {
			desc:  "cr_removed",
			input: "// a\r\r\n",
			want:  `comments before "// a\r"`,
		},
	} {
		t.Run(test.desc, func(t *testing.T) {
			f, err := ParseLax("go.mod", []byte(test.input), nil)
			if err != nil {
				t.Fatal(err)
			}

			buf := &bytes.Buffer{}
			printComments := func(prefix string, cs *Comments) {
				for _, c := range cs.Before {
					fmt.Fprintf(buf, "%s before %q\n", prefix, c.Token)
				}
				for _, c := range cs.Suffix {
					fmt.Fprintf(buf, "%s suffix %q\n", prefix, c.Token)
				}
				for _, c := range cs.After {
					fmt.Fprintf(buf, "%s after %q\n", prefix, c.Token)
				}
			}

			printComments("file", &f.Syntax.Comments)
			for _, stmt := range f.Syntax.Stmt {
				switch stmt := stmt.(type) {
				case *CommentBlock:
					printComments("comments", stmt.Comment())
				case *Line:
					printComments("line", stmt.Comment())
				case *LineBlock:
					printComments("block", stmt.Comment())
					printComments("lparen", stmt.LParen.Comment())
					for _, line := range stmt.Line {
						printComments("blockline", line.Comment())
					}
					printComments("rparen", stmt.RParen.Comment())
				}
			}

			got := strings.TrimSpace(buf.String())
			want := strings.TrimSpace(test.want)
			if got != want {
				t.Errorf("got:\n%s\nwant:\n%s", got, want)
			}
		})
	}
}

func TestCleanup(t *testing.T) {
	for _, test := range []struct {
		desc  string
		want  string
		input []Expr
	}{
		{
			desc: "simple_lines",
			want: `line: module a
line: require b v1.0.0
`,
			input: []Expr{
				&Line{
					Token: []string{"module", "a"},
				},
				&Line{
					Token: []string{"require", "b", "v1.0.0"},
				},
				&Line{
					Token: nil,
				},
			},
		}, {
			desc: "line_block",
			want: `line: module a
block: require
blockline: b v1.0.0
blockline: c v1.0.0
`,
			input: []Expr{
				&Line{
					Token: []string{"module", "a"},
				},
				&LineBlock{
					Token: []string{"require"},
					Line: []*Line{
						{
							Token:   []string{"b", "v1.0.0"},
							InBlock: true,
						},
						{
							Token:   nil,
							InBlock: true,
						},
						{
							Token:   []string{"c", "v1.0.0"},
							InBlock: true,
						},
					},
				},
			},
		}, {
			desc: "collapse",
			want: `line: module a
line: require b v1.0.0
`,
			input: []Expr{
				&Line{
					Token: []string{"module", "a"},
				},
				&LineBlock{
					Token: []string{"require"},
					Line: []*Line{
						{
							Token:   []string{"b", "v1.0.0"},
							InBlock: true,
						},
						{
							Token:   nil,
							InBlock: true,
						},
					},
				},
			},
		},
	} {
		t.Run(test.desc, func(t *testing.T) {
			syntax := &FileSyntax{
				Stmt: test.input,
			}
			syntax.Cleanup()

			buf := &bytes.Buffer{}
			for _, stmt := range syntax.Stmt {
				switch stmt := stmt.(type) {
				case *Line:
					fmt.Fprintf(buf, "line: %v\n", strings.Join(stmt.Token, " "))
				case *LineBlock:
					fmt.Fprintf(buf, "block: %v\n", strings.Join(stmt.Token, " "))
					for _, line := range stmt.Line {
						fmt.Fprintf(buf, "blockline: %v\n", strings.Join(line.Token, " "))
					}
				}
			}

			got := strings.TrimSpace(buf.String())
			want := strings.TrimSpace(test.want)
			if got != want {
				t.Errorf("got:\n%s\nwant:\n%s", got, want)
			}
		})
	}
}

// Issue 45130: File.Cleanup breaks references so future edits do nothing
func TestCleanupMaintainsRefs(t *testing.T) {
	lineB := &Line{
		Token:   []string{"b", "v1.0.0"},
		InBlock: true,
	}
	syntax := &FileSyntax{
		Stmt: []Expr{
			&LineBlock{
				Token: []string{"require"},
				Line: []*Line{
					lineB,
					{
						Token:   nil,
						InBlock: true,
					},
				},
			},
		},
	}
	syntax.Cleanup()

	if syntax.Stmt[0] != lineB {
		t.Errorf("got:\n%v\nwant:\n%v", syntax.Stmt[0], lineB)
	}
}
