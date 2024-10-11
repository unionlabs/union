// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package nilness

import (
	_ "embed"
	"fmt"
	"go/token"
	"go/types"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/buildssa"
	"golang.org/x/tools/go/analysis/passes/internal/analysisutil"
	"golang.org/x/tools/go/ssa"
	"golang.org/x/tools/internal/aliases"
	"golang.org/x/tools/internal/typeparams"
)

//go:embed doc.go
var doc string

var Analyzer = &analysis.Analyzer{
	Name:     "nilness",
	Doc:      analysisutil.MustExtractDoc(doc, "nilness"),
	URL:      "https://pkg.go.dev/golang.org/x/tools/go/analysis/passes/nilness",
	Run:      run,
	Requires: []*analysis.Analyzer{buildssa.Analyzer},
}

func run(pass *analysis.Pass) (interface{}, error) {
	ssainput := pass.ResultOf[buildssa.Analyzer].(*buildssa.SSA)
	for _, fn := range ssainput.SrcFuncs {
		runFunc(pass, fn)
	}
	return nil, nil
}

func runFunc(pass *analysis.Pass, fn *ssa.Function) {
	reportf := func(category string, pos token.Pos, format string, args ...interface{}) {
		// We ignore nil-checking ssa.Instructions
		// that don't correspond to syntax.
		if pos.IsValid() {
			pass.Report(analysis.Diagnostic{
				Pos:      pos,
				Category: category,
				Message:  fmt.Sprintf(format, args...),
			})
		}
	}

	// notNil reports an error if v is provably nil.
	notNil := func(stack []fact, instr ssa.Instruction, v ssa.Value, descr string) {
		if nilnessOf(stack, v) == isnil {
			reportf("nilderef", instr.Pos(), descr)
		}
	}

	// visit visits reachable blocks of the CFG in dominance order,
	// maintaining a stack of dominating nilness facts.
	//
	// By traversing the dom tree, we can pop facts off the stack as
	// soon as we've visited a subtree.  Had we traversed the CFG,
	// we would need to retain the set of facts for each block.
	seen := make([]bool, len(fn.Blocks)) // seen[i] means visit should ignore block i
	var visit func(b *ssa.BasicBlock, stack []fact)
	visit = func(b *ssa.BasicBlock, stack []fact) {
		if seen[b.Index] {
			return
		}
		seen[b.Index] = true

		// Report nil dereferences.
		for _, instr := range b.Instrs {
			switch instr := instr.(type) {
			case ssa.CallInstruction:
				// A nil receiver may be okay for type params.
				cc := instr.Common()
				if !(cc.IsInvoke() && typeparams.IsTypeParam(cc.Value.Type())) {
					notNil(stack, instr, cc.Value, "nil dereference in "+cc.Description())
				}
			case *ssa.FieldAddr:
				notNil(stack, instr, instr.X, "nil dereference in field selection")
			case *ssa.IndexAddr:
				switch typeparams.CoreType(instr.X.Type()).(type) {
				case *types.Pointer: // *array
					notNil(stack, instr, instr.X, "nil dereference in array index operation")
				case *types.Slice:
					// This is not necessarily a runtime error, because
					// it is usually dominated by a bounds check.
					if isRangeIndex(instr) {
						notNil(stack, instr, instr.X, "range of nil slice")
					} else {
						notNil(stack, instr, instr.X, "index of nil slice")
					}
				}
			case *ssa.MapUpdate:
				notNil(stack, instr, instr.Map, "nil dereference in map update")
			case *ssa.Range:
				// (Not a runtime error, but a likely mistake.)
				notNil(stack, instr, instr.X, "range over nil map")
			case *ssa.Slice:
				// A nilcheck occurs in ptr[:] iff ptr is a pointer to an array.
				if is[*types.Pointer](instr.X.Type().Underlying()) {
					notNil(stack, instr, instr.X, "nil dereference in slice operation")
				}
			case *ssa.Store:
				notNil(stack, instr, instr.Addr, "nil dereference in store")
			case *ssa.TypeAssert:
				if !instr.CommaOk {
					notNil(stack, instr, instr.X, "nil dereference in type assertion")
				}
			case *ssa.UnOp:
				switch instr.Op {
				case token.MUL: // *X
					notNil(stack, instr, instr.X, "nil dereference in load")
				case token.ARROW: // <-ch
					// (Not a runtime error, but a likely mistake.)
					notNil(stack, instr, instr.X, "receive from nil channel")
				}
			case *ssa.Send:
				// (Not a runtime error, but a likely mistake.)
				notNil(stack, instr, instr.Chan, "send to nil channel")
			}
		}

		// Look for panics with nil value
		for _, instr := range b.Instrs {
			switch instr := instr.(type) {
			case *ssa.Panic:
				if nilnessOf(stack, instr.X) == isnil {
					reportf("nilpanic", instr.Pos(), "panic with nil value")
				}
			case *ssa.SliceToArrayPointer:
				nn := nilnessOf(stack, instr.X)
				if nn == isnil && slice2ArrayPtrLen(instr) > 0 {
					reportf("conversionpanic", instr.Pos(), "nil slice being cast to an array of len > 0 will always panic")
				}
			}
		}

		// For nil comparison blocks, report an error if the condition
		// is degenerate, and push a nilness fact on the stack when
		// visiting its true and false successor blocks.
		if binop, tsucc, fsucc := eq(b); binop != nil {
			xnil := nilnessOf(stack, binop.X)
			ynil := nilnessOf(stack, binop.Y)

			if ynil != unknown && xnil != unknown && (xnil == isnil || ynil == isnil) {
				// Degenerate condition:
				// the nilness of both operands is known,
				// and at least one of them is nil.
				var adj string
				if (xnil == ynil) == (binop.Op == token.EQL) {
					adj = "tautological"
				} else {
					adj = "impossible"
				}
				reportf("cond", binop.Pos(), "%s condition: %s %s %s", adj, xnil, binop.Op, ynil)

				// If tsucc's or fsucc's sole incoming edge is impossible,
				// it is unreachable.  Prune traversal of it and
				// all the blocks it dominates.
				// (We could be more precise with full dataflow
				// analysis of control-flow joins.)
				var skip *ssa.BasicBlock
				if xnil == ynil {
					skip = fsucc
				} else {
					skip = tsucc
				}
				for _, d := range b.Dominees() {
					if d == skip && len(d.Preds) == 1 {
						continue
					}
					visit(d, stack)
				}
				return
			}

			// "if x == nil" or "if nil == y" condition; x, y are unknown.
			if xnil == isnil || ynil == isnil {
				var newFacts facts
				if xnil == isnil {
					// x is nil, y is unknown:
					// t successor learns y is nil.
					newFacts = expandFacts(fact{binop.Y, isnil})
				} else {
					// y is nil, x is unknown:
					// t successor learns x is nil.
					newFacts = expandFacts(fact{binop.X, isnil})
				}

				for _, d := range b.Dominees() {
					// Successor blocks learn a fact
					// only at non-critical edges.
					// (We could do be more precise with full dataflow
					// analysis of control-flow joins.)
					s := stack
					if len(d.Preds) == 1 {
						if d == tsucc {
							s = append(s, newFacts...)
						} else if d == fsucc {
							s = append(s, newFacts.negate()...)
						}
					}
					visit(d, s)
				}
				return
			}
		}

		// In code of the form:
		//
		// 	if ptr, ok := x.(*T); ok { ... } else { fsucc }
		//
		// the fsucc block learns that ptr == nil,
		// since that's its zero value.
		if If, ok := b.Instrs[len(b.Instrs)-1].(*ssa.If); ok {
			// Handle "if ok" and "if !ok" variants.
			cond, fsucc := If.Cond, b.Succs[1]
			if unop, ok := cond.(*ssa.UnOp); ok && unop.Op == token.NOT {
				cond, fsucc = unop.X, b.Succs[0]
			}

			// Match pattern:
			//   t0 = typeassert (pointerlike)
			//   t1 = extract t0 #0  // ptr
			//   t2 = extract t0 #1  // ok
			//   if t2 goto tsucc, fsucc
			if extract1, ok := cond.(*ssa.Extract); ok && extract1.Index == 1 {
				if assert, ok := extract1.Tuple.(*ssa.TypeAssert); ok &&
					isNillable(assert.AssertedType) {
					for _, pinstr := range *assert.Referrers() {
						if extract0, ok := pinstr.(*ssa.Extract); ok &&
							extract0.Index == 0 &&
							extract0.Tuple == extract1.Tuple {
							for _, d := range b.Dominees() {
								if len(d.Preds) == 1 && d == fsucc {
									visit(d, append(stack, fact{extract0, isnil}))
								}
							}
						}
					}
				}
			}
		}

		for _, d := range b.Dominees() {
			visit(d, stack)
		}
	}

	// Visit the entry block.  No need to visit fn.Recover.
	if fn.Blocks != nil {
		visit(fn.Blocks[0], make([]fact, 0, 20)) // 20 is plenty
	}
}

// A fact records that a block is dominated
// by the condition v == nil or v != nil.
type fact struct {
	value   ssa.Value
	nilness nilness
}

func (f fact) negate() fact { return fact{f.value, -f.nilness} }

type nilness int

const (
	isnonnil         = -1
	unknown  nilness = 0
	isnil            = 1
)

var nilnessStrings = []string{"non-nil", "unknown", "nil"}

func (n nilness) String() string { return nilnessStrings[n+1] }

// nilnessOf reports whether v is definitely nil, definitely not nil,
// or unknown given the dominating stack of facts.
func nilnessOf(stack []fact, v ssa.Value) nilness {

	switch v := v.(type) {
	// unwrap ChangeInterface and Slice values recursively, to detect if underlying
	// values have any facts recorded or are otherwise known with regard to nilness.
	//
	// This work must be in addition to expanding facts about
	// ChangeInterfaces during inference/fact gathering because this covers
	// cases where the nilness of a value is intrinsic, rather than based
	// on inferred facts, such as a zero value interface variable. That
	// said, this work alone would only inform us when facts are about
	// underlying values, rather than outer values, when the analysis is
	// transitive in both directions.
	case *ssa.ChangeInterface:
		if underlying := nilnessOf(stack, v.X); underlying != unknown {
			return underlying
		}
	case *ssa.MakeInterface:
		// A MakeInterface is non-nil unless its operand is a type parameter.
		tparam, ok := aliases.Unalias(v.X.Type()).(*types.TypeParam)
		if !ok {
			return isnonnil
		}

		// A MakeInterface of a type parameter is non-nil if
		// the type parameter cannot be instantiated as an
		// interface type (#66835).
		if terms, err := typeparams.NormalTerms(tparam.Constraint()); err == nil && len(terms) > 0 {
			return isnonnil
		}

		// If the type parameter can be instantiated as an
		// interface (and thus also as a concrete type),
		// we can't determine the nilness.

	case *ssa.Slice:
		if underlying := nilnessOf(stack, v.X); underlying != unknown {
			return underlying
		}
	case *ssa.SliceToArrayPointer:
		nn := nilnessOf(stack, v.X)
		if slice2ArrayPtrLen(v) > 0 {
			if nn == isnil {
				// We know that *(*[1]byte)(nil) is going to panic because of the
				// conversion. So return unknown to the caller, prevent useless
				// nil deference reporting due to * operator.
				return unknown
			}
			// Otherwise, the conversion will yield a non-nil pointer to array.
			// Note that the instruction can still panic if array length greater
			// than slice length. If the value is used by another instruction,
			// that instruction can assume the panic did not happen when that
			// instruction is reached.
			return isnonnil
		}
		// In case array length is zero, the conversion result depends on nilness of the slice.
		if nn != unknown {
			return nn
		}
	}

	// Is value intrinsically nil or non-nil?
	switch v := v.(type) {
	case *ssa.Alloc,
		*ssa.FieldAddr,
		*ssa.FreeVar,
		*ssa.Function,
		*ssa.Global,
		*ssa.IndexAddr,
		*ssa.MakeChan,
		*ssa.MakeClosure,
		*ssa.MakeMap,
		*ssa.MakeSlice:
		return isnonnil

	case *ssa.Const:
		if v.IsNil() {
			return isnil // nil or zero value of a pointer-like type
		} else {
			return unknown // non-pointer
		}
	}

	// Search dominating control-flow facts.
	for _, f := range stack {
		if f.value == v {
			return f.nilness
		}
	}
	return unknown
}

func slice2ArrayPtrLen(v *ssa.SliceToArrayPointer) int64 {
	return v.Type().(*types.Pointer).Elem().Underlying().(*types.Array).Len()
}

// If b ends with an equality comparison, eq returns the operation and
// its true (equal) and false (not equal) successors.
func eq(b *ssa.BasicBlock) (op *ssa.BinOp, tsucc, fsucc *ssa.BasicBlock) {
	if If, ok := b.Instrs[len(b.Instrs)-1].(*ssa.If); ok {
		if binop, ok := If.Cond.(*ssa.BinOp); ok {
			switch binop.Op {
			case token.EQL:
				return binop, b.Succs[0], b.Succs[1]
			case token.NEQ:
				return binop, b.Succs[1], b.Succs[0]
			}
		}
	}
	return nil, nil, nil
}

// expandFacts takes a single fact and returns the set of facts that can be
// known about it or any of its related values. Some operations, like
// ChangeInterface, have transitive nilness, such that if you know the
// underlying value is nil, you also know the value itself is nil, and vice
// versa. This operation allows callers to match on any of the related values
// in analyses, rather than just the one form of the value that happened to
// appear in a comparison.
//
// This work must be in addition to unwrapping values within nilnessOf because
// while this work helps give facts about transitively known values based on
// inferred facts, the recursive check within nilnessOf covers cases where
// nilness facts are intrinsic to the underlying value, such as a zero value
// interface variables.
//
// ChangeInterface is the only expansion currently supported, but others, like
// Slice, could be added. At this time, this tool does not check slice
// operations in a way this expansion could help. See
// https://play.golang.org/p/mGqXEp7w4fR for an example.
func expandFacts(f fact) []fact {
	ff := []fact{f}

Loop:
	for {
		switch v := f.value.(type) {
		case *ssa.ChangeInterface:
			f = fact{v.X, f.nilness}
			ff = append(ff, f)
		default:
			break Loop
		}
	}

	return ff
}

type facts []fact

func (ff facts) negate() facts {
	nn := make([]fact, len(ff))
	for i, f := range ff {
		nn[i] = f.negate()
	}
	return nn
}

func is[T any](x any) bool {
	_, ok := x.(T)
	return ok
}

func isNillable(t types.Type) bool {
	// TODO(adonovan): CoreType (+ case *Interface) looks wrong.
	// This should probably use Underlying, and handle TypeParam
	// by computing the union across its normal terms.
	switch t := typeparams.CoreType(t).(type) {
	case *types.Pointer,
		*types.Map,
		*types.Signature,
		*types.Chan,
		*types.Interface,
		*types.Slice:
		return true
	case *types.Basic:
		return t == types.Typ[types.UnsafePointer]
	}
	return false
}

// isRangeIndex reports whether the instruction is a slice indexing
// operation slice[i] within a "for range slice" loop. The operation
// could be explicit, such as slice[i] within (or even after) the
// loop, or it could be implicit, such as "for i, v := range slice {}".
// (These cannot be reliably distinguished.)
func isRangeIndex(instr *ssa.IndexAddr) bool {
	// Here we reverse-engineer the go/ssa lowering of range-over-slice:
	//
	//      n = len(x)
	//      jump loop
	// loop:                                                "rangeindex.loop"
	//      phi = φ(-1, incr) #rangeindex
	//      incr = phi + 1
	//      cond = incr < n
	//      if cond goto body else done
	// body:                                                "rangeindex.body"
	//      instr = &x[incr]
	//      ...
	// done:
	if incr, ok := instr.Index.(*ssa.BinOp); ok && incr.Op == token.ADD {
		if b := incr.Block(); b.Comment == "rangeindex.loop" {
			if If, ok := b.Instrs[len(b.Instrs)-1].(*ssa.If); ok {
				if cond := If.Cond.(*ssa.BinOp); cond.X == incr && cond.Op == token.LSS {
					if call, ok := cond.Y.(*ssa.Call); ok {
						common := call.Common()
						if blt, ok := common.Value.(*ssa.Builtin); ok && blt.Name() == "len" {
							return common.Args[0] == instr.X
						}
					}
				}
			}
		}
	}
	return false
}
