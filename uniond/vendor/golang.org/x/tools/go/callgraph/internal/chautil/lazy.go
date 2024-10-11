// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package chautil provides helper functions related to
// class hierarchy analysis (CHA) for use in x/tools.
package chautil

import (
	"go/types"

	"golang.org/x/tools/go/ssa"
	"golang.org/x/tools/go/types/typeutil"
)

// LazyCallees returns a function that maps a call site (in a function in fns)
// to its callees within fns. The set of callees is computed using the CHA algorithm,
// i.e., on the entire implements relation between interfaces and concrete types
// in fns. Please see golang.org/x/tools/go/callgraph/cha for more information.
//
// The resulting function is not concurrency safe.
func LazyCallees(fns map[*ssa.Function]bool) func(site ssa.CallInstruction) []*ssa.Function {
	// funcsBySig contains all functions, keyed by signature.  It is
	// the effective set of address-taken functions used to resolve
	// a dynamic call of a particular signature.
	var funcsBySig typeutil.Map // value is []*ssa.Function

	// methodsByID contains all methods, grouped by ID for efficient
	// lookup.
	//
	// We must key by ID, not name, for correct resolution of interface
	// calls to a type with two (unexported) methods spelled the same but
	// from different packages. The fact that the concrete type implements
	// the interface does not mean the call dispatches to both methods.
	methodsByID := make(map[string][]*ssa.Function)

	// An imethod represents an interface method I.m.
	// (There's no go/types object for it;
	// a *types.Func may be shared by many interfaces due to interface embedding.)
	type imethod struct {
		I  *types.Interface
		id string
	}
	// methodsMemo records, for every abstract method call I.m on
	// interface type I, the set of concrete methods C.m of all
	// types C that satisfy interface I.
	//
	// Abstract methods may be shared by several interfaces,
	// hence we must pass I explicitly, not guess from m.
	//
	// methodsMemo is just a cache, so it needn't be a typeutil.Map.
	methodsMemo := make(map[imethod][]*ssa.Function)
	lookupMethods := func(I *types.Interface, m *types.Func) []*ssa.Function {
		id := m.Id()
		methods, ok := methodsMemo[imethod{I, id}]
		if !ok {
			for _, f := range methodsByID[id] {
				C := f.Signature.Recv().Type() // named or *named
				if types.Implements(C, I) {
					methods = append(methods, f)
				}
			}
			methodsMemo[imethod{I, id}] = methods
		}
		return methods
	}

	for f := range fns {
		if f.Signature.Recv() == nil {
			// Package initializers can never be address-taken.
			if f.Name() == "init" && f.Synthetic == "package initializer" {
				continue
			}
			funcs, _ := funcsBySig.At(f.Signature).([]*ssa.Function)
			funcs = append(funcs, f)
			funcsBySig.Set(f.Signature, funcs)
		} else if obj := f.Object(); obj != nil {
			id := obj.(*types.Func).Id()
			methodsByID[id] = append(methodsByID[id], f)
		}
	}

	return func(site ssa.CallInstruction) []*ssa.Function {
		call := site.Common()
		if call.IsInvoke() {
			tiface := call.Value.Type().Underlying().(*types.Interface)
			return lookupMethods(tiface, call.Method)
		} else if g := call.StaticCallee(); g != nil {
			return []*ssa.Function{g}
		} else if _, ok := call.Value.(*ssa.Builtin); !ok {
			fns, _ := funcsBySig.At(call.Signature()).([]*ssa.Function)
			return fns
		}
		return nil
	}
}
