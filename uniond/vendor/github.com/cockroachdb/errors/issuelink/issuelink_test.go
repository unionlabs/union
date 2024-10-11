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

package issuelink_test

import (
	"context"
	goErr "errors"
	"fmt"
	"strings"
	"testing"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/issuelink"
	"github.com/cockroachdb/errors/markers"
	"github.com/cockroachdb/errors/testutils"
	"github.com/pkg/errors"
)

func TestIssueLink(t *testing.T) {
	tt := testutils.T{T: t}

	origErr := errors.New("world")

	err := errors.Wrap(
		issuelink.WithIssueLink(
			errors.WithStack(
				issuelink.WithIssueLink(origErr, issuelink.IssueLink{IssueURL: "123", Detail: "foo"}),
			),
			issuelink.IssueLink{IssueURL: "456", Detail: "bar"},
		),
		"hello")

	theTest := func(tt testutils.T, err error) {
		tt.Check(markers.Is(err, origErr))
		tt.CheckStringEqual(err.Error(), "hello: world")

		tt.Check(issuelink.HasIssueLink(err))
		if _, ok := markers.If(err, func(err error) (interface{}, bool) { return nil, issuelink.IsIssueLink(err) }); !ok {
			t.Error("woops")
		}

		details := issuelink.GetAllIssueLinks(err)
		tt.CheckDeepEqual(details, []issuelink.IssueLink{
			{IssueURL: "456", Detail: "bar"},
			{IssueURL: "123", Detail: "foo"},
		})

		errV := fmt.Sprintf("%+v", err)
		tt.Check(strings.Contains(errV, "issue: 123"))
		tt.Check(strings.Contains(errV, "issue: 456"))
		tt.Check(strings.Contains(errV, "detail: foo"))
		tt.Check(strings.Contains(errV, "detail: bar"))
	}

	tt.Run("local", func(tt testutils.T) { theTest(tt, err) })

	enc := errbase.EncodeError(context.Background(), err)
	newErr := errbase.DecodeError(context.Background(), enc)

	tt.Run("remote", func(tt testutils.T) { theTest(tt, newErr) })

}

func TestFormat(t *testing.T) {
	tt := testutils.T{t}

	baseErr := goErr.New("woo")
	link := issuelink.IssueLink{IssueURL: "http://mysite"}
	const woo = `woo`
	const waawoo = `waa: woo`
	testCases := []struct {
		name          string
		err           error
		expFmtSimple  string
		expFmtVerbose string
	}{
		{"link",
			issuelink.WithIssueLink(baseErr, link),
			woo, `
woo
(1) issue: http://mysite
Wraps: (2) woo
Error types: (1) *issuelink.withIssueLink (2) *errors.errorString`},
		{"link-details",
			issuelink.WithIssueLink(baseErr, issuelink.IssueLink{IssueURL: "http://mysite", Detail: "see more"}),
			woo, `
woo
(1) issue: http://mysite
  | detail: see more
Wraps: (2) woo
Error types: (1) *issuelink.withIssueLink (2) *errors.errorString`},

		{"link + wrapper",
			issuelink.WithIssueLink(&werrFmt{baseErr, "waa"}, link),
			waawoo, `
waa: woo
(1) issue: http://mysite
Wraps: (2) waa
  | -- this is waa's
  | multi-line payload
Wraps: (3) woo
Error types: (1) *issuelink.withIssueLink (2) *issuelink_test.werrFmt (3) *errors.errorString`},

		{"wrapper + link",
			&werrFmt{issuelink.WithIssueLink(baseErr, link), "waa"},
			waawoo, `
waa: woo
(1) waa
  | -- this is waa's
  | multi-line payload
Wraps: (2) issue: http://mysite
Wraps: (3) woo
Error types: (1) *issuelink_test.werrFmt (2) *issuelink.withIssueLink (3) *errors.errorString`},
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
