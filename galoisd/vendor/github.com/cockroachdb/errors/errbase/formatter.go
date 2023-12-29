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

// This file is taken from golang.org/x/xerrors,
// at commit 3ee3066db522c6628d440a3a91c4abdd7f5ef22f (2019-05-10).
// From the original code:
// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package errbase

// A Formatter formats error messages.
//
// NB: Consider implementing SafeFormatter instead. This will ensure
// that error displays can distinguish bits that are PII-safe.
type Formatter interface {
	error

	// FormatError prints the receiver's first error.
	// The return value decides what happens in the case
	// FormatError() is used to produce a "short" message,
	// eg. when it is used to implement Error():
	//
	// - if it returns nil, then the short message
	//   contains no more than that produced for this error,
	//   even if the error has a further causal chain.
	//
	// - if it returns non-nil, then the short message
	//   contains the value printed by this error,
	//   followed by that of its causal chain.
	//   (e.g. thiserror: itscause: furthercause)
	//
	// Note that all the causal chain is reported in verbose reports in
	// any case.
	FormatError(p Printer) (next error)
}

// SafeFormatter is implemented by error leaf or wrapper types that want
// to separate safe and non-safe information when printed out.
//
// When multiple errors are chained (e.g. via errors.Wrap), intermediate
// layers in the error that do not implement SafeError are considered
// “unsafe”
type SafeFormatter interface {
	// SafeFormatError prints the receiver's first error.
	//
	// The provided Printer behaves like a redact.SafePrinter its
	// Print() and Printf() methods conditionally add redaction markers
	// around unsafe bits.
	//
	// The return value of SafeFormatError() decides what happens in the
	// case the method is used to produce a "short" message, eg. when it
	// is used to implement Error():
	//
	// - if it returns nil, then the short message
	//   contains no more than that produced for this error,
	//   even if the error has a further causal chain.
	//
	// - if it returns non-nil, then the short message
	//   contains the value printed by this error,
	//   followed by that of its causal chain.
	//   (e.g. thiserror: itscause: furthercause)
	//
	// Note that all the causal chain is reported in verbose reports in
	// any case.
	SafeFormatError(p Printer) (next error)
}

// A Printer formats error messages.
//
// The most common implementation of Printer is the one provided by package fmt
// during Printf (as of Go 1.13). Localization packages such as golang.org/x/text/message
// typically provide their own implementations.
type Printer interface {
	// Print appends args to the message output.
	Print(args ...interface{})

	// Printf writes a formatted string.
	Printf(format string, args ...interface{})

	// Detail reports whether error detail is requested.
	// After the first call to Detail, all text written to the Printer
	// is formatted as additional detail, or ignored when
	// detail has not been requested.
	// If Detail returns false, the caller can avoid printing the detail at all.
	Detail() bool
}
