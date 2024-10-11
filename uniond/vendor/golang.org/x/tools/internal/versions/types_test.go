// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package versions_test

import (
	"fmt"
	"go/ast"
	"go/importer"
	"go/parser"
	"go/token"
	"go/types"
	"testing"

	"golang.org/x/tools/internal/testenv"
	"golang.org/x/tools/internal/versions"
)

func Test(t *testing.T) {
	testenv.NeedsGo1Point(t, 22)

	var contents = map[string]string{
		"gobuild.go": `
	//go:build go1.23
	package p
	`,
		"noversion.go": `
	package p
	`,
	}
	type fileTest struct {
		fname string
		want  string
	}
	for _, item := range []struct {
		goversion string
		pversion  string
		tests     []fileTest
	}{
		//  {"", "", []fileTest{{"noversion.go", ""}, {"gobuild.go", ""}}}, // TODO(matloob): re-enable this test (with modifications) once CL 607955 has been submitted
		{"go1.22", "go1.22", []fileTest{{"noversion.go", "go1.22"}, {"gobuild.go", "go1.23"}}},
	} {
		name := fmt.Sprintf("types.Config{GoVersion:%q}", item.goversion)
		t.Run(name, func(t *testing.T) {
			fset := token.NewFileSet()
			files := make([]*ast.File, len(item.tests))
			for i, test := range item.tests {
				files[i] = parse(t, fset, test.fname, contents[test.fname])
			}
			pkg, info := typeCheck(t, fset, files, item.goversion)
			if got, want := versions.GoVersion(pkg), item.pversion; versions.Compare(got, want) != 0 {
				t.Errorf("GoVersion()=%q. expected %q", got, want)
			}
			if got := versions.FileVersion(info, nil); got != "" {
				t.Errorf(`FileVersions(nil)=%q. expected ""`, got)
			}
			for i, test := range item.tests {
				if got, want := versions.FileVersion(info, files[i]), test.want; got != want {
					t.Errorf("FileVersions(%s)=%q. expected %q", test.fname, got, want)
				}
			}
		})
	}
}

func parse(t *testing.T, fset *token.FileSet, name, src string) *ast.File {
	file, err := parser.ParseFile(fset, name, src, 0)
	if err != nil {
		t.Fatal(err)
	}
	return file
}

func typeCheck(t *testing.T, fset *token.FileSet, files []*ast.File, goversion string) (*types.Package, *types.Info) {
	conf := types.Config{
		Importer:  importer.Default(),
		GoVersion: goversion,
	}
	info := types.Info{}
	versions.InitFileVersions(&info)
	pkg, err := conf.Check("", fset, files, &info)
	if err != nil {
		t.Fatal(err)
	}
	return pkg, &info
}
