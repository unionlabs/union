// Copyright 2013 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ssa

// This file defines synthesis of Functions that delegate to declared
// methods; they come in three kinds:
//
// (1) wrappers: methods that wrap declared methods, performing
//     implicit pointer indirections and embedded field selections.
//
// (2) thunks: funcs that wrap declared methods.  Like wrappers,
//     thunks perform indirections and field selections. The thunk's
//     first parameter is used as the receiver for the method call.
//
// (3) bounds: funcs that wrap declared methods.  The bound's sole
//     free variable, supplied by a closure, is used as the receiver
//     for the method call.  No indirections or field selections are
//     performed since they can be done before the call.

import (
	"fmt"

	"go/token"
	"go/types"

	"golang.org/x/tools/internal/typeparams"
)

// -- wrappers -----------------------------------------------------------

// createWrapper returns a synthetic method that delegates to the
// declared method denoted by meth.Obj(), first performing any
// necessary pointer indirections or field selections implied by meth.
//
// The resulting method's receiver type is meth.Recv().
//
// This function is versatile but quite subtle!  Consider the
// following axes of variation when making changes:
//   - optional receiver indirection
//   - optional implicit field selections
//   - meth.Obj() may denote a concrete or an interface method
//   - the result may be a thunk or a wrapper.
func createWrapper(prog *Program, sel *selection) *Function {
	obj := sel.obj.(*types.Func)      // the declared function
	sig := sel.typ.(*types.Signature) // type of this wrapper

	var recv *types.Var // wrapper's receiver or thunk's params[0]
	name := obj.Name()
	var description string
	if sel.kind == types.MethodExpr {
		name += "$thunk"
		description = "thunk"
		recv = sig.Params().At(0)
	} else {
		description = "wrapper"
		recv = sig.Recv()
	}

	description = fmt.Sprintf("%s for %s", description, sel.obj)
	if prog.mode&LogSource != 0 {
		defer logStack("create %s to (%s)", description, recv.Type())()
	}
	/* method wrapper */
	return &Function{
		name:      name,
		method:    sel,
		object:    obj,
		Signature: sig,
		Synthetic: description,
		Prog:      prog,
		pos:       obj.Pos(),
		// wrappers have no syntax
		build:     (*builder).buildWrapper,
		syntax:    nil,
		info:      nil,
		goversion: "",
	}
}

// buildWrapper builds fn.Body for a method wrapper.
func (b *builder) buildWrapper(fn *Function) {
	var recv *types.Var // wrapper's receiver or thunk's params[0]
	var start int       // first regular param
	if fn.method.kind == types.MethodExpr {
		recv = fn.Signature.Params().At(0)
		start = 1
	} else {
		recv = fn.Signature.Recv()
	}

	fn.startBody()
	fn.addSpilledParam(recv)
	createParams(fn, start)

	indices := fn.method.index

	var v Value = fn.Locals[0] // spilled receiver
	if isPointer(fn.method.recv) {
		v = emitLoad(fn, v)

		// For simple indirection wrappers, perform an informative nil-check:
		// "value method (T).f called using nil *T pointer"
		if len(indices) == 1 && !isPointer(recvType(fn.object)) {
			var c Call
			c.Call.Value = &Builtin{
				name: "ssa:wrapnilchk",
				sig: types.NewSignature(nil,
					types.NewTuple(anonVar(fn.method.recv), anonVar(tString), anonVar(tString)),
					types.NewTuple(anonVar(fn.method.recv)), false),
			}
			c.Call.Args = []Value{
				v,
				stringConst(typeparams.MustDeref(fn.method.recv).String()),
				stringConst(fn.method.obj.Name()),
			}
			c.setType(v.Type())
			v = fn.emit(&c)
		}
	}

	// Invariant: v is a pointer, either
	//   value of *A receiver param, or
	// address of  A spilled receiver.

	// We use pointer arithmetic (FieldAddr possibly followed by
	// Load) in preference to value extraction (Field possibly
	// preceded by Load).

	v = emitImplicitSelections(fn, v, indices[:len(indices)-1], token.NoPos)

	// Invariant: v is a pointer, either
	//   value of implicit *C field, or
	// address of implicit  C field.

	var c Call
	if r := recvType(fn.object); !types.IsInterface(r) { // concrete method
		if !isPointer(r) {
			v = emitLoad(fn, v)
		}
		c.Call.Value = fn.Prog.objectMethod(fn.object, b)
		c.Call.Args = append(c.Call.Args, v)
	} else {
		c.Call.Method = fn.object
		c.Call.Value = emitLoad(fn, v) // interface (possibly a typeparam)
	}
	for _, arg := range fn.Params[1:] {
		c.Call.Args = append(c.Call.Args, arg)
	}
	emitTailCall(fn, &c)
	fn.finishBody()
}

// createParams creates parameters for wrapper method fn based on its
// Signature.Params, which do not include the receiver.
// start is the index of the first regular parameter to use.
func createParams(fn *Function, start int) {
	tparams := fn.Signature.Params()
	for i, n := start, tparams.Len(); i < n; i++ {
		fn.addParamVar(tparams.At(i))
	}
}

// -- bounds -----------------------------------------------------------

// createBound returns a bound method wrapper (or "bound"), a synthetic
// function that delegates to a concrete or interface method denoted
// by obj.  The resulting function has no receiver, but has one free
// variable which will be used as the method's receiver in the
// tail-call.
//
// Use MakeClosure with such a wrapper to construct a bound method
// closure.  e.g.:
//
//	type T int          or:  type T interface { meth() }
//	func (t T) meth()
//	var t T
//	f := t.meth
//	f() // calls t.meth()
//
// f is a closure of a synthetic wrapper defined as if by:
//
//	f := func() { return t.meth() }
//
// Unlike createWrapper, createBound need perform no indirection or field
// selections because that can be done before the closure is
// constructed.
func createBound(prog *Program, obj *types.Func) *Function {
	description := fmt.Sprintf("bound method wrapper for %s", obj)
	if prog.mode&LogSource != 0 {
		defer logStack("%s", description)()
	}
	/* bound method wrapper */
	fn := &Function{
		name:      obj.Name() + "$bound",
		object:    obj,
		Signature: changeRecv(obj.Type().(*types.Signature), nil), // drop receiver
		Synthetic: description,
		Prog:      prog,
		pos:       obj.Pos(),
		// wrappers have no syntax
		build:     (*builder).buildBound,
		syntax:    nil,
		info:      nil,
		goversion: "",
	}
	fn.FreeVars = []*FreeVar{{name: "recv", typ: recvType(obj), parent: fn}} // (cyclic)
	return fn
}

// buildBound builds fn.Body for a bound method closure.
func (b *builder) buildBound(fn *Function) {
	fn.startBody()
	createParams(fn, 0)
	var c Call

	recv := fn.FreeVars[0]
	if !types.IsInterface(recvType(fn.object)) { // concrete
		c.Call.Value = fn.Prog.objectMethod(fn.object, b)
		c.Call.Args = []Value{recv}
	} else {
		c.Call.Method = fn.object
		c.Call.Value = recv // interface (possibly a typeparam)
	}
	for _, arg := range fn.Params {
		c.Call.Args = append(c.Call.Args, arg)
	}
	emitTailCall(fn, &c)
	fn.finishBody()
}

// -- thunks -----------------------------------------------------------

// createThunk returns a thunk, a synthetic function that delegates to a
// concrete or interface method denoted by sel.obj.  The resulting
// function has no receiver, but has an additional (first) regular
// parameter.
//
// Precondition: sel.kind == types.MethodExpr.
//
//	type T int          or:  type T interface { meth() }
//	func (t T) meth()
//	f := T.meth
//	var t T
//	f(t) // calls t.meth()
//
// f is a synthetic wrapper defined as if by:
//
//	f := func(t T) { return t.meth() }
func createThunk(prog *Program, sel *selection) *Function {
	if sel.kind != types.MethodExpr {
		panic(sel)
	}

	fn := createWrapper(prog, sel)
	if fn.Signature.Recv() != nil {
		panic(fn) // unexpected receiver
	}

	return fn
}

func changeRecv(s *types.Signature, recv *types.Var) *types.Signature {
	return types.NewSignature(recv, s.Params(), s.Results(), s.Variadic())
}

// A local version of *types.Selection.
// Needed for some additional control, such as creating a MethodExpr for an instantiation.
type selection struct {
	kind     types.SelectionKind
	recv     types.Type
	typ      types.Type
	obj      types.Object
	index    []int
	indirect bool
}

func toSelection(sel *types.Selection) *selection {
	return &selection{
		kind:     sel.Kind(),
		recv:     sel.Recv(),
		typ:      sel.Type(),
		obj:      sel.Obj(),
		index:    sel.Index(),
		indirect: sel.Indirect(),
	}
}

// -- instantiations --------------------------------------------------

// buildInstantiationWrapper builds the body of an instantiation
// wrapper fn. The body calls the original generic function,
// bracketed by ChangeType conversions on its arguments and results.
func (b *builder) buildInstantiationWrapper(fn *Function) {
	orig := fn.topLevelOrigin
	sig := fn.Signature

	fn.startBody()
	if sig.Recv() != nil {
		fn.addParamVar(sig.Recv())
	}
	createParams(fn, 0)

	// Create body. Add a call to origin generic function
	// and make type changes between argument and parameters,
	// as well as return values.
	var c Call
	c.Call.Value = orig
	if res := orig.Signature.Results(); res.Len() == 1 {
		c.typ = res.At(0).Type()
	} else {
		c.typ = res
	}

	// parameter of instance becomes an argument to the call
	// to the original generic function.
	argOffset := 0
	for i, arg := range fn.Params {
		var typ types.Type
		if i == 0 && sig.Recv() != nil {
			typ = orig.Signature.Recv().Type()
			argOffset = 1
		} else {
			typ = orig.Signature.Params().At(i - argOffset).Type()
		}
		c.Call.Args = append(c.Call.Args, emitTypeCoercion(fn, arg, typ))
	}

	results := fn.emit(&c)
	var ret Return
	switch res := sig.Results(); res.Len() {
	case 0:
		// no results, do nothing.
	case 1:
		ret.Results = []Value{emitTypeCoercion(fn, results, res.At(0).Type())}
	default:
		for i := 0; i < sig.Results().Len(); i++ {
			v := emitExtract(fn, results, i)
			ret.Results = append(ret.Results, emitTypeCoercion(fn, v, res.At(i).Type()))
		}
	}

	fn.emit(&ret)
	fn.currentBlock = nil

	fn.finishBody()
}
