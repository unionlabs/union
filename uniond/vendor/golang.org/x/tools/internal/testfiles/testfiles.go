// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package testfiles provides utilities for writing Go tests with files
// in testdata.
package testfiles

import (
	"io"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"golang.org/x/tools/txtar"
)

// CopyToTmp copies the files and directories in src to a new temporary testing
// directory dst, and returns dst on success.
//
// After copying the files, it processes each of the 'old,new,' rename
// directives in order. Each rename directive moves the relative path "old"
// to the relative path "new" within the directory.
//
// Renaming allows tests to hide files whose names have
// special meaning, such as "go.mod" files or "testdata" directories
// from the go command, or ill-formed Go source files from gofmt.
//
// For example if we copy the directory testdata:
//
//	testdata/
//	    go.mod.test
//	    a/a.go
//	    b/b.go
//
// with the rename "go.mod.test,go.mod", the resulting files will be:
//
//	dst/
//	    go.mod
//	    a/a.go
//	    b/b.go
func CopyToTmp(t testing.TB, src fs.FS, rename ...string) string {
	dstdir := t.TempDir()

	if err := copyFS(dstdir, src); err != nil {
		t.Fatal(err)
	}
	for _, r := range rename {
		old, new, found := strings.Cut(r, ",")
		if !found {
			t.Fatalf("rename directive %q does not contain delimiter %q", r, ",")
		}
		oldpath := filepath.Join(dstdir, old)
		newpath := filepath.Join(dstdir, new)
		if err := os.Rename(oldpath, newpath); err != nil {
			t.Fatal(err)
		}
	}

	return dstdir
}

// Copy the files in src to dst.
// Use os.CopyFS when 1.23 can be used in x/tools.
func copyFS(dstdir string, src fs.FS) error {
	return fs.WalkDir(src, ".", func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		newpath := filepath.Join(dstdir, path)
		if d.IsDir() {
			return os.MkdirAll(newpath, 0777)
		}
		r, err := src.Open(path)
		if err != nil {
			return err
		}
		defer r.Close()

		w, err := os.Create(newpath)
		if err != nil {
			return err
		}
		defer w.Close()
		_, err = io.Copy(w, r)
		return err
	})
}

// ExtractTxtarFileToTmp read a txtar archive on a given path,
// extracts it to a temporary directory, and returns the
// temporary directory.
func ExtractTxtarFileToTmp(t testing.TB, file string) string {
	ar, err := txtar.ParseFile(file)
	if err != nil {
		t.Fatal(err)
	}

	fs, err := txtar.FS(ar)
	if err != nil {
		t.Fatal(err)
	}
	return CopyToTmp(t, fs)
}
