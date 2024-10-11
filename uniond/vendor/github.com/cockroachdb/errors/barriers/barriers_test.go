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

package barriers_test

import (
	"context"
	goErr "errors"
	"fmt"
	"strings"
	"testing"

	"github.com/cockroachdb/errors/barriers"
	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/markers"
	"github.com/cockroachdb/errors/testutils"
	"github.com/pkg/errors"
)

// This test demonstrates that a barrier hides it causes.
func TestHideCause(t *testing.T) {
	tt := testutils.T{T: t}

	origErr := errors.New("hello")
	err1 := errors.Wrap(origErr, "world")

	// Assertion: without barriers, the cause can be identified.
	tt.Assert(markers.Is(err1, origErr))

	// This test: a barrier hides the cause.
	err := barriers.Handled(err1)
	tt.Check(!markers.Is(err, origErr))
}

// This test demonstrates how the message is preserved (or not) depending
// on how the barrier is constructed.
func TestBarrierMessage(t *testing.T) {
	tt := testutils.T{T: t}

	origErr := errors.New("hello")

	b1 := barriers.Handled(origErr)
	tt.CheckEqual(b1.Error(), origErr.Error())

	b2 := barriers.HandledWithMessage(origErr, "woo")
	tt.CheckEqual(b2.Error(), "woo")
}

// This test demonstrates that the original error details
// are preserved through barriers, even when they go through the network.
func TestBarrierMaskedDetails(t *testing.T) {
	tt := testutils.T{T: t}

	origErr := errors.New("hello friends")

	b := barriers.HandledWithMessage(origErr, "message hidden")

	// Assertion: the friends message is hidden.
	tt.Assert(!strings.Contains(b.Error(), "friends"))

	// This test: the details are available when printing the error details.
	errV := fmt.Sprintf("%+v", b)
	tt.Check(strings.Contains(errV, "friends"))

	// Simulate a network traversal.
	enc := errbase.EncodeError(context.Background(), b)
	newB := errbase.DecodeError(context.Background(), enc)

	// The friends message is hidden.
	tt.Check(!strings.Contains(b.Error(), "friends"))

	// The cause is still hidden.
	tt.Check(!markers.Is(newB, origErr))

	// However the cause's details are still visible.
	errV = fmt.Sprintf("%+v", newB)
	tt.Check(strings.Contains(errV, "friends"))
}

// This test exercises HandledWithMessagef.
func TestHandledWithMessagef(t *testing.T) {
	tt := testutils.T{T: t}

	origErr := errors.New("hello friends")

	b1 := barriers.HandledWithMessage(origErr, "woo woo")
	b2 := barriers.HandledWithMessagef(origErr, "woo %s", "woo")

	tt.CheckStringEqual(b1.Error(), b2.Error())
}

func TestFormat(t *testing.T) {
	tt := testutils.T{t}

	const woo = `woo`
	const waawoo = `waa: woo`
	testCases := []struct {
		name          string
		err           error
		expFmtSimple  string
		expFmtVerbose string
	}{
		{"handled", barriers.Handled(goErr.New("woo")), woo, `
woo
(1) woo
  | -- cause hidden behind barrier
  | woo
  | (1) woo
  | Error types: (1) *errors.errorString
Error types: (1) *barriers.barrierErr`},

		{"handled + handled", barriers.Handled(barriers.Handled(goErr.New("woo"))), woo, `
woo
(1) woo
  | -- cause hidden behind barrier
  | woo
  | (1) woo
  |   | -- cause hidden behind barrier
  |   | woo
  |   | (1) woo
  |   | Error types: (1) *errors.errorString
  | Error types: (1) *barriers.barrierErr
Error types: (1) *barriers.barrierErr`},

		{"handledmsg", barriers.HandledWithMessage(goErr.New("woo"), "waa"), "waa", `
waa
(1) waa
  | -- cause hidden behind barrier
  | woo
  | (1) woo
  | Error types: (1) *errors.errorString
Error types: (1) *barriers.barrierErr`},

		{"handledmsg + handledmsg", barriers.HandledWithMessage(
			barriers.HandledWithMessage(
				goErr.New("woo"), "waa"), "wuu"), `wuu`, `
wuu
(1) wuu
  | -- cause hidden behind barrier
  | waa
  | (1) waa
  |   | -- cause hidden behind barrier
  |   | woo
  |   | (1) woo
  |   | Error types: (1) *errors.errorString
  | Error types: (1) *barriers.barrierErr
Error types: (1) *barriers.barrierErr`},

		{"handled + wrapper",
			barriers.Handled(
				&werrFmt{
					goErr.New("woo"),
					"waa"}),
			waawoo, `
waa: woo
(1) waa: woo
  | -- cause hidden behind barrier
  | waa: woo
  | (1) waa
  |   | -- this is waa's
  |   | multi-line wrapper payload
  | Wraps: (2) woo
  | Error types: (1) *barriers_test.werrFmt (2) *errors.errorString
Error types: (1) *barriers.barrierErr`},
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
		p.Printf("-- this is %s's\nmulti-line wrapper payload", e.msg)
	}
	return e.cause
}
