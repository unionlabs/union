/*Package cmp provides Comparisons for Assert and Check*/
package cmp // import "gotest.tools/v3/assert/cmp"

import (
	"errors"
	"fmt"
	"reflect"
	"regexp"
	"strings"

	"github.com/google/go-cmp/cmp"
	"gotest.tools/v3/internal/format"
)

// Comparison is a function which compares values and returns [ResultSuccess] if
// the actual value matches the expected value. If the values do not match the
// [Result] will contain a message about why it failed.
type Comparison func() Result

// DeepEqual compares two values using [github.com/google/go-cmp/cmp]
// and succeeds if the values are equal.
//
// The comparison can be customized using comparison Options.
// Package [gotest.tools/v3/assert/opt] provides some additional
// commonly used Options.
func DeepEqual(x, y interface{}, opts ...cmp.Option) Comparison {
	return func() (result Result) {
		defer func() {
			if panicmsg, handled := handleCmpPanic(recover()); handled {
				result = ResultFailure(panicmsg)
			}
		}()
		diff := cmp.Diff(x, y, opts...)
		if diff == "" {
			return ResultSuccess
		}
		return multiLineDiffResult(diff, x, y)
	}
}

func handleCmpPanic(r interface{}) (string, bool) {
	if r == nil {
		return "", false
	}
	panicmsg, ok := r.(string)
	if !ok {
		panic(r)
	}
	switch {
	case strings.HasPrefix(panicmsg, "cannot handle unexported field"):
		return panicmsg, true
	}
	panic(r)
}

func toResult(success bool, msg string) Result {
	if success {
		return ResultSuccess
	}
	return ResultFailure(msg)
}

// RegexOrPattern may be either a [*regexp.Regexp] or a string that is a valid
// regexp pattern.
type RegexOrPattern interface{}

// Regexp succeeds if value v matches regular expression re.
//
// Example:
//
//	assert.Assert(t, cmp.Regexp("^[0-9a-f]{32}$", str))
//	r := regexp.MustCompile("^[0-9a-f]{32}$")
//	assert.Assert(t, cmp.Regexp(r, str))
func Regexp(re RegexOrPattern, v string) Comparison {
	match := func(re *regexp.Regexp) Result {
		return toResult(
			re.MatchString(v),
			fmt.Sprintf("value %q does not match regexp %q", v, re.String()))
	}

	return func() Result {
		switch regex := re.(type) {
		case *regexp.Regexp:
			return match(regex)
		case string:
			re, err := regexp.Compile(regex)
			if err != nil {
				return ResultFailure(err.Error())
			}
			return match(re)
		default:
			return ResultFailure(fmt.Sprintf("invalid type %T for regex pattern", regex))
		}
	}
}

// Equal succeeds if x == y. See [gotest.tools/v3/assert.Equal] for full documentation.
func Equal(x, y interface{}) Comparison {
	return func() Result {
		switch {
		case x == y:
			return ResultSuccess
		case isMultiLineStringCompare(x, y):
			diff := format.UnifiedDiff(format.DiffConfig{A: x.(string), B: y.(string)})
			return multiLineDiffResult(diff, x, y)
		}
		return ResultFailureTemplate(`
			{{- printf "%v" .Data.x}} (
				{{- with callArg 0 }}{{ formatNode . }} {{end -}}
				{{- printf "%T" .Data.x -}}
			) != {{ printf "%v" .Data.y}} (
				{{- with callArg 1 }}{{ formatNode . }} {{end -}}
				{{- printf "%T" .Data.y -}}
			)`,
			map[string]interface{}{"x": x, "y": y})
	}
}

func isMultiLineStringCompare(x, y interface{}) bool {
	strX, ok := x.(string)
	if !ok {
		return false
	}
	strY, ok := y.(string)
	if !ok {
		return false
	}
	return strings.Contains(strX, "\n") || strings.Contains(strY, "\n")
}

func multiLineDiffResult(diff string, x, y interface{}) Result {
	return ResultFailureTemplate(`
--- {{ with callArg 0 }}{{ formatNode . }}{{else}}←{{end}}
+++ {{ with callArg 1 }}{{ formatNode . }}{{else}}→{{end}}
{{ .Data.diff }}`,
		map[string]interface{}{"diff": diff, "x": x, "y": y})
}

// Len succeeds if the sequence has the expected length.
func Len(seq interface{}, expected int) Comparison {
	return func() (result Result) {
		defer func() {
			if e := recover(); e != nil {
				result = ResultFailure(fmt.Sprintf("type %T does not have a length", seq))
			}
		}()
		value := reflect.ValueOf(seq)
		length := value.Len()
		if length == expected {
			return ResultSuccess
		}
		msg := fmt.Sprintf("expected %s (length %d) to have length %d", seq, length, expected)
		return ResultFailure(msg)
	}
}

// Contains succeeds if item is in collection. Collection may be a string, map,
// slice, or array.
//
// If collection is a string, item must also be a string, and is compared using
// [strings.Contains].
// If collection is a Map, contains will succeed if item is a key in the map.
// If collection is a slice or array, item is compared to each item in the
// sequence using [reflect.DeepEqual].
func Contains(collection interface{}, item interface{}) Comparison {
	return func() Result {
		colValue := reflect.ValueOf(collection)
		if !colValue.IsValid() {
			return ResultFailure("nil does not contain items")
		}
		msg := fmt.Sprintf("%v does not contain %v", collection, item)

		itemValue := reflect.ValueOf(item)
		switch colValue.Type().Kind() {
		case reflect.String:
			if itemValue.Type().Kind() != reflect.String {
				return ResultFailure("string may only contain strings")
			}
			return toResult(
				strings.Contains(colValue.String(), itemValue.String()),
				fmt.Sprintf("string %q does not contain %q", collection, item))

		case reflect.Map:
			if itemValue.Type() != colValue.Type().Key() {
				return ResultFailure(fmt.Sprintf(
					"%v can not contain a %v key", colValue.Type(), itemValue.Type()))
			}
			return toResult(colValue.MapIndex(itemValue).IsValid(), msg)

		case reflect.Slice, reflect.Array:
			for i := 0; i < colValue.Len(); i++ {
				if reflect.DeepEqual(colValue.Index(i).Interface(), item) {
					return ResultSuccess
				}
			}
			return ResultFailure(msg)
		default:
			return ResultFailure(fmt.Sprintf("type %T does not contain items", collection))
		}
	}
}

// Panics succeeds if f() panics.
func Panics(f func()) Comparison {
	return func() (result Result) {
		defer func() {
			if err := recover(); err != nil {
				result = ResultSuccess
			}
		}()
		f()
		return ResultFailure("did not panic")
	}
}

// Error succeeds if err is a non-nil error, and the error message equals the
// expected message.
func Error(err error, message string) Comparison {
	return func() Result {
		switch {
		case err == nil:
			return ResultFailure("expected an error, got nil")
		case err.Error() != message:
			return ResultFailure(fmt.Sprintf(
				"expected error %q, got %s", message, formatErrorMessage(err)))
		}
		return ResultSuccess
	}
}

// ErrorContains succeeds if err is a non-nil error, and the error message contains
// the expected substring.
func ErrorContains(err error, substring string) Comparison {
	return func() Result {
		switch {
		case err == nil:
			return ResultFailure("expected an error, got nil")
		case !strings.Contains(err.Error(), substring):
			return ResultFailure(fmt.Sprintf(
				"expected error to contain %q, got %s", substring, formatErrorMessage(err)))
		}
		return ResultSuccess
	}
}

type causer interface {
	Cause() error
}

func formatErrorMessage(err error) string {
	//nolint:errorlint // unwrapping is not appropriate here
	if _, ok := err.(causer); ok {
		return fmt.Sprintf("%q\n%+v", err, err)
	}
	// This error was not wrapped with github.com/pkg/errors
	return fmt.Sprintf("%q", err)
}

// Nil succeeds if obj is a nil interface, pointer, or function.
//
// Use [gotest.tools/v3/assert.NilError] for comparing errors. Use Len(obj, 0) for comparing slices,
// maps, and channels.
func Nil(obj interface{}) Comparison {
	msgFunc := func(value reflect.Value) string {
		return fmt.Sprintf("%v (type %s) is not nil", reflect.Indirect(value), value.Type())
	}
	return isNil(obj, msgFunc)
}

func isNil(obj interface{}, msgFunc func(reflect.Value) string) Comparison {
	return func() Result {
		if obj == nil {
			return ResultSuccess
		}
		value := reflect.ValueOf(obj)
		kind := value.Type().Kind()
		if kind >= reflect.Chan && kind <= reflect.Slice {
			if value.IsNil() {
				return ResultSuccess
			}
			return ResultFailure(msgFunc(value))
		}

		return ResultFailure(fmt.Sprintf("%v (type %s) can not be nil", value, value.Type()))
	}
}

// ErrorType succeeds if err is not nil and is of the expected type.
//
// Expected can be one of:
//
//	func(error) bool
//
// Function should return true if the error is the expected type.
//
//	type struct{}, type &struct{}
//
// A struct or a pointer to a struct.
// Fails if the error is not of the same type as expected.
//
//	type &interface{}
//
// A pointer to an interface type.
// Fails if err does not implement the interface.
//
//	reflect.Type
//
// Fails if err does not implement the [reflect.Type].
//
// Deprecated: Use [ErrorIs]
func ErrorType(err error, expected interface{}) Comparison {
	return func() Result {
		switch expectedType := expected.(type) {
		case func(error) bool:
			return cmpErrorTypeFunc(err, expectedType)
		case reflect.Type:
			if expectedType.Kind() == reflect.Interface {
				return cmpErrorTypeImplementsType(err, expectedType)
			}
			return cmpErrorTypeEqualType(err, expectedType)
		case nil:
			return ResultFailure("invalid type for expected: nil")
		}

		expectedType := reflect.TypeOf(expected)
		switch {
		case expectedType.Kind() == reflect.Struct, isPtrToStruct(expectedType):
			return cmpErrorTypeEqualType(err, expectedType)
		case isPtrToInterface(expectedType):
			return cmpErrorTypeImplementsType(err, expectedType.Elem())
		}
		return ResultFailure(fmt.Sprintf("invalid type for expected: %T", expected))
	}
}

func cmpErrorTypeFunc(err error, f func(error) bool) Result {
	if f(err) {
		return ResultSuccess
	}
	actual := "nil"
	if err != nil {
		actual = fmt.Sprintf("%s (%T)", err, err)
	}
	return ResultFailureTemplate(`error is {{ .Data.actual }}
		{{- with callArg 1 }}, not {{ formatNode . }}{{end -}}`,
		map[string]interface{}{"actual": actual})
}

func cmpErrorTypeEqualType(err error, expectedType reflect.Type) Result {
	if err == nil {
		return ResultFailure(fmt.Sprintf("error is nil, not %s", expectedType))
	}
	errValue := reflect.ValueOf(err)
	if errValue.Type() == expectedType {
		return ResultSuccess
	}
	return ResultFailure(fmt.Sprintf("error is %s (%T), not %s", err, err, expectedType))
}

func cmpErrorTypeImplementsType(err error, expectedType reflect.Type) Result {
	if err == nil {
		return ResultFailure(fmt.Sprintf("error is nil, not %s", expectedType))
	}
	errValue := reflect.ValueOf(err)
	if errValue.Type().Implements(expectedType) {
		return ResultSuccess
	}
	return ResultFailure(fmt.Sprintf("error is %s (%T), not %s", err, err, expectedType))
}

func isPtrToInterface(typ reflect.Type) bool {
	return typ.Kind() == reflect.Ptr && typ.Elem().Kind() == reflect.Interface
}

func isPtrToStruct(typ reflect.Type) bool {
	return typ.Kind() == reflect.Ptr && typ.Elem().Kind() == reflect.Struct
}

var (
	stdlibErrorNewType = reflect.TypeOf(errors.New(""))
	stdlibFmtErrorType = reflect.TypeOf(fmt.Errorf("%w", fmt.Errorf("")))
)

// ErrorIs succeeds if errors.Is(actual, expected) returns true. See
// [errors.Is] for accepted argument values.
func ErrorIs(actual error, expected error) Comparison {
	return func() Result {
		if errors.Is(actual, expected) {
			return ResultSuccess
		}

		// The type of stdlib errors is excluded because the type is not relevant
		// in those cases. The type is only important when it is a user defined
		// custom error type.
		return ResultFailureTemplate(`error is
			{{- if not .Data.a }} nil,{{ else }}
				{{- printf " \"%v\"" .Data.a }}
				{{- if notStdlibErrorType .Data.a }} ({{ printf "%T" .Data.a }}){{ end }},
			{{- end }} not {{ printf "\"%v\"" .Data.x }} (
			{{- with callArg 1 }}{{ formatNode . }}{{ end }}
			{{- if notStdlibErrorType .Data.x }}{{ printf " %T" .Data.x }}{{ end }})`,
			map[string]interface{}{"a": actual, "x": expected})
	}
}
