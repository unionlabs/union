// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.22
// +build go1.22

package imports

import (
	"context"
	"testing"
)

// Tests that go.work files and vendor directory are respected.
func TestModWorkspaceVendoring(t *testing.T) {
	mt := setup(t, nil, `
-- go.work --
go 1.22

use (
	./a
	./b
)
-- a/go.mod --
module example.com/a

go 1.22

require rsc.io/sampler v1.3.1
-- a/a.go --
package a

import _ "rsc.io/sampler"
-- b/go.mod --
module example.com/b

go 1.22
-- b/b.go --
package b
`, "")
	defer mt.cleanup()

	// generate vendor directory
	if _, err := mt.env.invokeGo(context.Background(), "work", "vendor"); err != nil {
		t.Fatal(err)
	}

	// update module resolver
	mt.env.ClearModuleInfo()
	mt.env.UpdateResolver(mt.env.resolver.ClearForNewScan())

	mt.assertModuleFoundInDir("example.com/a", "a", `main/a$`)
	mt.assertScanFinds("example.com/a", "a")
	mt.assertModuleFoundInDir("example.com/b", "b", `main/b$`)
	mt.assertScanFinds("example.com/b", "b")
	mt.assertModuleFoundInDir("rsc.io/sampler", "sampler", `/vendor/`)
}
