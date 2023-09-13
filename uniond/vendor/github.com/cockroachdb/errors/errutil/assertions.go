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
	"github.com/cockroachdb/errors/assert"
	"github.com/cockroachdb/errors/barriers"
	"github.com/cockroachdb/errors/withstack"
)

// AssertionFailedf creates an internal error.
//
// Detail is shown:
// - via `errors.GetSafeDetails()`, shows redacted strings.
// - when formatting with `%+v`.
// - in Sentry reports.
func AssertionFailedf(format string, args ...interface{}) error {
	return AssertionFailedWithDepthf(1, format, args...)
}

// AssertionFailedWithDepthf creates an internal error
// with a stack trace collected at the specified depth.
// See the doc of `AssertionFailedf()` for more details.
func AssertionFailedWithDepthf(depth int, format string, args ...interface{}) error {
	err := NewWithDepthf(1+depth, format, args...)
	err = assert.WithAssertionFailure(err)
	return err
}

// HandleAsAssertionFailure hides an error and turns it into
// an assertion failure. Both details from the original error and the
// context of the caller are preserved. The original error is not
// visible as cause any more. The original error message is preserved.
// See the doc of `AssertionFailedf()` for more details.
func HandleAsAssertionFailure(origErr error) error {
	return HandleAsAssertionFailureDepth(1, origErr)
}

// HandleAsAssertionFailureDepth is like HandleAsAssertionFailure but
// the depth at which the call stack is captured can be specified.
func HandleAsAssertionFailureDepth(depth int, origErr error) error {
	err := barriers.Handled(origErr)
	err = withstack.WithStackDepth(err, 1+depth)
	err = assert.WithAssertionFailure(err)
	return err
}

// NewAssertionErrorWithWrappedErrf wraps an error and turns it into
// an assertion error. Both details from the original error and the
// context of the caller are preserved. The original error is not
// visible as cause any more. The original error message is preserved.
// See the doc of `AssertionFailedf()` for more details.
func NewAssertionErrorWithWrappedErrf(origErr error, format string, args ...interface{}) error {
	return NewAssertionErrorWithWrappedErrDepthf(1, origErr, format, args...)
}

// NewAssertionErrorWithWrappedErrDepthf is like
// NewAssertionErrorWithWrappedErrf but the depth at which the call
// stack is captured can be specified.
// See the doc of `AssertionFailedf()` for more details.
func NewAssertionErrorWithWrappedErrDepthf(
	depth int, origErr error, format string, args ...interface{},
) error {
	err := barriers.Handled(origErr)
	err = WrapWithDepthf(depth+1, err, format, args...)
	err = assert.WithAssertionFailure(err)
	return err
}
