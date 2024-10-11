// Copyright (c) 2013-2021 The btcsuite developers
// Copyright (c) 2015-2021 The Decred developers

package btcec

import (
	secp "github.com/decred/dcrd/dcrec/secp256k1/v4"
)

// ModNScalar implements optimized 256-bit constant-time fixed-precision
// arithmetic over the secp256k1 group order. This means all arithmetic is
// performed modulo:
//
//	0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141
//
// It only implements the arithmetic needed for elliptic curve operations,
// however, the operations that are not implemented can typically be worked
// around if absolutely needed.  For example, subtraction can be performed by
// adding the negation.
//
// Should it be absolutely necessary, conversion to the standard library
// math/big.Int can be accomplished by using the Bytes method, slicing the
// resulting fixed-size array, and feeding it to big.Int.SetBytes.  However,
// that should typically be avoided when possible as conversion to big.Ints
// requires allocations, is not constant time, and is slower when working modulo
// the group order.
type ModNScalar = secp.ModNScalar

// NonceRFC6979 generates a nonce deterministically according to RFC 6979 using
// HMAC-SHA256 for the hashing function.  It takes a 32-byte hash as an input
// and returns a 32-byte nonce to be used for deterministic signing.  The extra
// and version arguments are optional, but allow additional data to be added to
// the input of the HMAC.  When provided, the extra data must be 32-bytes and
// version must be 16 bytes or they will be ignored.
//
// Finally, the extraIterations parameter provides a method to produce a stream
// of deterministic nonces to ensure the signing code is able to produce a nonce
// that results in a valid signature in the extremely unlikely event the
// original nonce produced results in an invalid signature (e.g. R == 0).
// Signing code should start with 0 and increment it if necessary.
func NonceRFC6979(privKey []byte, hash []byte, extra []byte, version []byte,
	extraIterations uint32) *ModNScalar {

	return secp.NonceRFC6979(privKey, hash, extra, version, extraIterations)
}
