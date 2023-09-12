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

package errutil

import (
	"github.com/cockroachdb/errors/secondary"
	"github.com/cockroachdb/errors/withstack"
	"github.com/cockroachdb/redact"
)

// New creates an error with a simple error message.
// A stack trace is retained.
//
// Note: the message string is assumed to not contain
// PII and is included in Sentry reports.
// Use errors.Newf("%s", <unsafestring>) for errors
// strings that may contain PII information.
//
// Detail output:
// - message via `Error()` and formatting using `%v`/`%s`/`%q`.
// - everything when formatting with `%+v`.
// - stack trace and message via `errors.GetSafeDetails()`.
// - stack trace and message in Sentry reports.
func New(msg string) error {
	return NewWithDepth(1, msg)
}

// NewWithDepth is like New() except the depth to capture the stack
// trace is configurable.
// See the doc of `New()` for more details.
func NewWithDepth(depth int, msg string) error {
	err := error(&leafError{redact.Sprint(redact.Safe(msg))})
	err = withstack.WithStackDepth(err, 1+depth)
	return err
}

// Newf creates an error with a formatted error message.
// A stack trace is retained.
//
// Note: the format string is assumed to not contain
// PII and is included in Sentry reports.
// Use errors.Newf("%s", <unsafestring>) for errors
// strings that may contain PII information.
//
// See the doc of `New()` for more details.
func Newf(format string, args ...interface{}) error {
	return NewWithDepthf(1, format, args...)
}

// NewWithDepthf is like Newf() except the depth to capture the stack
// trace is configurable.
// See the doc of `New()` for more details.
func NewWithDepthf(depth int, format string, args ...interface{}) error {
	// If there's the verb %w in here, shortcut to fmt.Errorf()
	// and store the safe details as extra payload. That's
	// because we don't want to re-implement the error wrapping
	// logic from 'fmt' in there.
	var err error
	var errRefs []error
	for _, a := range args {
		if e, ok := a.(error); ok {
			errRefs = append(errRefs, e)
		}
	}
	redactable, wrappedErr := redact.HelperForErrorf(format, args...)
	if wrappedErr != nil {
		err = &withNewMessage{cause: wrappedErr, message: redactable}
	} else {
		err = &leafError{redactable}
	}
	for _, e := range errRefs {
		err = secondary.WithSecondaryError(err, e)
	}
	err = withstack.WithStackDepth(err, 1+depth)
	return err
}

// Wrap wraps an error with a message prefix.
// A stack trace is retained.
//
// Note: the prefix string is assumed to not contain
// PII and is included in Sentry reports.
// Use errors.Wrapf(err, "%s", <unsafestring>) for errors
// strings that may contain PII information.
//
// Detail output:
// - original error message + prefix via `Error()` and formatting using `%v`/`%s`/`%q`.
// - everything when formatting with `%+v`.
// - stack trace and message via `errors.GetSafeDetails()`.
// - stack trace and message in Sentry reports.
func Wrap(err error, msg string) error {
	return WrapWithDepth(1, err, msg)
}

// WrapWithDepth is like Wrap except the depth to capture the stack
// trace is configurable.
// The the doc of `Wrap()` for more details.
func WrapWithDepth(depth int, err error, msg string) error {
	if err == nil {
		return nil
	}
	if msg != "" {
		err = WithMessage(err, msg)
	}
	err = withstack.WithStackDepth(err, depth+1)
	return err
}

// Wrapf wraps an error with a formatted message prefix. A stack
// trace is also retained. If the format is empty, no prefix is added,
// but the extra arguments are still processed for reportable strings.
//
// Note: the format string is assumed to not contain
// PII and is included in Sentry reports.
// Use errors.Wrapf(err, "%s", <unsafestring>) for errors
// strings that may contain PII information.
//
// Detail output:
// - original error message + prefix via `Error()` and formatting using `%v`/`%s`/`%q`.
// - everything when formatting with `%+v`.
// - stack trace, format, and redacted details via `errors.GetSafeDetails()`.
// - stack trace, format, and redacted details in Sentry reports.
func Wrapf(err error, format string, args ...interface{}) error {
	return WrapWithDepthf(1, err, format, args...)
}

// WrapWithDepthf is like Wrapf except the depth to capture the stack
// trace is configurable.
// The the doc of `Wrapf()` for more details.
func WrapWithDepthf(depth int, err error, format string, args ...interface{}) error {
	if err == nil {
		return nil
	}
	var errRefs []error
	for _, a := range args {
		if e, ok := a.(error); ok {
			errRefs = append(errRefs, e)
		}
	}
	if format != "" || len(args) > 0 {
		err = WithMessagef(err, format, args...)
	}
	for _, e := range errRefs {
		err = secondary.WithSecondaryError(err, e)
	}
	err = withstack.WithStackDepth(err, depth+1)
	return err
}
