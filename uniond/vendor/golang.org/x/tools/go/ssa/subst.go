// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ssa

import (
	"go/types"

	"golang.org/x/tools/go/types/typeutil"
	"golang.org/x/tools/internal/aliases"
)

// subster defines a type substitution operation of a set of type parameters
// to type parameter free replacement types. Substitution is done within
// the context of a package-level function instantiation. *Named types
// declared in the function are unique to the instantiation.
//
// For example, given a parameterized function F
//
//	  func F[S, T any]() any {
//	    type X struct{ s S; next *X }
//		var p *X
//	    return p
//	  }
//
// calling the instantiation F[string, int]() returns an interface
// value (*X[string,int], nil) where the underlying value of
// X[string,int] is a struct{s string; next *X[string,int]}.
//
// A nil *subster is a valid, empty substitution map. It always acts as
// the identity function. This allows for treating parameterized and
// non-parameterized functions identically while compiling to ssa.
//
// Not concurrency-safe.
//
// Note: Some may find it helpful to think through some of the most
// complex substitution cases using lambda calculus inspired notation.
// subst.typ() solves evaluating a type expression E
// within the body of a function Fn[m] with the type parameters m
// once we have applied the type arguments N.
// We can succinctly write this as a function application:
//
//	((λm. E) N)
//
// go/types does not provide this interface directly.
// So what subster provides is a type substitution operation
//
//	E[m:=N]
type subster struct {
	replacements map[*types.TypeParam]types.Type // values should contain no type params
	cache        map[types.Type]types.Type       // cache of subst results
	origin       *types.Func                     // types.Objects declared within this origin function are unique within this context
	ctxt         *types.Context                  // speeds up repeated instantiations
	uniqueness   typeutil.Map                    // determines the uniqueness of the instantiations within the function
	// TODO(taking): consider adding Pos
}

// Returns a subster that replaces tparams[i] with targs[i]. Uses ctxt as a cache.
// targs should not contain any types in tparams.
// fn is the generic function for which we are substituting.
func makeSubster(ctxt *types.Context, fn *types.Func, tparams *types.TypeParamList, targs []types.Type, debug bool) *subster {
	assert(tparams.Len() == len(targs), "makeSubster argument count must match")

	subst := &subster{
		replacements: make(map[*types.TypeParam]types.Type, tparams.Len()),
		cache:        make(map[types.Type]types.Type),
		origin:       fn.Origin(),
		ctxt:         ctxt,
	}
	for i := 0; i < tparams.Len(); i++ {
		subst.replacements[tparams.At(i)] = targs[i]
	}
	return subst
}

// typ returns the type of t with the type parameter tparams[i] substituted
// for the type targs[i] where subst was created using tparams and targs.
func (subst *subster) typ(t types.Type) (res types.Type) {
	if subst == nil {
		return t // A nil subst is type preserving.
	}
	if r, ok := subst.cache[t]; ok {
		return r
	}
	defer func() {
		subst.cache[t] = res
	}()

	switch t := t.(type) {
	case *types.TypeParam:
		if r := subst.replacements[t]; r != nil {
			return r
		}
		return t

	case *types.Basic:
		return t

	case *types.Array:
		if r := subst.typ(t.Elem()); r != t.Elem() {
			return types.NewArray(r, t.Len())
		}
		return t

	case *types.Slice:
		if r := subst.typ(t.Elem()); r != t.Elem() {
			return types.NewSlice(r)
		}
		return t

	case *types.Pointer:
		if r := subst.typ(t.Elem()); r != t.Elem() {
			return types.NewPointer(r)
		}
		return t

	case *types.Tuple:
		return subst.tuple(t)

	case *types.Struct:
		return subst.struct_(t)

	case *types.Map:
		key := subst.typ(t.Key())
		elem := subst.typ(t.Elem())
		if key != t.Key() || elem != t.Elem() {
			return types.NewMap(key, elem)
		}
		return t

	case *types.Chan:
		if elem := subst.typ(t.Elem()); elem != t.Elem() {
			return types.NewChan(t.Dir(), elem)
		}
		return t

	case *types.Signature:
		return subst.signature(t)

	case *types.Union:
		return subst.union(t)

	case *types.Interface:
		return subst.interface_(t)

	case *aliases.Alias:
		return subst.alias(t)

	case *types.Named:
		return subst.named(t)

	case *opaqueType:
		return t // opaque types are never substituted

	default:
		panic("unreachable")
	}
}

// types returns the result of {subst.typ(ts[i])}.
func (subst *subster) types(ts []types.Type) []types.Type {
	res := make([]types.Type, len(ts))
	for i := range ts {
		res[i] = subst.typ(ts[i])
	}
	return res
}

func (subst *subster) tuple(t *types.Tuple) *types.Tuple {
	if t != nil {
		if vars := subst.varlist(t); vars != nil {
			return types.NewTuple(vars...)
		}
	}
	return t
}

type varlist interface {
	At(i int) *types.Var
	Len() int
}

// fieldlist is an adapter for structs for the varlist interface.
type fieldlist struct {
	str *types.Struct
}

func (fl fieldlist) At(i int) *types.Var { return fl.str.Field(i) }
func (fl fieldlist) Len() int            { return fl.str.NumFields() }

func (subst *subster) struct_(t *types.Struct) *types.Struct {
	if t != nil {
		if fields := subst.varlist(fieldlist{t}); fields != nil {
			tags := make([]string, t.NumFields())
			for i, n := 0, t.NumFields(); i < n; i++ {
				tags[i] = t.Tag(i)
			}
			return types.NewStruct(fields, tags)
		}
	}
	return t
}

// varlist returns subst(in[i]) or return nils if subst(v[i]) == v[i] for all i.
func (subst *subster) varlist(in varlist) []*types.Var {
	var out []*types.Var // nil => no updates
	for i, n := 0, in.Len(); i < n; i++ {
		v := in.At(i)
		w := subst.var_(v)
		if v != w && out == nil {
			out = make([]*types.Var, n)
			for j := 0; j < i; j++ {
				out[j] = in.At(j)
			}
		}
		if out != nil {
			out[i] = w
		}
	}
	return out
}

func (subst *subster) var_(v *types.Var) *types.Var {
	if v != nil {
		if typ := subst.typ(v.Type()); typ != v.Type() {
			if v.IsField() {
				return types.NewField(v.Pos(), v.Pkg(), v.Name(), typ, v.Embedded())
			}
			return types.NewVar(v.Pos(), v.Pkg(), v.Name(), typ)
		}
	}
	return v
}

func (subst *subster) union(u *types.Union) *types.Union {
	var out []*types.Term // nil => no updates

	for i, n := 0, u.Len(); i < n; i++ {
		t := u.Term(i)
		r := subst.typ(t.Type())
		if r != t.Type() && out == nil {
			out = make([]*types.Term, n)
			for j := 0; j < i; j++ {
				out[j] = u.Term(j)
			}
		}
		if out != nil {
			out[i] = types.NewTerm(t.Tilde(), r)
		}
	}

	if out != nil {
		return types.NewUnion(out)
	}
	return u
}

func (subst *subster) interface_(iface *types.Interface) *types.Interface {
	if iface == nil {
		return nil
	}

	// methods for the interface. Initially nil if there is no known change needed.
	// Signatures for the method where recv is nil. NewInterfaceType fills in the receivers.
	var methods []*types.Func
	initMethods := func(n int) { // copy first n explicit methods
		methods = make([]*types.Func, iface.NumExplicitMethods())
		for i := 0; i < n; i++ {
			f := iface.ExplicitMethod(i)
			norecv := changeRecv(f.Type().(*types.Signature), nil)
			methods[i] = types.NewFunc(f.Pos(), f.Pkg(), f.Name(), norecv)
		}
	}
	for i := 0; i < iface.NumExplicitMethods(); i++ {
		f := iface.ExplicitMethod(i)
		// On interfaces, we need to cycle break on anonymous interface types
		// being in a cycle with their signatures being in cycles with their receivers
		// that do not go through a Named.
		norecv := changeRecv(f.Type().(*types.Signature), nil)
		sig := subst.typ(norecv)
		if sig != norecv && methods == nil {
			initMethods(i)
		}
		if methods != nil {
			methods[i] = types.NewFunc(f.Pos(), f.Pkg(), f.Name(), sig.(*types.Signature))
		}
	}

	var embeds []types.Type
	initEmbeds := func(n int) { // copy first n embedded types
		embeds = make([]types.Type, iface.NumEmbeddeds())
		for i := 0; i < n; i++ {
			embeds[i] = iface.EmbeddedType(i)
		}
	}
	for i := 0; i < iface.NumEmbeddeds(); i++ {
		e := iface.EmbeddedType(i)
		r := subst.typ(e)
		if e != r && embeds == nil {
			initEmbeds(i)
		}
		if embeds != nil {
			embeds[i] = r
		}
	}

	if methods == nil && embeds == nil {
		return iface
	}
	if methods == nil {
		initMethods(iface.NumExplicitMethods())
	}
	if embeds == nil {
		initEmbeds(iface.NumEmbeddeds())
	}
	return types.NewInterfaceType(methods, embeds).Complete()
}

func (subst *subster) alias(t *aliases.Alias) types.Type {
	// See subster.named. This follows the same strategy.
	tparams := aliases.TypeParams(t)
	targs := aliases.TypeArgs(t)
	tname := t.Obj()
	torigin := aliases.Origin(t)

	if !declaredWithin(tname, subst.origin) {
		// t is declared outside of the function origin. So t is a package level type alias.
		if targs.Len() == 0 {
			// No type arguments so no instantiation needed.
			return t
		}

		// Instantiate with the substituted type arguments.
		newTArgs := subst.typelist(targs)
		return subst.instantiate(torigin, newTArgs)
	}

	if targs.Len() == 0 {
		// t is declared within the function origin and has no type arguments.
		//
		// Example: This corresponds to A or B in F, but not A[int]:
		//
		//     func F[T any]() {
		//       type A[S any] = struct{t T, s S}
		//       type B = T
		//       var x A[int]
		//       ...
		//     }
		//
		// This is somewhat different than *Named as *Alias cannot be created recursively.

		// Copy and substitute type params.
		var newTParams []*types.TypeParam
		for i := 0; i < tparams.Len(); i++ {
			cur := tparams.At(i)
			cobj := cur.Obj()
			cname := types.NewTypeName(cobj.Pos(), cobj.Pkg(), cobj.Name(), nil)
			ntp := types.NewTypeParam(cname, nil)
			subst.cache[cur] = ntp // See the comment "Note: Subtle" in subster.named.
			newTParams = append(newTParams, ntp)
		}

		// Substitute rhs.
		rhs := subst.typ(aliases.Rhs(t))

		// Create the fresh alias.
		//
		// Until 1.27, the result of aliases.NewAlias(...).Type() cannot guarantee it is a *types.Alias.
		// However, as t is an *alias.Alias and t is well-typed, then aliases must have been enabled.
		// Follow this decision, and always enable aliases here.
		const enabled = true
		obj := aliases.NewAlias(enabled, tname.Pos(), tname.Pkg(), tname.Name(), rhs, newTParams)

		// Substitute into all of the constraints after they are created.
		for i, ntp := range newTParams {
			bound := tparams.At(i).Constraint()
			ntp.SetConstraint(subst.typ(bound))
		}
		return obj.Type()
	}

	// t is declared within the function origin and has type arguments.
	//
	// Example: This corresponds to A[int] in F. Cases A and B are handled above.
	//     func F[T any]() {
	//       type A[S any] = struct{t T, s S}
	//       type B = T
	//       var x A[int]
	//       ...
	//     }
	subOrigin := subst.typ(torigin)
	subTArgs := subst.typelist(targs)
	return subst.instantiate(subOrigin, subTArgs)
}

func (subst *subster) named(t *types.Named) types.Type {
	// A Named type is a user defined type.
	// Ignoring generics, Named types are canonical: they are identical if
	// and only if they have the same defining symbol.
	// Generics complicate things, both if the type definition itself is
	// parameterized, and if the type is defined within the scope of a
	// parameterized function. In this case, two named types are identical if
	// and only if their identifying symbols are identical, and all type
	// arguments bindings in scope of the named type definition (including the
	// type parameters of the definition itself) are equivalent.
	//
	// Notably:
	// 1. For type definition type T[P1 any] struct{}, T[A] and T[B] are identical
	//    only if A and B are identical.
	// 2. Inside the generic func Fn[m any]() any { type T struct{}; return T{} },
	//    the result of Fn[A] and Fn[B] have identical type if and only if A and
	//    B are identical.
	// 3. Both 1 and 2 could apply, such as in
	//    func F[m any]() any { type T[x any] struct{}; return T{} }
	//
	// A subster replaces type parameters within a function scope, and therefore must
	// also replace free type parameters in the definitions of local types.
	//
	// Note: There are some detailed notes sprinkled throughout that borrow from
	// lambda calculus notation. These contain some over simplifying math.
	//
	// LC: One way to think about subster is that it is  a way of evaluating
	//   ((λm. E) N) as E[m:=N].
	// Each Named type t has an object *TypeName within a scope S that binds an
	// underlying type expression U. U can refer to symbols within S (+ S's ancestors).
	// Let x = t.TypeParams() and A = t.TypeArgs().
	// Each Named type t is then either:
	//   U              where len(x) == 0 && len(A) == 0
	//   λx. U          where len(x) != 0 && len(A) == 0
	//   ((λx. U) A)    where len(x) == len(A)
	// In each case, we will evaluate t[m:=N].
	tparams := t.TypeParams() // x
	targs := t.TypeArgs()     // A

	if !declaredWithin(t.Obj(), subst.origin) {
		// t is declared outside of Fn[m].
		//
		// In this case, we can skip substituting t.Underlying().
		// The underlying type cannot refer to the type parameters.
		//
		// LC: Let free(E) be the set of free type parameters in an expression E.
		// Then whenever m ∉ free(E), then E = E[m:=N].
		// t ∉ Scope(fn) so therefore m ∉ free(U) and m ∩ x = ∅.
		if targs.Len() == 0 {
			// t has no type arguments. So it does not need to be instantiated.
			//
			// This is the normal case in real Go code, where t is not parameterized,
			// declared at some package scope, and m is a TypeParam from a parameterized
			// function F[m] or method.
			//
			// LC: m ∉ free(A) lets us conclude m ∉ free(t). So t=t[m:=N].
			return t
		}

		// t is declared outside of Fn[m] and has type arguments.
		// The type arguments may contain type parameters m so
		// substitute the type arguments, and instantiate the substituted
		// type arguments.
		//
		// LC: Evaluate this as ((λx. U) A') where A' = A[m := N].
		newTArgs := subst.typelist(targs)
		return subst.instantiate(t.Origin(), newTArgs)
	}

	// t is declared within Fn[m].

	if targs.Len() == 0 { // no type arguments?
		assert(t == t.Origin(), "local parameterized type abstraction must be an origin type")

		// t has no type arguments.
		// The underlying type of t may contain the function's type parameters,
		// replace these, and create a new type.
		//
		// Subtle: We short circuit substitution and use a newly created type in
		// subst, i.e. cache[t]=fresh, to preemptively replace t with fresh
		// in recursive types during traversal. This both breaks infinite cycles
		// and allows for constructing types with the replacement applied in
		// subst.typ(U).
		//
		// A new copy of the Named and Typename (and constraints) per function
		// instantiation matches the semantics of Go, which treats all function
		// instantiations F[N] as having distinct local types.
		//
		// LC: x.Len()=0 can be thought of as a special case of λx. U.
		// LC: Evaluate (λx. U)[m:=N] as (λx'. U') where U'=U[x:=x',m:=N].
		tname := t.Obj()
		obj := types.NewTypeName(tname.Pos(), tname.Pkg(), tname.Name(), nil)
		fresh := types.NewNamed(obj, nil, nil)
		var newTParams []*types.TypeParam
		for i := 0; i < tparams.Len(); i++ {
			cur := tparams.At(i)
			cobj := cur.Obj()
			cname := types.NewTypeName(cobj.Pos(), cobj.Pkg(), cobj.Name(), nil)
			ntp := types.NewTypeParam(cname, nil)
			subst.cache[cur] = ntp
			newTParams = append(newTParams, ntp)
		}
		fresh.SetTypeParams(newTParams)
		subst.cache[t] = fresh
		subst.cache[fresh] = fresh
		fresh.SetUnderlying(subst.typ(t.Underlying()))
		// Substitute into all of the constraints after they are created.
		for i, ntp := range newTParams {
			bound := tparams.At(i).Constraint()
			ntp.SetConstraint(subst.typ(bound))
		}
		return fresh
	}

	// t is defined within Fn[m] and t has type arguments (an instantiation).
	// We reduce this to the two cases above:
	// (1) substitute the function's type parameters into t.Origin().
	// (2) substitute t's type arguments A and instantiate the updated t.Origin() with these.
	//
	// LC: Evaluate ((λx. U) A)[m:=N] as (t' A') where t' = (λx. U)[m:=N] and A'=A [m:=N]
	subOrigin := subst.typ(t.Origin())
	subTArgs := subst.typelist(targs)
	return subst.instantiate(subOrigin, subTArgs)
}

func (subst *subster) instantiate(orig types.Type, targs []types.Type) types.Type {
	i, err := types.Instantiate(subst.ctxt, orig, targs, false)
	assert(err == nil, "failed to Instantiate named (Named or Alias) type")
	if c, _ := subst.uniqueness.At(i).(types.Type); c != nil {
		return c.(types.Type)
	}
	subst.uniqueness.Set(i, i)
	return i
}

func (subst *subster) typelist(l *types.TypeList) []types.Type {
	res := make([]types.Type, l.Len())
	for i := 0; i < l.Len(); i++ {
		res[i] = subst.typ(l.At(i))
	}
	return res
}

func (subst *subster) signature(t *types.Signature) types.Type {
	tparams := t.TypeParams()

	// We are choosing not to support tparams.Len() > 0 until a need has been observed in practice.
	//
	// There are some known usages for types.Types coming from types.{Eval,CheckExpr}.
	// To support tparams.Len() > 0, we just need to do the following [psuedocode]:
	//   targs := {subst.replacements[tparams[i]]]}; Instantiate(ctxt, t, targs, false)

	assert(tparams.Len() == 0, "Substituting types.Signatures with generic functions are currently unsupported.")

	// Either:
	// (1)non-generic function.
	//    no type params to substitute
	// (2)generic method and recv needs to be substituted.

	// Receivers can be either:
	// named
	// pointer to named
	// interface
	// nil
	// interface is the problematic case. We need to cycle break there!
	recv := subst.var_(t.Recv())
	params := subst.tuple(t.Params())
	results := subst.tuple(t.Results())
	if recv != t.Recv() || params != t.Params() || results != t.Results() {
		return types.NewSignatureType(recv, nil, nil, params, results, t.Variadic())
	}
	return t
}

// reaches returns true if a type t reaches any type t' s.t. c[t'] == true.
// It updates c to cache results.
//
// reaches is currently only part of the wellFormed debug logic, and
// in practice c is initially only type parameters. It is not currently
// relied on in production.
func reaches(t types.Type, c map[types.Type]bool) (res bool) {
	if c, ok := c[t]; ok {
		return c
	}

	// c is populated with temporary false entries as types are visited.
	// This avoids repeat visits and break cycles.
	c[t] = false
	defer func() {
		c[t] = res
	}()

	switch t := t.(type) {
	case *types.TypeParam, *types.Basic:
		return false
	case *types.Array:
		return reaches(t.Elem(), c)
	case *types.Slice:
		return reaches(t.Elem(), c)
	case *types.Pointer:
		return reaches(t.Elem(), c)
	case *types.Tuple:
		for i := 0; i < t.Len(); i++ {
			if reaches(t.At(i).Type(), c) {
				return true
			}
		}
	case *types.Struct:
		for i := 0; i < t.NumFields(); i++ {
			if reaches(t.Field(i).Type(), c) {
				return true
			}
		}
	case *types.Map:
		return reaches(t.Key(), c) || reaches(t.Elem(), c)
	case *types.Chan:
		return reaches(t.Elem(), c)
	case *types.Signature:
		if t.Recv() != nil && reaches(t.Recv().Type(), c) {
			return true
		}
		return reaches(t.Params(), c) || reaches(t.Results(), c)
	case *types.Union:
		for i := 0; i < t.Len(); i++ {
			if reaches(t.Term(i).Type(), c) {
				return true
			}
		}
	case *types.Interface:
		for i := 0; i < t.NumEmbeddeds(); i++ {
			if reaches(t.Embedded(i), c) {
				return true
			}
		}
		for i := 0; i < t.NumExplicitMethods(); i++ {
			if reaches(t.ExplicitMethod(i).Type(), c) {
				return true
			}
		}
	case *types.Named, *aliases.Alias:
		return reaches(t.Underlying(), c)
	default:
		panic("unreachable")
	}
	return false
}
