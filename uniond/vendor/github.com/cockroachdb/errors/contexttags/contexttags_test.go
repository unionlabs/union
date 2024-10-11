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

package contexttags_test

import (
	"context"
	goErr "errors"
	"fmt"
	"strings"
	"testing"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/contexttags"
	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/markers"
	"github.com/cockroachdb/errors/testutils"
	"github.com/cockroachdb/logtags"
	"github.com/cockroachdb/redact"
)

func TestWithContext(t *testing.T) {
	tt := testutils.T{T: t}

	// Create an example context with decoration.
	ctx := context.Background()
	ctx = logtags.AddTag(ctx, "foo", 123)
	ctx = logtags.AddTag(ctx, "x", 456)
	ctx = logtags.AddTag(ctx, "bar", nil)

	// Create another example context. We use two to demonstrate that an
	// error can store multiple sets of context tags.
	ctx2 := context.Background()
	ctx2 = logtags.AddTag(ctx2, "planet", "universe")

	// This will be our reference expected value.
	refTagsets := []*logtags.Buffer{
		logtags.SingleTagBuffer("planet", "universe"),
		logtags.SingleTagBuffer("foo", "123").Add("x", "456").Add("bar", ""),
	}

	// Construct the error object.
	origErr := errors.New("hello")
	decoratedErr := errors.WithContextTags(origErr, ctx)
	decoratedErr = errors.WithContextTags(decoratedErr, ctx2)

	theTest := func(tt testutils.T, err error) {
		// Ensure that the original error object can be found.
		// This test that the cause interface works properly.
		tt.Check(markers.Is(err, origErr))

		// Ensure that the decorated error can be found.
		// This checks that the wrapper identity
		// is properly preserved across the network.
		tt.Check(markers.Is(err, decoratedErr))

		// Check that the message is preserved.
		tt.CheckStringEqual(err.Error(), "hello")

		// Check that the tag pairs are preserved.
		tagsets := contexttags.GetContextTags(err)
		if len(tagsets) != len(refTagsets) {
			tt.CheckEqual(len(tagsets), len(refTagsets))
		} else {
			for i, actualB := range tagsets {
				refB := refTagsets[i]
				tt.CheckDeepEqual(actualB.Get(), refB.Get())
			}
		}
	}

	tt.Run("local", func(tt testutils.T) { theTest(tt, decoratedErr) })

	enc := errbase.EncodeError(context.Background(), decoratedErr)
	newErr := errbase.DecodeError(context.Background(), enc)

	tt.Run("remote", func(tt testutils.T) { theTest(tt, newErr) })
}

func TestTagRedactionInSafeDetails(t *testing.T) {
	tt := testutils.T{T: t}

	// Create an example context with decoration.
	ctx := context.Background()
	ctx = logtags.AddTag(ctx, "foo1", 123)
	ctx = logtags.AddTag(ctx, "x", 456)
	ctx = logtags.AddTag(ctx, "bar1", nil)
	ctx = logtags.AddTag(ctx, "foo2", errors.Safe(123))
	ctx = logtags.AddTag(ctx, "y", errors.Safe(456))
	ctx = logtags.AddTag(ctx, "bar2", nil)

	// Create another example context. We use two to demonstrate that an
	// error can store multiple sets of context tags.
	ctx2 := context.Background()
	ctx2 = logtags.AddTag(ctx2, "planet1", "universe")
	ctx2 = logtags.AddTag(ctx2, "planet2", errors.Safe("universe"))

	// rm is what's left over after redaction.
	rm := string(redact.RedactableBytes(redact.RedactedMarker()).StripMarkers())

	// This will be our reference expected value in "safe details" strings.
	refStrings := [][]string{
		[]string{"planet1=" + rm, "planet2=universe"},
		[]string{"foo1=" + rm, "x" + rm, "bar1", "foo2=123", "y456", "bar2"},
	}

	// Construct the error object.
	origErr := errors.New("hello")
	decoratedErr := errors.WithContextTags(origErr, ctx)
	decoratedErr = errors.WithContextTags(decoratedErr, ctx2)

	theTest := func(tt testutils.T, err error) {
		details := errors.GetAllSafeDetails(err)
		var strs [][]string
		for _, d := range details {
			strs = append(strs, d.SafeDetails)
		}
		// Discard the inner details. We only care about the WithContext
		// decorations.
		if len(strs) > 2 {
			strs = strs[:2]
		}
		tt.CheckDeepEqual(strs, refStrings)
	}

	tt.Run("local", func(tt testutils.T) { theTest(tt, decoratedErr) })

	enc := errbase.EncodeError(context.Background(), decoratedErr)
	newErr := errbase.DecodeError(context.Background(), enc)

	tt.Run("remote", func(tt testutils.T) { theTest(tt, newErr) })

}

func TestFormat(t *testing.T) {
	tt := testutils.T{t}

	ctx := logtags.AddTag(context.Background(), "thetag", nil)
	ctx = logtags.AddTag(ctx, "anothertag", nil)
	baseErr := goErr.New("woo")
	const woo = `woo`
	const waawoo = `waa: woo`
	testCases := []struct {
		name          string
		err           error
		expFmtSimple  string
		expFmtVerbose string
	}{
		{"tags",
			contexttags.WithContextTags(baseErr, ctx),
			woo, `
woo
(1) tags: [thetag,anothertag]
Wraps: (2) woo
Error types: (1) *contexttags.withContext (2) *errors.errorString`},

		{"tags + wrapper",
			contexttags.WithContextTags(&werrFmt{baseErr, "waa"}, ctx),
			waawoo, `
waa: woo
(1) tags: [thetag,anothertag]
Wraps: (2) waa
  | -- this is waa's
  | multi-line payload
Wraps: (3) woo
Error types: (1) *contexttags.withContext (2) *contexttags_test.werrFmt (3) *errors.errorString`},

		{"wrapper + tags",
			&werrFmt{contexttags.WithContextTags(baseErr, ctx), "waa"},
			waawoo, `
waa: woo
(1) waa
  | -- this is waa's
  | multi-line payload
Wraps: (2) tags: [thetag,anothertag]
Wraps: (3) woo
Error types: (1) *contexttags_test.werrFmt (2) *contexttags.withContext (3) *errors.errorString`},
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
