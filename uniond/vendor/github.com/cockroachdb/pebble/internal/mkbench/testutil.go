// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"bytes"
	"io"
	"os"
	"path/filepath"
	"runtime"
	"testing"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/oserror"
	"github.com/pmezard/go-difflib/difflib"
)

// filesEqual returns the diff between contents of a and b.
func filesEqual(a, b string) error {
	aBytes, err := os.ReadFile(a)
	if err != nil {
		return err
	}
	bBytes, err := os.ReadFile(b)
	if err != nil {
		return err
	}

	// Normalize newlines.
	aBytes = bytes.Replace(aBytes, []byte{13, 10} /* \r\n */, []byte{10} /* \n */, -1)
	bBytes = bytes.Replace(bBytes, []byte{13, 10}, []byte{10}, -1)

	d, _ := difflib.GetUnifiedDiffString(difflib.UnifiedDiff{
		A: difflib.SplitLines(string(aBytes)),
		B: difflib.SplitLines(string(bBytes)),
	})
	if d != "" {
		return errors.Errorf("a != b\ndiff = %s", d)
	}

	return nil
}

// copyDir recursively copies the fromPath to toPath, excluding certain paths.
func copyDir(fromPath, toPath string) error {
	walkFn := func(path, pathRel string, info os.FileInfo) error {
		// Preserve the directory structure.
		if info.IsDir() {
			err := os.Mkdir(filepath.Join(toPath, pathRel), 0700)
			if err != nil && !oserror.IsNotExist(err) {
				return err
			}
			return nil
		}

		// Copy files.
		fIn, err := os.Open(path)
		if err != nil {
			return err
		}
		defer func() { _ = fIn.Close() }()

		fOut, err := os.OpenFile(filepath.Join(toPath, pathRel), os.O_CREATE|os.O_WRONLY, 0700)
		if err != nil {
			return err
		}
		defer func() { _ = fOut.Close() }()

		_, err = io.Copy(fOut, fIn)
		return err
	}
	return walkDir(fromPath, walkFn)
}

func maybeSkip(t *testing.T) {
	// The paths in the per-run summary.json files are UNIX-oriented. To avoid
	// duplicating the test fixtures just for Windows, just skip the tests on
	// Windows.
	if runtime.GOOS == "windows" {
		t.Skipf("skipped on windows")
	}
}
