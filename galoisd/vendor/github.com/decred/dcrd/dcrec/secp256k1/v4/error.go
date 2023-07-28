// Copyright (c) 2020 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1

// ErrorKind identifies a kind of error.  It has full support for errors.Is and
// errors.As, so the caller can directly check against an error kind when
// determining the reason for an error.
type ErrorKind string

// These constants are used to identify a specific RuleError.
const (
	// ErrPubKeyInvalidLen indicates that the length of a serialized public
	// key is not one of the allowed lengths.
	ErrPubKeyInvalidLen = ErrorKind("ErrPubKeyInvalidLen")

	// ErrPubKeyInvalidFormat indicates an attempt was made to parse a public
	// key that does not specify one of the supported formats.
	ErrPubKeyInvalidFormat = ErrorKind("ErrPubKeyInvalidFormat")

	// ErrPubKeyXTooBig indicates that the x coordinate for a public key
	// is greater than or equal to the prime of the field underlying the group.
	ErrPubKeyXTooBig = ErrorKind("ErrPubKeyXTooBig")

	// ErrPubKeyYTooBig indicates that the y coordinate for a public key is
	// greater than or equal to the prime of the field underlying the group.
	ErrPubKeyYTooBig = ErrorKind("ErrPubKeyYTooBig")

	// ErrPubKeyNotOnCurve indicates that a public key is not a point on the
	// secp256k1 curve.
	ErrPubKeyNotOnCurve = ErrorKind("ErrPubKeyNotOnCurve")

	// ErrPubKeyMismatchedOddness indicates that a hybrid public key specified
	// an oddness of the y coordinate that does not match the actual oddness of
	// the provided y coordinate.
	ErrPubKeyMismatchedOddness = ErrorKind("ErrPubKeyMismatchedOddness")
)

// Error satisfies the error interface and prints human-readable errors.
func (e ErrorKind) Error() string {
	return string(e)
}

// Error identifies an error related to public key cryptography using a
// sec256k1 curve. It has full support for errors.Is and errors.As, so the
// caller can  ascertain the specific reason for the error by checking
// the underlying error.
type Error struct {
	Err         error
	Description string
}

// Error satisfies the error interface and prints human-readable errors.
func (e Error) Error() string {
	return e.Description
}

// Unwrap returns the underlying wrapped error.
func (e Error) Unwrap() error {
	return e.Err
}

// makeError creates an Error given a set of arguments.
func makeError(kind ErrorKind, desc string) Error {
	return Error{Err: kind, Description: desc}
}
