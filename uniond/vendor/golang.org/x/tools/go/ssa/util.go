// Copyright 2013 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ssa

// This file defines a number of miscellaneous utility functions.

import (
	"fmt"
	"go/ast"
	"go/token"
	"go/types"
	"io"
	"os"
	"sync"

	"golang.org/x/tools/go/ast/astutil"
	"golang.org/x/tools/go/types/typeutil"
	"golang.org/x/tools/internal/aliases"
	"golang.org/x/tools/internal/typeparams"
	"golang.org/x/tools/internal/typesinternal"
)

type unit struct{}

//// Sanity checking utilities

// assert panics with the mesage msg if p is false.
// Avoid combining with expensive string formatting.
func assert(p bool, msg string) {
	if !p {
		panic(msg)
	}
}

//// AST utilities

func unparen(e ast.Expr) ast.Expr { return astutil.Unparen(e) }

// isBlankIdent returns true iff e is an Ident with name "_".
// They have no associated types.Object, and thus no type.
func isBlankIdent(e ast.Expr) bool {
	id, ok := e.(*ast.Ident)
	return ok && id.Name == "_"
}

// rangePosition is the position to give for the `range` token in a RangeStmt.
var rangePosition = func(rng *ast.RangeStmt) token.Pos {
	// Before 1.20, this is unreachable.
	// rng.For is a close, but incorrect position.
	return rng.For
}

//// Type utilities.  Some of these belong in go/types.

// isNonTypeParamInterface reports whether t is an interface type but not a type parameter.
func isNonTypeParamInterface(t types.Type) bool {
	return !typeparams.IsTypeParam(t) && types.IsInterface(t)
}

// isBasic reports whether t is a basic type.
// t is assumed to be an Underlying type (not Named or Alias).
func isBasic(t types.Type) bool {
	_, ok := t.(*types.Basic)
	return ok
}

// isString reports whether t is exactly a string type.
// t is assumed to be an Underlying type (not Named or Alias).
func isString(t types.Type) bool {
	basic, ok := t.(*types.Basic)
	return ok && basic.Info()&types.IsString != 0
}

// isByteSlice reports whether t is of the form []~bytes.
// t is assumed to be an Underlying type (not Named or Alias).
func isByteSlice(t types.Type) bool {
	if b, ok := t.(*types.Slice); ok {
		e, _ := b.Elem().Underlying().(*types.Basic)
		return e != nil && e.Kind() == types.Byte
	}
	return false
}

// isRuneSlice reports whether t is of the form []~runes.
// t is assumed to be an Underlying type (not Named or Alias).
func isRuneSlice(t types.Type) bool {
	if b, ok := t.(*types.Slice); ok {
		e, _ := b.Elem().Underlying().(*types.Basic)
		return e != nil && e.Kind() == types.Rune
	}
	return false
}

// isBasicConvTypes returns true when a type set can be
// one side of a Convert operation. This is when:
// - All are basic, []byte, or []rune.
// - At least 1 is basic.
// - At most 1 is []byte or []rune.
func isBasicConvTypes(tset termList) bool {
	basics := 0
	all := underIs(tset, func(t types.Type) bool {
		if isBasic(t) {
			basics++
			return true
		}
		return isByteSlice(t) || isRuneSlice(t)
	})
	return all && basics >= 1 && tset.Len()-basics <= 1
}

// isPointer reports whether t's underlying type is a pointer.
func isPointer(t types.Type) bool {
	return is[*types.Pointer](t.Underlying())
}

// isPointerCore reports whether t's core type is a pointer.
//
// (Most pointer manipulation is related to receivers, in which case
// isPointer is appropriate. tecallers can use isPointer(t).
func isPointerCore(t types.Type) bool {
	return is[*types.Pointer](typeparams.CoreType(t))
}

func is[T any](x any) bool {
	_, ok := x.(T)
	return ok
}

// recvType returns the receiver type of method obj.
func recvType(obj *types.Func) types.Type {
	return obj.Type().(*types.Signature).Recv().Type()
}

// fieldOf returns the index'th field of the (core type of) a struct type;
// otherwise returns nil.
func fieldOf(typ types.Type, index int) *types.Var {
	if st, ok := typeparams.CoreType(typ).(*types.Struct); ok {
		if 0 <= index && index < st.NumFields() {
			return st.Field(index)
		}
	}
	return nil
}

// isUntyped reports whether typ is the type of an untyped constant.
func isUntyped(typ types.Type) bool {
	// No Underlying/Unalias: untyped constant types cannot be Named or Alias.
	b, ok := typ.(*types.Basic)
	return ok && b.Info()&types.IsUntyped != 0
}

// declaredWithin reports whether an object is declared within a function.
//
// obj must not be a method or a field.
func declaredWithin(obj types.Object, fn *types.Func) bool {
	if obj.Pos() != token.NoPos {
		return fn.Scope().Contains(obj.Pos()) // trust the positions if they exist.
	}
	if fn.Pkg() != obj.Pkg() {
		return false // fast path for different packages
	}

	// Traverse Parent() scopes for fn.Scope().
	for p := obj.Parent(); p != nil; p = p.Parent() {
		if p == fn.Scope() {
			return true
		}
	}
	return false
}

// logStack prints the formatted "start" message to stderr and
// returns a closure that prints the corresponding "end" message.
// Call using 'defer logStack(...)()' to show builder stack on panic.
// Don't forget trailing parens!
func logStack(format string, args ...interface{}) func() {
	msg := fmt.Sprintf(format, args...)
	io.WriteString(os.Stderr, msg)
	io.WriteString(os.Stderr, "\n")
	return func() {
		io.WriteString(os.Stderr, msg)
		io.WriteString(os.Stderr, " end\n")
	}
}

// newVar creates a 'var' for use in a types.Tuple.
func newVar(name string, typ types.Type) *types.Var {
	return types.NewParam(token.NoPos, nil, name, typ)
}

// anonVar creates an anonymous 'var' for use in a types.Tuple.
func anonVar(typ types.Type) *types.Var {
	return newVar("", typ)
}

var lenResults = types.NewTuple(anonVar(tInt))

// makeLen returns the len builtin specialized to type func(T)int.
func makeLen(T types.Type) *Builtin {
	lenParams := types.NewTuple(anonVar(T))
	return &Builtin{
		name: "len",
		sig:  types.NewSignature(nil, lenParams, lenResults, false),
	}
}

// receiverTypeArgs returns the type arguments to a method's receiver.
// Returns an empty list if the receiver does not have type arguments.
func receiverTypeArgs(method *types.Func) []types.Type {
	recv := method.Type().(*types.Signature).Recv()
	_, named := typesinternal.ReceiverNamed(recv)
	if named == nil {
		return nil // recv is anonymous struct/interface
	}
	ts := named.TypeArgs()
	if ts.Len() == 0 {
		return nil
	}
	targs := make([]types.Type, ts.Len())
	for i := 0; i < ts.Len(); i++ {
		targs[i] = ts.At(i)
	}
	return targs
}

// recvAsFirstArg takes a method signature and returns a function
// signature with receiver as the first parameter.
func recvAsFirstArg(sig *types.Signature) *types.Signature {
	params := make([]*types.Var, 0, 1+sig.Params().Len())
	params = append(params, sig.Recv())
	for i := 0; i < sig.Params().Len(); i++ {
		params = append(params, sig.Params().At(i))
	}
	return types.NewSignatureType(nil, nil, nil, types.NewTuple(params...), sig.Results(), sig.Variadic())
}

// instance returns whether an expression is a simple or qualified identifier
// that is a generic instantiation.
func instance(info *types.Info, expr ast.Expr) bool {
	// Compare the logic here against go/types.instantiatedIdent,
	// which also handles  *IndexExpr and *IndexListExpr.
	var id *ast.Ident
	switch x := expr.(type) {
	case *ast.Ident:
		id = x
	case *ast.SelectorExpr:
		id = x.Sel
	default:
		return false
	}
	_, ok := info.Instances[id]
	return ok
}

// instanceArgs returns the Instance[id].TypeArgs as a slice.
func instanceArgs(info *types.Info, id *ast.Ident) []types.Type {
	targList := info.Instances[id].TypeArgs
	if targList == nil {
		return nil
	}

	targs := make([]types.Type, targList.Len())
	for i, n := 0, targList.Len(); i < n; i++ {
		targs[i] = targList.At(i)
	}
	return targs
}

// Mapping of a type T to a canonical instance C s.t. types.Indentical(T, C).
// Thread-safe.
type canonizer struct {
	mu    sync.Mutex
	types typeutil.Map // map from type to a canonical instance
	lists typeListMap  // map from a list of types to a canonical instance
}

func newCanonizer() *canonizer {
	c := &canonizer{}
	h := typeutil.MakeHasher()
	c.types.SetHasher(h)
	c.lists.hasher = h
	return c
}

// List returns a canonical representative of a list of types.
// Representative of the empty list is nil.
func (c *canonizer) List(ts []types.Type) *typeList {
	if len(ts) == 0 {
		return nil
	}

	unaliasAll := func(ts []types.Type) []types.Type {
		// Is there some top level alias?
		var found bool
		for _, t := range ts {
			if _, ok := t.(*aliases.Alias); ok {
				found = true
				break
			}
		}
		if !found {
			return ts // no top level alias
		}

		cp := make([]types.Type, len(ts)) // copy with top level aliases removed.
		for i, t := range ts {
			cp[i] = aliases.Unalias(t)
		}
		return cp
	}
	l := unaliasAll(ts)

	c.mu.Lock()
	defer c.mu.Unlock()
	return c.lists.rep(l)
}

// Type returns a canonical representative of type T.
// Removes top-level aliases.
//
// For performance, reasons the canonical instance is order-dependent,
// and may contain deeply nested aliases.
func (c *canonizer) Type(T types.Type) types.Type {
	T = aliases.Unalias(T) // remove the top level alias.

	c.mu.Lock()
	defer c.mu.Unlock()

	if r := c.types.At(T); r != nil {
		return r.(types.Type)
	}
	c.types.Set(T, T)
	return T
}

// A type for representing a canonized list of types.
type typeList []types.Type

func (l *typeList) identical(ts []types.Type) bool {
	if l == nil {
		return len(ts) == 0
	}
	n := len(*l)
	if len(ts) != n {
		return false
	}
	for i, left := range *l {
		right := ts[i]
		if !types.Identical(left, right) {
			return false
		}
	}
	return true
}

type typeListMap struct {
	hasher  typeutil.Hasher
	buckets map[uint32][]*typeList
}

// rep returns a canonical representative of a slice of types.
func (m *typeListMap) rep(ts []types.Type) *typeList {
	if m == nil || len(ts) == 0 {
		return nil
	}

	if m.buckets == nil {
		m.buckets = make(map[uint32][]*typeList)
	}

	h := m.hash(ts)
	bucket := m.buckets[h]
	for _, l := range bucket {
		if l.identical(ts) {
			return l
		}
	}

	// not present. create a representative.
	cp := make(typeList, len(ts))
	copy(cp, ts)
	rep := &cp

	m.buckets[h] = append(bucket, rep)
	return rep
}

func (m *typeListMap) hash(ts []types.Type) uint32 {
	if m == nil {
		return 0
	}
	// Some smallish prime far away from typeutil.Hash.
	n := len(ts)
	h := uint32(13619) + 2*uint32(n)
	for i := 0; i < n; i++ {
		h += 3 * m.hasher.Hash(ts[i])
	}
	return h
}

// instantiateMethod instantiates m with targs and returns a canonical representative for this method.
func (canon *canonizer) instantiateMethod(m *types.Func, targs []types.Type, ctxt *types.Context) *types.Func {
	recv := recvType(m)
	if p, ok := aliases.Unalias(recv).(*types.Pointer); ok {
		recv = p.Elem()
	}
	named := aliases.Unalias(recv).(*types.Named)
	inst, err := types.Instantiate(ctxt, named.Origin(), targs, false)
	if err != nil {
		panic(err)
	}
	rep := canon.Type(inst)
	obj, _, _ := types.LookupFieldOrMethod(rep, true, m.Pkg(), m.Name())
	return obj.(*types.Func)
}

// Exposed to ssautil using the linkname hack.
func isSyntactic(pkg *Package) bool { return pkg.syntax }

// mapValues returns a new unordered array of map values.
func mapValues[K comparable, V any](m map[K]V) []V {
	vals := make([]V, 0, len(m))
	for _, fn := range m {
		vals = append(vals, fn)
	}
	return vals

}
