package assert

import (
	"errors"
	"fmt"
	"go/ast"

	"gotest.tools/v3/assert/cmp"
	"gotest.tools/v3/internal/format"
	"gotest.tools/v3/internal/source"
)

// RunComparison and return Comparison.Success. If the comparison fails a messages
// will be printed using t.Log.
func RunComparison(
	t LogT,
	argSelector argSelector,
	f cmp.Comparison,
	msgAndArgs ...interface{},
) bool {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	result := f()
	if result.Success() {
		return true
	}

	if source.IsUpdate() {
		if updater, ok := result.(updateExpected); ok {
			const stackIndex = 3 // Assert/Check, assert, RunComparison
			err := updater.UpdatedExpected(stackIndex)
			switch {
			case err == nil:
				return true
			case errors.Is(err, source.ErrNotFound):
				// do nothing, fallthrough to regular failure message
			default:
				t.Log("failed to update source", err)
				return false
			}
		}
	}

	var message string
	switch typed := result.(type) {
	case resultWithComparisonArgs:
		const stackIndex = 3 // Assert/Check, assert, RunComparison
		args, err := source.CallExprArgs(stackIndex)
		if err != nil {
			t.Log(err.Error())
		}
		message = typed.FailureMessage(filterPrintableExpr(argSelector(args)))
	case resultBasic:
		message = typed.FailureMessage()
	default:
		message = fmt.Sprintf("comparison returned invalid Result type: %T", result)
	}

	t.Log(format.WithCustomMessage(failureMessage+message, msgAndArgs...))
	return false
}

type resultWithComparisonArgs interface {
	FailureMessage(args []ast.Expr) string
}

type resultBasic interface {
	FailureMessage() string
}

type updateExpected interface {
	UpdatedExpected(stackIndex int) error
}

// filterPrintableExpr filters the ast.Expr slice to only include Expr that are
// easy to read when printed and contain relevant information to an assertion.
//
// Ident and SelectorExpr are included because they print nicely and the variable
// names may provide additional context to their values.
// BasicLit and CompositeLit are excluded because their source is equivalent to
// their value, which is already available.
// Other types are ignored for now, but could be added if they are relevant.
func filterPrintableExpr(args []ast.Expr) []ast.Expr {
	result := make([]ast.Expr, len(args))
	for i, arg := range args {
		if isShortPrintableExpr(arg) {
			result[i] = arg
			continue
		}

		if starExpr, ok := arg.(*ast.StarExpr); ok {
			result[i] = starExpr.X
			continue
		}
	}
	return result
}

func isShortPrintableExpr(expr ast.Expr) bool {
	switch expr.(type) {
	case *ast.Ident, *ast.SelectorExpr, *ast.IndexExpr, *ast.SliceExpr:
		return true
	case *ast.BinaryExpr, *ast.UnaryExpr:
		return true
	default:
		// CallExpr, ParenExpr, TypeAssertExpr, KeyValueExpr, StarExpr
		return false
	}
}

type argSelector func([]ast.Expr) []ast.Expr

// ArgsAfterT selects args starting at position 1. Used when the caller has a
// testing.T as the first argument, and the args to select should follow it.
func ArgsAfterT(args []ast.Expr) []ast.Expr {
	if len(args) < 1 {
		return nil
	}
	return args[1:]
}

// ArgsFromComparisonCall selects args from the CallExpression at position 1.
// Used when the caller has a testing.T as the first argument, and the args to
// select are passed to the cmp.Comparison at position 1.
func ArgsFromComparisonCall(args []ast.Expr) []ast.Expr {
	if len(args) <= 1 {
		return nil
	}
	if callExpr, ok := args[1].(*ast.CallExpr); ok {
		return callExpr.Args
	}
	return nil
}

// ArgsAtZeroIndex selects args from the CallExpression at position 1.
// Used when the caller accepts a single cmp.Comparison argument.
func ArgsAtZeroIndex(args []ast.Expr) []ast.Expr {
	if len(args) == 0 {
		return nil
	}
	if callExpr, ok := args[0].(*ast.CallExpr); ok {
		return callExpr.Args
	}
	return nil
}
