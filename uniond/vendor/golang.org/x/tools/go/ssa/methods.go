// Copyright 2013 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ssa

// This file defines utilities for population of method sets.

import (
	"fmt"
	"go/types"

	"golang.org/x/tools/go/types/typeutil"
	"golang.org/x/tools/internal/aliases"
)

// MethodValue returns the Function implementing method sel, building
// wrapper methods on demand. It returns nil if sel denotes an
// interface or generic method.
//
// Precondition: sel.Kind() == MethodVal.
//
// Thread-safe.
//
// Acquires prog.methodsMu.
func (prog *Program) MethodValue(sel *types.Selection) *Function {
	if sel.Kind() != types.MethodVal {
		panic(fmt.Sprintf("MethodValue(%s) kind != MethodVal", sel))
	}
	T := sel.Recv()
	if types.IsInterface(T) {
		return nil // interface method or type parameter
	}

	if prog.isParameterized(T) {
		return nil // generic method
	}

	if prog.mode&LogSource != 0 {
		defer logStack("MethodValue %s %v", T, sel)()
	}

	var b builder

	m := func() *Function {
		prog.methodsMu.Lock()
		defer prog.methodsMu.Unlock()

		// Get or create SSA method set.
		mset, ok := prog.methodSets.At(T).(*methodSet)
		if !ok {
			mset = &methodSet{mapping: make(map[string]*Function)}
			prog.methodSets.Set(T, mset)
		}

		// Get or create SSA method.
		id := sel.Obj().Id()
		fn, ok := mset.mapping[id]
		if !ok {
			obj := sel.Obj().(*types.Func)
			needsPromotion := len(sel.Index()) > 1
			needsIndirection := !isPointer(recvType(obj)) && isPointer(T)
			if needsPromotion || needsIndirection {
				fn = createWrapper(prog, toSelection(sel))
				fn.buildshared = b.shared()
				b.enqueue(fn)
			} else {
				fn = prog.objectMethod(obj, &b)
			}
			if fn.Signature.Recv() == nil {
				panic(fn)
			}
			mset.mapping[id] = fn
		} else {
			b.waitForSharedFunction(fn)
		}

		return fn
	}()

	b.iterate()

	return m
}

// objectMethod returns the Function for a given method symbol.
// The symbol may be an instance of a generic function. It need not
// belong to an existing SSA package created by a call to
// prog.CreatePackage.
//
// objectMethod panics if the function is not a method.
//
// Acquires prog.objectMethodsMu.
func (prog *Program) objectMethod(obj *types.Func, b *builder) *Function {
	sig := obj.Type().(*types.Signature)
	if sig.Recv() == nil {
		panic("not a method: " + obj.String())
	}

	// Belongs to a created package?
	if fn := prog.FuncValue(obj); fn != nil {
		return fn
	}

	// Instantiation of generic?
	if originObj := obj.Origin(); originObj != obj {
		origin := prog.objectMethod(originObj, b)
		assert(origin.typeparams.Len() > 0, "origin is not generic")
		targs := receiverTypeArgs(obj)
		return origin.instance(targs, b)
	}

	// Consult/update cache of methods created from types.Func.
	prog.objectMethodsMu.Lock()
	defer prog.objectMethodsMu.Unlock()
	fn, ok := prog.objectMethods[obj]
	if !ok {
		fn = createFunction(prog, obj, obj.Name(), nil, nil, "")
		fn.Synthetic = "from type information (on demand)"
		fn.buildshared = b.shared()
		b.enqueue(fn)

		if prog.objectMethods == nil {
			prog.objectMethods = make(map[*types.Func]*Function)
		}
		prog.objectMethods[obj] = fn
	} else {
		b.waitForSharedFunction(fn)
	}
	return fn
}

// LookupMethod returns the implementation of the method of type T
// identified by (pkg, name).  It returns nil if the method exists but
// is an interface method or generic method, and panics if T has no such method.
func (prog *Program) LookupMethod(T types.Type, pkg *types.Package, name string) *Function {
	sel := prog.MethodSets.MethodSet(T).Lookup(pkg, name)
	if sel == nil {
		panic(fmt.Sprintf("%s has no method %s", T, types.Id(pkg, name)))
	}
	return prog.MethodValue(sel)
}

// methodSet contains the (concrete) methods of a concrete type (non-interface, non-parameterized).
type methodSet struct {
	mapping map[string]*Function // populated lazily
}

// RuntimeTypes returns a new unordered slice containing all types in
// the program for which a runtime type is required.
//
// A runtime type is required for any non-parameterized, non-interface
// type that is converted to an interface, or for any type (including
// interface types) derivable from one through reflection.
//
// The methods of such types may be reachable through reflection or
// interface calls even if they are never called directly.
//
// Thread-safe.
//
// Acquires prog.runtimeTypesMu.
func (prog *Program) RuntimeTypes() []types.Type {
	prog.runtimeTypesMu.Lock()
	defer prog.runtimeTypesMu.Unlock()
	return prog.runtimeTypes.Keys()
}

// forEachReachable calls f for type T and each type reachable from
// its type through reflection.
//
// The function f must use memoization to break cycles and
// return false when the type has already been visited.
//
// TODO(adonovan): publish in typeutil and share with go/callgraph/rta.
func forEachReachable(msets *typeutil.MethodSetCache, T types.Type, f func(types.Type) bool) {
	var visit func(T types.Type, skip bool)
	visit = func(T types.Type, skip bool) {
		if !skip {
			if !f(T) {
				return
			}
		}

		// Recursion over signatures of each method.
		tmset := msets.MethodSet(T)
		for i := 0; i < tmset.Len(); i++ {
			sig := tmset.At(i).Type().(*types.Signature)
			// It is tempting to call visit(sig, false)
			// but, as noted in golang.org/cl/65450043,
			// the Signature.Recv field is ignored by
			// types.Identical and typeutil.Map, which
			// is confusing at best.
			//
			// More importantly, the true signature rtype
			// reachable from a method using reflection
			// has no receiver but an extra ordinary parameter.
			// For the Read method of io.Reader we want:
			//   func(Reader, []byte) (int, error)
			// but here sig is:
			//   func([]byte) (int, error)
			// with .Recv = Reader (though it is hard to
			// notice because it doesn't affect Signature.String
			// or types.Identical).
			//
			// TODO(adonovan): construct and visit the correct
			// non-method signature with an extra parameter
			// (though since unnamed func types have no methods
			// there is essentially no actual demand for this).
			//
			// TODO(adonovan): document whether or not it is
			// safe to skip non-exported methods (as RTA does).
			visit(sig.Params(), true)  // skip the Tuple
			visit(sig.Results(), true) // skip the Tuple
		}

		switch T := T.(type) {
		case *aliases.Alias:
			visit(aliases.Unalias(T), skip) // emulates the pre-Alias behavior

		case *types.Basic:
			// nop

		case *types.Interface:
			// nop---handled by recursion over method set.

		case *types.Pointer:
			visit(T.Elem(), false)

		case *types.Slice:
			visit(T.Elem(), false)

		case *types.Chan:
			visit(T.Elem(), false)

		case *types.Map:
			visit(T.Key(), false)
			visit(T.Elem(), false)

		case *types.Signature:
			if T.Recv() != nil {
				panic(fmt.Sprintf("Signature %s has Recv %s", T, T.Recv()))
			}
			visit(T.Params(), true)  // skip the Tuple
			visit(T.Results(), true) // skip the Tuple

		case *types.Named:
			// A pointer-to-named type can be derived from a named
			// type via reflection.  It may have methods too.
			visit(types.NewPointer(T), false)

			// Consider 'type T struct{S}' where S has methods.
			// Reflection provides no way to get from T to struct{S},
			// only to S, so the method set of struct{S} is unwanted,
			// so set 'skip' flag during recursion.
			visit(T.Underlying(), true) // skip the unnamed type

		case *types.Array:
			visit(T.Elem(), false)

		case *types.Struct:
			for i, n := 0, T.NumFields(); i < n; i++ {
				// TODO(adonovan): document whether or not
				// it is safe to skip non-exported fields.
				visit(T.Field(i).Type(), false)
			}

		case *types.Tuple:
			for i, n := 0, T.Len(); i < n; i++ {
				visit(T.At(i).Type(), false)
			}

		case *types.TypeParam, *types.Union:
			// forEachReachable must not be called on parameterized types.
			panic(T)

		default:
			panic(T)
		}
	}
	visit(T, false)
}
