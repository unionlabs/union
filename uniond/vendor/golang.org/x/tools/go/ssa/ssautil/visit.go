// Copyright 2013 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ssautil // import "golang.org/x/tools/go/ssa/ssautil"

import (
	"go/ast"
	"go/types"

	"golang.org/x/tools/go/ssa"

	_ "unsafe" // for linkname hack
)

// This file defines utilities for visiting the SSA representation of
// a Program.
//
// TODO(adonovan): test coverage.

// AllFunctions finds and returns the set of functions potentially
// needed by program prog, as determined by a simple linker-style
// reachability algorithm starting from the members and method-sets of
// each package.  The result may include anonymous functions and
// synthetic wrappers.
//
// Precondition: all packages are built.
//
// TODO(adonovan): this function is underspecified. It doesn't
// actually work like a linker, which computes reachability from main
// using something like go/callgraph/cha (without materializing the
// call graph). In fact, it treats all public functions and all
// methods of public non-parameterized types as roots, even though
// they may be unreachable--but only in packages created from syntax.
//
// I think we should deprecate AllFunctions function in favor of two
// clearly defined ones:
//
//  1. The first would efficiently compute CHA reachability from a set
//     of main packages, making it suitable for a whole-program
//     analysis context with InstantiateGenerics, in conjunction with
//     Program.Build.
//
//  2. The second would return only the set of functions corresponding
//     to source Func{Decl,Lit} syntax, like SrcFunctions in
//     go/analysis/passes/buildssa; this is suitable for
//     package-at-a-time (or handful of packages) context.
//     ssa.Package could easily expose it as a field.
//
// We could add them unexported for now and use them via the linkname hack.
func AllFunctions(prog *ssa.Program) map[*ssa.Function]bool {
	seen := make(map[*ssa.Function]bool)

	var function func(fn *ssa.Function)
	function = func(fn *ssa.Function) {
		if !seen[fn] {
			seen[fn] = true
			var buf [10]*ssa.Value // avoid alloc in common case
			for _, b := range fn.Blocks {
				for _, instr := range b.Instrs {
					for _, op := range instr.Operands(buf[:0]) {
						if fn, ok := (*op).(*ssa.Function); ok {
							function(fn)
						}
					}
				}
			}
		}
	}

	// TODO(adonovan): opt: provide a way to share a builder
	// across a sequence of MethodValue calls.

	methodsOf := func(T types.Type) {
		if !types.IsInterface(T) {
			mset := prog.MethodSets.MethodSet(T)
			for i := 0; i < mset.Len(); i++ {
				function(prog.MethodValue(mset.At(i)))
			}
		}
	}

	// Historically, Program.RuntimeTypes used to include the type
	// of any exported member of a package loaded from syntax that
	// has a non-parameterized type, plus all types
	// reachable from that type using reflection, even though
	// these runtime types may not be required for them.
	//
	// Rather than break existing programs that rely on
	// AllFunctions visiting extra methods that are unreferenced
	// by IR and unreachable via reflection, we moved the logic
	// here, unprincipled though it is.
	// (See doc comment for better ideas.)
	//
	// Nonetheless, after the move, we no longer visit every
	// method of any type recursively reachable from T, only the
	// methods of T and *T themselves, and we only apply this to
	// named types T, and not to the type of every exported
	// package member.
	exportedTypeHack := func(t *ssa.Type) {
		if isSyntactic(t.Package()) &&
			ast.IsExported(t.Name()) &&
			!types.IsInterface(t.Type()) {
			// Consider only named types.
			// (Ignore aliases and unsafe.Pointer.)
			if named, ok := t.Type().(*types.Named); ok {
				if named.TypeParams() == nil {
					methodsOf(named)                   //  T
					methodsOf(types.NewPointer(named)) // *T
				}
			}
		}
	}

	for _, pkg := range prog.AllPackages() {
		for _, mem := range pkg.Members {
			switch mem := mem.(type) {
			case *ssa.Function:
				// Visit all package-level declared functions.
				function(mem)

			case *ssa.Type:
				exportedTypeHack(mem)
			}
		}
	}

	// Visit all methods of types for which runtime types were
	// materialized, as they are reachable through reflection.
	for _, T := range prog.RuntimeTypes() {
		methodsOf(T)
	}

	return seen
}

// MainPackages returns the subset of the specified packages
// named "main" that define a main function.
// The result may include synthetic "testmain" packages.
func MainPackages(pkgs []*ssa.Package) []*ssa.Package {
	var mains []*ssa.Package
	for _, pkg := range pkgs {
		if pkg.Pkg.Name() == "main" && pkg.Func("main") != nil {
			mains = append(mains, pkg)
		}
	}
	return mains
}

// TODO(adonovan): propose a principled API for this. One possibility
// is a new field, Package.SrcFunctions []*Function, which would
// contain the list of SrcFunctions described in point 2 of the
// AllFunctions doc comment, or nil if the package is not from syntax.
// But perhaps overloading nil vs empty slice is too subtle.
//
//go:linkname isSyntactic golang.org/x/tools/go/ssa.isSyntactic
func isSyntactic(pkg *ssa.Package) bool
