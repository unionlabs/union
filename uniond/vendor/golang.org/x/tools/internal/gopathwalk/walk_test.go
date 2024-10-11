// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package gopathwalk

import (
	"os"
	"path/filepath"
	"reflect"
	"runtime"
	"sort"
	"strings"
	"sync"
	"testing"
)

func TestSymlinkTraversal(t *testing.T) {
	t.Parallel()

	gopath := t.TempDir()

	if err := mapToDir(gopath, map[string]string{
		"a/b/c":          "LINK:../../a/d",
		"a/b/pkg/pkg.go": "package pkg",
		"a/d/e":          "LINK:../../a/b",
		"a/d/pkg/pkg.go": "package pkg",
		"a/f/loop":       "LINK:../f",
		"a/f/pkg/pkg.go": "package pkg",
		"a/g/pkg/pkg.go": "LINK:../../f/pkg/pkg.go",
		"a/self":         "LINK:.",
	}); err != nil {
		switch runtime.GOOS {
		case "windows", "plan9":
			t.Skipf("skipping symlink-requiring test on %s", runtime.GOOS)
		}
		t.Fatal(err)
	}

	pkgc := make(chan []string, 1)
	pkgc <- nil
	add := func(root Root, dir string) {
		rel, err := filepath.Rel(filepath.Join(root.Path, "src"), dir)
		if err != nil {
			t.Error(err)
		}
		pkgc <- append(<-pkgc, filepath.ToSlash(rel))
	}

	Walk([]Root{{Path: gopath, Type: RootGOPATH}}, add, Options{Logf: t.Logf})

	pkgs := <-pkgc
	sort.Strings(pkgs)
	t.Logf("Found packages:\n\t%s", strings.Join(pkgs, "\n\t"))

	got := make(map[string]bool, len(pkgs))
	for _, pkg := range pkgs {
		got[pkg] = true
	}
	tests := []struct {
		path string
		want bool
		why  string
	}{
		{
			path: "a/b/pkg",
			want: true,
			why:  "found via regular directories",
		},
		{
			path: "a/b/c/pkg",
			want: true,
			why:  "found via non-cyclic dir link",
		},
		{
			path: "a/b/c/e/pkg",
			want: true,
			why:  "found via two non-cyclic dir links",
		},
		{
			path: "a/d/e/c/pkg",
			want: true,
			why:  "found via two non-cyclic dir links",
		},
		{
			path: "a/f/loop/pkg",
			want: true,
			why:  "found via a single parent-dir link",
		},
		{
			path: "a/f/loop/loop/pkg",
			want: false,
			why:  "would follow loop symlink twice",
		},
		{
			path: "a/self/b/pkg",
			want: true,
			why:  "follows self-link once",
		},
		{
			path: "a/self/self/b/pkg",
			want: false,
			why:  "would follow self-link twice",
		},
	}
	for _, tc := range tests {
		if got[tc.path] != tc.want {
			if tc.want {
				t.Errorf("MISSING: %s (%s)", tc.path, tc.why)
			} else {
				t.Errorf("UNEXPECTED: %s (%s)", tc.path, tc.why)
			}
		}
	}
}

// TestSkip tests that various goimports rules are followed in non-modules mode.
func TestSkip(t *testing.T) {
	t.Parallel()

	dir := t.TempDir()

	if err := mapToDir(dir, map[string]string{
		"ignoreme/f.go":     "package ignoreme",     // ignored by .goimportsignore
		"node_modules/f.go": "package nodemodules;", // ignored by hardcoded node_modules filter
		"v/f.go":            "package v;",           // ignored by hardcoded vgo cache rule
		"mod/f.go":          "package mod;",         // ignored by hardcoded vgo cache rule
		"shouldfind/f.go":   "package shouldfind;",  // not ignored

		".goimportsignore": "ignoreme\n",
	}); err != nil {
		t.Fatal(err)
	}

	var found []string
	var mu sync.Mutex
	walkDir(Root{filepath.Join(dir, "src"), RootGOPATH},
		func(root Root, dir string) {
			mu.Lock()
			defer mu.Unlock()
			found = append(found, dir[len(root.Path)+1:])
		}, func(root Root, dir string) bool {
			return false
		}, Options{
			ModulesEnabled: false,
			Logf:           t.Logf,
		})
	if want := []string{"shouldfind"}; !reflect.DeepEqual(found, want) {
		t.Errorf("expected to find only %v, got %v", want, found)
	}
}

// TestSkipFunction tests that scan successfully skips directories from user callback.
func TestSkipFunction(t *testing.T) {
	t.Parallel()

	dir := t.TempDir()

	if err := mapToDir(dir, map[string]string{
		"ignoreme/f.go":           "package ignoreme",    // ignored by skip
		"ignoreme/subignore/f.go": "package subignore",   // also ignored by skip
		"shouldfind/f.go":         "package shouldfind;", // not ignored
	}); err != nil {
		t.Fatal(err)
	}

	var found []string
	var mu sync.Mutex
	walkDir(Root{filepath.Join(dir, "src"), RootGOPATH},
		func(root Root, dir string) {
			mu.Lock()
			defer mu.Unlock()
			found = append(found, dir[len(root.Path)+1:])
		}, func(root Root, dir string) bool {
			return strings.HasSuffix(dir, "ignoreme")
		},
		Options{
			ModulesEnabled: false,
			Logf:           t.Logf,
		})
	if want := []string{"shouldfind"}; !reflect.DeepEqual(found, want) {
		t.Errorf("expected to find only %v, got %v", want, found)
	}
}

// TestWalkSymlinkConcurrentDeletion is a regression test for the panic reported
// in https://go.dev/issue/58054#issuecomment-1791513726.
func TestWalkSymlinkConcurrentDeletion(t *testing.T) {
	t.Parallel()

	src := t.TempDir()

	m := map[string]string{
		"dir/readme.txt": "dir is not a go package",
		"dirlink":        "LINK:dir",
	}
	if err := mapToDir(src, m); err != nil {
		switch runtime.GOOS {
		case "windows", "plan9":
			t.Skipf("skipping symlink-requiring test on %s", runtime.GOOS)
		}
		t.Fatal(err)
	}

	done := make(chan struct{})
	go func() {
		if err := os.RemoveAll(src); err != nil {
			t.Log(err)
		}
		close(done)
	}()
	defer func() {
		<-done
	}()

	add := func(root Root, dir string) {
		t.Errorf("unexpected call to add(%q, %q)", root.Path, dir)
	}
	Walk([]Root{{Path: src, Type: RootGOPATH}}, add, Options{Logf: t.Logf})
}

func mapToDir(destDir string, files map[string]string) error {
	var symlinkPaths []string
	for path, contents := range files {
		file := filepath.Join(destDir, "src", path)
		if err := os.MkdirAll(filepath.Dir(file), 0755); err != nil {
			return err
		}
		var err error
		if strings.HasPrefix(contents, "LINK:") {
			// To work around https://go.dev/issue/39183, wait to create symlinks
			// until we have created all non-symlink paths.
			symlinkPaths = append(symlinkPaths, path)
		} else {
			err = os.WriteFile(file, []byte(contents), 0644)
		}
		if err != nil {
			return err
		}
	}

	for _, path := range symlinkPaths {
		file := filepath.Join(destDir, "src", path)
		target := filepath.FromSlash(strings.TrimPrefix(files[path], "LINK:"))
		err := os.Symlink(target, file)
		if err != nil {
			return err
		}
	}

	return nil
}
