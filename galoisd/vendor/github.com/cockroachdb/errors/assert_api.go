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

import "github.com/cockroachdb/errors/assert"

// WithAssertionFailure decorates the error with an assertion failure marker.
// This is not intended to be used directly (see AssertionFailed() for
// further decoration).
//
// Detail is shown:
// - when formatting with `%+v`.
// - in Sentry reports.
func WithAssertionFailure(err error) error { return assert.WithAssertionFailure(err) }

// HasAssertionFailure returns true if the error or any of its causes
// is an assertion failure annotation.
func HasAssertionFailure(err error) bool { return assert.HasAssertionFailure(err) }

// IsAssertionFailure returns true if the error (not its causes) is an
// assertion failure annotation. Consider using markers.If or
// HasAssertionFailure to test both the error and its causes.
func IsAssertionFailure(err error) bool { return assert.IsAssertionFailure(err) }
