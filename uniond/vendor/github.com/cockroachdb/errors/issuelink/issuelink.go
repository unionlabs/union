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

package issuelink

import (
	"fmt"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/markers"
)

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
func WithIssueLink(err error, issue IssueLink) error {
	if err == nil {
		return nil
	}
	return &withIssueLink{cause: err, IssueLink: issue}
}

// HasIssueLink returns true iff the error or one of its
// causes has a linked issue payload.
func HasIssueLink(err error) bool {
	_, ok := markers.If(err, func(err error) (v interface{}, ok bool) {
		v, ok = err.(*withIssueLink)
		return
	})
	return ok
}

// IsIssueLink returns true iff the error (not its
// causes) has a linked issue payload.
func IsIssueLink(err error) bool {
	_, ok := err.(*withIssueLink)
	return ok
}

// GetAllIssueLinks retrieves the linked issue carried
// by the error or its direct causes.
func GetAllIssueLinks(err error) (issues []IssueLink) {
	for ; err != nil; err = errbase.UnwrapOnce(err) {
		if issue, ok := GetIssueLink(err); ok {
			issues = append(issues, issue)
		}
	}
	return
}

// UnimplementedError creates a new leaf error that indicates that
// some feature was not (yet) implemented.
//
// Detail is shown:
// - via `errors.GetSafeDetails()`
// - when formatting with `%+v`.
// - in Sentry reports.
// - via `errors.GetAllHints()` / `errors.FlattenHints()`
func UnimplementedError(issueLink IssueLink, msg string) error {
	return &unimplementedError{IssueLink: issueLink, msg: msg}
}

// UnimplementedErrorf creates a new leaf error that indicates that
// some feature was not (yet) implemented. The message is formatted.
func UnimplementedErrorf(issueLink IssueLink, format string, args ...interface{}) error {
	return &unimplementedError{IssueLink: issueLink, msg: fmt.Sprintf(format, args...)}
}

// IsUnimplementedError returns iff if err is an unimplemented error.
func IsUnimplementedError(err error) bool {
	_, ok := err.(*unimplementedError)
	return ok
}

// HasUnimplementedError returns iff if err or its cause is an
// unimplemented error.
func HasUnimplementedError(err error) bool {
	_, ok := errbase.UnwrapAll(err).(*unimplementedError)
	return ok
}

// GetIssueLink retrieves the linked issue annotation carried
// by the error, or false if there is no such annotation.
func GetIssueLink(err error) (IssueLink, bool) {
	switch w := err.(type) {
	case *withIssueLink:
		return w.IssueLink, true
	case *unimplementedError:
		return w.IssueLink, true
	}
	return IssueLink{}, false
}

// IssueLink is the payload for a linked issue annotation.
type IssueLink struct {
	// URL to the issue on a tracker.
	IssueURL string
	// Annotation that characterizes a sub-issue.
	Detail string
}
