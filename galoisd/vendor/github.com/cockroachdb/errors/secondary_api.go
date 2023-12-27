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

package errors

import "github.com/cockroachdb/errors/secondary"

// WithSecondaryError enhances the error given as first argument with
// an annotation that carries the error given as second argument.  The
// second error does not participate in cause analysis (Is, etc) and
// is only revealed when printing out the error or collecting safe
// (PII-free) details for reporting.
//
// If additionalErr is nil, the first error is returned as-is.
//
// Tip: consider using CombineErrors() below in the general case.
//
// Detail is shown:
// - via `errors.GetSafeDetails()`, shows details from secondary error.
// - when formatting with `%+v`.
// - in Sentry reports.
func WithSecondaryError(err error, additionalErr error) error {
	return secondary.WithSecondaryError(err, additionalErr)
}

// CombineErrors returns err, or, if err is nil, otherErr.
// if err is non-nil, otherErr is attached as secondary error.
// See the documentation of `WithSecondaryError()` for details.
func CombineErrors(err, otherErr error) error {
	return secondary.CombineErrors(err, otherErr)
}
