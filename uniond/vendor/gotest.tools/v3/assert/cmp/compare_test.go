package cmp

import (
	"errors"
	"fmt"
	"go/ast"
	"io"
	"os"
	"reflect"
	"regexp"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestDeepEqual(t *testing.T) {
	t.Run("failure", func(t *testing.T) {
		result := DeepEqual([]string{"a", "b"}, []string{"b", "a"})()
		if result.Success() {
			t.Errorf("expected failure")
		}

		args := []ast.Expr{&ast.Ident{Name: "result"}, &ast.Ident{Name: "exp"}}
		message := result.(templatedResult).FailureMessage(args)
		expected := "\n--- result\n+++ exp\n"
		if !strings.HasPrefix(message, expected) {
			t.Errorf("expected prefix \n%q\ngot\n%q\n", expected, message)
		}
	})

	t.Run("success", func(t *testing.T) {
		actual := DeepEqual([]string{"a"}, []string{"a"})()
		assertSuccess(t, actual)
	})
}

type Stub struct {
	unx int
}

func TestDeepEqualWithUnexported(t *testing.T) {
	result := DeepEqual(Stub{}, Stub{unx: 1})()
	assertFailureHasPrefix(t, result, `cannot handle unexported field at {cmp.Stub}.unx:`)
}

func TestRegexp(t *testing.T) {
	var testcases = []struct {
		name   string
		regex  interface{}
		value  string
		match  bool
		expErr string
	}{
		{
			name:  "pattern string match",
			regex: "^[0-9]+$",
			value: "12123423456",
			match: true,
		},
		{
			name:   "simple pattern string no match",
			regex:  "bob",
			value:  "Probably",
			expErr: `value "Probably" does not match regexp "bob"`,
		},
		{
			name:   "pattern string no match",
			regex:  "^1",
			value:  "2123423456",
			expErr: `value "2123423456" does not match regexp "^1"`,
		},
		{
			name:  "regexp match",
			regex: regexp.MustCompile("^d[0-9a-f]{8}$"),
			value: "d1632beef",
			match: true,
		},
		{
			name:   "invalid regexp",
			regex:  "^1(",
			value:  "2",
			expErr: "error parsing regexp: missing closing ): `^1(`",
		},
		{
			name:   "invalid type",
			regex:  struct{}{},
			value:  "some string",
			expErr: "invalid type struct {} for regex pattern",
		},
	}

	for _, tc := range testcases {
		t.Run(tc.name, func(t *testing.T) {
			res := Regexp(tc.regex, tc.value)()
			if tc.match {
				assertSuccess(t, res)
			} else {
				assertFailure(t, res, tc.expErr)
			}
		})
	}
}

func TestLen(t *testing.T) {
	var testcases = []struct {
		seq             interface{}
		length          int
		expectedSuccess bool
		expectedMessage string
	}{
		{
			seq:             []string{"A", "b", "c"},
			length:          3,
			expectedSuccess: true,
		},
		{
			seq:             []string{"A", "b", "c"},
			length:          2,
			expectedMessage: "expected [A b c] (length 3) to have length 2",
		},
		{
			seq:             map[string]int{"a": 1, "b": 2},
			length:          2,
			expectedSuccess: true,
		},
		{
			seq:             [3]string{"a", "b", "c"},
			length:          3,
			expectedSuccess: true,
		},
		{
			seq:             "abcd",
			length:          4,
			expectedSuccess: true,
		},
		{
			seq:             "abcd",
			length:          3,
			expectedMessage: "expected abcd (length 4) to have length 3",
		},
	}

	for _, testcase := range testcases {
		t.Run(fmt.Sprintf("%v len=%d", testcase.seq, testcase.length), func(t *testing.T) {
			result := Len(testcase.seq, testcase.length)()
			if testcase.expectedSuccess {
				assertSuccess(t, result)
			} else {
				assertFailure(t, result, testcase.expectedMessage)
			}
		})
	}
}

func TestPanics(t *testing.T) {
	panicker := func() {
		panic("AHHHHHHHHHHH")
	}

	result := Panics(panicker)()
	assertSuccess(t, result)

	result = Panics(func() {})()
	assertFailure(t, result, "did not panic")
}

type innerstub struct {
	num int
}

type stub struct {
	stub innerstub
	num  int
}

func TestDeepEqualEquivalenceToReflectDeepEqual(t *testing.T) {
	var testcases = []struct {
		left  interface{}
		right interface{}
	}{
		{nil, nil},
		{7, 7},
		{false, false},
		{stub{innerstub{1}, 2}, stub{innerstub{1}, 2}},
		{[]int{1, 2, 3}, []int{1, 2, 3}},
		{[]byte(nil), []byte(nil)},
		{nil, []byte(nil)},
		{1, uint64(1)},
		{7, "7"},
	}
	for _, testcase := range testcases {
		expected := reflect.DeepEqual(testcase.left, testcase.right)
		res := DeepEqual(testcase.left, testcase.right, cmpStub)()
		if res.Success() != expected {
			msg := res.(StringResult).FailureMessage()
			t.Errorf("deepEqual(%v, %v) did not return %v (message %s)",
				testcase.left, testcase.right, expected, msg)
		}
	}
}

var cmpStub = cmp.AllowUnexported(stub{}, innerstub{})

func TestContains(t *testing.T) {
	var testcases = []struct {
		seq         interface{}
		item        interface{}
		expected    bool
		expectedMsg string
	}{
		{
			seq:         error(nil),
			item:        0,
			expectedMsg: "nil does not contain items",
		},
		{
			seq:      "abcdef",
			item:     "cde",
			expected: true,
		},
		{
			seq:         "abcdef",
			item:        "foo",
			expectedMsg: `string "abcdef" does not contain "foo"`,
		},
		{
			seq:         "abcdef",
			item:        3,
			expectedMsg: `string may only contain strings`,
		},
		{
			seq:      map[rune]int{'a': 1, 'b': 2},
			item:     'b',
			expected: true,
		},
		{
			seq:         map[rune]int{'a': 1},
			item:        'c',
			expectedMsg: "map[97:1] does not contain 99",
		},
		{
			seq:         map[int]int{'a': 1, 'b': 2},
			item:        'b',
			expectedMsg: "map[int]int can not contain a int32 key",
		},
		{
			seq:      []interface{}{"a", 1, 'a', 1.0, true},
			item:     'a',
			expected: true,
		},
		{
			seq:         []interface{}{"a", 1, 'a', 1.0, true},
			item:        3,
			expectedMsg: "[a 1 97 1 true] does not contain 3",
		},
		{
			seq:      [3]byte{99, 10, 100},
			item:     byte(99),
			expected: true,
		},
		{
			seq:         [3]byte{99, 10, 100},
			item:        byte(98),
			expectedMsg: "[99 10 100] does not contain 98",
		},
	}
	for _, testcase := range testcases {
		name := fmt.Sprintf("%v in %v", testcase.item, testcase.seq)
		t.Run(name, func(t *testing.T) {
			result := Contains(testcase.seq, testcase.item)()
			if testcase.expected {
				assertSuccess(t, result)
			} else {
				assertFailure(t, result, testcase.expectedMsg)
			}
		})
	}
}

func TestEqualMultiLine(t *testing.T) {
	result := `abcd
1234
aaaa
bbbb`

	exp := `abcd
1111
aaaa
bbbb`

	expected := `
--- result
+++ exp
@@ -1,4 +1,4 @@
 abcd
-1234
+1111
 aaaa
 bbbb
`

	args := []ast.Expr{&ast.Ident{Name: "result"}, &ast.Ident{Name: "exp"}}
	res := Equal(result, exp)()
	assertFailureTemplate(t, res, args, expected)
}

func TestEqual_PointersNotEqual(t *testing.T) {
	x := 123
	y := 123

	res := Equal(&x, &y)()
	args := []ast.Expr{&ast.Ident{Name: "x"}, &ast.Ident{Name: "y"}}
	expected := fmt.Sprintf("%p (x *int) != %p (y *int)", &x, &y)
	assertFailureTemplate(t, res, args, expected)
}

// errorWithCause mimics the error formatting of github.com/pkg/errors
type errorWithCause struct {
	msg   string
	cause error
}

func (e errorWithCause) Error() string {
	return fmt.Sprintf("%v with cause: %v", e.msg, e.cause)
}

func (e errorWithCause) Cause() error {
	return e.cause
}

func (e errorWithCause) Format(s fmt.State, verb rune) {
	switch verb {
	case 'v':
		if s.Flag('+') {
			fmt.Fprintf(s, "%+v", e.Cause())
			fmt.Fprint(s, "\nstack trace")
			return
		}
		fallthrough
	case 's':
		io.WriteString(s, e.Error())
	case 'q':
		fmt.Fprintf(s, "%q", e.Error())
	}
}

func TestError(t *testing.T) {
	result := Error(nil, "the error message")()
	assertFailure(t, result, "expected an error, got nil")

	// A Wrapped error also includes the stack
	result = Error(errorWithCause{cause: errors.New("other"), msg: "wrapped"}, "the error message")()
	assertFailureHasPrefix(t, result,
		`expected error "the error message", got "wrapped with cause: other"
other
stack trace`)

	msg := "the message"
	result = Error(errors.New(msg), msg)()
	assertSuccess(t, result)
}

func TestErrorContains(t *testing.T) {
	result := ErrorContains(nil, "the error message")()
	assertFailure(t, result, "expected an error, got nil")

	result = ErrorContains(errors.New("other"), "the error")()
	assertFailureHasPrefix(t, result,
		`expected error to contain "the error", got "other"`)

	msg := "the full message"
	result = ErrorContains(errors.New(msg), "full")()
	assertSuccess(t, result)
}

func TestNil(t *testing.T) {
	result := Nil(nil)()
	assertSuccess(t, result)

	var s *string
	result = Nil(s)()
	assertSuccess(t, result)

	var closer io.Closer
	result = Nil(closer)()
	assertSuccess(t, result)

	result = Nil("wrong")()
	assertFailure(t, result, "wrong (type string) can not be nil")

	notnil := "notnil"
	result = Nil(&notnil)()
	assertFailure(t, result, "notnil (type *string) is not nil")

	result = Nil([]string{"a"})()
	assertFailure(t, result, "[a] (type []string) is not nil")
}

type testingT interface {
	Errorf(msg string, args ...interface{})
}

type helperT interface {
	Helper()
}

func assertSuccess(t testingT, res Result) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if !res.Success() {
		msg := res.(StringResult).FailureMessage()
		t.Errorf("expected success, but got failure with message %q", msg)
	}
}

func assertFailure(t testingT, res Result, expected string) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if res.Success() {
		t.Errorf("expected failure")
	}
	message := res.(StringResult).FailureMessage()
	if message != expected {
		t.Errorf("expected \n%q\ngot\n%q\n", expected, message)
	}
}

func assertFailureHasPrefix(t testingT, res Result, prefix string) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if res.Success() {
		t.Errorf("expected failure")
	}
	message := res.(StringResult).FailureMessage()
	if !strings.HasPrefix(message, prefix) {
		t.Errorf("expected \n%v\nto start with\n%v\n", message, prefix)
	}
}

func assertFailureTemplate(t testingT, res Result, args []ast.Expr, expected string) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if res.Success() {
		t.Errorf("expected failure")
	}
	message := res.(templatedResult).FailureMessage(args)
	if message != expected {
		t.Errorf("expected \n%q\ngot\n%q\n", expected, message)
	}
}

type stubError struct{}

func (s stubError) Error() string {
	return "stub error"
}

func isErrorOfTypeStub(err error) bool {
	return reflect.TypeOf(err) == reflect.TypeOf(stubError{})
}

type notStubError struct{}

func (s notStubError) Error() string {
	return "not stub error"
}

func isErrorOfTypeNotStub(err error) bool {
	return reflect.TypeOf(err) == reflect.TypeOf(notStubError{})
}

type specialStubIface interface {
	Special()
}

type stubPtrError struct{}

func (s *stubPtrError) Error() string {
	return "stub ptr error"
}

func TestErrorTypeWithNil(t *testing.T) {
	var testcases = []struct {
		name     string
		expType  interface{}
		expected string
	}{
		{
			name:     "with struct",
			expType:  stubError{},
			expected: "error is nil, not cmp.stubError",
		},
		{
			name:     "with pointer to struct",
			expType:  &stubPtrError{},
			expected: "error is nil, not *cmp.stubPtrError",
		},
		{
			name:     "with interface",
			expType:  (*specialStubIface)(nil),
			expected: "error is nil, not cmp.specialStubIface",
		},
		{
			name:     "with reflect.Type",
			expType:  reflect.TypeOf(stubError{}),
			expected: "error is nil, not cmp.stubError",
		},
	}
	for _, testcase := range testcases {
		t.Run(testcase.name, func(t *testing.T) {
			result := ErrorType(nil, testcase.expType)()
			assertFailure(t, result, testcase.expected)
		})
	}
}

func TestErrorTypeSuccess(t *testing.T) {
	var testcases = []struct {
		name    string
		expType interface{}
		err     error
	}{
		{
			name:    "with function",
			expType: isErrorOfTypeStub,
			err:     stubError{},
		},
		{
			name:    "with struct",
			expType: stubError{},
			err:     stubError{},
		},
		{
			name:    "with pointer to struct",
			expType: &stubPtrError{},
			err:     &stubPtrError{},
		},
		{
			name:    "with interface",
			expType: (*error)(nil),
			err:     stubError{},
		},
		{
			name:    "with reflect.Type struct",
			expType: reflect.TypeOf(stubError{}),
			err:     stubError{},
		},
		{
			name:    "with reflect.Type interface",
			expType: reflect.TypeOf((*error)(nil)).Elem(),
			err:     stubError{},
		},
	}
	for _, testcase := range testcases {
		t.Run(testcase.name, func(t *testing.T) {
			result := ErrorType(testcase.err, testcase.expType)()
			assertSuccess(t, result)
		})
	}
}

func TestErrorTypeFailure(t *testing.T) {
	var testcases = []struct {
		name     string
		expType  interface{}
		expected string
	}{
		{
			name:     "with struct",
			expType:  notStubError{},
			expected: "error is stub error (cmp.stubError), not cmp.notStubError",
		},
		{
			name:     "with pointer to struct",
			expType:  &stubPtrError{},
			expected: "error is stub error (cmp.stubError), not *cmp.stubPtrError",
		},
		{
			name:     "with interface",
			expType:  (*specialStubIface)(nil),
			expected: "error is stub error (cmp.stubError), not cmp.specialStubIface",
		},
		{
			name:     "with reflect.Type struct",
			expType:  reflect.TypeOf(notStubError{}),
			expected: "error is stub error (cmp.stubError), not cmp.notStubError",
		},
		{
			name:     "with reflect.Type interface",
			expType:  reflect.TypeOf((*specialStubIface)(nil)).Elem(),
			expected: "error is stub error (cmp.stubError), not cmp.specialStubIface",
		},
	}
	for _, testcase := range testcases {
		t.Run(testcase.name, func(t *testing.T) {
			result := ErrorType(stubError{}, testcase.expType)()
			assertFailure(t, result, testcase.expected)
		})
	}
}

func TestErrorTypeInvalid(t *testing.T) {
	result := ErrorType(stubError{}, nil)()
	assertFailure(t, result, "invalid type for expected: nil")

	result = ErrorType(stubError{}, "my type!")()
	assertFailure(t, result, "invalid type for expected: string")
}

func TestErrorTypeWithFunc(t *testing.T) {
	result := ErrorType(nil, isErrorOfTypeStub)()
	assertFailureTemplate(t, result,
		[]ast.Expr{nil, &ast.Ident{Name: "isErrorOfTypeStub"}},
		"error is nil, not isErrorOfTypeStub")

	result = ErrorType(stubError{}, isErrorOfTypeNotStub)()
	assertFailureTemplate(t, result,
		[]ast.Expr{nil, &ast.Ident{Name: "isErrorOfTypeNotStub"}},
		"error is stub error (cmp.stubError), not isErrorOfTypeNotStub")
}

func TestErrorIs(t *testing.T) {
	t.Run("equal", func(t *testing.T) {
		result := ErrorIs(stubError{}, stubError{})()
		assertSuccess(t, result)
	})
	t.Run("actual is nil, not stdlib error", func(t *testing.T) {
		result := ErrorIs(nil, stubError{})()
		args := []ast.Expr{
			&ast.Ident{Name: "err"},
			&ast.SelectorExpr{
				X:   &ast.Ident{Name: "mypkg"},
				Sel: &ast.Ident{Name: "StubError"},
			},
		}
		expected := `error is nil, not "stub error" (mypkg.StubError cmp.stubError)`
		assertFailureTemplate(t, result, args, expected)
	})
	t.Run("not equal, not stdlib error", func(t *testing.T) {
		result := ErrorIs(notStubError{}, stubError{})()
		args := []ast.Expr{
			&ast.Ident{Name: "err"},
			&ast.SelectorExpr{
				X:   &ast.Ident{Name: "mypkg"},
				Sel: &ast.Ident{Name: "StubError"},
			},
		}
		expected := `error is "not stub error" (cmp.notStubError), not "stub error" (mypkg.StubError cmp.stubError)`
		assertFailureTemplate(t, result, args, expected)
	})
	t.Run("actual is nil, stdlib error", func(t *testing.T) {
		result := ErrorIs(nil, os.ErrClosed)()
		args := []ast.Expr{
			&ast.Ident{Name: "err"},
			&ast.SelectorExpr{
				X:   &ast.Ident{Name: "os"},
				Sel: &ast.Ident{Name: "ErrClosed"},
			},
		}
		expected := `error is nil, not "file already closed" (os.ErrClosed)`
		assertFailureTemplate(t, result, args, expected)
	})
	t.Run("not equal, stdlib error", func(t *testing.T) {
		result := ErrorIs(fmt.Errorf("foo"), os.ErrClosed)()
		args := []ast.Expr{
			&ast.Ident{Name: "err"},
			&ast.SelectorExpr{
				X:   &ast.Ident{Name: "os"},
				Sel: &ast.Ident{Name: "ErrClosed"},
			},
		}
		expected := `error is "foo", not "file already closed" (os.ErrClosed)`
		assertFailureTemplate(t, result, args, expected)
	})
}
