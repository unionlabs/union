// Copyright (c) 2013-2014 The btcsuite developers
// Copyright (c) 2015-2022 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1

// References:
//   [SEC1] Elliptic Curve Cryptography
//     https://www.secg.org/sec1-v2.pdf
//
//   [SEC2] Recommended Elliptic Curve Domain Parameters
//     https://www.secg.org/sec2-v2.pdf
//
//   [ANSI X9.62-1998] Public Key Cryptography For The Financial Services
//     Industry: The Elliptic Curve Digital Signature Algorithm (ECDSA)

import (
	"fmt"
)

const (
	// PubKeyBytesLenCompressed is the number of bytes of a serialized
	// compressed public key.
	PubKeyBytesLenCompressed = 33

	// PubKeyBytesLenUncompressed is the number of bytes of a serialized
	// uncompressed public key.
	PubKeyBytesLenUncompressed = 65

	// PubKeyFormatCompressedEven is the identifier prefix byte for a public key
	// whose Y coordinate is even when serialized in the compressed format per
	// section 2.3.4 of [SEC1](https://secg.org/sec1-v2.pdf#subsubsection.2.3.4).
	PubKeyFormatCompressedEven byte = 0x02

	// PubKeyFormatCompressedOdd is the identifier prefix byte for a public key
	// whose Y coordinate is odd when serialized in the compressed format per
	// section 2.3.4 of [SEC1](https://secg.org/sec1-v2.pdf#subsubsection.2.3.4).
	PubKeyFormatCompressedOdd byte = 0x03

	// PubKeyFormatUncompressed is the identifier prefix byte for a public key
	// when serialized according in the uncompressed format per section 2.3.3 of
	// [SEC1](https://secg.org/sec1-v2.pdf#subsubsection.2.3.3).
	PubKeyFormatUncompressed byte = 0x04

	// PubKeyFormatHybridEven is the identifier prefix byte for a public key
	// whose Y coordinate is even when serialized according to the hybrid format
	// per section 4.3.6 of [ANSI X9.62-1998].
	//
	// NOTE: This format makes little sense in practice an therefore this
	// package will not produce public keys serialized in this format.  However,
	// it will parse them since they exist in the wild.
	PubKeyFormatHybridEven byte = 0x06

	// PubKeyFormatHybridOdd is the identifier prefix byte for a public key
	// whose Y coordingate is odd when serialized according to the hybrid format
	// per section 4.3.6 of [ANSI X9.62-1998].
	//
	// NOTE: This format makes little sense in practice an therefore this
	// package will not produce public keys serialized in this format.  However,
	// it will parse them since they exist in the wild.
	PubKeyFormatHybridOdd byte = 0x07
)

// PublicKey provides facilities for efficiently working with secp256k1 public
// keys within this package and includes functions to serialize in both
// uncompressed and compressed SEC (Standards for Efficient Cryptography)
// formats.
type PublicKey struct {
	x FieldVal
	y FieldVal
}

// NewPublicKey instantiates a new public key with the given x and y
// coordinates.
//
// It should be noted that, unlike ParsePubKey, since this accepts arbitrary x
// and y coordinates, it allows creation of public keys that are not valid
// points on the secp256k1 curve.  The IsOnCurve method of the returned instance
// can be used to determine validity.
func NewPublicKey(x, y *FieldVal) *PublicKey {
	var pubKey PublicKey
	pubKey.x.Set(x)
	pubKey.y.Set(y)
	return &pubKey
}

// ParsePubKey parses a secp256k1 public key encoded according to the format
// specified by ANSI X9.62-1998, which means it is also compatible with the
// SEC (Standards for Efficient Cryptography) specification which is a subset of
// the former.  In other words, it supports the uncompressed, compressed, and
// hybrid formats as follows:
//
// Compressed:
//
//	<format byte = 0x02/0x03><32-byte X coordinate>
//
// Uncompressed:
//
//	<format byte = 0x04><32-byte X coordinate><32-byte Y coordinate>
//
// Hybrid:
//
//	<format byte = 0x05/0x06><32-byte X coordinate><32-byte Y coordinate>
//
// NOTE: The hybrid format makes little sense in practice an therefore this
// package will not produce public keys serialized in this format.  However,
// this function will properly parse them since they exist in the wild.
func ParsePubKey(serialized []byte) (key *PublicKey, err error) {
	var x, y FieldVal
	switch len(serialized) {
	case PubKeyBytesLenUncompressed:
		// Reject unsupported public key formats for the given length.
		format := serialized[0]
		switch format {
		case PubKeyFormatUncompressed:
		case PubKeyFormatHybridEven, PubKeyFormatHybridOdd:
		default:
			str := fmt.Sprintf("invalid public key: unsupported format: %x",
				format)
			return nil, makeError(ErrPubKeyInvalidFormat, str)
		}

		// Parse the x and y coordinates while ensuring that they are in the
		// allowed range.
		if overflow := x.SetByteSlice(serialized[1:33]); overflow {
			str := "invalid public key: x >= field prime"
			return nil, makeError(ErrPubKeyXTooBig, str)
		}
		if overflow := y.SetByteSlice(serialized[33:]); overflow {
			str := "invalid public key: y >= field prime"
			return nil, makeError(ErrPubKeyYTooBig, str)
		}

		// Ensure the oddness of the y coordinate matches the specified format
		// for hybrid public keys.
		if format == PubKeyFormatHybridEven || format == PubKeyFormatHybridOdd {
			wantOddY := format == PubKeyFormatHybridOdd
			if y.IsOdd() != wantOddY {
				str := fmt.Sprintf("invalid public key: y oddness does not "+
					"match specified value of %v", wantOddY)
				return nil, makeError(ErrPubKeyMismatchedOddness, str)
			}
		}

		// Reject public keys that are not on the secp256k1 curve.
		if !isOnCurve(&x, &y) {
			str := fmt.Sprintf("invalid public key: [%v,%v] not on secp256k1 "+
				"curve", x, y)
			return nil, makeError(ErrPubKeyNotOnCurve, str)
		}

	case PubKeyBytesLenCompressed:
		// Reject unsupported public key formats for the given length.
		format := serialized[0]
		switch format {
		case PubKeyFormatCompressedEven, PubKeyFormatCompressedOdd:
		default:
			str := fmt.Sprintf("invalid public key: unsupported format: %x",
				format)
			return nil, makeError(ErrPubKeyInvalidFormat, str)
		}

		// Parse the x coordinate while ensuring that it is in the allowed
		// range.
		if overflow := x.SetByteSlice(serialized[1:33]); overflow {
			str := "invalid public key: x >= field prime"
			return nil, makeError(ErrPubKeyXTooBig, str)
		}

		// Attempt to calculate the y coordinate for the given x coordinate such
		// that the result pair is a point on the secp256k1 curve and the
		// solution with desired oddness is chosen.
		wantOddY := format == PubKeyFormatCompressedOdd
		if !DecompressY(&x, wantOddY, &y) {
			str := fmt.Sprintf("invalid public key: x coordinate %v is not on "+
				"the secp256k1 curve", x)
			return nil, makeError(ErrPubKeyNotOnCurve, str)
		}
		y.Normalize()

	default:
		str := fmt.Sprintf("malformed public key: invalid length: %d",
			len(serialized))
		return nil, makeError(ErrPubKeyInvalidLen, str)
	}

	return NewPublicKey(&x, &y), nil
}

// SerializeUncompressed serializes a public key in the 65-byte uncompressed
// format.
func (p PublicKey) SerializeUncompressed() []byte {
	// 0x04 || 32-byte x coordinate || 32-byte y coordinate
	var b [PubKeyBytesLenUncompressed]byte
	b[0] = PubKeyFormatUncompressed
	p.x.PutBytesUnchecked(b[1:33])
	p.y.PutBytesUnchecked(b[33:65])
	return b[:]
}

// SerializeCompressed serializes a public key in the 33-byte compressed format.
func (p PublicKey) SerializeCompressed() []byte {
	// Choose the format byte depending on the oddness of the Y coordinate.
	format := PubKeyFormatCompressedEven
	if p.y.IsOdd() {
		format = PubKeyFormatCompressedOdd
	}

	// 0x02 or 0x03 || 32-byte x coordinate
	var b [PubKeyBytesLenCompressed]byte
	b[0] = format
	p.x.PutBytesUnchecked(b[1:33])
	return b[:]
}

// IsEqual compares this public key instance to the one passed, returning true
// if both public keys are equivalent.  A public key is equivalent to another,
// if they both have the same X and Y coordinates.
func (p *PublicKey) IsEqual(otherPubKey *PublicKey) bool {
	return p.x.Equals(&otherPubKey.x) && p.y.Equals(&otherPubKey.y)
}

// AsJacobian converts the public key into a Jacobian point with Z=1 and stores
// the result in the provided result param.  This allows the public key to be
// treated a Jacobian point in the secp256k1 group in calculations.
func (p *PublicKey) AsJacobian(result *JacobianPoint) {
	result.X.Set(&p.x)
	result.Y.Set(&p.y)
	result.Z.SetInt(1)
}

// IsOnCurve returns whether or not the public key represents a point on the
// secp256k1 curve.
func (p *PublicKey) IsOnCurve() bool {
	return isOnCurve(&p.x, &p.y)
}
