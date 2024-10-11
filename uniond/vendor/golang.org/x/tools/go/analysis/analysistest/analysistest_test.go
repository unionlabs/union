// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package analysistest_test

import (
	"fmt"
	"go/token"
	"log"
	"os"
	"reflect"
	"strings"
	"testing"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/analysistest"
	"golang.org/x/tools/go/analysis/passes/findcall"
	"golang.org/x/tools/internal/testenv"
	"golang.org/x/tools/internal/testfiles"
	"golang.org/x/tools/txtar"
)

func init() {
	// Run() decides when tests use GOPATH mode or modules.
	// We turn off GOPROXY just for good measure.
	if err := os.Setenv("GOPROXY", "off"); err != nil {
		log.Fatal(err)
	}
}

// TestTheTest tests the analysistest testing infrastructure.
func TestTheTest(t *testing.T) {
	testenv.NeedsTool(t, "go")

	// We'll simulate a partly failing test of the findcall analysis,
	// which (by default) reports calls to functions named 'println'.
	findcall.Analyzer.Flags.Set("name", "println")

	filemap := map[string]string{
		"a/b.go": `package main // want package:"found"

func main() {
	// The expectation is ill-formed:
	print() // want: "diagnostic"
	print() // want foo"fact"
	print() // want foo:
	print() // want "\xZZ scan error"

	// A diagnostic is reported at this line, but the expectation doesn't match:
	println("hello, world") // want "wrong expectation text"

	// An unexpected diagnostic is reported at this line:
	println() // trigger an unexpected diagnostic

	// No diagnostic is reported at this line:
	print()	// want "unsatisfied expectation"

	// OK
	println("hello, world") // want "call of println"

	// OK /* */-form.
	println("안녕, 세계") /* want "call of println" */

	// OK  (nested comment)
	println("Γειά σου, Κόσμε") // some comment // want "call of println"

	// OK (nested comment in /**/)
	println("你好，世界") /* some comment // want "call of println" */

	// OK (multiple expectations on same line)
	println(); println() // want "call of println(...)" "call of println(...)"

	// A Line that is not formatted correctly in the golden file.
}

// OK (facts and diagnostics on same line)
func println(...interface{}) { println() } // want println:"found" "call of println(...)"

`,
		"a/b.go.golden": `package main // want package:"found"

func main() {
	// The expectation is ill-formed:
	print() // want: "diagnostic"
	print() // want foo"fact"
	print() // want foo:
	print() // want "\xZZ scan error"

	// A diagnostic is reported at this line, but the expectation doesn't match:
	println_TEST_("hello, world") // want "wrong expectation text"

	// An unexpected diagnostic is reported at this line:
	println_TEST_() // trigger an unexpected diagnostic

	// No diagnostic is reported at this line:
	print() // want "unsatisfied expectation"

	// OK
	println_TEST_("hello, world") // want "call of println"

	// OK /* */-form.
	println_TEST_("안녕, 세계") /* want "call of println" */

	// OK  (nested comment)
	println_TEST_("Γειά σου, Κόσμε") // some comment // want "call of println"

	// OK (nested comment in /**/)
	println_TEST_("你好，世界") /* some comment // want "call of println" */

	// OK (multiple expectations on same line)
	println_TEST_()
	println_TEST_() // want "call of println(...)" "call of println(...)"
	
			// A Line that is not formatted correctly in the golden file.
}

// OK (facts and diagnostics on same line)
func println(...interface{}) { println_TEST_() } // want println:"found" "call of println(...)"
`,
		"a/b_test.go": `package main

// Test file shouldn't mess with things (issue #40574)
`,
	}
	dir, cleanup, err := analysistest.WriteFiles(filemap)
	if err != nil {
		t.Fatal(err)
	}
	defer cleanup()

	var got []string
	t2 := errorfunc(func(s string) { got = append(got, s) }) // a fake *testing.T
	analysistest.RunWithSuggestedFixes(t2, dir, findcall.Analyzer, "a")

	want := []string{
		`a/b.go:5: in 'want' comment: unexpected ":"`,
		`a/b.go:6: in 'want' comment: got String after foo, want ':'`,
		`a/b.go:7: in 'want' comment: got EOF, want regular expression`,
		`a/b.go:8: in 'want' comment: invalid char escape`,
		"a/b.go:11:9: diagnostic \"call of println(...)\" does not match pattern `wrong expectation text`",
		`a/b.go:14:9: unexpected diagnostic: call of println(...)`,
		"a/b.go:11: no diagnostic was reported matching `wrong expectation text`",
		"a/b.go:17: no diagnostic was reported matching `unsatisfied expectation`",
		// duplicate copies of each message from the test package (see issue #40574)
		`a/b.go:5: in 'want' comment: unexpected ":"`,
		`a/b.go:6: in 'want' comment: got String after foo, want ':'`,
		`a/b.go:7: in 'want' comment: got EOF, want regular expression`,
		`a/b.go:8: in 'want' comment: invalid char escape`,
		"a/b.go:11:9: diagnostic \"call of println(...)\" does not match pattern `wrong expectation text`",
		`a/b.go:14:9: unexpected diagnostic: call of println(...)`,
		"a/b.go:11: no diagnostic was reported matching `wrong expectation text`",
		"a/b.go:17: no diagnostic was reported matching `unsatisfied expectation`",
	}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("got:\n%s\nwant:\n%s",
			strings.Join(got, "\n"),
			strings.Join(want, "\n"))
	}
}

// TestNoEnd tests that a missing SuggestedFix.End position is
// correctly interpreted as if equal to SuggestedFix.Pos (see issue #64199).
func TestNoEnd(t *testing.T) {
	noend := &analysis.Analyzer{
		Name: "noend",
		Doc:  "inserts /*hello*/ before first decl",
		Run: func(pass *analysis.Pass) (any, error) {
			decl := pass.Files[0].Decls[0]
			pass.Report(analysis.Diagnostic{
				Pos:     decl.Pos(),
				End:     token.NoPos,
				Message: "say hello",
				SuggestedFixes: []analysis.SuggestedFix{{
					Message: "say hello",
					TextEdits: []analysis.TextEdit{
						{
							Pos:     decl.Pos(),
							End:     token.NoPos,
							NewText: []byte("/*hello*/"),
						},
					},
				}},
			})
			return nil, nil
		},
	}

	filemap := map[string]string{
		"a/a.go": `package a

func F() {} // want "say hello"`,
		"a/a.go.golden": `package a

/*hello*/
func F() {} // want "say hello"`,
	}
	dir, cleanup, err := analysistest.WriteFiles(filemap)
	if err != nil {
		t.Fatal(err)
	}
	defer cleanup()

	analysistest.RunWithSuggestedFixes(t, dir, noend, "a")
}

func TestModule(t *testing.T) {
	const content = `
Test that analysis.pass.Module is populated.

-- go.mod --
module golang.org/fake/mod

go 1.21

require golang.org/xyz/fake v0.12.34

-- mod.go --
// We expect a module.Path and a module.GoVersion, but an empty module.Version.

package mod // want "golang.org/fake/mod,,1.21"

import "golang.org/xyz/fake/ver"

var _ ver.T

-- vendor/modules.txt --
# golang.org/xyz/fake v0.12.34
## explicit; go 1.18
golang.org/xyz/fake/ver

-- vendor/golang.org/xyz/fake/ver/ver.go --
// This package is vendored so that we can populate a non-empty
// Pass.Module.Version is in a test.

package ver //want "golang.org/xyz/fake,v0.12.34,1.18"

type T string
`
	fs, err := txtar.FS(txtar.Parse([]byte(content)))
	if err != nil {
		t.Fatal(err)
	}
	dir := testfiles.CopyToTmp(t, fs)

	filever := &analysis.Analyzer{
		Name: "mod",
		Doc:  "reports module information",
		Run: func(pass *analysis.Pass) (any, error) {
			msg := "no module info"
			if m := pass.Module; m != nil {
				msg = fmt.Sprintf("%s,%s,%s", m.Path, m.Version, m.GoVersion)
			}
			for _, file := range pass.Files {
				pass.Reportf(file.Package, "%s", msg)
			}
			return nil, nil
		},
	}
	analysistest.Run(t, dir, filever, "golang.org/fake/mod", "golang.org/xyz/fake/ver")
}

type errorfunc func(string)

func (f errorfunc) Errorf(format string, args ...interface{}) {
	f(fmt.Sprintf(format, args...))
}
