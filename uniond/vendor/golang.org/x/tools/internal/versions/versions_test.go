// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package versions_test

import (
	"go/ast"
	"go/parser"
	"go/token"
	"go/types"
	"testing"

	"golang.org/x/tools/internal/testenv"
	"golang.org/x/tools/internal/versions"
)

func TestIsValid(t *testing.T) {
	// valid versions
	for _, x := range []string{
		"go1.21",
		"go1.21.2",
		"go1.21rc",
		"go1.21rc2",
		"go0.0", // ??
		"go1",
		"go2",
		"go1.20.0-bigcorp",
	} {
		if !versions.IsValid(x) {
			t.Errorf("expected versions.IsValid(%q) to hold", x)
		}
	}

	// invalid versions
	for _, x := range []string{
		"",
		"bad",
		"1.21",
		"v1.21",
		"go",
		"goAA",
		"go2_3",
		"go1.BB",
		"go1.21.",
		"go1.21.2_2",
		"go1.21rc_2",
		"go1.21rc2_",
		"go1.600+auto",
	} {
		if versions.IsValid(x) {
			t.Errorf("expected versions.IsValid(%q) to not hold", x)
		}
	}
}

func TestVersionComparisons(t *testing.T) {
	for _, item := range []struct {
		x, y string
		want int
	}{
		// All comparisons of go2, go1.21.2, go1.21rc2, go1.21rc2, go1, go0.0, "", bad
		{"go2", "go2", 0},
		{"go2", "go1.21.2", +1},
		{"go2", "go1.21rc2", +1},
		{"go2", "go1.21rc", +1},
		{"go2", "go1.21", +1},
		{"go2", "go1", +1},
		{"go2", "go0.0", +1},
		{"go2", "", +1},
		{"go2", "bad", +1},
		{"go1.21.2", "go1.21.2", 0},
		{"go1.21.2", "go1.21rc2", +1},
		{"go1.21.2", "go1.21rc", +1},
		{"go1.21.2", "go1.21", +1},
		{"go1.21.2", "go1", +1},
		{"go1.21.2", "go0.0", +1},
		{"go1.21.2", "", +1},
		{"go1.21.2", "bad", +1},
		{"go1.21rc2", "go1.21rc2", 0},
		{"go1.21rc2", "go1.21rc", +1},
		{"go1.21rc2", "go1.21", +1},
		{"go1.21rc2", "go1", +1},
		{"go1.21rc2", "go0.0", +1},
		{"go1.21rc2", "", +1},
		{"go1.21rc2", "bad", +1},
		{"go1.21rc", "go1.21rc", 0},
		{"go1.21rc", "go1.21", +1},
		{"go1.21rc", "go1", +1},
		{"go1.21rc", "go0.0", +1},
		{"go1.21rc", "", +1},
		{"go1.21rc", "bad", +1},
		{"go1.21", "go1.21", 0},
		{"go1.21", "go1", +1},
		{"go1.21", "go0.0", +1},
		{"go1.21", "", +1},
		{"go1.21", "bad", +1},
		{"go1", "go1", 0},
		{"go1", "go0.0", +1},
		{"go1", "", +1},
		{"go1", "bad", +1},
		{"go0.0", "go0.0", 0},
		{"go0.0", "", +1},
		{"go0.0", "bad", +1},
		{"", "", 0},
		{"", "bad", 0},
		{"bad", "bad", 0},
		// Other tests.
		{"go1.20", "go1.20.0-bigcorp", 0},
		{"go1.21", "go1.21.0-bigcorp", -1},  // Starting in Go 1.21, patch missing is different from explicit .0.
		{"go1.21.0", "go1.21.0-bigcorp", 0}, // Starting in Go 1.21, patch missing is different from explicit .0.
		{"go1.19rc1", "go1.19", -1},
	} {
		got := versions.Compare(item.x, item.y)
		if got != item.want {
			t.Errorf("versions.Compare(%q, %q)=%d. expected %d", item.x, item.y, got, item.want)
		}
		reverse := versions.Compare(item.y, item.x)
		if reverse != -got {
			t.Errorf("versions.Compare(%q, %q)=%d. expected %d", item.y, item.x, reverse, -got)
		}
	}
}

func TestLang(t *testing.T) {
	for _, item := range []struct {
		x    string
		want string
	}{
		// valid
		{"go1.21rc2", "go1.21"},
		{"go1.21.2", "go1.21"},
		{"go1.21", "go1.21"},
		{"go1", "go1"},
		// invalid
		{"bad", ""},
		{"1.21", ""},
	} {
		if got := versions.Lang(item.x); got != item.want {
			t.Errorf("versions.Lang(%q)=%q. expected %q", item.x, got, item.want)
		}
	}

}

func TestKnown(t *testing.T) {
	for _, v := range [...]string{
		versions.Go1_18,
		versions.Go1_19,
		versions.Go1_20,
		versions.Go1_21,
		versions.Go1_22,
	} {
		if !versions.IsValid(v) {
			t.Errorf("Expected known version %q to be valid.", v)
		}
		if v != versions.Lang(v) {
			t.Errorf("Expected known version %q == Lang(%q).", v, versions.Lang(v))
		}
	}
}

func TestAtLeast(t *testing.T) {
	for _, item := range [...]struct {
		v, release string
		want       bool
	}{
		{versions.Future, versions.Go1_22, true},
		{versions.Go1_22, versions.Go1_22, true},
		{"go1.21", versions.Go1_22, false},
		{"invalid", versions.Go1_22, false},
	} {
		if got := versions.AtLeast(item.v, item.release); got != item.want {
			t.Errorf("AtLeast(%q, %q)=%v. wanted %v", item.v, item.release, got, item.want)
		}
	}
}

func TestBefore(t *testing.T) {
	for _, item := range [...]struct {
		v, release string
		want       bool
	}{
		{versions.Future, versions.Go1_22, false},
		{versions.Go1_22, versions.Go1_22, false},
		{"go1.21", versions.Go1_22, true},
		{"invalid", versions.Go1_22, true}, // invalid < Go1_22
	} {
		if got := versions.Before(item.v, item.release); got != item.want {
			t.Errorf("Before(%q, %q)=%v. wanted %v", item.v, item.release, got, item.want)
		}
	}
}

func TestFileVersions122(t *testing.T) {
	testenv.NeedsGo1Point(t, 22)

	const source = `
	package P
	`
	fset := token.NewFileSet()
	f, err := parser.ParseFile(fset, "hello.go", source, 0)
	if err != nil {
		t.Fatal(err)
	}

	for _, conf := range []types.Config{
		{GoVersion: versions.Go1_22},
		{}, // GoVersion is unset.
	} {
		info := &types.Info{}
		versions.InitFileVersions(info)

		_, err = conf.Check("P", fset, []*ast.File{f}, info)
		if err != nil {
			t.Fatal(err)
		}

		v := versions.FileVersion(info, f)
		if !versions.AtLeast(v, versions.Go1_22) {
			t.Errorf("versions.AtLeast(%q, %q) expected to hold", v, versions.Go1_22)
		}

		if versions.Before(v, versions.Go1_22) {
			t.Errorf("versions.AtLeast(%q, %q) expected to be false", v, versions.Go1_22)
		}

		if conf.GoVersion == "" && v != versions.Future {
			t.Error("Expected the FileVersion to be the Future when conf.GoVersion is unset")
		}
	}
}

func TestFileVersions121(t *testing.T) {
	testenv.SkipAfterGo1Point(t, 21)

	// If <1.22, info and file are ignored.
	v := versions.FileVersion(nil, nil)
	oneof := map[string]bool{
		versions.Go1_18: true,
		versions.Go1_19: true,
		versions.Go1_20: true,
		versions.Go1_21: true,
	}
	if !oneof[v] {
		t.Errorf("FileVersion(...)=%q expected to be a known go version <1.22", v)
	}

	if versions.AtLeast(v, versions.Go1_22) {
		t.Errorf("versions.AtLeast(%q, %q) expected to be false", v, versions.Go1_22)
	}

	if !versions.Before(v, versions.Go1_22) {
		t.Errorf("versions.Before(%q, %q) expected to hold", v, versions.Go1_22)
	}
}
