// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package testfiles_test

import (
	"fmt"
	"os"
	"path/filepath"
	"testing"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/analysistest"
	"golang.org/x/tools/internal/testenv"
	"golang.org/x/tools/internal/testfiles"
	"golang.org/x/tools/internal/versions"
	"golang.org/x/tools/txtar"
)

func TestTestDir(t *testing.T) {
	testenv.NeedsGo1Point(t, 23)

	// Files are initially {go.mod.test,sub.test/sub.go.test}.
	fs := os.DirFS(filepath.Join(analysistest.TestData(), "versions"))
	tmpdir := testfiles.CopyToTmp(t, fs,
		"go.mod.test,go.mod",                // After: {go.mod,sub.test/sub.go.test}
		"sub.test/sub.go.test,sub.test/abc", // After: {go.mod,sub.test/abc}
		"sub.test,sub",                      // After: {go.mod,sub/abc}
		"sub/abc,sub/sub.go",                // After: {go.mod,sub/sub.go}
	)

	filever := &analysis.Analyzer{
		Name: "filever",
		Doc:  "reports file go versions",
		Run: func(pass *analysis.Pass) (any, error) {
			for _, file := range pass.Files {
				ver := versions.FileVersion(pass.TypesInfo, file)
				name := filepath.Base(pass.Fset.Position(file.Package).Filename)
				pass.Reportf(file.Package, "%s@%s", name, ver)
			}
			return nil, nil
		},
	}
	res := analysistest.Run(t, tmpdir, filever, "golang.org/fake/versions", "golang.org/fake/versions/sub")
	got := 0
	for _, r := range res {
		got += len(r.Diagnostics)
	}

	if want := 4; got != want {
		t.Errorf("Got %d diagnostics. wanted %d", got, want)
	}
}

func TestTestDirErrors(t *testing.T) {
	const input = `
-- one.txt --
one
`
	// Files are initially {go.mod.test,sub.test/sub.go.test}.
	fs, err := txtar.FS(txtar.Parse([]byte(input)))
	if err != nil {
		t.Fatal(err)
	}

	directive := "no comma to split on"
	intercept := &fatalIntercept{t, nil}
	func() {
		defer func() { // swallow panics from fatalIntercept.Fatal
			if r := recover(); r != intercept {
				panic(r)
			}
		}()
		testfiles.CopyToTmp(intercept, fs, directive)
	}()

	got := fmt.Sprint(intercept.fatalfs)
	want := `[rename directive "no comma to split on" does not contain delimiter ","]`
	if got != want {
		t.Errorf("CopyToTmp(%q) had the Fatal messages %q. wanted %q", directive, got, want)
	}
}

// helper for TestTestDirErrors
type fatalIntercept struct {
	testing.TB
	fatalfs []string
}

func (i *fatalIntercept) Fatalf(format string, args ...any) {
	i.fatalfs = append(i.fatalfs, fmt.Sprintf(format, args...))
	// Do not mark the test as failing, but fail early.
	panic(i)
}
