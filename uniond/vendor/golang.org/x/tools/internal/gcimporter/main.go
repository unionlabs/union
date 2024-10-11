// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build ignore

// The gcimporter command reads the compiler's export data for the
// named packages and prints the decoded type information.
//
// It is provided for debugging export data problems.
package main

import (
	"bytes"
	"flag"
	"fmt"
	"go/token"
	"go/types"
	"log"
	"os"
	"sort"

	"golang.org/x/tools/go/gcexportdata"
	"golang.org/x/tools/go/packages"
	"golang.org/x/tools/go/types/typeutil"
	"golang.org/x/tools/internal/gcimporter"
)

func main() {
	flag.Parse()
	cfg := &packages.Config{
		Fset: token.NewFileSet(),
		// Don't request NeedTypes: we want to be certain that
		// we loaded the types ourselves, from export data.
		Mode: packages.NeedName | packages.NeedExportFile,
	}
	pkgs, err := packages.Load(cfg, flag.Args()...)
	if err != nil {
		log.Fatal(err)
	}
	if packages.PrintErrors(pkgs) > 0 {
		os.Exit(1)
	}

	for _, pkg := range pkgs {
		// Read types from compiler's unified export data file.
		// This Package may included non-exported functions if they
		// are called by inlinable exported functions.
		var tpkg1 *types.Package
		{
			export, err := os.ReadFile(pkg.ExportFile)
			if err != nil {
				log.Fatalf("can't read %q export data: %v", pkg.PkgPath, err)
			}
			r, err := gcexportdata.NewReader(bytes.NewReader(export))
			if err != nil {
				log.Fatalf("reading export data %s: %v", pkg.ExportFile, err)
			}
			tpkg1, err = gcexportdata.Read(r, cfg.Fset, make(map[string]*types.Package), pkg.PkgPath)
			if err != nil {
				log.Fatalf("decoding export data: %v", err)
			}
		}
		fmt.Println("# Read from compiler's unified export data:")
		printPackage(tpkg1)

		// Now reexport as indexed (deep) export data, and reimport.
		// The Package will contain only exported symbols.
		var tpkg2 *types.Package
		{
			var out bytes.Buffer
			if err := gcimporter.IExportData(&out, cfg.Fset, tpkg1); err != nil {
				log.Fatal(err)
			}
			var err error
			_, tpkg2, err = gcimporter.IImportData(cfg.Fset, make(map[string]*types.Package), out.Bytes(), tpkg1.Path())
			if err != nil {
				log.Fatal(err)
			}
		}
		fmt.Println("# After round-tripping through indexed export data:")
		printPackage(tpkg2)
	}
}

func printPackage(pkg *types.Package) {
	fmt.Printf("package %s %q\n", pkg.Name(), pkg.Path())

	if !pkg.Complete() {
		fmt.Printf("\thas incomplete exported type info\n")
	}

	// imports
	var lines []string
	for _, imp := range pkg.Imports() {
		lines = append(lines, fmt.Sprintf("\timport %q", imp.Path()))
	}
	sort.Strings(lines)
	for _, line := range lines {
		fmt.Println(line)
	}

	// types of package members
	qual := types.RelativeTo(pkg)
	scope := pkg.Scope()
	for _, name := range scope.Names() {
		obj := scope.Lookup(name)
		fmt.Printf("\t%s\n", types.ObjectString(obj, qual))
		if _, ok := obj.(*types.TypeName); ok {
			for _, meth := range typeutil.IntuitiveMethodSet(obj.Type(), nil) {
				fmt.Printf("\t%s\n", types.SelectionString(meth, qual))
			}
		}
	}

	fmt.Println()
}
