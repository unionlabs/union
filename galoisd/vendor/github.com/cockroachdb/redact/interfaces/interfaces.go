// Copyright 2021 The Cockroach Authors.
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

package interfaces

import "fmt"

// SafeFormatter is implemented by object types that want to separate
// safe and non-safe information when printed out by a Printf-like
// formatter.
type SafeFormatter interface {
	// SafeFormat is like the Format method of fmt.Formatter, except
	// that it operates using a SafePrinter instead of a fmt.State for
	// output.
	//
	// The verb argument is the control character that defines
	// the formatting mode in the surrounding Printf call.
	// For example, if this method is called to format %03d,
	// the verb is 'd'.
	SafeFormat(s SafePrinter, verb rune)
}

// SafePrinter is a stateful helper that abstracts an output stream in
// the context of printf-like formatting, but with the ability to
// separate safe and unsafe bits of data.
//
// This package provides one implementation of this using marker
// delimiters for unsafe data, see markers.go. We would like to aim
// for alternate implementations to generate more structured formats.
type SafePrinter interface {
	// SafePrinter inherits fmt.State to access format flags, however
	// calls to fmt.State's underlying Write() as unsafe.
	fmt.State

	// SafePrinter provides the SafeWriter interface.
	SafeWriter
}

// SafeWriter provides helper functions for use in implementations of
// SafeFormatter, to format mixes of safe and unsafe strings.
type SafeWriter interface {
	// SafeString emits a safe string.
	SafeString(SafeString)

	// SafeInt emits a safe integer.
	SafeInt(SafeInt)

	// SafeUint emits a safe unsigned integer.
	SafeUint(SafeUint)

	// SafeFloat emits a safe floating-point value.
	SafeFloat(SafeFloat)

	// SafeRune emits a safe rune.
	SafeRune(SafeRune)

	// Print emits its arguments separated by spaces.
	// For each argument it dynamically checks for the SafeFormatter or
	// SafeValue interface and either use that, or mark the argument
	// payload as unsafe.
	Print(args ...interface{})

	// For printf, a linter checks that the format string is
	// a constant literal, so the implementation can assume it's always
	// safe.
	Printf(format string, arg ...interface{})

	// UnsafeString writes an unsafe string.
	UnsafeString(string)

	// UnsafeByte writes an unsafe byte.
	UnsafeByte(byte)

	// UnsafeBytes writes an unsafe byte slice.
	UnsafeBytes([]byte)

	// UnsafeRune writes an unsafe rune.
	UnsafeRune(rune)
}

// SafeString represents a string that is not a sensitive value.
type SafeString string

// SafeValue makes SafeString a SafeValue.
func (SafeString) SafeValue() {}

// SafeInt represents an integer that is not a sensitive value.
type SafeInt int64

// SafeValue makes SafeInt a SafeValue.
func (SafeInt) SafeValue() {}

// SafeUint represents an integer that is not a sensitive value.
type SafeUint uint64

// SafeValue makes SafeUint a SafeValue.
func (SafeUint) SafeValue() {}

// SafeFloat represents a floating-point value that is not a sensitive value.
type SafeFloat float64

// SafeValue makes SafeFloat a SafeValue.
func (SafeFloat) SafeValue() {}

// SafeRune aliases rune. See the explanation for SafeString.
type SafeRune rune

// SafeValue makes SafeRune a SafeValue.
func (SafeRune) SafeValue() {}

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
type SafeValue interface {
	SafeValue()
}

// SafeMessager is an alternative to SafeFormatter used in previous
// versions of CockroachDB.
// NB: this interface is obsolete. Use SafeFormatter instead.
// TODO(knz): Remove this.
type SafeMessager = interface {
	SafeMessage() string
}
