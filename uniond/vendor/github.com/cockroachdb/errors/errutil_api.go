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

import (
	"github.com/cockroachdb/errors/barriers"
	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/errutil"
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
func New(msg string) error { return errutil.NewWithDepth(1, msg) }

// NewWithDepth is like New() except the depth to capture the stack
// trace is configurable.
// See the doc of `New()` for more details.
func NewWithDepth(depth int, msg string) error { return errutil.NewWithDepth(depth+1, msg) }

// Newf creates an error with a formatted error message.
// A stack trace is retained.
//
// Note: the format string is assumed to not contain
// PII and is included in Sentry reports.
// Use errors.Newf("%s", <unsafestring>) for errors
// strings that may contain PII information.
//
// See the doc of `New()` for more details.
func Newf(format string, args ...interface{}) error { return errutil.NewWithDepthf(1, format, args...) }

// NewWithDepthf is like Newf() except the depth to capture the stack
// trace is configurable.
// See the doc of `New()` for more details.
func NewWithDepthf(depth int, format string, args ...interface{}) error {
	return errutil.NewWithDepthf(depth+1, format, args...)
}

// Errorf aliases Newf().
func Errorf(format string, args ...interface{}) error {
	return errutil.NewWithDepthf(1, format, args...)
}

// Cause aliases UnwrapAll() for compatibility with github.com/pkg/errors.
func Cause(err error) error { return errbase.UnwrapAll(err) }

// Unwrap aliases UnwrapOnce() for compatibility with xerrors.
func Unwrap(err error) error { return errbase.UnwrapOnce(err) }

// Wrapper is the type of an error wrapper.
type Wrapper interface {
	Unwrap() error
}

// Opaque aliases barrier.Handled(), for compatibility with xerrors.
func Opaque(err error) error { return barriers.Handled(err) }

// WithMessage annotates err with a new message.
// If err is nil, WithMessage returns nil.
// The message is considered safe for reporting
// and is included in Sentry reports.
func WithMessage(err error, msg string) error { return errutil.WithMessage(err, msg) }

// WithMessagef annotates err with the format specifier.
// If err is nil, WithMessagef returns nil.
// The message is formatted as per redact.Sprintf,
// to separate safe and unsafe strings for Sentry reporting.
func WithMessagef(err error, format string, args ...interface{}) error {
	return errutil.WithMessagef(err, format, args...)
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
func Wrap(err error, msg string) error { return errutil.WrapWithDepth(1, err, msg) }

// WrapWithDepth is like Wrap except the depth to capture the stack
// trace is configurable.
// The the doc of `Wrap()` for more details.
func WrapWithDepth(depth int, err error, msg string) error {
	return errutil.WrapWithDepth(depth+1, err, msg)
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
	return errutil.WrapWithDepthf(1, err, format, args...)
}

// WrapWithDepthf is like Wrapf except the depth to capture the stack
// trace is configurable.
// The the doc of `Wrapf()` for more details.
func WrapWithDepthf(depth int, err error, format string, args ...interface{}) error {
	return errutil.WrapWithDepthf(depth+1, err, format, args...)
}

// AssertionFailedf creates an internal error.
//
// Detail is shown:
// - via `errors.GetSafeDetails()`, shows redacted strings.
// - when formatting with `%+v`.
// - in Sentry reports.
func AssertionFailedf(format string, args ...interface{}) error {
	return errutil.AssertionFailedWithDepthf(1, format, args...)
}

// AssertionFailedWithDepthf creates an internal error
// with a stack trace collected at the specified depth.
// See the doc of `AssertionFailedf()` for more details.
func AssertionFailedWithDepthf(depth int, format string, args ...interface{}) error {
	return errutil.AssertionFailedWithDepthf(depth+1, format, args...)
}

// NewAssertionErrorWithWrappedErrf wraps an error and turns it into
// an assertion error. Both details from the original error and the
// context of the caller are preserved. The original error is not
// visible as cause any more. The original error message is preserved.
// See the doc of `AssertionFailedf()` for more details.
func NewAssertionErrorWithWrappedErrf(origErr error, format string, args ...interface{}) error {
	return errutil.NewAssertionErrorWithWrappedErrDepthf(1, origErr, format, args...)
}

// HandleAsAssertionFailure hides an error and turns it into
// an assertion failure. Both details from the original error and the
// context of the caller are preserved. The original error is not
// visible as cause any more. The original error message is preserved.
// See the doc of `AssertionFailedf()` for more details.
func HandleAsAssertionFailure(origErr error) error {
	return errutil.HandleAsAssertionFailureDepth(1, origErr)
}

// HandleAsAssertionFailureDepth is like HandleAsAssertionFailure but
// the depth at which the call stack is captured can be specified.
func HandleAsAssertionFailureDepth(depth int, origErr error) error {
	return errutil.HandleAsAssertionFailureDepth(1+depth, origErr)
}

// As finds the first error in err's chain that matches the type to which target
// points, and if so, sets the target to its value and returns true. An error
// matches a type if it is assignable to the target type, or if it has a method
// As(interface{}) bool such that As(target) returns true. As will panic if target
// is not a non-nil pointer to a type which implements error or is of interface type.
//
// The As method should set the target to its value and return true if err
// matches the type to which target points.
//
// Note: this implementation differs from that of xerrors as follows:
// - it also supports recursing through causes with Cause().
// - if it detects an API use error, its panic object is a valid error.
func As(err error, target interface{}) bool { return errutil.As(err, target) }
