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
	"fmt"
	"strings"
	"testing"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/issuelink"
	"github.com/cockroachdb/errors/markers"
	"github.com/cockroachdb/errors/testutils"
	"github.com/pkg/errors"
)

func TestUnimplementedError(t *testing.T) {
	tt := testutils.T{T: t}

	err := issuelink.UnimplementedError(issuelink.IssueLink{IssueURL: "123", Detail: "foo"}, "world")

	err = errors.Wrap(err, "hello")

	theTest := func(tt testutils.T, err error) {
		tt.Check(issuelink.HasUnimplementedError(err))
		tt.Check(issuelink.IsUnimplementedError(errbase.UnwrapAll(err)))
		if _, ok := markers.If(err, func(err error) (interface{}, bool) { return nil, issuelink.IsUnimplementedError(err) }); !ok {
			t.Error("woops")
		}

		details := issuelink.GetAllIssueLinks(err)
		tt.CheckDeepEqual(details, []issuelink.IssueLink{
			{IssueURL: "123", Detail: "foo"},
		})

		errV := fmt.Sprintf("%+v", err)
		tt.Check(strings.Contains(errV, "issue: 123"))
		tt.Check(strings.Contains(errV, "detail: foo"))
	}

	tt.Run("local", func(tt testutils.T) { theTest(tt, err) })

	enc := errbase.EncodeError(context.Background(), err)
	newErr := errbase.DecodeError(context.Background(), enc)

	tt.Run("remote", func(tt testutils.T) { theTest(tt, newErr) })

}

func TestFormatUnimp(t *testing.T) {
	tt := testutils.T{t}

	link := issuelink.IssueLink{IssueURL: "http://mysite"}
	const woo = `woo`
	const waawoo = `waa: woo`
	testCases := []struct {
		name          string
		err           error
		expFmtSimple  string
		expFmtVerbose string
	}{
		{"unimp",
			issuelink.UnimplementedError(link, "woo"),
			woo, `
woo
(1) woo
  | unimplemented
  | issue: http://mysite
Error types: (1) *issuelink.unimplementedError`},
		{"unimp-details",
			issuelink.UnimplementedError(issuelink.IssueLink{IssueURL: "http://mysite", Detail: "see more"}, "woo"),
			woo, `
woo
(1) woo
  | unimplemented
  | issue: http://mysite
  | detail: see more
Error types: (1) *issuelink.unimplementedError`},

		{"wrapper + unimp",
			&werrFmt{issuelink.UnimplementedError(link, "woo"), "waa"},
			waawoo, `
waa: woo
(1) waa
  | -- this is waa's
  | multi-line payload
Wraps: (2) woo
  | unimplemented
  | issue: http://mysite
Error types: (1) *issuelink_test.werrFmt (2) *issuelink.unimplementedError`},
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
