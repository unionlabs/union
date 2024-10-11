// Copyright 2014 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package cha computes the call graph of a Go program using the Class
// Hierarchy Analysis (CHA) algorithm.
//
// CHA was first described in "Optimization of Object-Oriented Programs
// Using Static Class Hierarchy Analysis", Jeffrey Dean, David Grove,
// and Craig Chambers, ECOOP'95.
//
// CHA is related to RTA (see go/callgraph/rta); the difference is that
// CHA conservatively computes the entire "implements" relation between
// interfaces and concrete types ahead of time, whereas RTA uses dynamic
// programming to construct it on the fly as it encounters new functions
// reachable from main.  CHA may thus include spurious call edges for
// types that haven't been instantiated yet, or types that are never
// instantiated.
//
// Since CHA conservatively assumes that all functions are address-taken
// and all concrete types are put into interfaces, it is sound to run on
// partial programs, such as libraries without a main or test function.
package cha // import "golang.org/x/tools/go/callgraph/cha"

// TODO(zpavlinovic): update CHA for how it handles generic function bodies.

import (
	"golang.org/x/tools/go/callgraph"
	"golang.org/x/tools/go/callgraph/internal/chautil"
	"golang.org/x/tools/go/ssa"
	"golang.org/x/tools/go/ssa/ssautil"
)

// CallGraph computes the call graph of the specified program using the
// Class Hierarchy Analysis algorithm.
func CallGraph(prog *ssa.Program) *callgraph.Graph {
	cg := callgraph.New(nil) // TODO(adonovan) eliminate concept of rooted callgraph

	allFuncs := ssautil.AllFunctions(prog)

	calleesOf := lazyCallees(allFuncs)

	addEdge := func(fnode *callgraph.Node, site ssa.CallInstruction, g *ssa.Function) {
		gnode := cg.CreateNode(g)
		callgraph.AddEdge(fnode, site, gnode)
	}

	addEdges := func(fnode *callgraph.Node, site ssa.CallInstruction, callees []*ssa.Function) {
		// Because every call to a highly polymorphic and
		// frequently used abstract method such as
		// (io.Writer).Write is assumed to call every concrete
		// Write method in the program, the call graph can
		// contain a lot of duplication.
		for _, g := range callees {
			addEdge(fnode, site, g)
		}
	}

	for f := range allFuncs {
		fnode := cg.CreateNode(f)
		for _, b := range f.Blocks {
			for _, instr := range b.Instrs {
				if site, ok := instr.(ssa.CallInstruction); ok {
					if g := site.Common().StaticCallee(); g != nil {
						addEdge(fnode, site, g)
					} else {
						addEdges(fnode, site, calleesOf(site))
					}
				}
			}
		}
	}

	return cg
}

var lazyCallees = chautil.LazyCallees
