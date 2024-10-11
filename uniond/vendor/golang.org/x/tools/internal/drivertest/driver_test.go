// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package drivertest_test

// This file is both a test of drivertest and an example of how to use it in your own tests.

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"

	"golang.org/x/tools/go/packages"
	"golang.org/x/tools/internal/diff"
	"golang.org/x/tools/internal/diff/myers"
	"golang.org/x/tools/internal/drivertest"
	"golang.org/x/tools/internal/packagesinternal"
	"golang.org/x/tools/internal/testenv"
	"golang.org/x/tools/internal/testfiles"
	"golang.org/x/tools/txtar"
)

func TestMain(m *testing.M) {
	drivertest.RunIfChild()

	os.Exit(m.Run())
}

func TestDriverConformance(t *testing.T) {
	testenv.NeedsExec(t)

	const workspace = `
-- go.mod --
module example.com/m

go 1.20

-- m.go --
package m

-- lib/lib.go --
package lib
`

	fs, err := txtar.FS(txtar.Parse([]byte(workspace)))
	if err != nil {
		t.Fatal(err)
	}
	dir := testfiles.CopyToTmp(t, fs)

	// TODO(rfindley): on mac, this is required to fix symlink path mismatches.
	// But why? Where is the symlink being evaluated in go/packages?
	dir, err = filepath.EvalSymlinks(dir)
	if err != nil {
		t.Fatal(err)
	}

	baseConfig := packages.Config{
		Dir: dir,
		Mode: packages.NeedName |
			packages.NeedFiles |
			packages.NeedCompiledGoFiles |
			packages.NeedImports |
			packages.NeedDeps |
			packages.NeedTypesSizes |
			packages.NeedModule |
			packages.NeedEmbedFiles |
			packages.LoadMode(packagesinternal.DepsErrors) |
			packages.LoadMode(packagesinternal.ForTest),
	}

	tests := []struct {
		name    string
		query   string
		overlay string
	}{
		{
			name:  "load all",
			query: "./...",
		},
		{
			name:  "overlays",
			query: "./...",
			overlay: `
-- m.go --
package m

import . "lib"
-- a/a.go --
package a
`,
		},
		{
			name:  "std",
			query: "std",
		},
		{
			name:  "builtin",
			query: "builtin",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			cfg := baseConfig
			if test.overlay != "" {
				cfg.Overlay = make(map[string][]byte)
				for _, file := range txtar.Parse([]byte(test.overlay)).Files {
					name := filepath.Join(dir, filepath.FromSlash(file.Name))
					cfg.Overlay[name] = file.Data
				}
			}

			// Compare JSON-encoded packages with and without GOPACKAGESDRIVER.
			//
			// Note that this does not guarantee that the go/packages results
			// themselves are equivalent, only that their encoded JSON is equivalent.
			// Certain fields such as Module are intentionally omitted from external
			// drivers, because they don't make sense for an arbitrary build system.
			var jsons []string
			for _, env := range [][]string{
				{"GOPACKAGESDRIVER=off"},
				drivertest.Env(t),
			} {
				cfg.Env = append(os.Environ(), env...)
				pkgs, err := packages.Load(&cfg, test.query)
				if err != nil {
					t.Fatalf("failed to load (env: %v): %v", env, err)
				}
				data, err := json.MarshalIndent(pkgs, "", "\t")
				if err != nil {
					t.Fatalf("failed to marshal (env: %v): %v", env, err)
				}
				jsons = append(jsons, string(data))
			}

			listJSON := jsons[0]
			driverJSON := jsons[1]

			// Use the myers package for better line diffs.
			edits := myers.ComputeEdits(listJSON, driverJSON)
			d, err := diff.ToUnified("go list", "driver", listJSON, edits, 0)
			if err != nil {
				t.Fatal(err)
			}
			if d != "" {
				t.Errorf("mismatching JSON:\n%s", d)
			}
		})
	}
}
