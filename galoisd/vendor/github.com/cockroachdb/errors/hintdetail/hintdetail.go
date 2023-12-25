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

package hintdetail

import (
	"bytes"
	"fmt"

	"github.com/cockroachdb/errors/errbase"
)

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
func WithHint(err error, msg string) error {
	if err == nil {
		return nil
	}

	return &withHint{cause: err, hint: msg}
}

// WithHintf is a helper that formats the hint.
func WithHintf(err error, format string, args ...interface{}) error {
	if err == nil {
		return nil
	}

	return &withHint{cause: err, hint: fmt.Sprintf(format, args...)}
}

// GetAllHints retrieves the hints from the error using in post-order
// traversal. The hints are de-duplicated. Assertion failures, issue
// links and unimplemented errors are detected and receive standard
// hints.
func GetAllHints(err error) []string {
	return getAllHintsInternal(err, nil, make(map[string]struct{}))
}

// FlattenHints retrieves the hints as per GetAllHints() and
// concatenates them into a single string.
func FlattenHints(err error) string {
	var b bytes.Buffer
	sep := ""
	for _, h := range GetAllHints(err) {
		b.WriteString(sep)
		b.WriteString(h)
		sep = "\n--\n"
	}
	return b.String()
}

func getAllHintsInternal(err error, hints []string, seen map[string]struct{}) []string {
	if c := errbase.UnwrapOnce(err); c != nil {
		hints = getAllHintsInternal(c, hints, seen)
	}

	hint := ""
	if w, ok := err.(ErrorHinter); ok {
		hint = w.ErrorHint()
	}

	if hint != "" {
		// De-duplicate hints.
		if _, ok := seen[hint]; !ok {
			hints = append(hints, hint)
			seen[hint] = struct{}{}
		}
	}
	return hints
}

// ErrorHinter is implemented by types that can provide
// user-informing detail strings. This is implemented by withHint
// here, withIssueLink, assertionFailure and pgerror.Error.
type ErrorHinter interface {
	ErrorHint() string
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
func WithDetail(err error, msg string) error {
	if err == nil {
		return nil
	}

	return &withDetail{cause: err, detail: msg}
}

// WithDetailf is a helper that formats the detail string.
func WithDetailf(err error, format string, args ...interface{}) error {
	if err == nil {
		return nil
	}

	return &withDetail{cause: err, detail: fmt.Sprintf(format, args...)}
}

// GetAllDetails retrieves the details from the error using in post-order
// traversal.
func GetAllDetails(err error) []string {
	return getAllDetailsInternal(err, nil)
}

// FlattenDetails retrieves the details as per GetAllDetails() and
// concatenates them into a single string.
func FlattenDetails(err error) string {
	var b bytes.Buffer
	sep := ""
	for _, h := range GetAllDetails(err) {
		b.WriteString(sep)
		b.WriteString(h)
		sep = "\n--\n"
	}
	return b.String()
}

func getAllDetailsInternal(err error, details []string) []string {
	if c := errbase.UnwrapOnce(err); c != nil {
		details = getAllDetailsInternal(c, details)
	}
	if w, ok := err.(ErrorDetailer); ok {
		d := w.ErrorDetail()
		if d != "" {
			details = append(details, w.ErrorDetail())
		}
	}
	return details
}

// ErrorDetailer is implemented by types that can provide
// user-informing detail strings. This is implemented by withDetail
// here and pgerror.Error.
type ErrorDetailer interface {
	ErrorDetail() string
}
