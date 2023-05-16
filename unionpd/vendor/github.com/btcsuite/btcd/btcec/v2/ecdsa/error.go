// Copyright (c) 2013-2021 The btcsuite developers
// Copyright (c) 2015-2021 The Decred developers

package ecdsa

import (
	secp_ecdsa "github.com/decred/dcrd/dcrec/secp256k1/v4/ecdsa"
)

// ErrorKind identifies a kind of error.  It has full support for
// errors.Is and errors.As, so the caller can directly check against
// an error kind when determining the reason for an error.
type ErrorKind = secp_ecdsa.ErrorKind

// Error identifies an error related to an ECDSA signature. It has full
// support for errors.Is and errors.As, so the caller can ascertain the
// specific reason for the error by checking the underlying error.
type Error = secp_ecdsa.ErrorKind
