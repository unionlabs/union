// Package assert provides internal utilties for assertions.
package assert

import (
	"fmt"
	"go/ast"
	"go/token"
	"reflect"

	"gotest.tools/v3/assert/cmp"
	"gotest.tools/v3/internal/format"
	"gotest.tools/v3/internal/source"
)

// LogT is the subset of testing.T used by the assert package.
type LogT interface {
	Log(args ...interface{})
}

type helperT interface {
	Helper()
}

const failureMessage = "assertion failed: "

// Eval the comparison and print a failure messages if the comparison has failed.
func Eval(
	t LogT,
	argSelector argSelector,
	comparison interface{},
	msgAndArgs ...interface{},
) bool {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	var success bool
	switch check := comparison.(type) {
	case bool:
		if check {
			return true
		}
		logFailureFromBool(t, msgAndArgs...)

	// Undocumented legacy comparison without Result type
	case func() (success bool, message string):
		success = runCompareFunc(t, check, msgAndArgs...)

	case nil:
		return true

	case error:
		msg := failureMsgFromError(check)
		t.Log(format.WithCustomMessage(failureMessage+msg, msgAndArgs...))

	case cmp.Comparison:
		success = RunComparison(t, argSelector, check, msgAndArgs...)

	case func() cmp.Result:
		success = RunComparison(t, argSelector, check, msgAndArgs...)

	default:
		t.Log(fmt.Sprintf("invalid Comparison: %v (%T)", check, check))
	}
	return success
}

func runCompareFunc(
	t LogT,
	f func() (success bool, message string),
	msgAndArgs ...interface{},
) bool {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if success, message := f(); !success {
		t.Log(format.WithCustomMessage(failureMessage+message, msgAndArgs...))
		return false
	}
	return true
}

func logFailureFromBool(t LogT, msgAndArgs ...interface{}) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	const stackIndex = 3 // Assert()/Check(), assert(), logFailureFromBool()
	args, err := source.CallExprArgs(stackIndex)
	if err != nil {
		t.Log(err.Error())
		return
	}

	const comparisonArgIndex = 1 // Assert(t, comparison)
	if len(args) <= comparisonArgIndex {
		t.Log(failureMessage + "but assert failed to find the expression to print")
		return
	}

	msg, err := boolFailureMessage(args[comparisonArgIndex])
	if err != nil {
		t.Log(err.Error())
		msg = "expression is false"
	}

	t.Log(format.WithCustomMessage(failureMessage+msg, msgAndArgs...))
}

func failureMsgFromError(err error) string {
	// Handle errors with non-nil types
	v := reflect.ValueOf(err)
	if v.Kind() == reflect.Ptr && v.IsNil() {
		return fmt.Sprintf("error is not nil: error has type %T", err)
	}
	return "error is not nil: " + err.Error()
}

func boolFailureMessage(expr ast.Expr) (string, error) {
	if binaryExpr, ok := expr.(*ast.BinaryExpr); ok {
		x, err := source.FormatNode(binaryExpr.X)
		if err != nil {
			return "", err
		}
		y, err := source.FormatNode(binaryExpr.Y)
		if err != nil {
			return "", err
		}

		switch binaryExpr.Op {
		case token.NEQ:
			return x + " is " + y, nil
		case token.EQL:
			return x + " is not " + y, nil
		case token.GTR:
			return x + " is <= " + y, nil
		case token.LSS:
			return x + " is >= " + y, nil
		case token.GEQ:
			return x + " is less than " + y, nil
		case token.LEQ:
			return x + " is greater than " + y, nil
		}
	}

	if unaryExpr, ok := expr.(*ast.UnaryExpr); ok && unaryExpr.Op == token.NOT {
		x, err := source.FormatNode(unaryExpr.X)
		if err != nil {
			return "", err
		}
		return x + " is true", nil
	}

	if ident, ok := expr.(*ast.Ident); ok {
		return ident.Name + " is false", nil
	}

	formatted, err := source.FormatNode(expr)
	if err != nil {
		return "", err
	}
	return "expression is false: " + formatted, nil
}
