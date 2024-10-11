// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package buildssa defines an Analyzer that constructs the SSA
// representation of an error-free package and returns the set of all
// functions within it. It does not report any diagnostics itself but
// may be used as an input to other analyzers.
package buildssa

import (
	"go/ast"
	"go/types"
	"reflect"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/ssa"
)

var Analyzer = &analysis.Analyzer{
	Name:       "buildssa",
	Doc:        "build SSA-form IR for later passes",
	URL:        "https://pkg.go.dev/golang.org/x/tools/go/analysis/passes/buildssa",
	Run:        run,
	ResultType: reflect.TypeOf(new(SSA)),
}

// SSA provides SSA-form intermediate representation for all the
// source functions in the current package.
type SSA struct {
	Pkg      *ssa.Package
	SrcFuncs []*ssa.Function
}

func run(pass *analysis.Pass) (interface{}, error) {
	// We must create a new Program for each Package because the
	// analysis API provides no place to hang a Program shared by
	// all Packages. Consequently, SSA Packages and Functions do not
	// have a canonical representation across an analysis session of
	// multiple packages. This is unlikely to be a problem in
	// practice because the analysis API essentially forces all
	// packages to be analysed independently, so any given call to
	// Analysis.Run on a package will see only SSA objects belonging
	// to a single Program.

	// Some Analyzers may need GlobalDebug, in which case we'll have
	// to set it globally, but let's wait till we need it.
	mode := ssa.BuilderMode(0)

	prog := ssa.NewProgram(pass.Fset, mode)

	// Create SSA packages for direct imports.
	for _, p := range pass.Pkg.Imports() {
		prog.CreatePackage(p, nil, nil, true)
	}

	// Create and build the primary package.
	ssapkg := prog.CreatePackage(pass.Pkg, pass.Files, pass.TypesInfo, false)
	ssapkg.Build()

	// Compute list of source functions, including literals,
	// in source order.
	var funcs []*ssa.Function
	for _, f := range pass.Files {
		for _, decl := range f.Decls {
			if fdecl, ok := decl.(*ast.FuncDecl); ok {
				// (init functions have distinct Func
				// objects named "init" and distinct
				// ssa.Functions named "init#1", ...)

				fn := pass.TypesInfo.Defs[fdecl.Name].(*types.Func)
				if fn == nil {
					panic(fn)
				}

				f := ssapkg.Prog.FuncValue(fn)
				if f == nil {
					panic(fn)
				}

				var addAnons func(f *ssa.Function)
				addAnons = func(f *ssa.Function) {
					funcs = append(funcs, f)
					for _, anon := range f.AnonFuncs {
						addAnons(anon)
					}
				}
				addAnons(f)
			}
		}
	}

	return &SSA{Pkg: ssapkg, SrcFuncs: funcs}, nil
}
