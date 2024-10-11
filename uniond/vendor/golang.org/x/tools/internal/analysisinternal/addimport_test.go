// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package analysisinternal_test

import (
	"fmt"
	"go/ast"
	"go/importer"
	"go/parser"
	"go/token"
	"go/types"
	"runtime"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/internal/analysisinternal"
)

func TestAddImport(t *testing.T) {
	descr := func(s string) string {
		if _, _, line, ok := runtime.Caller(1); ok {
			return fmt.Sprintf("L%d %s", line, s)
		}
		panic("runtime.Caller failed")
	}

	// Each test case contains a «name pkgpath»
	// section to be replaced with a reference
	// to a valid import of pkgpath,
	// ideally of the specified name.
	for _, test := range []struct {
		descr, src, want string
	}{
		{
			descr: descr("simple add import"),
			src: `package a
func _() {
	«fmt fmt»
}`,
			want: `package a
import "fmt"

func _() {
	fmt
}`,
		},
		{
			descr: descr("existing import"),
			src: `package a

import "fmt"

func _(fmt.Stringer) {
	«fmt fmt»
}`,
			want: `package a

import "fmt"

func _(fmt.Stringer) {
	fmt
}`,
		},
		{
			descr: descr("existing blank import"),
			src: `package a

import _ "fmt"

func _() {
	«fmt fmt»
}`,
			want: `package a

import "fmt"

import _ "fmt"

func _() {
	fmt
}`,
		},
		{
			descr: descr("existing renaming import"),
			src: `package a

import fmtpkg "fmt"

var fmt int

func _(fmtpkg.Stringer) {
	«fmt fmt»
}`,
			want: `package a

import fmtpkg "fmt"

var fmt int

func _(fmtpkg.Stringer) {
	fmtpkg
}`,
		},
		{
			descr: descr("existing import is shadowed"),
			src: `package a

import "fmt"

var _ fmt.Stringer

func _(fmt int) {
	«fmt fmt»
}`,
			want: `package a

import fmt0 "fmt"

import "fmt"

var _ fmt.Stringer

func _(fmt int) {
	fmt0
}`,
		},
		{
			descr: descr("preferred name is shadowed"),
			src: `package a

import "fmt"

func _(fmt fmt.Stringer) {
	«fmt fmt»
}`,
			want: `package a

import fmt0 "fmt"

import "fmt"

func _(fmt fmt.Stringer) {
	fmt0
}`,
		},
		{
			descr: descr("import inserted before doc comments"),
			src: `package a

// hello
import ()

// world
func _() {
	«fmt fmt»
}`,
			want: `package a

import "fmt"

// hello
import ()

// world
func _() {
	fmt
}`,
		},
		{
			descr: descr("arbitrary preferred name => renaming import"),
			src: `package a

func _() {
	«foo encoding/json»
}`,
			want: `package a

import foo "encoding/json"

func _() {
	foo
}`,
		},
	} {
		t.Run(test.descr, func(t *testing.T) {
			// splice marker
			before, mid, ok1 := strings.Cut(test.src, "«")
			mid, after, ok2 := strings.Cut(mid, "»")
			if !ok1 || !ok2 {
				t.Fatal("no «name path» marker")
			}
			src := before + "/*!*/" + after
			name, path, _ := strings.Cut(mid, " ")

			// parse
			fset := token.NewFileSet()
			f, err := parser.ParseFile(fset, "a.go", src, parser.ParseComments)
			if err != nil {
				t.Log(err)
			}
			pos := fset.File(f.Pos()).Pos(len(before))

			// type-check
			info := &types.Info{
				Types:     make(map[ast.Expr]types.TypeAndValue),
				Scopes:    make(map[ast.Node]*types.Scope),
				Defs:      make(map[*ast.Ident]types.Object),
				Implicits: make(map[ast.Node]types.Object),
			}
			conf := &types.Config{
				Error:    func(err error) { t.Log(err) },
				Importer: importer.Default(),
			}
			conf.Check(f.Name.Name, fset, []*ast.File{f}, info)

			// add import
			name, edits := analysisinternal.AddImport(info, f, pos, path, name)

			var edit analysis.TextEdit
			switch len(edits) {
			case 0:
			case 1:
				edit = edits[0]
			default:
				t.Fatalf("expected at most one edit, got %d", len(edits))
			}

			// apply patch
			start := fset.Position(edit.Pos)
			end := fset.Position(edit.End)
			output := src[:start.Offset] + string(edit.NewText) + src[end.Offset:]
			output = strings.ReplaceAll(output, "/*!*/", name)
			if output != test.want {
				t.Errorf("\n--got--\n%s\n--want--\n%s\n--diff--\n%s",
					output, test.want, cmp.Diff(test.want, output))
			}
		})
	}
}
