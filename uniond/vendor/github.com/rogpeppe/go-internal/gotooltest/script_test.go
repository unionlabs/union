// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package gotooltest_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/rogpeppe/go-internal/gotooltest"
	"github.com/rogpeppe/go-internal/testscript"
)

func TestSimple(t *testing.T) {
	p := testscript.Params{
		Dir: "testdata",
		Setup: func(env *testscript.Env) error {
			// cover.txt will need testscript as a dependency.
			// Tell it where our module is, via an absolute path.
			wd, err := os.Getwd()
			if err != nil {
				return err
			}
			modPath := filepath.Dir(wd)
			env.Setenv("GOINTERNAL_MODULE", modPath)
			return nil
		},
	}

	if err := gotooltest.Setup(&p); err != nil {
		t.Fatal(err)
	}
	testscript.Run(t, p)
}
