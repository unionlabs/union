package assert

import (
	"bufio"
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"math"
	"os"
	"path/filepath"
	"reflect"
	"regexp"
	"runtime"
	"strings"
	"testing"
	"time"
)

var (
	i     interface{}
	zeros = []interface{}{
		false,
		byte(0),
		complex64(0),
		complex128(0),
		float32(0),
		float64(0),
		int(0),
		int8(0),
		int16(0),
		int32(0),
		int64(0),
		rune(0),
		uint(0),
		uint8(0),
		uint16(0),
		uint32(0),
		uint64(0),
		uintptr(0),
		"",
		[0]interface{}{},
		[]interface{}(nil),
		struct{ x int }{},
		(*interface{})(nil),
		(func())(nil),
		nil,
		interface{}(nil),
		map[interface{}]interface{}(nil),
		(chan interface{})(nil),
		(<-chan interface{})(nil),
		(chan<- interface{})(nil),
	}
	nonZeros = []interface{}{
		true,
		byte(1),
		complex64(1),
		complex128(1),
		float32(1),
		float64(1),
		int(1),
		int8(1),
		int16(1),
		int32(1),
		int64(1),
		rune(1),
		uint(1),
		uint8(1),
		uint16(1),
		uint32(1),
		uint64(1),
		uintptr(1),
		"s",
		[1]interface{}{1},
		[]interface{}{},
		struct{ x int }{1},
		(&i),
		(func() {}),
		interface{}(1),
		map[interface{}]interface{}{},
		(make(chan interface{})),
		(<-chan interface{})(make(chan interface{})),
		(chan<- interface{})(make(chan interface{})),
	}
)

// AssertionTesterInterface defines an interface to be used for testing assertion methods
type AssertionTesterInterface interface {
	TestMethod()
}

// AssertionTesterConformingObject is an object that conforms to the AssertionTesterInterface interface
type AssertionTesterConformingObject struct {
}

func (a *AssertionTesterConformingObject) TestMethod() {
}

// AssertionTesterNonConformingObject is an object that does not conform to the AssertionTesterInterface interface
type AssertionTesterNonConformingObject struct {
}

func TestObjectsAreEqual(t *testing.T) {
	cases := []struct {
		expected interface{}
		actual   interface{}
		result   bool
	}{
		// cases that are expected to be equal
		{"Hello World", "Hello World", true},
		{123, 123, true},
		{123.5, 123.5, true},
		{[]byte("Hello World"), []byte("Hello World"), true},
		{nil, nil, true},

		// cases that are expected not to be equal
		{map[int]int{5: 10}, map[int]int{10: 20}, false},
		{'x', "x", false},
		{"x", 'x', false},
		{0, 0.1, false},
		{0.1, 0, false},
		{time.Now, time.Now, false},
		{func() {}, func() {}, false},
		{uint32(10), int32(10), false},
	}

	for _, c := range cases {
		t.Run(fmt.Sprintf("ObjectsAreEqual(%#v, %#v)", c.expected, c.actual), func(t *testing.T) {
			res := ObjectsAreEqual(c.expected, c.actual)

			if res != c.result {
				t.Errorf("ObjectsAreEqual(%#v, %#v) should return %#v", c.expected, c.actual, c.result)
			}

		})
	}
}

func TestObjectsAreEqualValues(t *testing.T) {
	now := time.Now()

	cases := []struct {
		expected interface{}
		actual   interface{}
		result   bool
	}{
		{uint32(10), int32(10), true},
		{0, nil, false},
		{nil, 0, false},
		{now, now.In(time.Local), false}, // should not be time zone independent
		{int(270), int8(14), false},      // should handle overflow/underflow
		{int8(14), int(270), false},
		{[]int{270, 270}, []int8{14, 14}, false},
		{complex128(1e+100 + 1e+100i), complex64(complex(math.Inf(0), math.Inf(0))), false},
		{complex64(complex(math.Inf(0), math.Inf(0))), complex128(1e+100 + 1e+100i), false},
		{complex128(1e+100 + 1e+100i), 270, false},
		{270, complex128(1e+100 + 1e+100i), false},
		{complex128(1e+100 + 1e+100i), 3.14, false},
		{3.14, complex128(1e+100 + 1e+100i), false},
		{complex128(1e+10 + 1e+10i), complex64(1e+10 + 1e+10i), true},
		{complex64(1e+10 + 1e+10i), complex128(1e+10 + 1e+10i), true},
	}

	for _, c := range cases {
		t.Run(fmt.Sprintf("ObjectsAreEqualValues(%#v, %#v)", c.expected, c.actual), func(t *testing.T) {
			res := ObjectsAreEqualValues(c.expected, c.actual)

			if res != c.result {
				t.Errorf("ObjectsAreEqualValues(%#v, %#v) should return %#v", c.expected, c.actual, c.result)
			}
		})
	}
}

type Nested struct {
	Exported    interface{}
	notExported interface{}
}

type S struct {
	Exported1    interface{}
	Exported2    Nested
	notExported1 interface{}
	notExported2 Nested
}

type S2 struct {
	foo interface{}
}

type S3 struct {
	Exported1 *Nested
	Exported2 *Nested
}

type S4 struct {
	Exported1 []*Nested
}

type S5 struct {
	Exported Nested
}

type S6 struct {
	Exported   string
	unexported string
}

func TestObjectsExportedFieldsAreEqual(t *testing.T) {

	intValue := 1

	cases := []struct {
		expected interface{}
		actual   interface{}
		result   bool
	}{
		{S{1, Nested{2, 3}, 4, Nested{5, 6}}, S{1, Nested{2, 3}, 4, Nested{5, 6}}, true},
		{S{1, Nested{2, 3}, 4, Nested{5, 6}}, S{1, Nested{2, 3}, "a", Nested{5, 6}}, true},
		{S{1, Nested{2, 3}, 4, Nested{5, 6}}, S{1, Nested{2, 3}, 4, Nested{5, "a"}}, true},
		{S{1, Nested{2, 3}, 4, Nested{5, 6}}, S{1, Nested{2, 3}, 4, Nested{"a", "a"}}, true},
		{S{1, Nested{2, 3}, 4, Nested{5, 6}}, S{1, Nested{2, "a"}, 4, Nested{5, 6}}, true},
		{S{1, Nested{2, 3}, 4, Nested{5, 6}}, S{"a", Nested{2, 3}, 4, Nested{5, 6}}, false},
		{S{1, Nested{2, 3}, 4, Nested{5, 6}}, S{1, Nested{"a", 3}, 4, Nested{5, 6}}, false},
		{S{1, Nested{2, 3}, 4, Nested{5, 6}}, S2{1}, false},
		{1, S{1, Nested{2, 3}, 4, Nested{5, 6}}, false},

		{S3{&Nested{1, 2}, &Nested{3, 4}}, S3{&Nested{1, 2}, &Nested{3, 4}}, true},
		{S3{nil, &Nested{3, 4}}, S3{nil, &Nested{3, 4}}, true},
		{S3{&Nested{1, 2}, &Nested{3, 4}}, S3{&Nested{1, 2}, &Nested{3, "b"}}, true},
		{S3{&Nested{1, 2}, &Nested{3, 4}}, S3{&Nested{1, "a"}, &Nested{3, "b"}}, true},
		{S3{&Nested{1, 2}, &Nested{3, 4}}, S3{&Nested{"a", 2}, &Nested{3, 4}}, false},
		{S3{&Nested{1, 2}, &Nested{3, 4}}, S3{}, false},
		{S3{}, S3{}, true},

		{S4{[]*Nested{{1, 2}}}, S4{[]*Nested{{1, 2}}}, true},
		{S4{[]*Nested{{1, 2}}}, S4{[]*Nested{{1, 3}}}, true},
		{S4{[]*Nested{{1, 2}, {3, 4}}}, S4{[]*Nested{{1, "a"}, {3, "b"}}}, true},
		{S4{[]*Nested{{1, 2}, {3, 4}}}, S4{[]*Nested{{1, "a"}, {2, "b"}}}, false},

		{Nested{&intValue, 2}, Nested{&intValue, 2}, true},
		{Nested{&Nested{1, 2}, 3}, Nested{&Nested{1, "b"}, 3}, true},
		{Nested{&Nested{1, 2}, 3}, Nested{nil, 3}, false},

		{
			Nested{map[interface{}]*Nested{nil: nil}, 2},
			Nested{map[interface{}]*Nested{nil: nil}, 2},
			true,
		},
		{
			Nested{map[interface{}]*Nested{"a": nil}, 2},
			Nested{map[interface{}]*Nested{"a": nil}, 2},
			true,
		},
		{
			Nested{map[interface{}]*Nested{"a": nil}, 2},
			Nested{map[interface{}]*Nested{"a": {1, 2}}, 2},
			false,
		},
		{
			Nested{map[interface{}]Nested{"a": {1, 2}, "b": {3, 4}}, 2},
			Nested{map[interface{}]Nested{"a": {1, 5}, "b": {3, 7}}, 2},
			true,
		},
		{
			Nested{map[interface{}]Nested{"a": {1, 2}, "b": {3, 4}}, 2},
			Nested{map[interface{}]Nested{"a": {2, 2}, "b": {3, 4}}, 2},
			false,
		},
	}

	for _, c := range cases {
		t.Run(fmt.Sprintf("ObjectsExportedFieldsAreEqual(%#v, %#v)", c.expected, c.actual), func(t *testing.T) {
			res := ObjectsExportedFieldsAreEqual(c.expected, c.actual)

			if res != c.result {
				t.Errorf("ObjectsExportedFieldsAreEqual(%#v, %#v) should return %#v", c.expected, c.actual, c.result)
			}

		})
	}
}

func TestCopyExportedFields(t *testing.T) {
	intValue := 1

	cases := []struct {
		input    interface{}
		expected interface{}
	}{
		{
			input:    Nested{"a", "b"},
			expected: Nested{"a", nil},
		},
		{
			input:    Nested{&intValue, 2},
			expected: Nested{&intValue, nil},
		},
		{
			input:    Nested{nil, 3},
			expected: Nested{nil, nil},
		},
		{
			input:    S{1, Nested{2, 3}, 4, Nested{5, 6}},
			expected: S{1, Nested{2, nil}, nil, Nested{}},
		},
		{
			input:    S3{},
			expected: S3{},
		},
		{
			input:    S3{&Nested{1, 2}, &Nested{3, 4}},
			expected: S3{&Nested{1, nil}, &Nested{3, nil}},
		},
		{
			input:    S3{Exported1: &Nested{"a", "b"}},
			expected: S3{Exported1: &Nested{"a", nil}},
		},
		{
			input: S4{[]*Nested{
				nil,
				{1, 2},
			}},
			expected: S4{[]*Nested{
				nil,
				{1, nil},
			}},
		},
		{
			input: S4{[]*Nested{
				{1, 2}},
			},
			expected: S4{[]*Nested{
				{1, nil}},
			},
		},
		{
			input: S4{[]*Nested{
				{1, 2},
				{3, 4},
			}},
			expected: S4{[]*Nested{
				{1, nil},
				{3, nil},
			}},
		},
		{
			input:    S5{Exported: Nested{"a", "b"}},
			expected: S5{Exported: Nested{"a", nil}},
		},
		{
			input:    S6{"a", "b"},
			expected: S6{"a", ""},
		},
	}

	for _, c := range cases {
		t.Run("", func(t *testing.T) {
			output := copyExportedFields(c.input)
			if !ObjectsAreEqualValues(c.expected, output) {
				t.Errorf("%#v, %#v should be equal", c.expected, output)
			}
		})
	}
}

func TestEqualExportedValues(t *testing.T) {
	cases := []struct {
		value1        interface{}
		value2        interface{}
		expectedEqual bool
		expectedFail  string
	}{
		{
			value1:        S{1, Nested{2, 3}, 4, Nested{5, 6}},
			value2:        S{1, Nested{2, nil}, nil, Nested{}},
			expectedEqual: true,
		},
		{
			value1:        S{1, Nested{2, 3}, 4, Nested{5, 6}},
			value2:        S{1, Nested{1, nil}, nil, Nested{}},
			expectedEqual: false,
			expectedFail: `
	            	Diff:
	            	--- Expected
	            	+++ Actual
	            	@@ -3,3 +3,3 @@
	            	  Exported2: (assert.Nested) {
	            	-  Exported: (int) 2,
	            	+  Exported: (int) 1,
	            	   notExported: (interface {}) <nil>`,
		},
		{
			value1:        S3{&Nested{1, 2}, &Nested{3, 4}},
			value2:        S3{&Nested{"a", 2}, &Nested{3, 4}},
			expectedEqual: false,
			expectedFail: `
	            	Diff:
	            	--- Expected
	            	+++ Actual
	            	@@ -2,3 +2,3 @@
	            	  Exported1: (*assert.Nested)({
	            	-  Exported: (int) 1,
	            	+  Exported: (string) (len=1) "a",
	            	   notExported: (interface {}) <nil>`,
		},
		{
			value1: S4{[]*Nested{
				{1, 2},
				{3, 4},
			}},
			value2: S4{[]*Nested{
				{1, "a"},
				{2, "b"},
			}},
			expectedEqual: false,
			expectedFail: `
	            	Diff:
	            	--- Expected
	            	+++ Actual
	            	@@ -7,3 +7,3 @@
	            	   (*assert.Nested)({
	            	-   Exported: (int) 3,
	            	+   Exported: (int) 2,
	            	    notExported: (interface {}) <nil>`,
		},
		{
			value1:        S{[2]int{1, 2}, Nested{2, 3}, 4, Nested{5, 6}},
			value2:        S{[2]int{1, 2}, Nested{2, nil}, nil, Nested{}},
			expectedEqual: true,
		},
		{
			value1:        &S{1, Nested{2, 3}, 4, Nested{5, 6}},
			value2:        &S{1, Nested{2, nil}, nil, Nested{}},
			expectedEqual: true,
		},
		{
			value1:        &S{1, Nested{2, 3}, 4, Nested{5, 6}},
			value2:        &S{1, Nested{1, nil}, nil, Nested{}},
			expectedEqual: false,
			expectedFail: `
	            	Diff:
	            	--- Expected
	            	+++ Actual
	            	@@ -3,3 +3,3 @@
	            	  Exported2: (assert.Nested) {
	            	-  Exported: (int) 2,
	            	+  Exported: (int) 1,
	            	   notExported: (interface {}) <nil>`,
		},
	}

	for _, c := range cases {
		t.Run("", func(t *testing.T) {
			mockT := new(mockTestingT)

			actual := EqualExportedValues(mockT, c.value1, c.value2)
			if actual != c.expectedEqual {
				t.Errorf("Expected EqualExportedValues to be %t, but was %t", c.expectedEqual, actual)
			}

			actualFail := mockT.errorString()
			if !strings.Contains(actualFail, c.expectedFail) {
				t.Errorf("Contains failure should include %q but was %q", c.expectedFail, actualFail)
			}
		})
	}

}

func TestImplements(t *testing.T) {

	mockT := new(testing.T)

	if !Implements(mockT, (*AssertionTesterInterface)(nil), new(AssertionTesterConformingObject)) {
		t.Error("Implements method should return true: AssertionTesterConformingObject implements AssertionTesterInterface")
	}
	if Implements(mockT, (*AssertionTesterInterface)(nil), new(AssertionTesterNonConformingObject)) {
		t.Error("Implements method should return false: AssertionTesterNonConformingObject does not implements AssertionTesterInterface")
	}
	if Implements(mockT, (*AssertionTesterInterface)(nil), nil) {
		t.Error("Implements method should return false: nil does not implement AssertionTesterInterface")
	}

}

func TestNotImplements(t *testing.T) {

	mockT := new(testing.T)

	if !NotImplements(mockT, (*AssertionTesterInterface)(nil), new(AssertionTesterNonConformingObject)) {
		t.Error("NotImplements method should return true: AssertionTesterNonConformingObject does not implement AssertionTesterInterface")
	}
	if NotImplements(mockT, (*AssertionTesterInterface)(nil), new(AssertionTesterConformingObject)) {
		t.Error("NotImplements method should return false: AssertionTesterConformingObject implements AssertionTesterInterface")
	}
	if NotImplements(mockT, (*AssertionTesterInterface)(nil), nil) {
		t.Error("NotImplements method should return false: nil can't be checked to be implementing AssertionTesterInterface or not")
	}

}

func TestIsType(t *testing.T) {

	mockT := new(testing.T)

	if !IsType(mockT, new(AssertionTesterConformingObject), new(AssertionTesterConformingObject)) {
		t.Error("IsType should return true: AssertionTesterConformingObject is the same type as AssertionTesterConformingObject")
	}
	if IsType(mockT, new(AssertionTesterConformingObject), new(AssertionTesterNonConformingObject)) {
		t.Error("IsType should return false: AssertionTesterConformingObject is not the same type as AssertionTesterNonConformingObject")
	}

}

func TestEqual(t *testing.T) {
	type myType string

	mockT := new(testing.T)
	var m map[string]interface{}

	cases := []struct {
		expected interface{}
		actual   interface{}
		result   bool
		remark   string
	}{
		{"Hello World", "Hello World", true, ""},
		{123, 123, true, ""},
		{123.5, 123.5, true, ""},
		{[]byte("Hello World"), []byte("Hello World"), true, ""},
		{nil, nil, true, ""},
		{int32(123), int32(123), true, ""},
		{uint64(123), uint64(123), true, ""},
		{myType("1"), myType("1"), true, ""},
		{&struct{}{}, &struct{}{}, true, "pointer equality is based on equality of underlying value"},

		// Not expected to be equal
		{m["bar"], "something", false, ""},
		{myType("1"), myType("2"), false, ""},

		// A case that might be confusing, especially with numeric literals
		{10, uint(10), false, ""},
	}

	for _, c := range cases {
		t.Run(fmt.Sprintf("Equal(%#v, %#v)", c.expected, c.actual), func(t *testing.T) {
			res := Equal(mockT, c.expected, c.actual)

			if res != c.result {
				t.Errorf("Equal(%#v, %#v) should return %#v: %s", c.expected, c.actual, c.result, c.remark)
			}
		})
	}
}

func ptr(i int) *int {
	return &i
}

func TestSame(t *testing.T) {

	mockT := new(testing.T)

	if Same(mockT, ptr(1), ptr(1)) {
		t.Error("Same should return false")
	}
	if Same(mockT, 1, 1) {
		t.Error("Same should return false")
	}
	p := ptr(2)
	if Same(mockT, p, *p) {
		t.Error("Same should return false")
	}
	if !Same(mockT, p, p) {
		t.Error("Same should return true")
	}
}

func TestNotSame(t *testing.T) {

	mockT := new(testing.T)

	if !NotSame(mockT, ptr(1), ptr(1)) {
		t.Error("NotSame should return true; different pointers")
	}
	if !NotSame(mockT, 1, 1) {
		t.Error("NotSame should return true; constant inputs")
	}
	p := ptr(2)
	if !NotSame(mockT, p, *p) {
		t.Error("NotSame should return true; mixed-type inputs")
	}
	if NotSame(mockT, p, p) {
		t.Error("NotSame should return false")
	}
}

func Test_samePointers(t *testing.T) {
	p := ptr(2)

	type args struct {
		first  interface{}
		second interface{}
	}
	tests := []struct {
		name      string
		args      args
		assertion BoolAssertionFunc
	}{
		{
			name:      "1 != 2",
			args:      args{first: 1, second: 2},
			assertion: False,
		},
		{
			name:      "1 != 1 (not same ptr)",
			args:      args{first: 1, second: 1},
			assertion: False,
		},
		{
			name:      "ptr(1) == ptr(1)",
			args:      args{first: p, second: p},
			assertion: True,
		},
		{
			name:      "int(1) != float32(1)",
			args:      args{first: int(1), second: float32(1)},
			assertion: False,
		},
		{
			name:      "array != slice",
			args:      args{first: [2]int{1, 2}, second: []int{1, 2}},
			assertion: False,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.assertion(t, samePointers(tt.args.first, tt.args.second))
		})
	}
}

// bufferT implements TestingT. Its implementation of Errorf writes the output that would be produced by
// testing.T.Errorf to an internal bytes.Buffer.
type bufferT struct {
	buf bytes.Buffer
}

func (t *bufferT) Errorf(format string, args ...interface{}) {
	// implementation of decorate is copied from testing.T
	decorate := func(s string) string {
		_, file, line, ok := runtime.Caller(3) // decorate + log + public function.
		if ok {
			// Truncate file name at last file name separator.
			if index := strings.LastIndex(file, "/"); index >= 0 {
				file = file[index+1:]
			} else if index = strings.LastIndex(file, "\\"); index >= 0 {
				file = file[index+1:]
			}
		} else {
			file = "???"
			line = 1
		}
		buf := new(bytes.Buffer)
		// Every line is indented at least one tab.
		buf.WriteByte('\t')
		fmt.Fprintf(buf, "%s:%d: ", file, line)
		lines := strings.Split(s, "\n")
		if l := len(lines); l > 1 && lines[l-1] == "" {
			lines = lines[:l-1]
		}
		for i, line := range lines {
			if i > 0 {
				// Second and subsequent lines are indented an extra tab.
				buf.WriteString("\n\t\t")
			}
			buf.WriteString(line)
		}
		buf.WriteByte('\n')
		return buf.String()
	}
	t.buf.WriteString(decorate(fmt.Sprintf(format, args...)))
}

func TestStringEqual(t *testing.T) {
	for i, currCase := range []struct {
		equalWant  string
		equalGot   string
		msgAndArgs []interface{}
		want       string
	}{
		{equalWant: "hi, \nmy name is", equalGot: "what,\nmy name is", want: "\tassertions.go:\\d+: \n\t+Error Trace:\t\n\t+Error:\\s+Not equal:\\s+\n\\s+expected: \"hi, \\\\nmy name is\"\n\\s+actual\\s+: \"what,\\\\nmy name is\"\n\\s+Diff:\n\\s+-+ Expected\n\\s+\\++ Actual\n\\s+@@ -1,2 \\+1,2 @@\n\\s+-hi, \n\\s+\\+what,\n\\s+my name is"},
	} {
		mockT := &bufferT{}
		Equal(mockT, currCase.equalWant, currCase.equalGot, currCase.msgAndArgs...)
		Regexp(t, regexp.MustCompile(currCase.want), mockT.buf.String(), "Case %d", i)
	}
}

func TestEqualFormatting(t *testing.T) {
	for i, currCase := range []struct {
		equalWant  string
		equalGot   string
		msgAndArgs []interface{}
		want       string
	}{
		{equalWant: "want", equalGot: "got", want: "\tassertions.go:\\d+: \n\t+Error Trace:\t\n\t+Error:\\s+Not equal:\\s+\n\\s+expected: \"want\"\n\\s+actual\\s+: \"got\"\n\\s+Diff:\n\\s+-+ Expected\n\\s+\\++ Actual\n\\s+@@ -1 \\+1 @@\n\\s+-want\n\\s+\\+got\n"},
		{equalWant: "want", equalGot: "got", msgAndArgs: []interface{}{"hello, %v!", "world"}, want: "\tassertions.go:[0-9]+: \n\t+Error Trace:\t\n\t+Error:\\s+Not equal:\\s+\n\\s+expected: \"want\"\n\\s+actual\\s+: \"got\"\n\\s+Diff:\n\\s+-+ Expected\n\\s+\\++ Actual\n\\s+@@ -1 \\+1 @@\n\\s+-want\n\\s+\\+got\n\\s+Messages:\\s+hello, world!\n"},
		{equalWant: "want", equalGot: "got", msgAndArgs: []interface{}{123}, want: "\tassertions.go:[0-9]+: \n\t+Error Trace:\t\n\t+Error:\\s+Not equal:\\s+\n\\s+expected: \"want\"\n\\s+actual\\s+: \"got\"\n\\s+Diff:\n\\s+-+ Expected\n\\s+\\++ Actual\n\\s+@@ -1 \\+1 @@\n\\s+-want\n\\s+\\+got\n\\s+Messages:\\s+123\n"},
		{equalWant: "want", equalGot: "got", msgAndArgs: []interface{}{struct{ a string }{"hello"}}, want: "\tassertions.go:[0-9]+: \n\t+Error Trace:\t\n\t+Error:\\s+Not equal:\\s+\n\\s+expected: \"want\"\n\\s+actual\\s+: \"got\"\n\\s+Diff:\n\\s+-+ Expected\n\\s+\\++ Actual\n\\s+@@ -1 \\+1 @@\n\\s+-want\n\\s+\\+got\n\\s+Messages:\\s+{a:hello}\n"},
	} {
		mockT := &bufferT{}
		Equal(mockT, currCase.equalWant, currCase.equalGot, currCase.msgAndArgs...)
		Regexp(t, regexp.MustCompile(currCase.want), mockT.buf.String(), "Case %d", i)
	}
}

func TestFormatUnequalValues(t *testing.T) {
	expected, actual := formatUnequalValues("foo", "bar")
	Equal(t, `"foo"`, expected, "value should not include type")
	Equal(t, `"bar"`, actual, "value should not include type")

	expected, actual = formatUnequalValues(123, 123)
	Equal(t, `123`, expected, "value should not include type")
	Equal(t, `123`, actual, "value should not include type")

	expected, actual = formatUnequalValues(int64(123), int32(123))
	Equal(t, `int64(123)`, expected, "value should include type")
	Equal(t, `int32(123)`, actual, "value should include type")

	expected, actual = formatUnequalValues(int64(123), nil)
	Equal(t, `int64(123)`, expected, "value should include type")
	Equal(t, `<nil>(<nil>)`, actual, "value should include type")

	type testStructType struct {
		Val string
	}

	expected, actual = formatUnequalValues(&testStructType{Val: "test"}, &testStructType{Val: "test"})
	Equal(t, `&assert.testStructType{Val:"test"}`, expected, "value should not include type annotation")
	Equal(t, `&assert.testStructType{Val:"test"}`, actual, "value should not include type annotation")
}

func TestNotNil(t *testing.T) {

	mockT := new(testing.T)

	if !NotNil(mockT, new(AssertionTesterConformingObject)) {
		t.Error("NotNil should return true: object is not nil")
	}
	if NotNil(mockT, nil) {
		t.Error("NotNil should return false: object is nil")
	}
	if NotNil(mockT, (*struct{})(nil)) {
		t.Error("NotNil should return false: object is (*struct{})(nil)")
	}

}

func TestNil(t *testing.T) {

	mockT := new(testing.T)

	if !Nil(mockT, nil) {
		t.Error("Nil should return true: object is nil")
	}
	if !Nil(mockT, (*struct{})(nil)) {
		t.Error("Nil should return true: object is (*struct{})(nil)")
	}
	if Nil(mockT, new(AssertionTesterConformingObject)) {
		t.Error("Nil should return false: object is not nil")
	}

}

func TestTrue(t *testing.T) {

	mockT := new(testing.T)

	if !True(mockT, true) {
		t.Error("True should return true")
	}
	if True(mockT, false) {
		t.Error("True should return false")
	}

}

func TestFalse(t *testing.T) {

	mockT := new(testing.T)

	if !False(mockT, false) {
		t.Error("False should return true")
	}
	if False(mockT, true) {
		t.Error("False should return false")
	}

}

func TestExactly(t *testing.T) {

	mockT := new(testing.T)

	a := float32(1)
	b := float64(1)
	c := float32(1)
	d := float32(2)
	cases := []struct {
		expected interface{}
		actual   interface{}
		result   bool
	}{
		{a, b, false},
		{a, d, false},
		{a, c, true},
		{nil, a, false},
		{a, nil, false},
	}

	for _, c := range cases {
		t.Run(fmt.Sprintf("Exactly(%#v, %#v)", c.expected, c.actual), func(t *testing.T) {
			res := Exactly(mockT, c.expected, c.actual)

			if res != c.result {
				t.Errorf("Exactly(%#v, %#v) should return %#v", c.expected, c.actual, c.result)
			}
		})
	}
}

func TestNotEqual(t *testing.T) {

	mockT := new(testing.T)

	cases := []struct {
		expected interface{}
		actual   interface{}
		result   bool
	}{
		// cases that are expected not to match
		{"Hello World", "Hello World!", true},
		{123, 1234, true},
		{123.5, 123.55, true},
		{[]byte("Hello World"), []byte("Hello World!"), true},
		{nil, new(AssertionTesterConformingObject), true},

		// cases that are expected to match
		{nil, nil, false},
		{"Hello World", "Hello World", false},
		{123, 123, false},
		{123.5, 123.5, false},
		{[]byte("Hello World"), []byte("Hello World"), false},
		{new(AssertionTesterConformingObject), new(AssertionTesterConformingObject), false},
		{&struct{}{}, &struct{}{}, false},
		{func() int { return 23 }, func() int { return 24 }, false},
		// A case that might be confusing, especially with numeric literals
		{int(10), uint(10), true},
	}

	for _, c := range cases {
		t.Run(fmt.Sprintf("NotEqual(%#v, %#v)", c.expected, c.actual), func(t *testing.T) {
			res := NotEqual(mockT, c.expected, c.actual)

			if res != c.result {
				t.Errorf("NotEqual(%#v, %#v) should return %#v", c.expected, c.actual, c.result)
			}
		})
	}
}

func TestNotEqualValues(t *testing.T) {
	mockT := new(testing.T)

	cases := []struct {
		expected interface{}
		actual   interface{}
		result   bool
	}{
		// cases that are expected not to match
		{"Hello World", "Hello World!", true},
		{123, 1234, true},
		{123.5, 123.55, true},
		{[]byte("Hello World"), []byte("Hello World!"), true},
		{nil, new(AssertionTesterConformingObject), true},

		// cases that are expected to match
		{nil, nil, false},
		{"Hello World", "Hello World", false},
		{123, 123, false},
		{123.5, 123.5, false},
		{[]byte("Hello World"), []byte("Hello World"), false},
		{new(AssertionTesterConformingObject), new(AssertionTesterConformingObject), false},
		{&struct{}{}, &struct{}{}, false},

		// Different behavior from NotEqual()
		{func() int { return 23 }, func() int { return 24 }, true},
		{int(10), int(11), true},
		{int(10), uint(10), false},

		{struct{}{}, struct{}{}, false},
	}

	for _, c := range cases {
		t.Run(fmt.Sprintf("NotEqualValues(%#v, %#v)", c.expected, c.actual), func(t *testing.T) {
			res := NotEqualValues(mockT, c.expected, c.actual)

			if res != c.result {
				t.Errorf("NotEqualValues(%#v, %#v) should return %#v", c.expected, c.actual, c.result)
			}
		})
	}
}

func TestContainsNotContains(t *testing.T) {

	type A struct {
		Name, Value string
	}
	list := []string{"Foo", "Bar"}

	complexList := []*A{
		{"b", "c"},
		{"d", "e"},
		{"g", "h"},
		{"j", "k"},
	}
	simpleMap := map[interface{}]interface{}{"Foo": "Bar"}
	var zeroMap map[interface{}]interface{}

	cases := []struct {
		expected interface{}
		actual   interface{}
		result   bool
	}{
		{"Hello World", "Hello", true},
		{"Hello World", "Salut", false},
		{list, "Bar", true},
		{list, "Salut", false},
		{complexList, &A{"g", "h"}, true},
		{complexList, &A{"g", "e"}, false},
		{simpleMap, "Foo", true},
		{simpleMap, "Bar", false},
		{zeroMap, "Bar", false},
	}

	for _, c := range cases {
		t.Run(fmt.Sprintf("Contains(%#v, %#v)", c.expected, c.actual), func(t *testing.T) {
			mockT := new(testing.T)
			res := Contains(mockT, c.expected, c.actual)

			if res != c.result {
				if res {
					t.Errorf("Contains(%#v, %#v) should return true:\n\t%#v contains %#v", c.expected, c.actual, c.expected, c.actual)
				} else {
					t.Errorf("Contains(%#v, %#v) should return false:\n\t%#v does not contain %#v", c.expected, c.actual, c.expected, c.actual)
				}
			}
		})
	}

	for _, c := range cases {
		t.Run(fmt.Sprintf("NotContains(%#v, %#v)", c.expected, c.actual), func(t *testing.T) {
			mockT := new(testing.T)
			res := NotContains(mockT, c.expected, c.actual)

			// NotContains should be inverse of Contains. If it's not, something is wrong
			if res == Contains(mockT, c.expected, c.actual) {
				if res {
					t.Errorf("NotContains(%#v, %#v) should return true:\n\t%#v does not contains %#v", c.expected, c.actual, c.expected, c.actual)
				} else {
					t.Errorf("NotContains(%#v, %#v) should return false:\n\t%#v contains %#v", c.expected, c.actual, c.expected, c.actual)
				}
			}
		})
	}
}

func TestContainsNotContainsFailMessage(t *testing.T) {
	mockT := new(mockTestingT)

	type nonContainer struct {
		Value string
	}

	cases := []struct {
		assertion func(t TestingT, s, contains interface{}, msgAndArgs ...interface{}) bool
		container interface{}
		instance  interface{}
		expected  string
	}{
		{
			assertion: Contains,
			container: "Hello World",
			instance:  errors.New("Hello"),
			expected:  "\"Hello World\" does not contain &errors.errorString{s:\"Hello\"}",
		},
		{
			assertion: Contains,
			container: map[string]int{"one": 1},
			instance:  "two",
			expected:  "map[string]int{\"one\":1} does not contain \"two\"\n",
		},
		{
			assertion: NotContains,
			container: map[string]int{"one": 1},
			instance:  "one",
			expected:  "map[string]int{\"one\":1} should not contain \"one\"",
		},
		{
			assertion: Contains,
			container: nonContainer{Value: "Hello"},
			instance:  "Hello",
			expected:  "assert.nonContainer{Value:\"Hello\"} could not be applied builtin len()\n",
		},
		{
			assertion: NotContains,
			container: nonContainer{Value: "Hello"},
			instance:  "Hello",
			expected:  "assert.nonContainer{Value:\"Hello\"} could not be applied builtin len()\n",
		},
	}
	for _, c := range cases {
		name := filepath.Base(runtime.FuncForPC(reflect.ValueOf(c.assertion).Pointer()).Name())
		t.Run(fmt.Sprintf("%v(%T, %T)", name, c.container, c.instance), func(t *testing.T) {
			c.assertion(mockT, c.container, c.instance)
			actualFail := mockT.errorString()
			if !strings.Contains(actualFail, c.expected) {
				t.Errorf("Contains failure should include %q but was %q", c.expected, actualFail)
			}
		})
	}
}

func TestContainsNotContainsOnNilValue(t *testing.T) {
	mockT := new(mockTestingT)

	Contains(mockT, nil, "key")
	expectedFail := "<nil> could not be applied builtin len()"
	actualFail := mockT.errorString()
	if !strings.Contains(actualFail, expectedFail) {
		t.Errorf("Contains failure should include %q but was %q", expectedFail, actualFail)
	}

	NotContains(mockT, nil, "key")
	if !strings.Contains(actualFail, expectedFail) {
		t.Errorf("Contains failure should include %q but was %q", expectedFail, actualFail)
	}
}

func TestSubsetNotSubset(t *testing.T) {
	cases := []struct {
		list    interface{}
		subset  interface{}
		result  bool
		message string
	}{
		// cases that are expected to contain
		{[]int{1, 2, 3}, nil, true, `nil is the empty set which is a subset of every set`},
		{[]int{1, 2, 3}, []int{}, true, `[] is a subset of ['\x01' '\x02' '\x03']`},
		{[]int{1, 2, 3}, []int{1, 2}, true, `['\x01' '\x02'] is a subset of ['\x01' '\x02' '\x03']`},
		{[]int{1, 2, 3}, []int{1, 2, 3}, true, `['\x01' '\x02' '\x03'] is a subset of ['\x01' '\x02' '\x03']`},
		{[]string{"hello", "world"}, []string{"hello"}, true, `["hello"] is a subset of ["hello" "world"]`},
		{map[string]string{
			"a": "x",
			"c": "z",
			"b": "y",
		}, map[string]string{
			"a": "x",
			"b": "y",
		}, true, `map["a":"x" "b":"y"] is a subset of map["a":"x" "b":"y" "c":"z"]`},

		// cases that are expected not to contain
		{[]string{"hello", "world"}, []string{"hello", "testify"}, false, `[]string{"hello", "world"} does not contain "testify"`},
		{[]int{1, 2, 3}, []int{4, 5}, false, `[]int{1, 2, 3} does not contain 4`},
		{[]int{1, 2, 3}, []int{1, 5}, false, `[]int{1, 2, 3} does not contain 5`},
		{map[string]string{
			"a": "x",
			"c": "z",
			"b": "y",
		}, map[string]string{
			"a": "x",
			"b": "z",
		}, false, `map[string]string{"a":"x", "b":"y", "c":"z"} does not contain map[string]string{"a":"x", "b":"z"}`},
		{map[string]string{
			"a": "x",
			"b": "y",
		}, map[string]string{
			"a": "x",
			"b": "y",
			"c": "z",
		}, false, `map[string]string{"a":"x", "b":"y"} does not contain map[string]string{"a":"x", "b":"y", "c":"z"}`},
	}

	for _, c := range cases {
		t.Run("SubSet: "+c.message, func(t *testing.T) {

			mockT := new(mockTestingT)
			res := Subset(mockT, c.list, c.subset)

			if res != c.result {
				t.Errorf("Subset should return %t: %s", c.result, c.message)
			}
			if !c.result {
				expectedFail := c.message
				actualFail := mockT.errorString()
				if !strings.Contains(actualFail, expectedFail) {
					t.Log(actualFail)
					t.Errorf("Subset failure should contain %q but was %q", expectedFail, actualFail)
				}
			}
		})
	}
	for _, c := range cases {
		t.Run("NotSubSet: "+c.message, func(t *testing.T) {
			mockT := new(mockTestingT)
			res := NotSubset(mockT, c.list, c.subset)

			// NotSubset should match the inverse of Subset. If it doesn't, something is wrong
			if res == Subset(mockT, c.list, c.subset) {
				t.Errorf("NotSubset should return %t: %s", !c.result, c.message)
			}
			if c.result {
				expectedFail := c.message
				actualFail := mockT.errorString()
				if !strings.Contains(actualFail, expectedFail) {
					t.Log(actualFail)
					t.Errorf("NotSubset failure should contain %q but was %q", expectedFail, actualFail)
				}
			}
		})
	}
}

func TestNotSubsetNil(t *testing.T) {
	mockT := new(testing.T)
	NotSubset(mockT, []string{"foo"}, nil)
	if !mockT.Failed() {
		t.Error("NotSubset on nil set should have failed the test")
	}
}

func Test_containsElement(t *testing.T) {

	list1 := []string{"Foo", "Bar"}
	list2 := []int{1, 2}
	simpleMap := map[interface{}]interface{}{"Foo": "Bar"}

	ok, found := containsElement("Hello World", "World")
	True(t, ok)
	True(t, found)

	ok, found = containsElement(list1, "Foo")
	True(t, ok)
	True(t, found)

	ok, found = containsElement(list1, "Bar")
	True(t, ok)
	True(t, found)

	ok, found = containsElement(list2, 1)
	True(t, ok)
	True(t, found)

	ok, found = containsElement(list2, 2)
	True(t, ok)
	True(t, found)

	ok, found = containsElement(list1, "Foo!")
	True(t, ok)
	False(t, found)

	ok, found = containsElement(list2, 3)
	True(t, ok)
	False(t, found)

	ok, found = containsElement(list2, "1")
	True(t, ok)
	False(t, found)

	ok, found = containsElement(simpleMap, "Foo")
	True(t, ok)
	True(t, found)

	ok, found = containsElement(simpleMap, "Bar")
	True(t, ok)
	False(t, found)

	ok, found = containsElement(1433, "1")
	False(t, ok)
	False(t, found)
}

func TestElementsMatch(t *testing.T) {
	mockT := new(testing.T)

	cases := []struct {
		expected interface{}
		actual   interface{}
		result   bool
	}{
		// matching
		{nil, nil, true},

		{nil, nil, true},
		{[]int{}, []int{}, true},
		{[]int{1}, []int{1}, true},
		{[]int{1, 1}, []int{1, 1}, true},
		{[]int{1, 2}, []int{1, 2}, true},
		{[]int{1, 2}, []int{2, 1}, true},
		{[2]int{1, 2}, [2]int{2, 1}, true},
		{[]string{"hello", "world"}, []string{"world", "hello"}, true},
		{[]string{"hello", "hello"}, []string{"hello", "hello"}, true},
		{[]string{"hello", "hello", "world"}, []string{"hello", "world", "hello"}, true},
		{[3]string{"hello", "hello", "world"}, [3]string{"hello", "world", "hello"}, true},
		{[]int{}, nil, true},

		// not matching
		{[]int{1}, []int{1, 1}, false},
		{[]int{1, 2}, []int{2, 2}, false},
		{[]string{"hello", "hello"}, []string{"hello"}, false},
	}

	for _, c := range cases {
		t.Run(fmt.Sprintf("ElementsMatch(%#v, %#v)", c.expected, c.actual), func(t *testing.T) {
			res := ElementsMatch(mockT, c.actual, c.expected)

			if res != c.result {
				t.Errorf("ElementsMatch(%#v, %#v) should return %v", c.actual, c.expected, c.result)
			}
		})
	}
}

func TestDiffLists(t *testing.T) {
	tests := []struct {
		name   string
		listA  interface{}
		listB  interface{}
		extraA []interface{}
		extraB []interface{}
	}{
		{
			name:   "equal empty",
			listA:  []string{},
			listB:  []string{},
			extraA: nil,
			extraB: nil,
		},
		{
			name:   "equal same order",
			listA:  []string{"hello", "world"},
			listB:  []string{"hello", "world"},
			extraA: nil,
			extraB: nil,
		},
		{
			name:   "equal different order",
			listA:  []string{"hello", "world"},
			listB:  []string{"world", "hello"},
			extraA: nil,
			extraB: nil,
		},
		{
			name:   "extra A",
			listA:  []string{"hello", "hello", "world"},
			listB:  []string{"hello", "world"},
			extraA: []interface{}{"hello"},
			extraB: nil,
		},
		{
			name:   "extra A twice",
			listA:  []string{"hello", "hello", "hello", "world"},
			listB:  []string{"hello", "world"},
			extraA: []interface{}{"hello", "hello"},
			extraB: nil,
		},
		{
			name:   "extra B",
			listA:  []string{"hello", "world"},
			listB:  []string{"hello", "hello", "world"},
			extraA: nil,
			extraB: []interface{}{"hello"},
		},
		{
			name:   "extra B twice",
			listA:  []string{"hello", "world"},
			listB:  []string{"hello", "hello", "world", "hello"},
			extraA: nil,
			extraB: []interface{}{"hello", "hello"},
		},
		{
			name:   "integers 1",
			listA:  []int{1, 2, 3, 4, 5},
			listB:  []int{5, 4, 3, 2, 1},
			extraA: nil,
			extraB: nil,
		},
		{
			name:   "integers 2",
			listA:  []int{1, 2, 1, 2, 1},
			listB:  []int{2, 1, 2, 1, 2},
			extraA: []interface{}{1},
			extraB: []interface{}{2},
		},
	}
	for _, test := range tests {
		test := test
		t.Run(test.name, func(t *testing.T) {
			actualExtraA, actualExtraB := diffLists(test.listA, test.listB)
			Equal(t, test.extraA, actualExtraA, "extra A does not match for listA=%v listB=%v",
				test.listA, test.listB)
			Equal(t, test.extraB, actualExtraB, "extra B does not match for listA=%v listB=%v",
				test.listA, test.listB)
		})
	}
}

func TestCondition(t *testing.T) {
	mockT := new(testing.T)

	if !Condition(mockT, func() bool { return true }, "Truth") {
		t.Error("Condition should return true")
	}

	if Condition(mockT, func() bool { return false }, "Lie") {
		t.Error("Condition should return false")
	}

}

func TestDidPanic(t *testing.T) {

	const panicMsg = "Panic!"

	if funcDidPanic, msg, _ := didPanic(func() {
		panic(panicMsg)
	}); !funcDidPanic || msg != panicMsg {
		t.Error("didPanic should return true, panicMsg")
	}

	if funcDidPanic, msg, _ := didPanic(func() {
		panic(nil)
	}); !funcDidPanic || msg != nil {
		t.Error("didPanic should return true, nil")
	}

	if funcDidPanic, _, _ := didPanic(func() {
	}); funcDidPanic {
		t.Error("didPanic should return false")
	}

}

func TestPanics(t *testing.T) {

	mockT := new(testing.T)

	if !Panics(mockT, func() {
		panic("Panic!")
	}) {
		t.Error("Panics should return true")
	}

	if Panics(mockT, func() {
	}) {
		t.Error("Panics should return false")
	}

}

func TestPanicsWithValue(t *testing.T) {

	mockT := new(testing.T)

	if !PanicsWithValue(mockT, "Panic!", func() {
		panic("Panic!")
	}) {
		t.Error("PanicsWithValue should return true")
	}

	if !PanicsWithValue(mockT, nil, func() {
		panic(nil)
	}) {
		t.Error("PanicsWithValue should return true")
	}

	if PanicsWithValue(mockT, "Panic!", func() {
	}) {
		t.Error("PanicsWithValue should return false")
	}

	if PanicsWithValue(mockT, "at the disco", func() {
		panic("Panic!")
	}) {
		t.Error("PanicsWithValue should return false")
	}
}

func TestPanicsWithError(t *testing.T) {

	mockT := new(testing.T)

	if !PanicsWithError(mockT, "panic", func() {
		panic(errors.New("panic"))
	}) {
		t.Error("PanicsWithError should return true")
	}

	if PanicsWithError(mockT, "Panic!", func() {
	}) {
		t.Error("PanicsWithError should return false")
	}

	if PanicsWithError(mockT, "at the disco", func() {
		panic(errors.New("panic"))
	}) {
		t.Error("PanicsWithError should return false")
	}

	if PanicsWithError(mockT, "Panic!", func() {
		panic("panic")
	}) {
		t.Error("PanicsWithError should return false")
	}
}

func TestNotPanics(t *testing.T) {

	mockT := new(testing.T)

	if !NotPanics(mockT, func() {
	}) {
		t.Error("NotPanics should return true")
	}

	if NotPanics(mockT, func() {
		panic("Panic!")
	}) {
		t.Error("NotPanics should return false")
	}

}

func TestNoError(t *testing.T) {

	mockT := new(testing.T)

	// start with a nil error
	var err error

	True(t, NoError(mockT, err), "NoError should return True for nil arg")

	// now set an error
	err = errors.New("some error")

	False(t, NoError(mockT, err), "NoError with error should return False")

	// returning an empty error interface
	err = func() error {
		var err *customError
		return err
	}()

	if err == nil { // err is not nil here!
		t.Errorf("Error should be nil due to empty interface: %s", err)
	}

	False(t, NoError(mockT, err), "NoError should fail with empty error interface")
}

type customError struct{}

func (*customError) Error() string { return "fail" }

func TestError(t *testing.T) {

	mockT := new(testing.T)

	// start with a nil error
	var err error

	False(t, Error(mockT, err), "Error should return False for nil arg")

	// now set an error
	err = errors.New("some error")

	True(t, Error(mockT, err), "Error with error should return True")

	// go vet check
	True(t, Errorf(mockT, err, "example with %s", "formatted message"), "Errorf with error should return True")

	// returning an empty error interface
	err = func() error {
		var err *customError
		return err
	}()

	if err == nil { // err is not nil here!
		t.Errorf("Error should be nil due to empty interface: %s", err)
	}

	True(t, Error(mockT, err), "Error should pass with empty error interface")
}

func TestEqualError(t *testing.T) {
	mockT := new(testing.T)

	// start with a nil error
	var err error
	False(t, EqualError(mockT, err, ""),
		"EqualError should return false for nil arg")

	// now set an error
	err = errors.New("some error")
	False(t, EqualError(mockT, err, "Not some error"),
		"EqualError should return false for different error string")
	True(t, EqualError(mockT, err, "some error"),
		"EqualError should return true")
}

func TestErrorContains(t *testing.T) {
	mockT := new(testing.T)

	// start with a nil error
	var err error
	False(t, ErrorContains(mockT, err, ""),
		"ErrorContains should return false for nil arg")

	// now set an error
	err = errors.New("some error: another error")
	False(t, ErrorContains(mockT, err, "bad error"),
		"ErrorContains should return false for different error string")
	True(t, ErrorContains(mockT, err, "some error"),
		"ErrorContains should return true")
	True(t, ErrorContains(mockT, err, "another error"),
		"ErrorContains should return true")
}

func Test_isEmpty(t *testing.T) {

	chWithValue := make(chan struct{}, 1)
	chWithValue <- struct{}{}

	True(t, isEmpty(""))
	True(t, isEmpty(nil))
	True(t, isEmpty([]string{}))
	True(t, isEmpty(0))
	True(t, isEmpty(int32(0)))
	True(t, isEmpty(int64(0)))
	True(t, isEmpty(false))
	True(t, isEmpty(map[string]string{}))
	True(t, isEmpty(new(time.Time)))
	True(t, isEmpty(time.Time{}))
	True(t, isEmpty(make(chan struct{})))
	True(t, isEmpty([1]int{}))
	False(t, isEmpty("something"))
	False(t, isEmpty(errors.New("something")))
	False(t, isEmpty([]string{"something"}))
	False(t, isEmpty(1))
	False(t, isEmpty(true))
	False(t, isEmpty(map[string]string{"Hello": "World"}))
	False(t, isEmpty(chWithValue))
	False(t, isEmpty([1]int{42}))
}

func TestEmpty(t *testing.T) {

	mockT := new(testing.T)
	chWithValue := make(chan struct{}, 1)
	chWithValue <- struct{}{}
	var tiP *time.Time
	var tiNP time.Time
	var s *string
	var f *os.File
	sP := &s
	x := 1
	xP := &x

	type TString string
	type TStruct struct {
		x int
	}

	True(t, Empty(mockT, ""), "Empty string is empty")
	True(t, Empty(mockT, nil), "Nil is empty")
	True(t, Empty(mockT, []string{}), "Empty string array is empty")
	True(t, Empty(mockT, 0), "Zero int value is empty")
	True(t, Empty(mockT, false), "False value is empty")
	True(t, Empty(mockT, make(chan struct{})), "Channel without values is empty")
	True(t, Empty(mockT, s), "Nil string pointer is empty")
	True(t, Empty(mockT, f), "Nil os.File pointer is empty")
	True(t, Empty(mockT, tiP), "Nil time.Time pointer is empty")
	True(t, Empty(mockT, tiNP), "time.Time is empty")
	True(t, Empty(mockT, TStruct{}), "struct with zero values is empty")
	True(t, Empty(mockT, TString("")), "empty aliased string is empty")
	True(t, Empty(mockT, sP), "ptr to nil value is empty")
	True(t, Empty(mockT, [1]int{}), "array is state")

	False(t, Empty(mockT, "something"), "Non Empty string is not empty")
	False(t, Empty(mockT, errors.New("something")), "Non nil object is not empty")
	False(t, Empty(mockT, []string{"something"}), "Non empty string array is not empty")
	False(t, Empty(mockT, 1), "Non-zero int value is not empty")
	False(t, Empty(mockT, true), "True value is not empty")
	False(t, Empty(mockT, chWithValue), "Channel with values is not empty")
	False(t, Empty(mockT, TStruct{x: 1}), "struct with initialized values is empty")
	False(t, Empty(mockT, TString("abc")), "non-empty aliased string is empty")
	False(t, Empty(mockT, xP), "ptr to non-nil value is not empty")
	False(t, Empty(mockT, [1]int{42}), "array is not state")
}

func TestNotEmpty(t *testing.T) {

	mockT := new(testing.T)
	chWithValue := make(chan struct{}, 1)
	chWithValue <- struct{}{}

	False(t, NotEmpty(mockT, ""), "Empty string is empty")
	False(t, NotEmpty(mockT, nil), "Nil is empty")
	False(t, NotEmpty(mockT, []string{}), "Empty string array is empty")
	False(t, NotEmpty(mockT, 0), "Zero int value is empty")
	False(t, NotEmpty(mockT, false), "False value is empty")
	False(t, NotEmpty(mockT, make(chan struct{})), "Channel without values is empty")
	False(t, NotEmpty(mockT, [1]int{}), "array is state")

	True(t, NotEmpty(mockT, "something"), "Non Empty string is not empty")
	True(t, NotEmpty(mockT, errors.New("something")), "Non nil object is not empty")
	True(t, NotEmpty(mockT, []string{"something"}), "Non empty string array is not empty")
	True(t, NotEmpty(mockT, 1), "Non-zero int value is not empty")
	True(t, NotEmpty(mockT, true), "True value is not empty")
	True(t, NotEmpty(mockT, chWithValue), "Channel with values is not empty")
	True(t, NotEmpty(mockT, [1]int{42}), "array is not state")
}

func Test_getLen(t *testing.T) {
	falseCases := []interface{}{
		nil,
		0,
		true,
		false,
		'A',
		struct{}{},
	}
	for _, v := range falseCases {
		l, ok := getLen(v)
		False(t, ok, "Expected getLen fail to get length of %#v", v)
		Equal(t, 0, l, "getLen should return 0 for %#v", v)
	}

	ch := make(chan int, 5)
	ch <- 1
	ch <- 2
	ch <- 3
	trueCases := []struct {
		v interface{}
		l int
	}{
		{[]int{1, 2, 3}, 3},
		{[...]int{1, 2, 3}, 3},
		{"ABC", 3},
		{map[int]int{1: 2, 2: 4, 3: 6}, 3},
		{ch, 3},

		{[]int{}, 0},
		{map[int]int{}, 0},
		{make(chan int), 0},

		{[]int(nil), 0},
		{map[int]int(nil), 0},
		{(chan int)(nil), 0},
	}

	for _, c := range trueCases {
		l, ok := getLen(c.v)
		True(t, ok, "Expected getLen success to get length of %#v", c.v)
		Equal(t, c.l, l)
	}
}

func TestLen(t *testing.T) {
	mockT := new(testing.T)

	False(t, Len(mockT, nil, 0), "nil does not have length")
	False(t, Len(mockT, 0, 0), "int does not have length")
	False(t, Len(mockT, true, 0), "true does not have length")
	False(t, Len(mockT, false, 0), "false does not have length")
	False(t, Len(mockT, 'A', 0), "Rune does not have length")
	False(t, Len(mockT, struct{}{}, 0), "Struct does not have length")

	ch := make(chan int, 5)
	ch <- 1
	ch <- 2
	ch <- 3

	cases := []struct {
		v               interface{}
		l               int
		expected1234567 string // message when expecting 1234567 items
	}{
		{[]int{1, 2, 3}, 3, `"[1 2 3]" should have 1234567 item(s), but has 3`},
		{[...]int{1, 2, 3}, 3, `"[1 2 3]" should have 1234567 item(s), but has 3`},
		{"ABC", 3, `"ABC" should have 1234567 item(s), but has 3`},
		{map[int]int{1: 2, 2: 4, 3: 6}, 3, `"map[1:2 2:4 3:6]" should have 1234567 item(s), but has 3`},
		{ch, 3, ""},

		{[]int{}, 0, `"[]" should have 1234567 item(s), but has 0`},
		{map[int]int{}, 0, `"map[]" should have 1234567 item(s), but has 0`},
		{make(chan int), 0, ""},

		{[]int(nil), 0, `"[]" should have 1234567 item(s), but has 0`},
		{map[int]int(nil), 0, `"map[]" should have 1234567 item(s), but has 0`},
		{(chan int)(nil), 0, `"<nil>" should have 1234567 item(s), but has 0`},
	}

	for _, c := range cases {
		True(t, Len(mockT, c.v, c.l), "%#v have %d items", c.v, c.l)
		False(t, Len(mockT, c.v, c.l+1), "%#v have %d items", c.v, c.l)
		if c.expected1234567 != "" {
			msgMock := new(mockTestingT)
			Len(msgMock, c.v, 1234567)
			Contains(t, msgMock.errorString(), c.expected1234567)
		}
	}
}

func TestWithinDuration(t *testing.T) {

	mockT := new(testing.T)
	a := time.Now()
	b := a.Add(10 * time.Second)

	True(t, WithinDuration(mockT, a, b, 10*time.Second), "A 10s difference is within a 10s time difference")
	True(t, WithinDuration(mockT, b, a, 10*time.Second), "A 10s difference is within a 10s time difference")

	False(t, WithinDuration(mockT, a, b, 9*time.Second), "A 10s difference is not within a 9s time difference")
	False(t, WithinDuration(mockT, b, a, 9*time.Second), "A 10s difference is not within a 9s time difference")

	False(t, WithinDuration(mockT, a, b, -9*time.Second), "A 10s difference is not within a 9s time difference")
	False(t, WithinDuration(mockT, b, a, -9*time.Second), "A 10s difference is not within a 9s time difference")

	False(t, WithinDuration(mockT, a, b, -11*time.Second), "A 10s difference is not within a 9s time difference")
	False(t, WithinDuration(mockT, b, a, -11*time.Second), "A 10s difference is not within a 9s time difference")
}

func TestWithinRange(t *testing.T) {

	mockT := new(testing.T)
	n := time.Now()
	s := n.Add(-time.Second)
	e := n.Add(time.Second)

	True(t, WithinRange(mockT, n, n, n), "Exact same actual, start, and end values return true")

	True(t, WithinRange(mockT, n, s, e), "Time in range is within the time range")
	True(t, WithinRange(mockT, s, s, e), "The start time is within the time range")
	True(t, WithinRange(mockT, e, s, e), "The end time is within the time range")

	False(t, WithinRange(mockT, s.Add(-time.Nanosecond), s, e, "Just before the start time is not within the time range"))
	False(t, WithinRange(mockT, e.Add(time.Nanosecond), s, e, "Just after the end time is not within the time range"))

	False(t, WithinRange(mockT, n, e, s, "Just after the end time is not within the time range"))
}

func TestInDelta(t *testing.T) {
	mockT := new(testing.T)

	True(t, InDelta(mockT, 1.001, 1, 0.01), "|1.001 - 1| <= 0.01")
	True(t, InDelta(mockT, 1, 1.001, 0.01), "|1 - 1.001| <= 0.01")
	True(t, InDelta(mockT, 1, 2, 1), "|1 - 2| <= 1")
	False(t, InDelta(mockT, 1, 2, 0.5), "Expected |1 - 2| <= 0.5 to fail")
	False(t, InDelta(mockT, 2, 1, 0.5), "Expected |2 - 1| <= 0.5 to fail")
	False(t, InDelta(mockT, "", nil, 1), "Expected non numerals to fail")
	False(t, InDelta(mockT, 42, math.NaN(), 0.01), "Expected NaN for actual to fail")
	False(t, InDelta(mockT, math.NaN(), 42, 0.01), "Expected NaN for expected to fail")
	True(t, InDelta(mockT, math.NaN(), math.NaN(), 0.01), "Expected NaN for both to pass")

	cases := []struct {
		a, b  interface{}
		delta float64
	}{
		{uint(2), uint(1), 1},
		{uint8(2), uint8(1), 1},
		{uint16(2), uint16(1), 1},
		{uint32(2), uint32(1), 1},
		{uint64(2), uint64(1), 1},

		{int(2), int(1), 1},
		{int8(2), int8(1), 1},
		{int16(2), int16(1), 1},
		{int32(2), int32(1), 1},
		{int64(2), int64(1), 1},

		{float32(2), float32(1), 1},
		{float64(2), float64(1), 1},
	}

	for _, tc := range cases {
		True(t, InDelta(mockT, tc.a, tc.b, tc.delta), "Expected |%V - %V| <= %v", tc.a, tc.b, tc.delta)
	}
}

func TestInDeltaSlice(t *testing.T) {
	mockT := new(testing.T)

	True(t, InDeltaSlice(mockT,
		[]float64{1.001, math.NaN(), 0.999},
		[]float64{1, math.NaN(), 1},
		0.1), "{1.001, NaN, 0.009} is element-wise close to {1, NaN, 1} in delta=0.1")

	True(t, InDeltaSlice(mockT,
		[]float64{1, math.NaN(), 2},
		[]float64{0, math.NaN(), 3},
		1), "{1, NaN, 2} is element-wise close to {0, NaN, 3} in delta=1")

	False(t, InDeltaSlice(mockT,
		[]float64{1, math.NaN(), 2},
		[]float64{0, math.NaN(), 3},
		0.1), "{1, NaN, 2} is not element-wise close to {0, NaN, 3} in delta=0.1")

	False(t, InDeltaSlice(mockT, "", nil, 1), "Expected non numeral slices to fail")
}

func TestInDeltaMapValues(t *testing.T) {
	mockT := new(testing.T)

	for _, tc := range []struct {
		title  string
		expect interface{}
		actual interface{}
		f      func(TestingT, bool, ...interface{}) bool
		delta  float64
	}{
		{
			title: "Within delta",
			expect: map[string]float64{
				"foo": 1.0,
				"bar": 2.0,
				"baz": math.NaN(),
			},
			actual: map[string]float64{
				"foo": 1.01,
				"bar": 1.99,
				"baz": math.NaN(),
			},
			delta: 0.1,
			f:     True,
		},
		{
			title: "Within delta",
			expect: map[int]float64{
				1: 1.0,
				2: 2.0,
			},
			actual: map[int]float64{
				1: 1.0,
				2: 1.99,
			},
			delta: 0.1,
			f:     True,
		},
		{
			title: "Different number of keys",
			expect: map[int]float64{
				1: 1.0,
				2: 2.0,
			},
			actual: map[int]float64{
				1: 1.0,
			},
			delta: 0.1,
			f:     False,
		},
		{
			title: "Within delta with zero value",
			expect: map[string]float64{
				"zero": 0,
			},
			actual: map[string]float64{
				"zero": 0,
			},
			delta: 0.1,
			f:     True,
		},
		{
			title: "With missing key with zero value",
			expect: map[string]float64{
				"zero": 0,
				"foo":  0,
			},
			actual: map[string]float64{
				"zero": 0,
				"bar":  0,
			},
			f: False,
		},
	} {
		tc.f(t, InDeltaMapValues(mockT, tc.expect, tc.actual, tc.delta), tc.title+"\n"+diff(tc.expect, tc.actual))
	}
}

func TestInEpsilon(t *testing.T) {
	mockT := new(testing.T)

	cases := []struct {
		a, b    interface{}
		epsilon float64
	}{
		{uint8(2), uint16(2), .001},
		{2.1, 2.2, 0.1},
		{2.2, 2.1, 0.1},
		{-2.1, -2.2, 0.1},
		{-2.2, -2.1, 0.1},
		{uint64(100), uint8(101), 0.01},
		{0.1, -0.1, 2},
		{0.1, 0, 2},
		{math.NaN(), math.NaN(), 1},
		{time.Second, time.Second + time.Millisecond, 0.002},
	}

	for _, tc := range cases {
		True(t, InEpsilon(t, tc.a, tc.b, tc.epsilon, "Expected %V and %V to have a relative difference of %v", tc.a, tc.b, tc.epsilon), "test: %q", tc)
	}

	cases = []struct {
		a, b    interface{}
		epsilon float64
	}{
		{uint8(2), int16(-2), .001},
		{uint64(100), uint8(102), 0.01},
		{2.1, 2.2, 0.001},
		{2.2, 2.1, 0.001},
		{2.1, -2.2, 1},
		{2.1, "bla-bla", 0},
		{0.1, -0.1, 1.99},
		{0, 0.1, 2}, // expected must be different to zero
		{time.Second, time.Second + 10*time.Millisecond, 0.002},
		{math.NaN(), 0, 1},
		{0, math.NaN(), 1},
		{0, 0, math.NaN()},
	}

	for _, tc := range cases {
		False(t, InEpsilon(mockT, tc.a, tc.b, tc.epsilon, "Expected %V and %V to have a relative difference of %v", tc.a, tc.b, tc.epsilon))
	}

}

func TestInEpsilonSlice(t *testing.T) {
	mockT := new(testing.T)

	True(t, InEpsilonSlice(mockT,
		[]float64{2.2, math.NaN(), 2.0},
		[]float64{2.1, math.NaN(), 2.1},
		0.06), "{2.2, NaN, 2.0} is element-wise close to {2.1, NaN, 2.1} in epsilon=0.06")

	False(t, InEpsilonSlice(mockT,
		[]float64{2.2, 2.0},
		[]float64{2.1, 2.1},
		0.04), "{2.2, 2.0} is not element-wise close to {2.1, 2.1} in epsilon=0.04")

	False(t, InEpsilonSlice(mockT, "", nil, 1), "Expected non numeral slices to fail")
}

func TestRegexp(t *testing.T) {
	mockT := new(testing.T)

	cases := []struct {
		rx, str string
	}{
		{"^start", "start of the line"},
		{"end$", "in the end"},
		{"[0-9]{3}[.-]?[0-9]{2}[.-]?[0-9]{2}", "My phone number is 650.12.34"},
	}

	for _, tc := range cases {
		True(t, Regexp(mockT, tc.rx, tc.str))
		True(t, Regexp(mockT, regexp.MustCompile(tc.rx), tc.str))
		False(t, NotRegexp(mockT, tc.rx, tc.str))
		False(t, NotRegexp(mockT, regexp.MustCompile(tc.rx), tc.str))
	}

	cases = []struct {
		rx, str string
	}{
		{"^asdfastart", "Not the start of the line"},
		{"end$", "in the end."},
		{"[0-9]{3}[.-]?[0-9]{2}[.-]?[0-9]{2}", "My phone number is 650.12a.34"},
	}

	for _, tc := range cases {
		False(t, Regexp(mockT, tc.rx, tc.str), "Expected \"%s\" to not match \"%s\"", tc.rx, tc.str)
		False(t, Regexp(mockT, regexp.MustCompile(tc.rx), tc.str))
		True(t, NotRegexp(mockT, tc.rx, tc.str))
		True(t, NotRegexp(mockT, regexp.MustCompile(tc.rx), tc.str))
	}
}

func testAutogeneratedFunction() {
	defer func() {
		if err := recover(); err == nil {
			panic("did not panic")
		}
		CallerInfo()
	}()
	t := struct {
		io.Closer
	}{}
	c := t
	c.Close()
}

func TestCallerInfoWithAutogeneratedFunctions(t *testing.T) {
	NotPanics(t, func() {
		testAutogeneratedFunction()
	})
}

func TestZero(t *testing.T) {
	mockT := new(testing.T)

	for _, test := range zeros {
		True(t, Zero(mockT, test, "%#v is not the %v zero value", test, reflect.TypeOf(test)))
	}

	for _, test := range nonZeros {
		False(t, Zero(mockT, test, "%#v is not the %v zero value", test, reflect.TypeOf(test)))
	}
}

func TestNotZero(t *testing.T) {
	mockT := new(testing.T)

	for _, test := range zeros {
		False(t, NotZero(mockT, test, "%#v is not the %v zero value", test, reflect.TypeOf(test)))
	}

	for _, test := range nonZeros {
		True(t, NotZero(mockT, test, "%#v is not the %v zero value", test, reflect.TypeOf(test)))
	}
}

func TestFileExists(t *testing.T) {
	mockT := new(testing.T)
	True(t, FileExists(mockT, "assertions.go"))

	mockT = new(testing.T)
	False(t, FileExists(mockT, "random_file"))

	mockT = new(testing.T)
	False(t, FileExists(mockT, "../_codegen"))

	var tempFiles []string

	link, err := getTempSymlinkPath("assertions.go")
	if err != nil {
		t.Fatal("could not create temp symlink, err:", err)
	}
	tempFiles = append(tempFiles, link)
	mockT = new(testing.T)
	True(t, FileExists(mockT, link))

	link, err = getTempSymlinkPath("non_existent_file")
	if err != nil {
		t.Fatal("could not create temp symlink, err:", err)
	}
	tempFiles = append(tempFiles, link)
	mockT = new(testing.T)
	True(t, FileExists(mockT, link))

	errs := cleanUpTempFiles(tempFiles)
	if len(errs) > 0 {
		t.Fatal("could not clean up temporary files")
	}
}

func TestNoFileExists(t *testing.T) {
	mockT := new(testing.T)
	False(t, NoFileExists(mockT, "assertions.go"))

	mockT = new(testing.T)
	True(t, NoFileExists(mockT, "non_existent_file"))

	mockT = new(testing.T)
	True(t, NoFileExists(mockT, "../_codegen"))

	var tempFiles []string

	link, err := getTempSymlinkPath("assertions.go")
	if err != nil {
		t.Fatal("could not create temp symlink, err:", err)
	}
	tempFiles = append(tempFiles, link)
	mockT = new(testing.T)
	False(t, NoFileExists(mockT, link))

	link, err = getTempSymlinkPath("non_existent_file")
	if err != nil {
		t.Fatal("could not create temp symlink, err:", err)
	}
	tempFiles = append(tempFiles, link)
	mockT = new(testing.T)
	False(t, NoFileExists(mockT, link))

	errs := cleanUpTempFiles(tempFiles)
	if len(errs) > 0 {
		t.Fatal("could not clean up temporary files")
	}
}

func getTempSymlinkPath(file string) (string, error) {
	link := file + "_symlink"
	err := os.Symlink(file, link)
	return link, err
}

func cleanUpTempFiles(paths []string) []error {
	var res []error
	for _, path := range paths {
		err := os.Remove(path)
		if err != nil {
			res = append(res, err)
		}
	}
	return res
}

func TestDirExists(t *testing.T) {
	mockT := new(testing.T)
	False(t, DirExists(mockT, "assertions.go"))

	mockT = new(testing.T)
	False(t, DirExists(mockT, "non_existent_dir"))

	mockT = new(testing.T)
	True(t, DirExists(mockT, "../_codegen"))

	var tempFiles []string

	link, err := getTempSymlinkPath("assertions.go")
	if err != nil {
		t.Fatal("could not create temp symlink, err:", err)
	}
	tempFiles = append(tempFiles, link)
	mockT = new(testing.T)
	False(t, DirExists(mockT, link))

	link, err = getTempSymlinkPath("non_existent_dir")
	if err != nil {
		t.Fatal("could not create temp symlink, err:", err)
	}
	tempFiles = append(tempFiles, link)
	mockT = new(testing.T)
	False(t, DirExists(mockT, link))

	errs := cleanUpTempFiles(tempFiles)
	if len(errs) > 0 {
		t.Fatal("could not clean up temporary files")
	}
}

func TestNoDirExists(t *testing.T) {
	mockT := new(testing.T)
	True(t, NoDirExists(mockT, "assertions.go"))

	mockT = new(testing.T)
	True(t, NoDirExists(mockT, "non_existent_dir"))

	mockT = new(testing.T)
	False(t, NoDirExists(mockT, "../_codegen"))

	var tempFiles []string

	link, err := getTempSymlinkPath("assertions.go")
	if err != nil {
		t.Fatal("could not create temp symlink, err:", err)
	}
	tempFiles = append(tempFiles, link)
	mockT = new(testing.T)
	True(t, NoDirExists(mockT, link))

	link, err = getTempSymlinkPath("non_existent_dir")
	if err != nil {
		t.Fatal("could not create temp symlink, err:", err)
	}
	tempFiles = append(tempFiles, link)
	mockT = new(testing.T)
	True(t, NoDirExists(mockT, link))

	errs := cleanUpTempFiles(tempFiles)
	if len(errs) > 0 {
		t.Fatal("could not clean up temporary files")
	}
}

func TestJSONEq_EqualSONString(t *testing.T) {
	mockT := new(testing.T)
	True(t, JSONEq(mockT, `{"hello": "world", "foo": "bar"}`, `{"hello": "world", "foo": "bar"}`))
}

func TestJSONEq_EquivalentButNotEqual(t *testing.T) {
	mockT := new(testing.T)
	True(t, JSONEq(mockT, `{"hello": "world", "foo": "bar"}`, `{"foo": "bar", "hello": "world"}`))
}

func TestJSONEq_HashOfArraysAndHashes(t *testing.T) {
	mockT := new(testing.T)
	True(t, JSONEq(mockT, "{\r\n\t\"numeric\": 1.5,\r\n\t\"array\": [{\"foo\": \"bar\"}, 1, \"string\", [\"nested\", \"array\", 5.5]],\r\n\t\"hash\": {\"nested\": \"hash\", \"nested_slice\": [\"this\", \"is\", \"nested\"]},\r\n\t\"string\": \"foo\"\r\n}",
		"{\r\n\t\"numeric\": 1.5,\r\n\t\"hash\": {\"nested\": \"hash\", \"nested_slice\": [\"this\", \"is\", \"nested\"]},\r\n\t\"string\": \"foo\",\r\n\t\"array\": [{\"foo\": \"bar\"}, 1, \"string\", [\"nested\", \"array\", 5.5]]\r\n}"))
}

func TestJSONEq_Array(t *testing.T) {
	mockT := new(testing.T)
	True(t, JSONEq(mockT, `["foo", {"hello": "world", "nested": "hash"}]`, `["foo", {"nested": "hash", "hello": "world"}]`))
}

func TestJSONEq_HashAndArrayNotEquivalent(t *testing.T) {
	mockT := new(testing.T)
	False(t, JSONEq(mockT, `["foo", {"hello": "world", "nested": "hash"}]`, `{"foo": "bar", {"nested": "hash", "hello": "world"}}`))
}

func TestJSONEq_HashesNotEquivalent(t *testing.T) {
	mockT := new(testing.T)
	False(t, JSONEq(mockT, `{"foo": "bar"}`, `{"foo": "bar", "hello": "world"}`))
}

func TestJSONEq_ActualIsNotJSON(t *testing.T) {
	mockT := new(testing.T)
	False(t, JSONEq(mockT, `{"foo": "bar"}`, "Not JSON"))
}

func TestJSONEq_ExpectedIsNotJSON(t *testing.T) {
	mockT := new(testing.T)
	False(t, JSONEq(mockT, "Not JSON", `{"foo": "bar", "hello": "world"}`))
}

func TestJSONEq_ExpectedAndActualNotJSON(t *testing.T) {
	mockT := new(testing.T)
	False(t, JSONEq(mockT, "Not JSON", "Not JSON"))
}

func TestJSONEq_ArraysOfDifferentOrder(t *testing.T) {
	mockT := new(testing.T)
	False(t, JSONEq(mockT, `["foo", {"hello": "world", "nested": "hash"}]`, `[{ "hello": "world", "nested": "hash"}, "foo"]`))
}

func TestYAMLEq_EqualYAMLString(t *testing.T) {
	mockT := new(testing.T)
	True(t, YAMLEq(mockT, `{"hello": "world", "foo": "bar"}`, `{"hello": "world", "foo": "bar"}`))
}

func TestYAMLEq_EquivalentButNotEqual(t *testing.T) {
	mockT := new(testing.T)
	True(t, YAMLEq(mockT, `{"hello": "world", "foo": "bar"}`, `{"foo": "bar", "hello": "world"}`))
}

func TestYAMLEq_HashOfArraysAndHashes(t *testing.T) {
	mockT := new(testing.T)
	expected := `
numeric: 1.5
array:
  - foo: bar
  - 1
  - "string"
  - ["nested", "array", 5.5]
hash:
  nested: hash
  nested_slice: [this, is, nested]
string: "foo"
`

	actual := `
numeric: 1.5
hash:
  nested: hash
  nested_slice: [this, is, nested]
string: "foo"
array:
  - foo: bar
  - 1
  - "string"
  - ["nested", "array", 5.5]
`
	True(t, YAMLEq(mockT, expected, actual))
}

func TestYAMLEq_Array(t *testing.T) {
	mockT := new(testing.T)
	True(t, YAMLEq(mockT, `["foo", {"hello": "world", "nested": "hash"}]`, `["foo", {"nested": "hash", "hello": "world"}]`))
}

func TestYAMLEq_HashAndArrayNotEquivalent(t *testing.T) {
	mockT := new(testing.T)
	False(t, YAMLEq(mockT, `["foo", {"hello": "world", "nested": "hash"}]`, `{"foo": "bar", {"nested": "hash", "hello": "world"}}`))
}

func TestYAMLEq_HashesNotEquivalent(t *testing.T) {
	mockT := new(testing.T)
	False(t, YAMLEq(mockT, `{"foo": "bar"}`, `{"foo": "bar", "hello": "world"}`))
}

func TestYAMLEq_ActualIsSimpleString(t *testing.T) {
	mockT := new(testing.T)
	False(t, YAMLEq(mockT, `{"foo": "bar"}`, "Simple String"))
}

func TestYAMLEq_ExpectedIsSimpleString(t *testing.T) {
	mockT := new(testing.T)
	False(t, YAMLEq(mockT, "Simple String", `{"foo": "bar", "hello": "world"}`))
}

func TestYAMLEq_ExpectedAndActualSimpleString(t *testing.T) {
	mockT := new(testing.T)
	True(t, YAMLEq(mockT, "Simple String", "Simple String"))
}

func TestYAMLEq_ArraysOfDifferentOrder(t *testing.T) {
	mockT := new(testing.T)
	False(t, YAMLEq(mockT, `["foo", {"hello": "world", "nested": "hash"}]`, `[{ "hello": "world", "nested": "hash"}, "foo"]`))
}

type diffTestingStruct struct {
	A string
	B int
}

func (d *diffTestingStruct) String() string {
	return d.A
}

func TestDiff(t *testing.T) {
	expected := `

Diff:
--- Expected
+++ Actual
@@ -1,3 +1,3 @@
 (struct { foo string }) {
- foo: (string) (len=5) "hello"
+ foo: (string) (len=3) "bar"
 }
`
	actual := diff(
		struct{ foo string }{"hello"},
		struct{ foo string }{"bar"},
	)
	Equal(t, expected, actual)

	expected = `

Diff:
--- Expected
+++ Actual
@@ -2,5 +2,5 @@
  (int) 1,
- (int) 2,
  (int) 3,
- (int) 4
+ (int) 5,
+ (int) 7
 }
`
	actual = diff(
		[]int{1, 2, 3, 4},
		[]int{1, 3, 5, 7},
	)
	Equal(t, expected, actual)

	expected = `

Diff:
--- Expected
+++ Actual
@@ -2,4 +2,4 @@
  (int) 1,
- (int) 2,
- (int) 3
+ (int) 3,
+ (int) 5
 }
`
	actual = diff(
		[]int{1, 2, 3, 4}[0:3],
		[]int{1, 3, 5, 7}[0:3],
	)
	Equal(t, expected, actual)

	expected = `

Diff:
--- Expected
+++ Actual
@@ -1,6 +1,6 @@
 (map[string]int) (len=4) {
- (string) (len=4) "four": (int) 4,
+ (string) (len=4) "five": (int) 5,
  (string) (len=3) "one": (int) 1,
- (string) (len=5) "three": (int) 3,
- (string) (len=3) "two": (int) 2
+ (string) (len=5) "seven": (int) 7,
+ (string) (len=5) "three": (int) 3
 }
`

	actual = diff(
		map[string]int{"one": 1, "two": 2, "three": 3, "four": 4},
		map[string]int{"one": 1, "three": 3, "five": 5, "seven": 7},
	)
	Equal(t, expected, actual)

	expected = `

Diff:
--- Expected
+++ Actual
@@ -1,3 +1,3 @@
 (*errors.errorString)({
- s: (string) (len=19) "some expected error"
+ s: (string) (len=12) "actual error"
 })
`

	actual = diff(
		errors.New("some expected error"),
		errors.New("actual error"),
	)
	Equal(t, expected, actual)

	expected = `

Diff:
--- Expected
+++ Actual
@@ -2,3 +2,3 @@
  A: (string) (len=11) "some string",
- B: (int) 10
+ B: (int) 15
 }
`

	actual = diff(
		diffTestingStruct{A: "some string", B: 10},
		diffTestingStruct{A: "some string", B: 15},
	)
	Equal(t, expected, actual)

	expected = `

Diff:
--- Expected
+++ Actual
@@ -1,2 +1,2 @@
-(time.Time) 2020-09-24 00:00:00 +0000 UTC
+(time.Time) 2020-09-25 00:00:00 +0000 UTC
 
`

	actual = diff(
		time.Date(2020, 9, 24, 0, 0, 0, 0, time.UTC),
		time.Date(2020, 9, 25, 0, 0, 0, 0, time.UTC),
	)
	Equal(t, expected, actual)
}

func TestTimeEqualityErrorFormatting(t *testing.T) {
	mockT := new(mockTestingT)

	Equal(mockT, time.Second*2, time.Millisecond)

	expectedErr := "\\s+Error Trace:\\s+Error:\\s+Not equal:\\s+\n\\s+expected: 2s\n\\s+actual\\s+: 1ms\n"
	Regexp(t, regexp.MustCompile(expectedErr), mockT.errorString())
}

func TestDiffEmptyCases(t *testing.T) {
	Equal(t, "", diff(nil, nil))
	Equal(t, "", diff(struct{ foo string }{}, nil))
	Equal(t, "", diff(nil, struct{ foo string }{}))
	Equal(t, "", diff(1, 2))
	Equal(t, "", diff(1, 2))
	Equal(t, "", diff([]int{1}, []bool{true}))
}

// Ensure there are no data races
func TestDiffRace(t *testing.T) {
	t.Parallel()

	expected := map[string]string{
		"a": "A",
		"b": "B",
		"c": "C",
	}

	actual := map[string]string{
		"d": "D",
		"e": "E",
		"f": "F",
	}

	// run diffs in parallel simulating tests with t.Parallel()
	numRoutines := 10
	rChans := make([]chan string, numRoutines)
	for idx := range rChans {
		rChans[idx] = make(chan string)
		go func(ch chan string) {
			defer close(ch)
			ch <- diff(expected, actual)
		}(rChans[idx])
	}

	for _, ch := range rChans {
		for msg := range ch {
			NotZero(t, msg) // dummy assert
		}
	}
}

type mockTestingT struct {
	errorFmt string
	args     []interface{}
}

func (m *mockTestingT) errorString() string {
	return fmt.Sprintf(m.errorFmt, m.args...)
}

func (m *mockTestingT) Errorf(format string, args ...interface{}) {
	m.errorFmt = format
	m.args = args
}

func (m *mockTestingT) Failed() bool {
	return m.errorFmt != ""
}

func TestFailNowWithPlainTestingT(t *testing.T) {
	mockT := &mockTestingT{}

	Panics(t, func() {
		FailNow(mockT, "failed")
	}, "should panic since mockT is missing FailNow()")
}

type mockFailNowTestingT struct {
}

func (m *mockFailNowTestingT) Errorf(format string, args ...interface{}) {}

func (m *mockFailNowTestingT) FailNow() {}

func TestFailNowWithFullTestingT(t *testing.T) {
	mockT := &mockFailNowTestingT{}

	NotPanics(t, func() {
		FailNow(mockT, "failed")
	}, "should call mockT.FailNow() rather than panicking")
}

func TestBytesEqual(t *testing.T) {
	var cases = []struct {
		a, b []byte
	}{
		{make([]byte, 2), make([]byte, 2)},
		{make([]byte, 2), make([]byte, 2, 3)},
		{nil, make([]byte, 0)},
	}
	for i, c := range cases {
		Equal(t, reflect.DeepEqual(c.a, c.b), ObjectsAreEqual(c.a, c.b), "case %d failed", i+1)
	}
}

func BenchmarkBytesEqual(b *testing.B) {
	const size = 1024 * 8
	s := make([]byte, size)
	for i := range s {
		s[i] = byte(i % 255)
	}
	s2 := make([]byte, size)
	copy(s2, s)

	mockT := &mockFailNowTestingT{}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		Equal(mockT, s, s2)
	}
}

func BenchmarkNotNil(b *testing.B) {
	for i := 0; i < b.N; i++ {
		NotNil(b, b)
	}
}

func ExampleComparisonAssertionFunc() {
	t := &testing.T{} // provided by test

	adder := func(x, y int) int {
		return x + y
	}

	type args struct {
		x int
		y int
	}

	tests := []struct {
		name      string
		args      args
		expect    int
		assertion ComparisonAssertionFunc
	}{
		{"2+2=4", args{2, 2}, 4, Equal},
		{"2+2!=5", args{2, 2}, 5, NotEqual},
		{"2+3==5", args{2, 3}, 5, Exactly},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.assertion(t, tt.expect, adder(tt.args.x, tt.args.y))
		})
	}
}

func TestComparisonAssertionFunc(t *testing.T) {
	type iface interface {
		Name() string
	}

	tests := []struct {
		name      string
		expect    interface{}
		got       interface{}
		assertion ComparisonAssertionFunc
	}{
		{"implements", (*iface)(nil), t, Implements},
		{"isType", (*testing.T)(nil), t, IsType},
		{"equal", t, t, Equal},
		{"equalValues", t, t, EqualValues},
		{"notEqualValues", t, nil, NotEqualValues},
		{"exactly", t, t, Exactly},
		{"notEqual", t, nil, NotEqual},
		{"notContains", []int{1, 2, 3}, 4, NotContains},
		{"subset", []int{1, 2, 3, 4}, []int{2, 3}, Subset},
		{"notSubset", []int{1, 2, 3, 4}, []int{0, 3}, NotSubset},
		{"elementsMatch", []byte("abc"), []byte("bac"), ElementsMatch},
		{"regexp", "^t.*y$", "testify", Regexp},
		{"notRegexp", "^t.*y$", "Testify", NotRegexp},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.assertion(t, tt.expect, tt.got)
		})
	}
}

func ExampleValueAssertionFunc() {
	t := &testing.T{} // provided by test

	dumbParse := func(input string) interface{} {
		var x interface{}
		_ = json.Unmarshal([]byte(input), &x)
		return x
	}

	tests := []struct {
		name      string
		arg       string
		assertion ValueAssertionFunc
	}{
		{"true is not nil", "true", NotNil},
		{"empty string is nil", "", Nil},
		{"zero is not nil", "0", NotNil},
		{"zero is zero", "0", Zero},
		{"false is zero", "false", Zero},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.assertion(t, dumbParse(tt.arg))
		})
	}
}

func TestValueAssertionFunc(t *testing.T) {
	tests := []struct {
		name      string
		value     interface{}
		assertion ValueAssertionFunc
	}{
		{"notNil", true, NotNil},
		{"nil", nil, Nil},
		{"empty", []int{}, Empty},
		{"notEmpty", []int{1}, NotEmpty},
		{"zero", false, Zero},
		{"notZero", 42, NotZero},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.assertion(t, tt.value)
		})
	}
}

func ExampleBoolAssertionFunc() {
	t := &testing.T{} // provided by test

	isOkay := func(x int) bool {
		return x >= 42
	}

	tests := []struct {
		name      string
		arg       int
		assertion BoolAssertionFunc
	}{
		{"-1 is bad", -1, False},
		{"42 is good", 42, True},
		{"41 is bad", 41, False},
		{"45 is cool", 45, True},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.assertion(t, isOkay(tt.arg))
		})
	}
}

func TestBoolAssertionFunc(t *testing.T) {
	tests := []struct {
		name      string
		value     bool
		assertion BoolAssertionFunc
	}{
		{"true", true, True},
		{"false", false, False},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.assertion(t, tt.value)
		})
	}
}

func ExampleErrorAssertionFunc() {
	t := &testing.T{} // provided by test

	dumbParseNum := func(input string, v interface{}) error {
		return json.Unmarshal([]byte(input), v)
	}

	tests := []struct {
		name      string
		arg       string
		assertion ErrorAssertionFunc
	}{
		{"1.2 is number", "1.2", NoError},
		{"1.2.3 not number", "1.2.3", Error},
		{"true is not number", "true", Error},
		{"3 is number", "3", NoError},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var x float64
			tt.assertion(t, dumbParseNum(tt.arg, &x))
		})
	}
}

func TestErrorAssertionFunc(t *testing.T) {
	tests := []struct {
		name      string
		err       error
		assertion ErrorAssertionFunc
	}{
		{"noError", nil, NoError},
		{"error", errors.New("whoops"), Error},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.assertion(t, tt.err)
		})
	}
}

func TestEventuallyFalse(t *testing.T) {
	mockT := new(testing.T)

	condition := func() bool {
		return false
	}

	False(t, Eventually(mockT, condition, 100*time.Millisecond, 20*time.Millisecond))
}

func TestEventuallyTrue(t *testing.T) {
	state := 0
	condition := func() bool {
		defer func() {
			state += 1
		}()
		return state == 2
	}

	True(t, Eventually(t, condition, 100*time.Millisecond, 20*time.Millisecond))
}

// errorsCapturingT is a mock implementation of TestingT that captures errors reported with Errorf.
type errorsCapturingT struct {
	errors []error
}

func (t *errorsCapturingT) Errorf(format string, args ...interface{}) {
	t.errors = append(t.errors, fmt.Errorf(format, args...))
}

func (t *errorsCapturingT) Helper() {}

func TestEventuallyWithTFalse(t *testing.T) {
	mockT := new(errorsCapturingT)

	condition := func(collect *CollectT) {
		Fail(collect, "condition fixed failure")
	}

	False(t, EventuallyWithT(mockT, condition, 100*time.Millisecond, 20*time.Millisecond))
	Len(t, mockT.errors, 2)
}

func TestEventuallyWithTTrue(t *testing.T) {
	mockT := new(errorsCapturingT)

	state := 0
	condition := func(collect *CollectT) {
		defer func() {
			state += 1
		}()
		True(collect, state == 2)
	}

	True(t, EventuallyWithT(mockT, condition, 100*time.Millisecond, 20*time.Millisecond))
	Len(t, mockT.errors, 0)
}

func TestEventuallyWithT_ConcurrencySafe(t *testing.T) {
	mockT := new(errorsCapturingT)

	condition := func(collect *CollectT) {
		Fail(collect, "condition fixed failure")
	}

	// To trigger race conditions, we run EventuallyWithT with a nanosecond tick.
	False(t, EventuallyWithT(mockT, condition, 100*time.Millisecond, time.Nanosecond))
	Len(t, mockT.errors, 2)
}

func TestEventuallyWithT_ReturnsTheLatestFinishedConditionErrors(t *testing.T) {
	// We'll use a channel to control whether a condition should sleep or not.
	mustSleep := make(chan bool, 2)
	mustSleep <- false
	mustSleep <- true
	close(mustSleep)

	condition := func(collect *CollectT) {
		if <-mustSleep {
			// Sleep to ensure that the second condition runs longer than timeout.
			time.Sleep(time.Second)
			return
		}

		// The first condition will fail. We expect to get this error as a result.
		Fail(collect, "condition fixed failure")
	}

	mockT := new(errorsCapturingT)
	False(t, EventuallyWithT(mockT, condition, 100*time.Millisecond, 20*time.Millisecond))
	Len(t, mockT.errors, 2)
}

func TestNeverFalse(t *testing.T) {
	condition := func() bool {
		return false
	}

	True(t, Never(t, condition, 100*time.Millisecond, 20*time.Millisecond))
}

// TestNeverTrue checks Never with a condition that returns true on second call.
func TestNeverTrue(t *testing.T) {
	mockT := new(testing.T)

	// A list of values returned by condition.
	// Channel protects against concurrent access.
	returns := make(chan bool, 2)
	returns <- false
	returns <- true
	defer close(returns)

	// Will return true on second call.
	condition := func() bool {
		return <-returns
	}

	False(t, Never(mockT, condition, 100*time.Millisecond, 20*time.Millisecond))
}

// Check that a long running condition doesn't block Eventually.
// See issue 805 (and its long tail of following issues)
func TestEventuallyTimeout(t *testing.T) {
	mockT := new(testing.T)

	NotPanics(t, func() {
		done, done2 := make(chan struct{}), make(chan struct{})

		// A condition function that returns after the Eventually timeout
		condition := func() bool {
			// Wait until Eventually times out and terminates
			<-done
			close(done2)
			return true
		}

		False(t, Eventually(mockT, condition, time.Millisecond, time.Microsecond))

		close(done)
		<-done2
	})
}

func Test_validateEqualArgs(t *testing.T) {
	if validateEqualArgs(func() {}, func() {}) == nil {
		t.Error("non-nil functions should error")
	}

	if validateEqualArgs(func() {}, func() {}) == nil {
		t.Error("non-nil functions should error")
	}

	if validateEqualArgs(nil, nil) != nil {
		t.Error("nil functions are equal")
	}
}

func Test_truncatingFormat(t *testing.T) {

	original := strings.Repeat("a", bufio.MaxScanTokenSize-102)
	result := truncatingFormat(original)
	Equal(t, fmt.Sprintf("%#v", original), result, "string should not be truncated")

	original = original + "x"
	result = truncatingFormat(original)
	NotEqual(t, fmt.Sprintf("%#v", original), result, "string should have been truncated.")

	if !strings.HasSuffix(result, "<... truncated>") {
		t.Error("truncated string should have <... truncated> suffix")
	}
}

// parseLabeledOutput does the inverse of labeledOutput - it takes a formatted
// output string and turns it back into a slice of labeledContent.
func parseLabeledOutput(output string) []labeledContent {
	labelPattern := regexp.MustCompile(`^\t([^\t]*): *\t(.*)$`)
	contentPattern := regexp.MustCompile(`^\t *\t(.*)$`)
	var contents []labeledContent
	lines := strings.Split(output, "\n")
	i := -1
	for _, line := range lines {
		if line == "" {
			// skip blank lines
			continue
		}
		matches := labelPattern.FindStringSubmatch(line)
		if len(matches) == 3 {
			// a label
			contents = append(contents, labeledContent{
				label:   matches[1],
				content: matches[2] + "\n",
			})
			i++
			continue
		}
		matches = contentPattern.FindStringSubmatch(line)
		if len(matches) == 2 {
			// just content
			if i >= 0 {
				contents[i].content += matches[1] + "\n"
				continue
			}
		}
		// Couldn't parse output
		return nil
	}
	return contents
}

type captureTestingT struct {
	msg string
}

func (ctt *captureTestingT) Errorf(format string, args ...interface{}) {
	ctt.msg = fmt.Sprintf(format, args...)
}

func (ctt *captureTestingT) checkResultAndErrMsg(t *testing.T, expectedRes, res bool, expectedErrMsg string) {
	t.Helper()
	if res != expectedRes {
		t.Errorf("Should return %t", expectedRes)
		return
	}
	contents := parseLabeledOutput(ctt.msg)
	if res == true {
		if contents != nil {
			t.Errorf("Should not log an error")
		}
		return
	}
	if contents == nil {
		t.Errorf("Should log an error. Log output: %v", ctt.msg)
		return
	}
	for _, content := range contents {
		if content.label == "Error" {
			if expectedErrMsg == content.content {
				return
			}
			t.Errorf("Logged Error: %v", content.content)
		}
	}
	t.Errorf("Should log Error: %v", expectedErrMsg)
}

func TestErrorIs(t *testing.T) {
	tests := []struct {
		err          error
		target       error
		result       bool
		resultErrMsg string
	}{
		{
			err:    io.EOF,
			target: io.EOF,
			result: true,
		},
		{
			err:    fmt.Errorf("wrap: %w", io.EOF),
			target: io.EOF,
			result: true,
		},
		{
			err:    io.EOF,
			target: io.ErrClosedPipe,
			result: false,
			resultErrMsg: "" +
				"Target error should be in err chain:\n" +
				"expected: \"io: read/write on closed pipe\"\n" +
				"in chain: \"EOF\"\n",
		},
		{
			err:    nil,
			target: io.EOF,
			result: false,
			resultErrMsg: "" +
				"Target error should be in err chain:\n" +
				"expected: \"EOF\"\n" +
				"in chain: \n",
		},
		{
			err:    io.EOF,
			target: nil,
			result: false,
			resultErrMsg: "" +
				"Target error should be in err chain:\n" +
				"expected: \"\"\n" +
				"in chain: \"EOF\"\n",
		},
		{
			err:    nil,
			target: nil,
			result: true,
		},
		{
			err:    fmt.Errorf("abc: %w", errors.New("def")),
			target: io.EOF,
			result: false,
			resultErrMsg: "" +
				"Target error should be in err chain:\n" +
				"expected: \"EOF\"\n" +
				"in chain: \"abc: def\"\n" +
				"\t\"def\"\n",
		},
	}
	for _, tt := range tests {
		tt := tt
		t.Run(fmt.Sprintf("ErrorIs(%#v,%#v)", tt.err, tt.target), func(t *testing.T) {
			mockT := new(captureTestingT)
			res := ErrorIs(mockT, tt.err, tt.target)
			mockT.checkResultAndErrMsg(t, tt.result, res, tt.resultErrMsg)
		})
	}
}

func TestNotErrorIs(t *testing.T) {
	tests := []struct {
		err          error
		target       error
		result       bool
		resultErrMsg string
	}{
		{
			err:    io.EOF,
			target: io.EOF,
			result: false,
			resultErrMsg: "" +
				"Target error should not be in err chain:\n" +
				"found: \"EOF\"\n" +
				"in chain: \"EOF\"\n",
		},
		{
			err:    fmt.Errorf("wrap: %w", io.EOF),
			target: io.EOF,
			result: false,
			resultErrMsg: "" +
				"Target error should not be in err chain:\n" +
				"found: \"EOF\"\n" +
				"in chain: \"wrap: EOF\"\n" +
				"\t\"EOF\"\n",
		},
		{
			err:    io.EOF,
			target: io.ErrClosedPipe,
			result: true,
		},
		{
			err:    nil,
			target: io.EOF,
			result: true,
		},
		{
			err:    io.EOF,
			target: nil,
			result: true,
		},
		{
			err:    nil,
			target: nil,
			result: false,
			resultErrMsg: "" +
				"Target error should not be in err chain:\n" +
				"found: \"\"\n" +
				"in chain: \n",
		},
		{
			err:    fmt.Errorf("abc: %w", errors.New("def")),
			target: io.EOF,
			result: true,
		},
	}
	for _, tt := range tests {
		tt := tt
		t.Run(fmt.Sprintf("NotErrorIs(%#v,%#v)", tt.err, tt.target), func(t *testing.T) {
			mockT := new(captureTestingT)
			res := NotErrorIs(mockT, tt.err, tt.target)
			mockT.checkResultAndErrMsg(t, tt.result, res, tt.resultErrMsg)
		})
	}
}

func TestErrorAs(t *testing.T) {
	mockT := new(testing.T)
	tests := []struct {
		err    error
		result bool
	}{
		{fmt.Errorf("wrap: %w", &customError{}), true},
		{io.EOF, false},
		{nil, false},
	}
	for _, tt := range tests {
		tt := tt
		var target *customError
		t.Run(fmt.Sprintf("ErrorAs(%#v,%#v)", tt.err, target), func(t *testing.T) {
			res := ErrorAs(mockT, tt.err, &target)
			if res != tt.result {
				t.Errorf("ErrorAs(%#v,%#v) should return %t)", tt.err, target, tt.result)
			}
		})
	}
}
