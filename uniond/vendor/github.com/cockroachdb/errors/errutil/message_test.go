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

package errutil_test

import (
	goErr "errors"
	"fmt"
	"regexp"
	"strings"
	"testing"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/errutil"
	"github.com/cockroachdb/errors/testutils"
)

// TestFormat checks edge cases of the utilities in this package.
// More tests are exercised in the "fmttests" package.
func TestFormat(t *testing.T) {
	tt := testutils.T{t}

	testCases := []struct {
		name          string
		err           error
		expFmtSimple  string
		expFmtVerbose string
	}{
		{"newf-empty",
			errutil.Newf(emptyString),
			``, `

(1) attached stack trace
  -- stack trace:
  | github.com/cockroachdb/errors/errutil_test.TestFormat
  | <tab><path>
  | testing.tRunner
  | <tab><path>
  | runtime.goexit
  | <tab><path>
Wraps: (2)
Error types: (1) *withstack.withStack (2) *errutil.leafError`,
		},

		{"newf-empty-arg",
			errutil.Newf(emptyString, 123),
			`%!(EXTRA int=123)`, `
%!(EXTRA int=123)
(1) attached stack trace
  -- stack trace:
  | github.com/cockroachdb/errors/errutil_test.TestFormat
  | <tab><path>
  | testing.tRunner
  | <tab><path>
  | runtime.goexit
  | <tab><path>
Wraps: (2) %!(EXTRA int=123)
Error types: (1) *withstack.withStack (2) *errutil.leafError`,
		},

		{"wrapf-empty",
			errutil.Wrapf(goErr.New("woo"), emptyString),
			`woo`, `
woo
(1) attached stack trace
  -- stack trace:
  | github.com/cockroachdb/errors/errutil_test.TestFormat
  | <tab><path>
  | testing.tRunner
  | <tab><path>
  | runtime.goexit
  | <tab><path>
Wraps: (2) woo
Error types: (1) *withstack.withStack (2) *errors.errorString`,
		},

		{"wrapf-empty-arg",
			errutil.Wrapf(goErr.New("woo"), emptyString, 123),
			`%!(EXTRA int=123): woo`, `
%!(EXTRA int=123): woo
(1) attached stack trace
  -- stack trace:
  | github.com/cockroachdb/errors/errutil_test.TestFormat
  | <tab><path>
  | testing.tRunner
  | <tab><path>
  | runtime.goexit
  | <tab><path>
Wraps: (2) %!(EXTRA int=123)
Wraps: (3) woo
Error types: (1) *withstack.withStack (2) *errutil.withPrefix (3) *errors.errorString`,
		},

		{"assert + wrap",
			errutil.NewAssertionErrorWithWrappedErrf(&werrFmt{goErr.New("woo"), "wuu"}, "waa: %s", "hello"),
			`waa: hello: wuu: woo`, `
waa: hello: wuu: woo
(1) assertion failure
Wraps: (2) attached stack trace
  -- stack trace:
  | github.com/cockroachdb/errors/errutil_test.TestFormat
  | <tab><path>
  | testing.tRunner
  | <tab><path>
  | runtime.goexit
  | <tab><path>
Wraps: (3) waa: hello
Wraps: (4) wuu: woo
  | -- cause hidden behind barrier
  | wuu: woo
  | (1) wuu
  |   | -- this is wuu's
  |   | multi-line payload
  | Wraps: (2) woo
  | Error types: (1) *errutil_test.werrFmt (2) *errors.errorString
Error types: (1) *assert.withAssertionFailure (2) *withstack.withStack (3) *errutil.withPrefix (4) *barriers.barrierErr`,
		},

		{"assert + wrap empty",
			errutil.NewAssertionErrorWithWrappedErrf(&werrFmt{goErr.New("woo"), "wuu"}, emptyString),
			`wuu: woo`, `
wuu: woo
(1) assertion failure
Wraps: (2) attached stack trace
  -- stack trace:
  | github.com/cockroachdb/errors/errutil_test.TestFormat
  | <tab><path>
  | testing.tRunner
  | <tab><path>
  | runtime.goexit
  | <tab><path>
Wraps: (3) wuu: woo
  | -- cause hidden behind barrier
  | wuu: woo
  | (1) wuu
  |   | -- this is wuu's
  |   | multi-line payload
  | Wraps: (2) woo
  | Error types: (1) *errutil_test.werrFmt (2) *errors.errorString
Error types: (1) *assert.withAssertionFailure (2) *withstack.withStack (3) *barriers.barrierErr`,
		},

		{"assert + wrap empty+arg",
			errutil.NewAssertionErrorWithWrappedErrf(&werrFmt{goErr.New("woo"), "wuu"}, emptyString, 123),
			`%!(EXTRA int=123): wuu: woo`, `
%!(EXTRA int=123): wuu: woo
(1) assertion failure
Wraps: (2) attached stack trace
  -- stack trace:
  | github.com/cockroachdb/errors/errutil_test.TestFormat
  | <tab><path>
  | testing.tRunner
  | <tab><path>
  | runtime.goexit
  | <tab><path>
Wraps: (3) %!(EXTRA int=123)
Wraps: (4) wuu: woo
  | -- cause hidden behind barrier
  | wuu: woo
  | (1) wuu
  |   | -- this is wuu's
  |   | multi-line payload
  | Wraps: (2) woo
  | Error types: (1) *errutil_test.werrFmt (2) *errors.errorString
Error types: (1) *assert.withAssertionFailure (2) *withstack.withStack (3) *errutil.withPrefix (4) *barriers.barrierErr`,
		},
	}

	for _, test := range testCases {
		tt.Run(test.name, func(tt testutils.T) {
			err := test.err

			// %s is simple formatting
			tt.CheckStringEqual(fmt.Sprintf("%s", err), test.expFmtSimple)

			// %v is simple formatting too, for compatibility with the past.
			tt.CheckStringEqual(fmt.Sprintf("%v", err), test.expFmtSimple)

			// %q is always like %s but quotes the result.
			ref := fmt.Sprintf("%q", test.expFmtSimple)
			tt.CheckStringEqual(fmt.Sprintf("%q", err), ref)

			// %+v is the verbose mode.
			refV := strings.TrimPrefix(test.expFmtVerbose, "\n")
			spv := fmt.Sprintf("%+v", err)
			spv = fileref.ReplaceAllString(spv, "<path>")
			spv = strings.ReplaceAll(spv, "\t", "<tab>")
			tt.CheckStringEqual(spv, refV)
		})
	}
}

var fileref = regexp.MustCompile(`([a-zA-Z0-9\._/@-]*\.(?:go|s):\d+)`)

type werrFmt struct {
	cause error
	msg   string
}

var _ errbase.Formatter = (*werrFmt)(nil)

func (e *werrFmt) Error() string                 { return fmt.Sprintf("%s: %v", e.msg, e.cause) }
func (e *werrFmt) Unwrap() error                 { return e.cause }
func (e *werrFmt) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }
func (e *werrFmt) FormatError(p errbase.Printer) error {
	p.Print(e.msg)
	if p.Detail() {
		p.Printf("-- this is %s's\nmulti-line payload", e.msg)
	}
	return e.cause
}

var emptyString = ""
