// Copyright 2019 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package testutils

import (
	"bytes"
	"fmt"
	"os"
	"reflect"
	"regexp"
	"runtime"
	"strings"
	"sync"
	"testing"

	"github.com/kr/pretty"
)

// This file provides assertions and checks for Go tests, two
// facilities which also provide some lines of source code around the
// location of a failing check when an error occurs.
//
// It makes the test code more concise and also makes the
// troubleshooting of failed tests easier.
//
// For example, "before":
//
//      if a,b := fmt.Errorf("woo"), fmt.Errorf("waa"); a != b {
//          t.Errorf("expected: %s, got: %s", a, b)
//      }
//
// When this fails, only the details available via the objects'
// String() function would be printed. The test failure would also not
// detail what comparison operator was used.
//
// With this library, "after":
//
//    tt := testutils.T{T:t}
//    tt.CheckEqual(fmt.Errorf("woo"), fmt.Errorf("waa"))
//
// Upon a failure:
//
//    assert_test.go:11: values not equal
//        got: &errors.errorString{s:"woo"}
//        expected: &errors.errorString{s:"waa"}
//        context:
//
//            tt := testutils.T{T:t}
//        > 	tt.CheckEqual(fmt.Errorf("woo"), fmt.Errorf("waa"))
//
//
// Observe:
// - the objects being compared are detailed using a deep pretty-printer.
// - three lines of source context are printed automatically under the failing check.
//

// T is a wrapper around testing.T which provides additional
// methods defined in this package. See below for details.
type T struct {
	*testing.T
}

// Run runs the specified sub-test and passes it a T instance
// from this package.
func (t *T) Run(testName string, testFn func(t T)) {
	t.Helper()
	t.T.Run(testName, func(t *testing.T) {
		t.Helper()
		testFn(T{T: t})
	})
}

// Assert calls t.Fatal if the condition is false. Also
// the text of the assertion is printed.
func (t *T) Assert(cond bool) {
	t.Helper()
	if !cond {
		t.failWithf(true, "assertion failed")
	}
}

// AssertEqual asserts that the value is equal to some reference.
func (t *T) AssertEqual(val, ref interface{}) {
	t.Helper()
	if val != ref {
		t.failWithf(true, "assertion failed: values not equal\ngot: %# v\nexpected: %# v",
			pretty.Formatter(val), pretty.Formatter(ref))
	}
}

// AssertDeepEqual asserts that the value is equal to some reference.
func (t *T) AssertDeepEqual(val, ref interface{}) {
	t.Helper()
	if !reflect.DeepEqual(val, ref) {
		t.failWithf(true, "assertion failed: values not equal\ngot: %# v\nexpected: %# v",
			pretty.Formatter(val), pretty.Formatter(ref))
	}
}

// Check calls t.Error if the condition is false. Also
// the text of the condition is printed.
func (t *T) Check(cond bool) {
	t.Helper()
	if !cond {
		t.failWithf(false, "check failed")
	}
}

// CheckEqual checks that the value is equal to some reference.
func (t *T) CheckEqual(val, ref interface{}) {
	t.Helper()
	if val != ref {
		t.failWithf(false, "values not equal\n     got: %# v\nexpected: %# v",
			pretty.Formatter(val), pretty.Formatter(ref))
	}
}

func (t *T) CheckContains(s, substr string) {
	t.Helper()
	if !strings.Contains(s, substr) {
		t.failWithf(false, "value does not contain substring\n     got: %s\\nexpected: %s", s, substr)
	}
}

// CheckEqual checks that the string value is equal to some reference.
func (t *T) CheckStringEqual(val, ref string) {
	t.Helper()
	if val != ref {
		pat := regexp.QuoteMeta(ref)
		pat = strings.ReplaceAll(pat, "\n", `\n`)
		pat = strings.ReplaceAll(pat, "'", `'"'"'`)
		pat = strings.ReplaceAll(pat, "\t", `\t`)
		repl := strings.ReplaceAll(val, "\n", `\n`)
		repl = strings.ReplaceAll(repl, "'", `'"'"'`)
		repl = strings.ReplaceAll(repl, "\t", `\t`)
		_, file, _, _ := runtime.Caller(1)
		t.failWithf(false, "values not equal; got:\n  %s\nexpected:\n  %s\nTo update expected, run:\n  perl -0777 -pi -ne 's@%s@%s@ms' %s",
			strings.ReplaceAll(val, "\n", "\n  "),
			strings.ReplaceAll(ref, "\n", "\n  "),
			pat, repl, file,
		)

	}
}

// CheckDeepEqual checks that the value is equal to some reference.
func (t *T) CheckDeepEqual(val, ref interface{}) {
	t.Helper()
	if !reflect.DeepEqual(val, ref) {
		t.failWithf(false, "values not equal\ngot: %# v\nexpected: %# v",
			pretty.Formatter(val), pretty.Formatter(ref))
	}
}

// CheckRegexpEqual checks that the value is matched by some regular
// expression.
func (t *T) CheckRegexpEqual(val, regexs string) {
	t.Helper()
	m, err := regexp.MatchString(regexs, val)
	if err != nil {
		t.Fatalf("error compiling regular expression: %+v", err)
	}
	if !m {
		t.failWithf(false, "values does not match regular expression\ngot: %q\nexpected to match regexp:\n%s",
			pretty.Formatter(val), pretty.Formatter(regexs))
	}
}

func (t *T) failWithf(failTest bool, format string, args ...interface{}) {
	t.Helper()
	_, file, line, _ := runtime.Caller(2)
	var msg bytes.Buffer
	fmt.Fprintf(&msg, format, args...)
	contextLines, lineIdx := fileContext(file, line, 1)
	if len(contextLines) > 0 {
		msg.WriteString("\ncontext:\n")
		for i, line := range contextLines {
			switch {
			case i == lineIdx:
				fmt.Fprintf(&msg, "> %s\n", line)
			default:
				fmt.Fprintf(&msg, "  %s\n", line)
			}
		}
	}
	if failTest {
		t.Fatal(msg.String())
	} else {
		t.Error(msg.String())
	}
}

// The following code is copied from github.com/getsentry/raven-go.

var fileCacheLock sync.Mutex
var fileCache = make(map[string][][]byte)

func fileContext(filename string, line, context int) ([][]byte, int) {
	fileCacheLock.Lock()
	defer fileCacheLock.Unlock()
	lines, ok := fileCache[filename]
	if !ok {
		data, err := os.ReadFile(filename)
		if err != nil {
			// cache errors as nil slice: code below handles it correctly
			// otherwise when missing the source or running as a different user, we try
			// reading the file on each error which is unnecessary
			fileCache[filename] = nil
			return nil, 0
		}
		lines = bytes.Split(data, []byte{'\n'})
		fileCache[filename] = lines
	}

	if lines == nil {
		// cached error from ReadFile: return no lines
		return nil, 0
	}

	line-- // stack trace lines are 1-indexed
	start := line - context
	var idx int
	if start < 0 {
		start = 0
		idx = line
	} else {
		idx = context
	}
	end := line + context + 1
	if line >= len(lines) {
		return nil, 0
	}
	if end > len(lines) {
		end = len(lines)
	}
	return lines[start:end], idx
}
