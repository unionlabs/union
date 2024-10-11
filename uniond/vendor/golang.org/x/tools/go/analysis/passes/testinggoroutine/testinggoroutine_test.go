// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package testinggoroutine_test

import (
	"testing"

	"golang.org/x/tools/go/analysis/analysistest"
	"golang.org/x/tools/go/analysis/passes/testinggoroutine"
)

func init() {
	testinggoroutine.Analyzer.Flags.Set("subtest", "true")
}

func Test(t *testing.T) {
	testdata := analysistest.TestData()
	pkgs := []string{"a", "typeparams"}
	analysistest.Run(t, testdata, testinggoroutine.Analyzer, pkgs...)
}
