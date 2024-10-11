// Copyright (c) 2020-2023 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

/*
Package ecdsa provides secp256k1-optimized ECDSA signing and verification.

This package provides data structures and functions necessary to produce and
verify deterministic canonical signatures in accordance with RFC6979 and
BIP0062, optimized specifically for the secp256k1 curve using the Elliptic Curve
Digital Signature Algorithm (ECDSA), as defined in FIPS 186-3.  See
https://www.secg.org/sec2-v2.pdf for details on the secp256k1 standard.

It also provides functions to parse and serialize the ECDSA signatures with the
more strict Distinguished Encoding Rules (DER) of ISO/IEC 8825-1 and some
additional restrictions specific to secp256k1.

In addition, it supports a custom "compact" signature format which allows
efficient recovery of the public key from a given valid signature and message
hash combination.

A comprehensive suite of tests is provided to ensure proper functionality.

# ECDSA use in Decred

At the time of this writing, ECDSA signatures are heavily used for proving coin
ownership in Decred as the vast majority of transactions consist of what is
effectively transferring ownership of coins to a public key associated with a
private key only known to the recipient of the coins along with an encumbrance
that requires an ECDSA signature that proves the new owner possesses the private
key without actually revealing it.

# Errors

The errors returned by this package are of type ecdsa.Error and fully support
the standard library errors.Is and errors.As functions.  This allows the caller
to programmatically determine the specific error by examining the ErrorKind
field of the type asserted ecdsa.Error while still providing rich error messages
with contextual information.  See ErrorKind in the package documentation for a
full list.
*/
package ecdsa
