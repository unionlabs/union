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

package domains_test

import (
	goErr "errors"
	"fmt"
	"strings"
	"testing"

	"github.com/cockroachdb/errors/domains"
	"github.com/cockroachdb/errors/domains/internal"
	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/markers"
	"github.com/cockroachdb/errors/testutils"
	"github.com/kr/pretty"
	"github.com/pkg/errors"
)

// This test demonstrates that simple errors using the standard Go
// types have no domain.
func TestNoDomain(t *testing.T) {
	err := errors.New("hello")

	tt := testutils.T{T: t}

	tt.CheckEqual(domains.GetDomain(err), domains.NoDomain)
	tt.Check(!domains.NotInDomain(err, domains.NoDomain))
}

// This test demonstrates how it is possible to create a custom domain
// from an arbitrary string.
func TestNamedDomain(t *testing.T) {
	myDomain := domains.NamedDomain("here")
	err := errors.New("hello")
	err = domains.WithDomain(err, myDomain)

	tt := testutils.T{T: t}

	tt.CheckEqual(domains.GetDomain(err), myDomain)
	tt.Check(!domains.NotInDomain(err, myDomain))
	tt.Check(domains.NotInDomain(err, domains.NoDomain))
}

// This test demonstrates that the domain information is visible
// "through" layers of wrapping.
func TestWrappedDomain(t *testing.T) {
	myDomain := domains.NamedDomain("here")
	err := errors.New("hello")
	err = domains.WithDomain(err, myDomain)
	err = errors.Wrap(err, "world")

	tt := testutils.T{T: t}

	tt.Check(!domains.NotInDomain(err, myDomain))
}

// This test demonstrates how it is possible to leave the domain
// implicit, in which case a domain is computed automatically based on
// which package instantiates the error.
func TestPackageDomain(t *testing.T) {
	otherErr := internal.NewError("hello")

	// Then errors fall into the local "package domain" where the error
	// was instantiated.
	otherDomain := internal.ThisDomain

	tt := testutils.T{T: t}

	tt.CheckEqual(domains.GetDomain(otherErr), otherDomain)
	tt.Check(!domains.NotInDomain(otherErr, otherDomain))
	tt.Check(domains.NotInDomain(otherErr, domains.NoDomain))

	hereDomain := domains.PackageDomain()

	tt.Check(hereDomain != otherDomain)

	hereErr := domains.New("hello")

	tt.CheckEqual(domains.GetDomain(hereErr), hereDomain)
	tt.Check(!domains.NotInDomain(hereErr, hereDomain))
	tt.Check(domains.NotInDomain(hereErr, domains.NoDomain))
}

// This test demonstrates how the original domain becomes invisible
// via WithDomain(), but the original error remains visible as cause.
func TestWithDomain(t *testing.T) {
	origErr := domains.New("hello")
	t.Logf("origErr: %# v", pretty.Formatter(origErr))
	origDomain := domains.GetDomain(origErr)

	otherDomain := domains.NamedDomain("woo")
	err := domains.WithDomain(origErr, otherDomain)
	t.Logf("err: %# v", pretty.Formatter(err))

	tt := testutils.T{T: t}

	// The original domain becomes invisible.
	tt.Check(domains.NotInDomain(err, origDomain))

	// The cause remains visible.
	tt.Check(markers.Is(err, origErr))

	// Moreover, the error message is preserved fully.
	tt.CheckEqual(err.Error(), "hello")
}

// This test demonstrates how the original domain becomes invisible
// via HandledInDomain(), and even the original error becomes invisible.
func TestHandledInDomain(t *testing.T) {
	origErr := domains.New("hello")
	t.Logf("origErr: %# v", pretty.Formatter(origErr))
	origDomain := domains.GetDomain(origErr)

	otherDomain := domains.NamedDomain("woo")
	err := domains.HandledInDomain(origErr, otherDomain)
	t.Logf("err: %# v", pretty.Formatter(err))

	tt := testutils.T{T: t}

	// The original domain becomes invisible.
	tt.Check(domains.NotInDomain(err, origDomain))

	// The cause becomes invisible.
	tt.Check(!markers.Is(err, origErr))

	// However, the error message is preserved fully.
	tt.CheckEqual(err.Error(), "hello")
}

// This test demonstrates that Handled() overrides an error's original
// domain with the current package's local domain.
func TestHandled(t *testing.T) {
	wooDomain := domains.NamedDomain("woo")
	origErr := domains.WithDomain(errors.New("hello"), wooDomain)

	tt := testutils.T{T: t}

	// Sanity check.
	tt.Assert(!domains.NotInDomain(origErr, wooDomain))

	// Handled overrides the domain with the current (package) domain.
	otherErr := domains.Handled(origErr)

	// The original domain disappears.
	tt.Check(domains.NotInDomain(otherErr, wooDomain))

	// The local domain appears.
	tt.Check(!domains.NotInDomain(otherErr, domains.PackageDomain()))
}

// This test demonstrates that same error annotated with different
// domains appears to be different errors for the purpose of Is()
// comparisons.
func TestDomainsBreakErrorEquivalence(t *testing.T) {
	baseErr := errors.New("hello")

	err1 := domains.WithDomain(baseErr, domains.NamedDomain("woo"))
	err2 := domains.WithDomain(baseErr, domains.NamedDomain("waa"))

	tt := testutils.T{T: t}

	tt.Check(!markers.Is(err1, err2))
	tt.Check(!markers.Is(err2, err1))
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
		{"keys",
			domains.WithDomain(baseErr, domains.NoDomain),
			woo, `
woo
(1) error domain: <none>
Wraps: (2) woo
Error types: (1) *domains.withDomain (2) *errors.errorString`},

		{"keys + wrapper",
			domains.WithDomain(&werrFmt{baseErr, "waa"}, domains.NoDomain),
			waawoo, `
waa: woo
(1) error domain: <none>
Wraps: (2) waa
  | -- this is waa's
  | multi-line payload
Wraps: (3) woo
Error types: (1) *domains.withDomain (2) *domains_test.werrFmt (3) *errors.errorString`},

		{"wrapper + keys",
			&werrFmt{domains.WithDomain(baseErr, domains.NoDomain), "waa"},
			waawoo, `
waa: woo
(1) waa
  | -- this is waa's
  | multi-line payload
Wraps: (2) error domain: <none>
Wraps: (3) woo
Error types: (1) *domains_test.werrFmt (2) *domains.withDomain (3) *errors.errorString`},
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
