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
	"github.com/cockroachdb/errors/safedetails"
	"github.com/cockroachdb/redact"
)

// WithSafeDetails annotates an error with the given reportable details.
// The format is made available as a PII-free string, alongside
// with a PII-free representation of every additional argument.
// Arguments can be reported as-is (without redaction) by wrapping
// them using the Safe() function.
//
// If the format is empty and there are no arguments, the
// error argument is returned unchanged.
//
// Detail is shown:
// - via `errors.GetSafeDetails()`
// - when formatting with `%+v`.
// - in Sentry reports.
func WithSafeDetails(err error, format string, args ...interface{}) error {
	return safedetails.WithSafeDetails(err, format, args...)
}

// SafeMessager aliases redact.SafeMessager.
//
// NB: this is obsolete. Use redact.SafeFormatter or
// errors.SafeFormatter instead.
type SafeMessager = redact.SafeMessager

// Safe wraps the given object into an opaque struct that implements
// SafeMessager: its contents can be included as-is in PII-free
// strings in error objects and reports.
//
// NB: this is obsolete. Use redact.Safe instead.
func Safe(v interface{}) redact.SafeValue { return safedetails.Safe(v) }

// Redact returns a redacted version of the supplied item that is safe to use in
// anonymized reporting.
//
// NB: this interface is obsolete. Use redact.Sprint() directly.
func Redact(r interface{}) string { return safedetails.Redact(r) }
