// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// The drivertest package provides a fake implementation of the go/packages
// driver protocol that delegates to the go list driver. It may be used to test
// programs such as gopls that specialize behavior when a go/packages driver is
// in use.
//
// The driver is run as a child of the current process, by calling [RunIfChild]
// at process start, and running go/packages with the environment variables set
// by [Env].
package drivertest

import (
	"encoding/json"
	"flag"
	"log"
	"os"

	"golang.org/x/tools/go/packages"
)

const runAsDriverEnv = "DRIVERTEST_RUN_AS_DRIVER"

// RunIfChild runs the current process as a go/packages driver, if configured
// to do so by the current environment (see [Env]).
//
// Otherwise, RunIfChild is a no op.
func RunIfChild() {
	if os.Getenv(runAsDriverEnv) != "" {
		main()
		os.Exit(0)
	}
}

// Env returns additional environment variables for use in [packages.Config]
// to enable the use of drivertest as the driver.
//
// t abstracts a *testing.T or log.Default().
func Env(t interface{ Fatal(...any) }) []string {
	exe, err := os.Executable()
	if err != nil {
		t.Fatal(err)
	}
	return []string{"GOPACKAGESDRIVER=" + exe, runAsDriverEnv + "=1"}
}

func main() {
	flag.Parse()

	dec := json.NewDecoder(os.Stdin)
	var request packages.DriverRequest
	if err := dec.Decode(&request); err != nil {
		log.Fatalf("decoding request: %v", err)
	}

	config := packages.Config{
		Mode:       request.Mode,
		Env:        append(request.Env, "GOPACKAGESDRIVER=off"), // avoid recursive invocation
		BuildFlags: request.BuildFlags,
		Tests:      request.Tests,
		Overlay:    request.Overlay,
	}
	pkgs, err := packages.Load(&config, flag.Args()...)
	if err != nil {
		log.Fatalf("load failed: %v", err)
	}

	var roots []string
	for _, pkg := range pkgs {
		roots = append(roots, pkg.ID)
	}
	var allPackages []*packages.Package
	packages.Visit(pkgs, nil, func(pkg *packages.Package) {
		newImports := make(map[string]*packages.Package)
		for path, imp := range pkg.Imports {
			newImports[path] = &packages.Package{ID: imp.ID}
		}
		pkg.Imports = newImports
		allPackages = append(allPackages, pkg)
	})

	enc := json.NewEncoder(os.Stdout)
	response := packages.DriverResponse{
		Roots:    roots,
		Packages: allPackages,
	}
	if err := enc.Encode(response); err != nil {
		log.Fatalf("encoding response: %v", err)
	}
}
