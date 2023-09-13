// Copyright 2020 The Cockroach Authors.
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

package redact

import (
	"fmt"
	"reflect"

	"github.com/cockroachdb/redact/builder"
	i "github.com/cockroachdb/redact/interfaces"
	b "github.com/cockroachdb/redact/internal/buffer"
	fw "github.com/cockroachdb/redact/internal/fmtforward"
	m "github.com/cockroachdb/redact/internal/markers"
	w "github.com/cockroachdb/redact/internal/redact"
	ifmt "github.com/cockroachdb/redact/internal/rfmt"
)

// SafeFormatter is implemented by object types that want to separate
// safe and non-safe information when printed out by a Printf-like
// formatter.
type SafeFormatter = i.SafeFormatter

// SafeValue is a marker interface to be implemented by types that
// alias base Go types and whose natural representation via Printf is
// always safe for reporting.
//
// This is recognized by the SafePrinter interface as an alternative
// to SafeFormatter.
//
// It is provided to decorate "leaf" Go types, such as aliases to int.
//
// Typically, a linter enforces that a type can only implement this
// interface if it aliases a base go type. More complex types should
// implement SafeFormatter instead.
//
// It is advised to build an automatic process during builds to
// collect all the types that implement this interface, as well as all
// uses of this type, and produce a report. Changes to this report
// should receive maximal amount of scrutiny during code reviews.
type SafeValue = i.SafeValue

// SafeMessager is an alternative to SafeFormatter used in previous
// versions of CockroachDB.
// NB: this interface is obsolete. Use SafeFormatter instead.
// TODO(knz): Remove this.
type SafeMessager = i.SafeMessager

// SafePrinter is a stateful helper that abstracts an output stream in
// the context of printf-like formatting, but with the ability to
// separate safe and unsafe bits of data.
//
// This package provides one implementation of this using marker
// delimiters for unsafe data, see markers.go. We would like to aim
// for alternate implementations to generate more structured formats.
type SafePrinter = i.SafePrinter

// SafeWriter provides helper functions for use in implementations of
// SafeFormatter, to format mixes of safe and unsafe strings.
type SafeWriter = i.SafeWriter

// SafeString represents a string that is not a sensitive value.
type SafeString = i.SafeString

// SafeInt represents an integer that is not a sensitive value.
type SafeInt = i.SafeInt

// SafeUint represents an integer that is not a sensitive value.
type SafeUint = i.SafeUint

// SafeFloat represents a floating-point value that is not a sensitive value.
type SafeFloat = i.SafeFloat

// SafeRune aliases rune. See the explanation for SafeString.
type SafeRune = i.SafeRune

// RedactableString is a string that contains a mix of safe and unsafe
// bits of data, but where it is known that unsafe bits are enclosed
// by redaction markers ‹ and ›, and occurrences of the markers
// inside the original data items have been escaped.
//
// Instances of RedactableString should not be constructed directly;
// instead use the facilities from print.go (Sprint, Sprintf)
// or the methods below.
type RedactableString = m.RedactableString

// RedactableBytes is like RedactableString but is a byte slice.
//
// Instances of RedactableBytes should not be constructed directly;
// instead use the facilities from print.go (Sprint, Sprintf)
// or the methods below.
type RedactableBytes = m.RedactableBytes

// StartMarker returns the start delimiter for an unsafe string.
func StartMarker() []byte { return m.StartMarker() }

// EndMarker returns the end delimiter for an unsafe string.
func EndMarker() []byte { return m.EndMarker() }

// RedactedMarker returns the special string used by Redact.
func RedactedMarker() []byte { return m.RedactedMarker() }

// EscapeMarkers escapes the special delimiters from the provided
// byte slice.
func EscapeMarkers(s []byte) []byte { return m.EscapeMarkers(s) }

// EscapeBytes escapes markers inside the given byte slice and encloses
// the entire byte slice between redaction markers.
// EscapeBytes escapes markers inside the given byte slice and encloses
// the entire byte slice between redaction markers.
func EscapeBytes(s []byte) RedactableBytes { return ifmt.EscapeBytes(s) }

// ManualBuffer is a variable-sized buffer of bytes with
// Write methods. Writes are escaped in different ways
// depending on the output mode.
// Note: This type is only meant for "advanced" usage.
// For most common uses, consider using StringBuilder instead.
type ManualBuffer = b.Buffer

// Unsafe turns any value that would otherwise be considered safe,
// into an unsafe value.
func Unsafe(a interface{}) interface{} { return w.Unsafe(a) }

// Safe turns any value into an object that is considered as safe by
// the formatter.
//
// This is provided as an “escape hatch” for cases where the other
// interfaces and conventions fail. Increased usage of this mechanism
// should be taken as a signal that a new abstraction is missing.
// The implementation is also slow.
func Safe(a interface{}) SafeValue { return w.Safe(a) }

// RegisterRedactErrorFn registers an error redaction function for use
// during automatic redaction by this package.
// Provided e.g. by cockroachdb/errors.
func RegisterRedactErrorFn(fn func(err error, p i.SafePrinter, verb rune)) {
	ifmt.RegisterRedactErrorFn(fn)
}

// MakeFormat is a helper for use by implementations of the
// SafeFormatter interface. It reproduces the format currently active
// in fmt.State and verb. This is provided because Go's standard
// fmt.State does not make the original format string available to us.
//
// If the return value justV is true, then the current state
// was found to be %v exactly; in that case the caller
// can avoid a full-blown Printf call and use just Print instead
// to take a shortcut.
func MakeFormat(s fmt.State, verb rune) (justV bool, format string) {
	return fw.MakeFormat(s, verb)
}

// StringBuilder accumulates strings with optional redaction markers.
//
// It implements io.Writer but marks direct writes as redactable.
// To distinguish safe and unsafe bits, it also implements the SafeWriter
// interface.
type StringBuilder = builder.StringBuilder

// RegisterSafeType registers a data type to always be considered safe
// during the production of redactable strings.
func RegisterSafeType(t reflect.Type) {
	ifmt.RegisterSafeType(t)
}
