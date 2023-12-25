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

import "github.com/cockroachdb/errors/issuelink"

// WithIssueLink adds an annotation to a know issue
// on a web issue tracker.
//
// The url and detail strings may contain PII and will
// be considered reportable.
//
// Detail is shown:
// - via `errors.GetSafeDetails()`
// - when formatting with `%+v`.
// - in Sentry reports.
// - via `errors.GetAllHints()` / `errors.FlattenHints()`
func WithIssueLink(err error, issue IssueLink) error { return issuelink.WithIssueLink(err, issue) }

// IssueLink is the payload for a linked issue annotation.
type IssueLink = issuelink.IssueLink

// UnimplementedError creates a new leaf error that indicates that
// some feature was not (yet) implemented.
//
// Detail is shown:
// - via `errors.GetSafeDetails()`
// - when formatting with `%+v`.
// - in Sentry reports.
// - via `errors.GetAllHints()` / `errors.FlattenHints()`
func UnimplementedError(issueLink IssueLink, msg string) error {
	return issuelink.UnimplementedError(issueLink, msg)
}

// UnimplementedErrorf creates a new leaf error that indicates that
// some feature was not (yet) implemented. The message is formatted.
func UnimplementedErrorf(issueLink IssueLink, format string, args ...interface{}) error {
	return issuelink.UnimplementedErrorf(issueLink, format, args...)
}

// GetAllIssueLinks retrieves the linked issue carried
// by the error or its direct causes.
func GetAllIssueLinks(err error) (issues []IssueLink) { return issuelink.GetAllIssueLinks(err) }

// HasIssueLink returns true iff the error or one of its
// causes has a linked issue payload.
func HasIssueLink(err error) bool { return issuelink.HasIssueLink(err) }

// IsIssueLink returns true iff the error (not its
// causes) has a linked issue payload.
func IsIssueLink(err error) bool { return issuelink.IsIssueLink(err) }

// HasUnimplementedError returns iff if err or its cause is an
// unimplemented error.
func HasUnimplementedError(err error) bool { return issuelink.HasUnimplementedError(err) }

// IsUnimplementedError returns iff if err is an unimplemented error.
func IsUnimplementedError(err error) bool { return issuelink.IsUnimplementedError(err) }

// UnimplementedErrorHint is the hint emitted upon unimplemented errors.
const UnimplementedErrorHint = issuelink.UnimplementedErrorHint
