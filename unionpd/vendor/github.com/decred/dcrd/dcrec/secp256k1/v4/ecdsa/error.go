// Copyright (c) 2020-2022 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package ecdsa

// ErrorKind identifies a kind of error.  It has full support for
// errors.Is and errors.As, so the caller can directly check against
// an error kind when determining the reason for an error.
type ErrorKind string

// These constants are used to identify a specific Error.
const (
	// ErrSigTooShort is returned when a signature that should be a DER
	// signature is too short.
	ErrSigTooShort = ErrorKind("ErrSigTooShort")

	// ErrSigTooLong is returned when a signature that should be a DER signature
	// is too long.
	ErrSigTooLong = ErrorKind("ErrSigTooLong")

	// ErrSigInvalidSeqID is returned when a signature that should be a DER
	// signature does not have the expected ASN.1 sequence ID.
	ErrSigInvalidSeqID = ErrorKind("ErrSigInvalidSeqID")

	// ErrSigInvalidDataLen is returned when a signature that should be a DER
	// signature does not specify the correct number of remaining bytes for the
	// R and S portions.
	ErrSigInvalidDataLen = ErrorKind("ErrSigInvalidDataLen")

	// ErrSigMissingSTypeID is returned when a signature that should be a DER
	// signature does not provide the ASN.1 type ID for S.
	ErrSigMissingSTypeID = ErrorKind("ErrSigMissingSTypeID")

	// ErrSigMissingSLen is returned when a signature that should be a DER
	// signature does not provide the length of S.
	ErrSigMissingSLen = ErrorKind("ErrSigMissingSLen")

	// ErrSigInvalidSLen is returned when a signature that should be a DER
	// signature does not specify the correct number of bytes for the S portion.
	ErrSigInvalidSLen = ErrorKind("ErrSigInvalidSLen")

	// ErrSigInvalidRIntID is returned when a signature that should be a DER
	// signature does not have the expected ASN.1 integer ID for R.
	ErrSigInvalidRIntID = ErrorKind("ErrSigInvalidRIntID")

	// ErrSigZeroRLen is returned when a signature that should be a DER
	// signature has an R length of zero.
	ErrSigZeroRLen = ErrorKind("ErrSigZeroRLen")

	// ErrSigNegativeR is returned when a signature that should be a DER
	// signature has a negative value for R.
	ErrSigNegativeR = ErrorKind("ErrSigNegativeR")

	// ErrSigTooMuchRPadding is returned when a signature that should be a DER
	// signature has too much padding for R.
	ErrSigTooMuchRPadding = ErrorKind("ErrSigTooMuchRPadding")

	// ErrSigRIsZero is returned when a signature has R set to the value zero.
	ErrSigRIsZero = ErrorKind("ErrSigRIsZero")

	// ErrSigRTooBig is returned when a signature has R with a value that is
	// greater than or equal to the group order.
	ErrSigRTooBig = ErrorKind("ErrSigRTooBig")

	// ErrSigInvalidSIntID is returned when a signature that should be a DER
	// signature does not have the expected ASN.1 integer ID for S.
	ErrSigInvalidSIntID = ErrorKind("ErrSigInvalidSIntID")

	// ErrSigZeroSLen is returned when a signature that should be a DER
	// signature has an S length of zero.
	ErrSigZeroSLen = ErrorKind("ErrSigZeroSLen")

	// ErrSigNegativeS is returned when a signature that should be a DER
	// signature has a negative value for S.
	ErrSigNegativeS = ErrorKind("ErrSigNegativeS")

	// ErrSigTooMuchSPadding is returned when a signature that should be a DER
	// signature has too much padding for S.
	ErrSigTooMuchSPadding = ErrorKind("ErrSigTooMuchSPadding")

	// ErrSigSIsZero is returned when a signature has S set to the value zero.
	ErrSigSIsZero = ErrorKind("ErrSigSIsZero")

	// ErrSigSTooBig is returned when a signature has S with a value that is
	// greater than or equal to the group order.
	ErrSigSTooBig = ErrorKind("ErrSigSTooBig")

	// ErrSigInvalidLen is returned when a signature that should be a compact
	// signature is not the required length.
	ErrSigInvalidLen = ErrorKind("ErrSigInvalidLen")

	// ErrSigInvalidRecoveryCode is returned when a signature that should be a
	// compact signature has an invalid value for the public key recovery code.
	ErrSigInvalidRecoveryCode = ErrorKind("ErrSigInvalidRecoveryCode")

	// ErrSigOverflowsPrime is returned when a signature that should be a
	// compact signature has the overflow bit set but adding the order to it
	// would overflow the underlying field prime.
	ErrSigOverflowsPrime = ErrorKind("ErrSigOverflowsPrime")

	// ErrPointNotOnCurve is returned when attempting to recover a public key
	// from a compact signature results in a point that is not on the elliptic
	// curve.
	ErrPointNotOnCurve = ErrorKind("ErrPointNotOnCurve")
)

// Error satisfies the error interface and prints human-readable errors.
func (e ErrorKind) Error() string {
	return string(e)
}

// Error identifies an error related to an ECDSA signature. It has full
// support for errors.Is and errors.As, so the caller can ascertain the
// specific reason for the error by checking the underlying error.
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

// signatureError creates an Error given a set of arguments.
func signatureError(kind ErrorKind, desc string) Error {
	return Error{Err: kind, Description: desc}
}
