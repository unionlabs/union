//go:build ignore

// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// The cfg command prints the control-flow graph of the first function
// or method whose name matches 'funcname' in the specified package.
//
// Usage: cfg package funcname
//
// Example:
//
//	$ go build -o cfg ./go/cfg/main.go
//	$ cfg ./go/cfg stmt | dot -Tsvg > cfg.svg && open cfg.svg
package main

import (
	"flag"
	"fmt"
	"go/ast"
	"log"
	"os"

	"golang.org/x/tools/go/cfg"
	"golang.org/x/tools/go/packages"
)

func main() {
	flag.Parse()
	if len(flag.Args()) != 2 {
		log.Fatal("Usage: package funcname")
	}
	pattern, funcname := flag.Args()[0], flag.Args()[1]
	pkgs, err := packages.Load(&packages.Config{Mode: packages.LoadSyntax}, pattern)
	if err != nil {
		log.Fatal(err)
	}
	if packages.PrintErrors(pkgs) > 0 {
		os.Exit(1)
	}
	for _, pkg := range pkgs {
		for _, f := range pkg.Syntax {
			for _, decl := range f.Decls {
				if decl, ok := decl.(*ast.FuncDecl); ok {
					if decl.Name.Name == funcname {
						g := cfg.New(decl.Body, mayReturn)
						fmt.Println(g.Dot(pkg.Fset))
						os.Exit(0)
					}
				}
			}
		}
	}
	log.Fatalf("no function %q found in %s", funcname, pattern)
}

// A trivial mayReturn predicate that looks only at syntax, not types.
func mayReturn(call *ast.CallExpr) bool {
	switch fun := call.Fun.(type) {
	case *ast.Ident:
		return fun.Name != "panic"
	case *ast.SelectorExpr:
		return fun.Sel.Name != "Fatal"
	}
	return true
}
