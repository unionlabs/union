// Copyright 2014 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package static computes the call graph of a Go program containing
// only static call edges.
package static

import (
	"go/types"

	"golang.org/x/tools/go/callgraph"
	"golang.org/x/tools/go/ssa"
)

// CallGraph computes the static call graph of the specified program.
//
// The resulting graph includes:
// - all package-level functions;
// - all methods of package-level non-parameterized non-interface types;
// - pointer wrappers (*C).F for source-level methods C.F;
// - and all functions reachable from them following only static calls.
//
// It does not consider exportedness, nor treat main packages specially.
func CallGraph(prog *ssa.Program) *callgraph.Graph {
	cg := callgraph.New(nil)

	// Recursively follow all static calls.
	seen := make(map[int]bool) // node IDs already seen
	var visit func(fnode *callgraph.Node)
	visit = func(fnode *callgraph.Node) {
		if !seen[fnode.ID] {
			seen[fnode.ID] = true

			for _, b := range fnode.Func.Blocks {
				for _, instr := range b.Instrs {
					if site, ok := instr.(ssa.CallInstruction); ok {
						if g := site.Common().StaticCallee(); g != nil {
							gnode := cg.CreateNode(g)
							callgraph.AddEdge(fnode, site, gnode)
							visit(gnode)
						}
					}
				}
			}
		}
	}

	// If we were ever to redesign this function, we should allow
	// the caller to provide the set of root functions and just
	// perform the reachability step. This would allow them to
	// work forwards from main entry points:
	//
	// rootNames := []string{"init", "main"}
	// for _, main := range ssautil.MainPackages(prog.AllPackages()) {
	// 	for _, rootName := range rootNames {
	// 		visit(cg.CreateNode(main.Func(rootName)))
	// 	}
	// }
	//
	// or to control whether to include non-exported
	// functions/methods, wrapper methods, and so on.
	// Unfortunately that's not consistent with its historical
	// behavior and existing tests.
	//
	// The logic below is a slight simplification and
	// rationalization of ssautil.AllFunctions. (Having to include
	// (*T).F wrapper methods is unfortunate--they are not source
	// functions, and if they're reachable, they'll be in the
	// graph--but the existing tests will break without it.)

	methodsOf := func(T types.Type) {
		if !types.IsInterface(T) {
			mset := prog.MethodSets.MethodSet(T)
			for i := 0; i < mset.Len(); i++ {
				visit(cg.CreateNode(prog.MethodValue(mset.At(i))))
			}
		}
	}

	// Start from package-level symbols.
	for _, pkg := range prog.AllPackages() {
		for _, mem := range pkg.Members {
			switch mem := mem.(type) {
			case *ssa.Function:
				// package-level function
				visit(cg.CreateNode(mem))

			case *ssa.Type:
				// methods of package-level non-interface non-parameterized types
				if !types.IsInterface(mem.Type()) {
					if named, ok := mem.Type().(*types.Named); ok &&
						named.TypeParams() == nil {
						methodsOf(named)                   //  T
						methodsOf(types.NewPointer(named)) // *T
					}
				}
			}
		}
	}

	return cg
}
