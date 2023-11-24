/*
Package assert provides assertions for comparing expected values to actual
values in tests. When an assertion fails a helpful error message is printed.

# Example usage

All the assertions in this package use [testing.T.Helper] to mark themselves as
test helpers. This allows the testing package to print the filename and line
number of the file function that failed.

	assert.NilError(t, err)
	// filename_test.go:212: assertion failed: error is not nil: file not found

If any assertion is called from a helper function, make sure to call t.Helper
from the helper function so that the filename and line number remain correct.

The examples below show assert used with some common types and the failure
messages it produces. The filename and line number portion of the failure
message is omitted from these examples for brevity.

	// booleans

	assert.Assert(t, ok)
	// assertion failed: ok is false
	assert.Assert(t, !missing)
	// assertion failed: missing is true

	// primitives

	assert.Equal(t, count, 1)
	// assertion failed: 0 (count int) != 1 (int)
	assert.Equal(t, msg, "the message")
	// assertion failed: my message (msg string) != the message (string)
	assert.Assert(t, total != 10) // use Assert for NotEqual
	// assertion failed: total is 10
	assert.Assert(t, count > 20, "count=%v", count)
	// assertion failed: count is <= 20: count=1

	// errors

	assert.NilError(t, closer.Close())
	// assertion failed: error is not nil: close /file: errno 11
	assert.Error(t, err, "the exact error message")
	// assertion failed: expected error "the exact error message", got "oops"
	assert.ErrorContains(t, err, "includes this")
	// assertion failed: expected error to contain "includes this", got "oops"
	assert.ErrorIs(t, err, os.ErrNotExist)
	// assertion failed: error is "oops", not "file does not exist" (os.ErrNotExist)

	// complex types

	assert.DeepEqual(t, result, myStruct{Name: "title"})
	// assertion failed: ... (diff of the two structs)
	assert.Assert(t, is.Len(items, 3))
	// assertion failed: expected [] (length 0) to have length 3
	assert.Assert(t, len(sequence) != 0) // use Assert for NotEmpty
	// assertion failed: len(sequence) is 0
	assert.Assert(t, is.Contains(mapping, "key"))
	// assertion failed: map[other:1] does not contain key

	// pointers and interface

	assert.Assert(t, ref == nil)
	// assertion failed: ref is not nil
	assert.Assert(t, ref != nil) // use Assert for NotNil
	// assertion failed: ref is nil

# Assert and Check

[Assert] and [Check] are very similar, they both accept a [cmp.Comparison], and fail
the test when the comparison fails. The one difference is that Assert uses
[testing.T.FailNow] to fail the test, which will end the test execution immediately.
Check uses [testing.T.Fail] to fail the test, which allows it to return the
result of the comparison, then proceed with the rest of the test case.

Like [testing.T.FailNow], [Assert] must be called from the goroutine running the test,
not from other goroutines created during the test. [Check] is safe to use from any
goroutine.

# Comparisons

Package [gotest.tools/v3/assert/cmp] provides
many common comparisons. Additional comparisons can be written to compare
values in other ways. See the example Assert (CustomComparison).

# Automated migration from testify

gty-migrate-from-testify is a command which translates Go source code from
testify assertions to the assertions provided by this package.

See http://pkg.go.dev/gotest.tools/v3/assert/cmd/gty-migrate-from-testify.
*/
package assert // import "gotest.tools/v3/assert"

import (
	gocmp "github.com/google/go-cmp/cmp"
	"gotest.tools/v3/assert/cmp"
	"gotest.tools/v3/internal/assert"
)

// BoolOrComparison can be a bool, [cmp.Comparison], or error. See [Assert] for
// details about how this type is used.
type BoolOrComparison interface{}

// TestingT is the subset of [testing.T] (see also [testing.TB]) used by the assert package.
type TestingT interface {
	FailNow()
	Fail()
	Log(args ...interface{})
}

type helperT interface {
	Helper()
}

// Assert performs a comparison. If the comparison fails, the test is marked as
// failed, a failure message is logged, and execution is stopped immediately.
//
// The comparison argument may be one of three types:
//
//	bool
//	  True is success. False is a failure. The failure message will contain
//	  the literal source code of the expression.
//
//	cmp.Comparison
//	  Uses cmp.Result.Success() to check for success or failure.
//	  The comparison is responsible for producing a helpful failure message.
//	  http://pkg.go.dev/gotest.tools/v3/assert/cmp provides many common comparisons.
//
//	error
//	  A nil value is considered success, and a non-nil error is a failure.
//	  The return value of error.Error is used as the failure message.
//
// Extra details can be added to the failure message using msgAndArgs. msgAndArgs
// may be either a single string, or a format string and args that will be
// passed to [fmt.Sprintf].
//
// Assert uses [testing.TB.FailNow] to fail the test. Like t.FailNow, Assert must be called
// from the goroutine running the test function, not from other
// goroutines created during the test. Use [Check] from other goroutines.
func Assert(t TestingT, comparison BoolOrComparison, msgAndArgs ...interface{}) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if !assert.Eval(t, assert.ArgsFromComparisonCall, comparison, msgAndArgs...) {
		t.FailNow()
	}
}

// Check performs a comparison. If the comparison fails the test is marked as
// failed, a failure message is printed, and Check returns false. If the comparison
// is successful Check returns true. Check may be called from any goroutine.
//
// See [Assert] for details about the comparison arg and failure messages.
func Check(t TestingT, comparison BoolOrComparison, msgAndArgs ...interface{}) bool {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if !assert.Eval(t, assert.ArgsFromComparisonCall, comparison, msgAndArgs...) {
		t.Fail()
		return false
	}
	return true
}

// NilError fails the test immediately if err is not nil, and includes err.Error
// in the failure message.
//
// NilError uses [testing.TB.FailNow] to fail the test. Like t.FailNow, NilError must be
// called from the goroutine running the test function, not from other
// goroutines created during the test. Use [Check] from other goroutines.
func NilError(t TestingT, err error, msgAndArgs ...interface{}) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if !assert.Eval(t, assert.ArgsAfterT, err, msgAndArgs...) {
		t.FailNow()
	}
}

// Equal uses the == operator to assert two values are equal and fails the test
// if they are not equal.
//
// If the comparison fails Equal will use the variable names and types of
// x and y as part of the failure message to identify the actual and expected
// values.
//
//	assert.Equal(t, actual, expected)
//	// main_test.go:41: assertion failed: 1 (actual int) != 21 (expected int32)
//
// If either x or y are a multi-line string the failure message will include a
// unified diff of the two values. If the values only differ by whitespace
// the unified diff will be augmented by replacing whitespace characters with
// visible characters to identify the whitespace difference.
//
// Equal uses [testing.T.FailNow] to fail the test. Like t.FailNow, Equal must be
// called from the goroutine running the test function, not from other
// goroutines created during the test. Use [Check] with [cmp.Equal] from other
// goroutines.
func Equal(t TestingT, x, y interface{}, msgAndArgs ...interface{}) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if !assert.Eval(t, assert.ArgsAfterT, cmp.Equal(x, y), msgAndArgs...) {
		t.FailNow()
	}
}

// DeepEqual uses [github.com/google/go-cmp/cmp]
// to assert two values are equal and fails the test if they are not equal.
//
// Package [gotest.tools/v3/assert/opt] provides some additional
// commonly used Options.
//
// DeepEqual uses [testing.T.FailNow] to fail the test. Like t.FailNow, DeepEqual must be
// called from the goroutine running the test function, not from other
// goroutines created during the test. Use [Check] with [cmp.DeepEqual] from other
// goroutines.
func DeepEqual(t TestingT, x, y interface{}, opts ...gocmp.Option) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if !assert.Eval(t, assert.ArgsAfterT, cmp.DeepEqual(x, y, opts...)) {
		t.FailNow()
	}
}

// Error fails the test if err is nil, or if err.Error is not equal to expected.
// Both err.Error and expected will be included in the failure message.
// Error performs an exact match of the error text. Use [ErrorContains] if only
// part of the error message is relevant. Use [ErrorType] or [ErrorIs] to compare
// errors by type.
//
// Error uses [testing.T.FailNow] to fail the test. Like t.FailNow, Error must be
// called from the goroutine running the test function, not from other
// goroutines created during the test. Use [Check] with [cmp.Error] from other
// goroutines.
func Error(t TestingT, err error, expected string, msgAndArgs ...interface{}) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if !assert.Eval(t, assert.ArgsAfterT, cmp.Error(err, expected), msgAndArgs...) {
		t.FailNow()
	}
}

// ErrorContains fails the test if err is nil, or if err.Error does not
// contain the expected substring. Both err.Error and the expected substring
// will be included in the failure message.
//
// ErrorContains uses [testing.T.FailNow] to fail the test. Like t.FailNow, ErrorContains
// must be called from the goroutine running the test function, not from other
// goroutines created during the test. Use [Check] with [cmp.ErrorContains] from other
// goroutines.
func ErrorContains(t TestingT, err error, substring string, msgAndArgs ...interface{}) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if !assert.Eval(t, assert.ArgsAfterT, cmp.ErrorContains(err, substring), msgAndArgs...) {
		t.FailNow()
	}
}

// ErrorType fails the test if err is nil, or err is not the expected type.
// New code should use ErrorIs instead.
//
// Expected can be one of:
//
//	func(error) bool
//	  The function should return true if the error is the expected type.
//
//	struct{} or *struct{}
//	  A struct or a pointer to a struct. The assertion fails if the error is
//	  not of the same type.
//
//	*interface{}
//	  A pointer to an interface type. The assertion fails if err does not
//	  implement the interface.
//
//	reflect.Type
//	  The assertion fails if err does not implement the reflect.Type.
//
// ErrorType uses [testing.T.FailNow] to fail the test. Like t.FailNow, ErrorType
// must be called from the goroutine running the test function, not from other
// goroutines created during the test. Use [Check] with [cmp.ErrorType] from other
// goroutines.
//
// Deprecated: Use [ErrorIs]
func ErrorType(t TestingT, err error, expected interface{}, msgAndArgs ...interface{}) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if !assert.Eval(t, assert.ArgsAfterT, cmp.ErrorType(err, expected), msgAndArgs...) {
		t.FailNow()
	}
}

// ErrorIs fails the test if err is nil, or the error does not match expected
// when compared using errors.Is. See [errors.Is] for
// accepted arguments.
//
// ErrorIs uses [testing.T.FailNow] to fail the test. Like t.FailNow, ErrorIs
// must be called from the goroutine running the test function, not from other
// goroutines created during the test. Use [Check] with [cmp.ErrorIs] from other
// goroutines.
func ErrorIs(t TestingT, err error, expected error, msgAndArgs ...interface{}) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if !assert.Eval(t, assert.ArgsAfterT, cmp.ErrorIs(err, expected), msgAndArgs...) {
		t.FailNow()
	}
}
