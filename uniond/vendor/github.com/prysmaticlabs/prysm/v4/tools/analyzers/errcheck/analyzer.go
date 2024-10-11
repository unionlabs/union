// Package errcheck implements an static analysis analyzer to ensure that errors are handled in go
// code. This analyzer was adapted from https://github.com/kisielk/errcheck (MIT License).
package errcheck

import (
	"errors"
	"fmt"
	"go/ast"
	"go/token"
	"go/types"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/inspect"
	"golang.org/x/tools/go/ast/inspector"
)

// Doc explaining the tool.
const Doc = "This tool enforces all errors must be handled and that type assertions test that " +
	"the type implements the given interface to prevent runtime panics."

// Analyzer runs static analysis.
var Analyzer = &analysis.Analyzer{
	Name:     "errcheck",
	Doc:      Doc,
	Requires: []*analysis.Analyzer{inspect.Analyzer},
	Run:      run,
}

var exclusions = make(map[string]bool)

func init() {
	for _, exc := range [...]string{
		// bytes
		"(*bytes.Buffer).Write",
		"(*bytes.Buffer).WriteByte",
		"(*bytes.Buffer).WriteRune",
		"(*bytes.Buffer).WriteString",

		// fmt
		"fmt.Errorf",
		"fmt.Print",
		"fmt.Printf",
		"fmt.Println",
		"fmt.Fprint(*bytes.Buffer)",
		"fmt.Fprintf(*bytes.Buffer)",
		"fmt.Fprintln(*bytes.Buffer)",
		"fmt.Fprint(*strings.Builder)",
		"fmt.Fprintf(*strings.Builder)",
		"fmt.Fprintln(*strings.Builder)",
		"fmt.Fprint(os.Stderr)",
		"fmt.Fprintf(os.Stderr)",
		"fmt.Fprintln(os.Stderr)",

		// math/rand
		"math/rand.Read",
		"(*math/rand.Rand).Read",

		// hash
		"(hash.Hash).Write",
	} {
		exclusions[exc] = true
	}
}

func run(pass *analysis.Pass) (interface{}, error) {
	inspection, ok := pass.ResultOf[inspect.Analyzer].(*inspector.Inspector)
	if !ok {
		return nil, errors.New("analyzer is not type *inspector.Inspector")
	}

	nodeFilter := []ast.Node{
		(*ast.CallExpr)(nil),
		(*ast.ExprStmt)(nil),
		(*ast.GoStmt)(nil),
		(*ast.DeferStmt)(nil),
		(*ast.AssignStmt)(nil),
	}

	inspection.Preorder(nodeFilter, func(node ast.Node) {
		switch stmt := node.(type) {
		case *ast.ExprStmt:
			if call, ok := stmt.X.(*ast.CallExpr); ok {
				if !ignoreCall(pass, call) && callReturnsError(pass, call) {
					reportUnhandledError(pass, call.Lparen, call)
				}
			}
		case *ast.GoStmt:
			if !ignoreCall(pass, stmt.Call) && callReturnsError(pass, stmt.Call) {
				reportUnhandledError(pass, stmt.Call.Lparen, stmt.Call)
			}
		case *ast.DeferStmt:
			if !ignoreCall(pass, stmt.Call) && callReturnsError(pass, stmt.Call) {
				reportUnhandledError(pass, stmt.Call.Lparen, stmt.Call)
			}
		case *ast.AssignStmt:
			if len(stmt.Rhs) == 1 {
				// single value on rhs; check against lhs identifiers
				if call, ok := stmt.Rhs[0].(*ast.CallExpr); ok {
					if ignoreCall(pass, call) {
						break
					}
					isError := errorsByArg(pass, call)
					for i := 0; i < len(stmt.Lhs); i++ {
						if id, ok := stmt.Lhs[i].(*ast.Ident); ok {
							// We shortcut calls to recover() because errorsByArg can't
							// check its return types for errors since it returns interface{}.
							if id.Name == "_" && (isRecover(pass, call) || isError[i]) {
								reportUnhandledError(pass, id.NamePos, call)
							}
						}
					}
				} else if assert, ok := stmt.Rhs[0].(*ast.TypeAssertExpr); ok {
					if assert.Type == nil {
						// type switch
						break
					}
					if len(stmt.Lhs) < 2 {
						// assertion result not read
						reportUnhandledTypeAssertion(pass, stmt.Rhs[0].Pos())
					} else if id, ok := stmt.Lhs[1].(*ast.Ident); ok && id.Name == "_" {
						// assertion result ignored
						reportUnhandledTypeAssertion(pass, id.NamePos)
					}
				}
			} else {
				// multiple value on rhs; in this case a call can't return
				// multiple values. Assume len(stmt.Lhs) == len(stmt.Rhs)
				for i := 0; i < len(stmt.Lhs); i++ {
					if id, ok := stmt.Lhs[i].(*ast.Ident); ok {
						if call, ok := stmt.Rhs[i].(*ast.CallExpr); ok {
							if ignoreCall(pass, call) {
								continue
							}
							if id.Name == "_" && callReturnsError(pass, call) {
								reportUnhandledError(pass, id.NamePos, call)
							}
						} else if assert, ok := stmt.Rhs[i].(*ast.TypeAssertExpr); ok {
							if assert.Type == nil {
								// Shouldn't happen anyway, no multi assignment in type switches
								continue
							}
							reportUnhandledError(pass, id.NamePos, nil)
						}
					}
				}
			}
		default:
		}
	})

	return nil, nil
}

func reportUnhandledError(pass *analysis.Pass, pos token.Pos, call *ast.CallExpr) {
	pass.Reportf(pos, "Unhandled error for function call %s", fullName(pass, call))
}

func reportUnhandledTypeAssertion(pass *analysis.Pass, pos token.Pos) {
	pass.Reportf(pos, "Unhandled type assertion check. You must test whether or not an "+
		"interface implements the asserted type.")
}

func fullName(pass *analysis.Pass, call *ast.CallExpr) string {
	_, fn, ok := selectorAndFunc(pass, call)
	if !ok {
		return ""
	}
	return fn.FullName()
}

// selectorAndFunc tries to get the selector and function from call expression.
// For example, given the call expression representing "a.b()", the selector
// is "a.b" and the function is "b" itself.
//
// The final return value will be true if it is able to do extract a selector
// from the call and look up the function object it refers to.
//
// If the call does not include a selector (like if it is a plain "f()" function call)
// then the final return value will be false.
func selectorAndFunc(pass *analysis.Pass, call *ast.CallExpr) (*ast.SelectorExpr, *types.Func, bool) {
	if call == nil || call.Fun == nil {
		return nil, nil, false
	}
	sel, ok := call.Fun.(*ast.SelectorExpr)
	if !ok {
		return nil, nil, false
	}

	fn, ok := pass.TypesInfo.ObjectOf(sel.Sel).(*types.Func)
	if !ok {
		return nil, nil, false
	}

	return sel, fn, true

}

func ignoreCall(pass *analysis.Pass, call *ast.CallExpr) bool {
	for _, name := range namesForExcludeCheck(pass, call) {
		if exclusions[name] {
			return true
		}
	}
	return false
}

var errorType = types.Universe.Lookup("error").Type().Underlying().(*types.Interface)

func isErrorType(t types.Type) bool {
	return types.Implements(t, errorType)
}

func callReturnsError(pass *analysis.Pass, call *ast.CallExpr) bool {
	if isRecover(pass, call) {
		return true
	}

	for _, isError := range errorsByArg(pass, call) {
		if isError {
			return true
		}
	}

	return false
}

// errorsByArg returns a slice s such that
// len(s) == number of return types of call
// s[i] == true iff return type at position i from left is an error type
func errorsByArg(pass *analysis.Pass, call *ast.CallExpr) []bool {
	switch t := pass.TypesInfo.Types[call].Type.(type) {
	case *types.Named:
		// Single return
		return []bool{isErrorType(t)}
	case *types.Pointer:
		// Single return via pointer
		return []bool{isErrorType(t)}
	case *types.Tuple:
		// Multiple returns
		s := make([]bool, t.Len())
		for i := 0; i < t.Len(); i++ {
			switch et := t.At(i).Type().(type) {
			case *types.Named:
				// Single return
				s[i] = isErrorType(et)
			case *types.Pointer:
				// Single return via pointer
				s[i] = isErrorType(et)
			default:
				s[i] = false
			}
		}
		return s
	}
	return []bool{false}
}

func isRecover(pass *analysis.Pass, call *ast.CallExpr) bool {
	if fun, ok := call.Fun.(*ast.Ident); ok {
		if _, ok := pass.TypesInfo.Uses[fun].(*types.Builtin); ok {
			return fun.Name == "recover"
		}
	}
	return false
}

func namesForExcludeCheck(pass *analysis.Pass, call *ast.CallExpr) []string {
	sel, fn, ok := selectorAndFunc(pass, call)
	if !ok {
		return nil
	}

	name := fullName(pass, call)
	if name == "" {
		return nil
	}

	// This will be missing for functions without a receiver (like fmt.Printf),
	// so just fall back to the function's fullName in that case.
	selection, ok := pass.TypesInfo.Selections[sel]
	if !ok {
		return []string{name}
	}

	// This will return with ok false if the function isn't defined
	// on an interface, so just fall back to the fullName.
	ts, ok := walkThroughEmbeddedInterfaces(selection)
	if !ok {
		return []string{name}
	}

	result := make([]string, len(ts))
	for i, t := range ts {
		// Like in fullName, vendored packages will have /vendor/ in their name,
		// thus not matching vendored standard library packages. If we
		// want to support vendored stdlib packages, we need to implement
		// additional logic here.
		result[i] = fmt.Sprintf("(%s).%s", t.String(), fn.Name())
	}
	return result
}

// walkThroughEmbeddedInterfaces returns a slice of Interfaces that
// we need to walk through in order to reach the actual definition,
// in an Interface, of the method selected by the given selection.
//
// false will be returned in the second return value if:
//   - the right side of the selection is not a function
//   - the actual definition of the function is not in an Interface
//
// The returned slice will contain all the interface types that need
// to be walked through to reach the actual definition.
//
// For example, say we have:
//
//	type Inner interface {Method()}
//	type Middle interface {Inner}
//	type Outer interface {Middle}
//	type T struct {Outer}
//	type U struct {T}
//	type V struct {U}
//
// And then the selector:
//
//	V.Method
//
// We'll return [Outer, Middle, Inner] by first walking through the embedded structs
// until we reach the Outer interface, then descending through the embedded interfaces
// until we find the one that actually explicitly defines Method.
func walkThroughEmbeddedInterfaces(sel *types.Selection) ([]types.Type, bool) {
	fn, ok := sel.Obj().(*types.Func)
	if !ok {
		return nil, false
	}

	// Start off at the receiver.
	currentT := sel.Recv()

	// First, we can walk through any Struct fields provided
	// by the selection Index() method. We ignore the last
	// index because it would give the method itself.
	indexes := sel.Index()
	for _, fieldIndex := range indexes[:len(indexes)-1] {
		currentT = typeAtFieldIndex(currentT, fieldIndex)
	}

	// Now currentT is either a type implementing the actual function,
	// an Invalid type (if the receiver is a package), or an interface.
	//
	// If it's not an Interface, then we're done, as this function
	// only cares about Interface-defined functions.
	//
	// If it is an Interface, we potentially need to continue digging until
	// we find the Interface that actually explicitly defines the function.
	interfaceT, ok := maybeUnname(currentT).(*types.Interface)
	if !ok {
		return nil, false
	}

	// The first interface we pass through is this one we've found. We return the possibly
	// wrapping types.Named because it is more useful to work with for callers.
	result := []types.Type{currentT}

	// If this interface itself explicitly defines the given method
	// then we're done digging.
	for !explicitlyDefinesMethod(interfaceT, fn) {
		// Otherwise, we find which of the embedded interfaces _does_
		// define the method, add it to our list, and loop.
		namedInterfaceT, ok := embeddedInterfaceDefiningMethod(interfaceT, fn)
		if !ok {
			// This should be impossible as long as we type-checked: either the
			// interface or one of its embedded ones must implement the method...
			panic(fmt.Sprintf("either %v or one of its embedded interfaces must implement %v", currentT, fn))
		}
		result = append(result, namedInterfaceT)
		interfaceT, ok = namedInterfaceT.Underlying().(*types.Interface)
		if !ok {
			panic(fmt.Sprintf("either %v or one of its embedded interfaces must implement %v", currentT, fn))
		}
	}

	return result, true
}

func typeAtFieldIndex(startingAt types.Type, fieldIndex int) types.Type {
	t := maybeUnname(maybeDereference(startingAt))
	s, ok := t.(*types.Struct)
	if !ok {
		panic(fmt.Sprintf("cannot get Field of a type that is not a struct, got a %T", t))
	}

	return s.Field(fieldIndex).Type()
}

// embeddedInterfaceDefiningMethod searches through any embedded interfaces of the
// passed interface searching for one that defines the given function. If found, the
// types.Named wrapping that interface will be returned along with true in the second value.
//
// If no such embedded interface is found, nil and false are returned.
func embeddedInterfaceDefiningMethod(interfaceT *types.Interface, fn *types.Func) (*types.Named, bool) {
	for i := 0; i < interfaceT.NumEmbeddeds(); i++ {
		embedded, ok := interfaceT.EmbeddedType(i).(*types.Named)
		if !ok {
			return nil, false
		}
		if definesMethod(embedded.Underlying().(*types.Interface), fn) {
			return embedded, true
		}
	}
	return nil, false
}

func explicitlyDefinesMethod(interfaceT *types.Interface, fn *types.Func) bool {
	for i := 0; i < interfaceT.NumExplicitMethods(); i++ {
		if interfaceT.ExplicitMethod(i) == fn {
			return true
		}
	}
	return false
}

func definesMethod(interfaceT *types.Interface, fn *types.Func) bool {
	for i := 0; i < interfaceT.NumMethods(); i++ {
		if interfaceT.Method(i) == fn {
			return true
		}
	}
	return false
}

func maybeDereference(t types.Type) types.Type {
	p, ok := t.(*types.Pointer)
	if ok {
		return p.Elem()
	}
	return t
}

func maybeUnname(t types.Type) types.Type {
	n, ok := t.(*types.Named)
	if ok {
		return n.Underlying()
	}
	return t
}
