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

import "github.com/cockroachdb/errors/hintdetail"

// ErrorHinter is implemented by types that can provide
// user-informing detail strings. This is implemented by withHint
// here, withIssueLink, assertionFailure and pgerror.Error.
type ErrorHinter = hintdetail.ErrorHinter

// ErrorDetailer is implemented by types that can provide
// user-informing detail strings.
type ErrorDetailer = hintdetail.ErrorDetailer

// WithHint decorates an error with a textual hint.
// The hint may contain PII and thus will not reportable.
// The suggested use case for hint is to relay information to end users.
//
// Hint is shown:
// - when formatting with `%+v`.
// - with `GetAllHints()` / `FlattenHints()` below.
//
// Note: the hint does not appear in the main error message returned
// with Error(). Use GetAllHints() or FlattenHints() to retrieve it.
func WithHint(err error, msg string) error { return hintdetail.WithHint(err, msg) }

// WithHintf is a helper that formats the hint.
// See the documentation of WithHint() for details.
func WithHintf(err error, format string, args ...interface{}) error {
	return hintdetail.WithHintf(err, format, args...)
}

// WithDetail decorates an error with a textual detail.
// The detail may contain PII and thus will not reportable.
// The suggested use case for detail is to augment errors with information
// useful for debugging.
//
// Detail is shown:
// - when formatting with `%+v`.
// - with `GetAllDetails()` / `FlattenDetails()` below.
//
// Note: the detail does not appear in the main error message returned
// with Error(). Use GetAllDetails() or FlattenDetails() to retrieve
// it.
func WithDetail(err error, msg string) error { return hintdetail.WithDetail(err, msg) }

// WithDetailf is a helper that formats the detail string.
// See the documentation of WithDetail() for details.
func WithDetailf(err error, format string, args ...interface{}) error {
	return hintdetail.WithDetailf(err, format, args...)
}

// GetAllHints retrieves the hints from the error using in post-order
// traversal. The hints are de-duplicated. Assertion failures, issue
// links and unimplemented errors are detected and receive standard
// hints.
func GetAllHints(err error) []string { return hintdetail.GetAllHints(err) }

// FlattenHints retrieves the hints as per GetAllHints() and
// concatenates them into a single string.
func FlattenHints(err error) string { return hintdetail.FlattenHints(err) }

// GetAllDetails retrieves the details from the error using in post-order
// traversal.
func GetAllDetails(err error) []string { return hintdetail.GetAllDetails(err) }

// FlattenDetails retrieves the details as per GetAllDetails() and
// concatenates them into a single string.
func FlattenDetails(err error) string { return hintdetail.FlattenDetails(err) }
