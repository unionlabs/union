package main

import (
	"go/types"
	"testing"

	"github.com/google/go-cmp/cmp"
	"golang.org/x/exp/apidiff"
)

func TestIsInternalPackage(t *testing.T) {
	for _, tst := range []struct {
		name, pkg, mod string
		want           bool
	}{
		{
			name: "not internal no module",
			pkg:  "foo",
			want: false,
		},
		{
			name: "not internal with module",
			pkg:  "example.com/bar/foo",
			mod:  "example.com/bar",
			want: false,
		},
		{
			name: "internal no module",
			pkg:  "internal",
			want: true,
		},
		{
			name: "leading internal no module",
			pkg:  "internal/foo",
			want: true,
		},
		{
			name: "middle internal no module",
			pkg:  "foo/internal/bar",
			want: true,
		},
		{
			name: "ending internal no module",
			pkg:  "foo/internal",
			want: true,
		},

		{
			name: "leading internal with module",
			pkg:  "example.com/baz/internal/foo",
			mod:  "example.com/baz",
			want: true,
		},
		{
			name: "middle internal with module",
			pkg:  "example.com/baz/foo/internal/bar",
			mod:  "example.com/baz",
			want: true,
		},
		{
			name: "ending internal with module",
			pkg:  "example.com/baz/foo/internal",
			mod:  "example.com/baz",
			want: true,
		},
		{
			name: "not package internal with internal module",
			pkg:  "example.com/internal/foo",
			mod:  "example.com/internal",
			want: false,
		},
	} {
		t.Run(tst.name, func(t *testing.T) {
			if got := isInternalPackage(tst.pkg, tst.mod); got != tst.want {
				t.Errorf("expected %v, got %v for %s/%s", tst.want, got, tst.mod, tst.pkg)
			}
		})
	}
}

func TestFilterInternal(t *testing.T) {
	for _, tst := range []struct {
		name  string
		mod   *apidiff.Module
		allow bool
		want  []*types.Package
	}{
		{
			name: "allow internal",
			mod: &apidiff.Module{
				Path: "example.com/foo",
				Packages: []*types.Package{
					types.NewPackage("example.com/foo/bar", "bar"),
					types.NewPackage("example.com/foo/internal", "internal"),
					types.NewPackage("example.com/foo/internal/buz", "buz"),
					types.NewPackage("example.com/foo/bar/internal", "internal"),
				},
			},
			allow: true,
			want: []*types.Package{
				types.NewPackage("example.com/foo/bar", "bar"),
				types.NewPackage("example.com/foo/internal", "internal"),
				types.NewPackage("example.com/foo/internal/buz", "buz"),
				types.NewPackage("example.com/foo/bar/internal", "internal"),
			},
		},
		{
			name: "filter internal",
			mod: &apidiff.Module{
				Path: "example.com/foo",
				Packages: []*types.Package{
					types.NewPackage("example.com/foo/bar", "bar"),
					types.NewPackage("example.com/foo/internal", "internal"),
					types.NewPackage("example.com/foo/internal/buz", "buz"),
					types.NewPackage("example.com/foo/bar/internal", "internal"),
				},
			},
			want: []*types.Package{
				types.NewPackage("example.com/foo/bar", "bar"),
			},
		},
		{
			name: "filter internal nothing left",
			mod: &apidiff.Module{
				Path: "example.com/foo",
				Packages: []*types.Package{
					types.NewPackage("example.com/foo/internal", "internal"),
				},
			},
			want: nil,
		},
	} {
		t.Run(tst.name, func(t *testing.T) {
			filterInternal(tst.mod, tst.allow)
			if diff := cmp.Diff(tst.mod.Packages, tst.want, cmp.Comparer(comparePath)); diff != "" {
				t.Errorf("got(-),want(+):\n%s", diff)
			}
		})
	}
}

func comparePath(x, y *types.Package) bool {
	return x.Path() == y.Path()
}
