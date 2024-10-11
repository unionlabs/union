// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package stdversion_test

import (
	"path/filepath"
	"testing"

	"golang.org/x/tools/go/analysis/analysistest"
	"golang.org/x/tools/go/analysis/passes/stdversion"
	"golang.org/x/tools/internal/testenv"
	"golang.org/x/tools/internal/testfiles"
)

func Test(t *testing.T) {
	t.Skip("Disabled for golang.org/cl/603895. Fix and re-enable.")
	// The test relies on go1.21 std symbols, but the analyzer
	// itself requires the go1.22 implementation of versions.FileVersions.
	testenv.NeedsGo1Point(t, 22)

	dir := testfiles.ExtractTxtarFileToTmp(t, filepath.Join(analysistest.TestData(), "test.txtar"))
	analysistest.Run(t, dir, stdversion.Analyzer,
		"example.com/a",
		"example.com/sub",
		"example.com/sub20",
		"example.com/old")
}
