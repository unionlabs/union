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

package assert_test

import (
	"context"
	goErr "errors"
	"fmt"
	"strings"
	"testing"

	"github.com/cockroachdb/errors/assert"
	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/markers"
	"github.com/cockroachdb/errors/testutils"
	"github.com/pkg/errors"
)

func TestAssert(t *testing.T) {
	tt := testutils.T{T: t}

	baseErr := errors.New("world")
	err := errors.Wrap(assert.WithAssertionFailure(baseErr), "hello")

	tt.Check(markers.Is(err, baseErr))

	tt.Check(assert.HasAssertionFailure(err))

	if _, ok := markers.If(err, func(err error) (interface{}, bool) { return nil, assert.IsAssertionFailure(err) }); !ok {
		t.Error("woops")
	}

	tt.CheckEqual(err.Error(), "hello: world")

	enc := errbase.EncodeError(context.Background(), err)
	newErr := errbase.DecodeError(context.Background(), enc)

	tt.Check(markers.Is(newErr, baseErr))

	tt.Check(assert.HasAssertionFailure(newErr))

	if _, ok := markers.If(newErr, func(err error) (interface{}, bool) { return nil, assert.IsAssertionFailure(err) }); !ok {
		t.Error("woops")
	}

	tt.CheckEqual(newErr.Error(), "hello: world")
}

func TestFormat(t *testing.T) {
	tt := testutils.T{t}

	baseErr := goErr.New("woo")
	const woo = `woo`
	const waawoo = `waa: woo`
	testCases := []struct {
		name          string
		err           error
		expFmtSimple  string
		expFmtVerbose string
	}{
		{"assert",
			assert.WithAssertionFailure(baseErr),
			woo, `
woo
(1) assertion failure
Wraps: (2) woo
Error types: (1) *assert.withAssertionFailure (2) *errors.errorString`},

		{"assert + wrapper",
			assert.WithAssertionFailure(&werrFmt{baseErr, "waa"}),
			waawoo, `
waa: woo
(1) assertion failure
Wraps: (2) waa
  | -- this is waa's
  | multi-line payload
Wraps: (3) woo
Error types: (1) *assert.withAssertionFailure (2) *assert_test.werrFmt (3) *errors.errorString`},

		{"wrapper + assert",
			&werrFmt{assert.WithAssertionFailure(baseErr), "waa"},
			waawoo, `
waa: woo
(1) waa
  | -- this is waa's
  | multi-line payload
Wraps: (2) assertion failure
Wraps: (3) woo
Error types: (1) *assert_test.werrFmt (2) *assert.withAssertionFailure (3) *errors.errorString`},
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
			tt.CheckStringEqual(spv, refV)
		})
	}
}

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
