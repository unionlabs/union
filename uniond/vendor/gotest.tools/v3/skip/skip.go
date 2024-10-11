/*
Package skip provides functions for skipping a test and printing the source code
of the condition used to skip the test.
*/
package skip // import "gotest.tools/v3/skip"

import (
	"fmt"
	"path"
	"reflect"
	"runtime"
	"strings"

	"gotest.tools/v3/internal/format"
	"gotest.tools/v3/internal/source"
)

type skipT interface {
	Skip(args ...interface{})
	Log(args ...interface{})
}

// Result of skip function
type Result interface {
	Skip() bool
	Message() string
}

type helperT interface {
	Helper()
}

// BoolOrCheckFunc can be a bool, func() bool, or func() Result. Other types will panic
type BoolOrCheckFunc interface{}

// If the condition expression evaluates to true, skip the test.
//
// The condition argument may be one of three types: bool, func() bool, or
// func() SkipResult.
// When called with a bool, the test will be skip if the condition evaluates to true.
// When called with a func() bool, the test will be skip if the function returns true.
// When called with a func() Result, the test will be skip if the Skip method
// of the result returns true.
// The skip message will contain the source code of the expression.
// Extra message text can be passed as a format string with args.
func If(t skipT, condition BoolOrCheckFunc, msgAndArgs ...interface{}) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	switch check := condition.(type) {
	case bool:
		ifCondition(t, check, msgAndArgs...)
	case func() bool:
		if check() {
			t.Skip(format.WithCustomMessage(getFunctionName(check), msgAndArgs...))
		}
	case func() Result:
		result := check()
		if result.Skip() {
			msg := getFunctionName(check) + ": " + result.Message()
			t.Skip(format.WithCustomMessage(msg, msgAndArgs...))
		}
	default:
		panic(fmt.Sprintf("invalid type for condition arg: %T", check))
	}
}

func getFunctionName(function interface{}) string {
	funcPath := runtime.FuncForPC(reflect.ValueOf(function).Pointer()).Name()
	return strings.SplitN(path.Base(funcPath), ".", 2)[1]
}

func ifCondition(t skipT, condition bool, msgAndArgs ...interface{}) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if !condition {
		return
	}
	const (
		stackIndex = 2
		argPos     = 1
	)
	source, err := source.FormattedCallExprArg(stackIndex, argPos)
	if err != nil {
		t.Log(err.Error())
		t.Skip(format.Message(msgAndArgs...))
	}
	t.Skip(format.WithCustomMessage(source, msgAndArgs...))
}
