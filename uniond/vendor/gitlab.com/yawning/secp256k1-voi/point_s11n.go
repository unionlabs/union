// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secp256k1

import (
	"crypto/subtle"
	"errors"
	"fmt"

	"gitlab.com/yawning/secp256k1-voi/internal/field"
	"gitlab.com/yawning/secp256k1-voi/internal/helpers"
)

// See: https://www.secg.org/sec1-v2.pdf
//
// There apparently is a "hybrid" format in X9.62 which is uncompressed
// but with the prefix encoding if y is odd or even.  However:
// - That's fucking moronic.
// - Not part of SEC 1.
// - A PDF copy of X9.62 costs 100 USD, assuming I don't get it from
// a domain that ends in `ru` or similar.
// - If you absolutely need to deal with a point in that format, it's
// trivial to convert to either of the supported encodings.

const (
	// CompressedPointSize is the size of a compressed point in bytes,
	// in the SEC 1, Version 2.0, Section 2.3.3 encoding (`Y_EvenOrOdd | X`).
	CompressedPointSize = 33

	// UncompressedPointSize is the size of an uncompressed point in
	// bytes in the SEC 1, Version 2.0, Section 2.3.3 encoding
	// (`0x04 | X | Y`).
	UncompressedPointSize = 65

	// IdentityPointSize is the size of the point at infinity in bytes,
	// in the SEC 1, Version 2.0, Section 2.3.3 encoding (`0x00`).
	IdentityPointSize = 1

	// CoordSize is the size of a coordinate in bytes, in the SEC 1,
	// Version 2.0, Section 2.3.5 encoding.
	CoordSize = 32

	prefixIdentity       = 0x00
	prefixCompressedEven = 0x02
	prefixCompressedOdd  = 0x03
	prefixUncompressed   = 0x04
)

var (
	// feB is the constant `b`, part of the curve equation.
	feB = field.NewElementFromUint64(7)

	// feN is the constant `n`, part of the curve parameters.
	feN = field.NewElement().MustSetCanonicalBytes((*[field.ElementSize]byte)(nBytes))

	errPointNotOnCurve   = errors.New("secp256k1: point not on curve")
	errInvalidEncoding   = errors.New("secp256k1: invalid point encoding")
	errInvalidPrefix     = errors.New("secp256k1: invalid encoded point prefix")
	errInvalidRecoveryID = errors.New("secp256k1: invalid recovery ID")
)

// UncompressedBytes returns the SEC 1, Version 2.0, Section 2.3.3
// uncompressed or infinity encoding of `v`.
func (v *Point) UncompressedBytes() []byte {
	// Blah outline blah escape analysis blah.
	var dst [UncompressedPointSize]byte
	return v.getUncompressedBytes(&dst)
}

func (v *Point) getUncompressedBytes(dst *[UncompressedPointSize]byte) []byte {
	assertPointsValid(v)

	if v.IsIdentity() != 0 {
		return append(dst[:0], prefixIdentity)
	}

	scaled := newRcvr().rescale(v)

	buf := append(dst[:0], prefixUncompressed) //nolint:gocritic
	buf = append(buf, scaled.x.Bytes()...)
	buf = append(buf, scaled.y.Bytes()...)

	return buf
}

// CompressedBytes returns the SEC 1, Version 2.0, Section 2.3.3
// compressed or infinity encoding of `v`.
func (v *Point) CompressedBytes() []byte {
	// Blah outline blah escape analysis blah.
	var dst [CompressedPointSize]byte
	return v.getCompressedBytes(&dst)
}

func (v *Point) getCompressedBytes(dst *[CompressedPointSize]byte) []byte {
	assertPointsValid(v)

	if v.IsIdentity() != 0 {
		return append(dst[:0], prefixIdentity)
	}

	scaled := newRcvr().rescale(v)

	y := subtle.ConstantTimeSelect(
		int(scaled.y.IsOdd()),
		prefixCompressedOdd,
		prefixCompressedEven,
	)

	buf := append(dst[:0], byte(y)) //nolint:gocritic
	buf = append(buf, scaled.x.Bytes()...)

	return buf
}

// XBytes returns the SEC 1, Version 2.0, Section 2.3.5 encoding of the
// x-coordinate, or an error if the point is the point at infinity.
func (v *Point) XBytes() ([]byte, error) {
	assertPointsValid(v)

	if v.IsIdentity() != 0 {
		return nil, errPointNotOnCurve
	}

	// Blah outline blah escape analysis blah.
	var dst [CoordSize]byte
	return v.getXBytes(&dst)
}

func (v *Point) getXBytes(dst *[CoordSize]byte) ([]byte, error) {
	scaled := newRcvr().rescale(v) // XXX/perf: Don't need to rescale Y.
	return append(dst[:0], scaled.x.Bytes()...), nil
}

// SetCompressedBytes sets `p = src`, where `src` is a valid SEC 1,
// Verson 2.0, Section 2.3.3 compressed encoding of a point.  If `src`
// is not a valid compressed encodiong of a point, SetCompressedBytes
// returns nil and an error, and the receiver is unchanged.
func (v *Point) SetCompressedBytes(src []byte) (*Point, error) {
	if len(src) != CompressedPointSize {
		return nil, errInvalidEncoding
	}

	switch src[0] {
	case prefixCompressedOdd:
	case prefixCompressedEven:
	default:
		return nil, errInvalidPrefix
	}

	xBytes := (*[field.ElementSize]byte)(src[1:33])
	x, err := field.NewElementFromCanonicalBytes(xBytes)
	if err != nil {
		return nil, fmt.Errorf("secp256k1: invalid x-coordinate: %w", err)
	}

	y, hasSqrt := field.NewElement().Sqrt(maybeYY(x))
	if hasSqrt != 1 {
		return nil, errPointNotOnCurve
	}

	yNeg := field.NewElement().Negate(y)
	tagEq := subtle.ConstantTimeByteEq(byte(y.IsOdd()), src[0]&1)

	v.x.Set(x)
	v.y.ConditionalSelect(yNeg, y, helpers.Uint64IsNonzero(uint64(tagEq)))
	v.z.One()
	v.isValid = true

	return v, nil
}

// SetUncompressedBytes sets `p = src`, where `src` is a valid SEC 1,
// Verson 2.0, Section 2.3.3 uncompressed encoding of a point.  If `src`
// is not a valid uncompressed encodiong of a point, SetUncompressedBytes
// returns nil and an error, and the receiver is unchanged.
func (v *Point) SetUncompressedBytes(src []byte) (*Point, error) {
	if len(src) != UncompressedPointSize {
		return nil, errInvalidEncoding
	}

	if src[0] != prefixUncompressed {
		return nil, errInvalidPrefix
	}

	xBytes := (*[field.ElementSize]byte)(src[1:33])
	x, err := field.NewElementFromCanonicalBytes(xBytes)
	if err != nil {
		return nil, fmt.Errorf("secp256k1: invalid x-coordinate: %w", err)
	}
	yBytes := (*[field.ElementSize]byte)(src[33:65])
	y, err := field.NewElementFromCanonicalBytes(yBytes)
	if err != nil {
		return nil, fmt.Errorf("secp256k1: invalid y-coordinate: %w", err)
	}

	// Check the points against the curve equation.
	if xyOnCurve(x, y) != 1 {
		return nil, errPointNotOnCurve
	}

	v.x.Set(x)
	v.y.Set(y)
	v.z.One()
	v.isValid = true

	return v, nil
}

// SetBytes sets `p = src`, where `src` is a valid SEC 1, Version 2.0,
// Section 2.3.3 encoding of a point.  If `src` is not a valid encoding
// of `p`, SetBytes returns nil and an error, and the receiver is
// unchanged.
func (v *Point) SetBytes(src []byte) (*Point, error) {
	switch len(src) {
	case IdentityPointSize:
		if src[0] != prefixIdentity {
			return nil, errInvalidPrefix
		}
		v.Identity()
		return v, nil
	case CompressedPointSize:
		return v.SetCompressedBytes(src)
	case UncompressedPointSize:
		return v.SetUncompressedBytes(src)
	}

	return nil, errInvalidEncoding
}

// NewPointFromBytes creates a new Point from either of the SEC 1
// encodings (uncompressed or compressed).
func NewPointFromBytes(src []byte) (*Point, error) {
	p, err := newRcvr().SetBytes(src)
	if err != nil {
		return nil, err
	}

	return p, nil
}

// RecoverPoint reconstructs a point from the Scalar representation of
// the x-coordinate, and a "recovery ID" in the range `[0,3]`.
func RecoverPoint(xScalar *Scalar, recoveryID byte) (*Point, error) {
	if recoveryID >= 4 {
		return nil, errInvalidRecoveryID
	}

	// The 0th bit indicates if the y-coordinate was odd.
	yIsOdd := recoveryID & 1

	xFe, err := field.NewElementFromCanonicalBytes((*[field.ElementSize]byte)(xScalar.Bytes()))
	if err != nil {
		// This can NEVER happen as Scalar.Bytes() returns the canonical
		// representation, and `n < p`.
		panic(fmt.Errorf("secp256k1: invalid x-coordinate scalar: %w", err))
	}

	// The 1st bit indicates if the x-coordinate was larger than n.
	//
	// This is unlikely in the extreme, but wycheproof has test cases
	// that can trigger this ("k*G has a large x-coordinate").
	xGtN := uint64(recoveryID&2) >> 1
	xFeN := field.NewElement().Add(xFe, feN)
	xFe.ConditionalSelect(xFe, xFeN, xGtN)

	// Sanity check.
	sc, didReduce := NewScalarFromBytes((*[ScalarSize]byte)(xFe.Bytes()))
	if (helpers.Uint64Equal(didReduce, xGtN) & sc.Equal(xScalar)) == 0 {
		return nil, errInvalidRecoveryID // Could give a more specific error...
	}

	// Now that we have what probably is the x-coordinate, and the
	// parity of the y-coordinate, we can just treat this as any
	// other compressed point.
	ptCompressed := make([]byte, 0, CompressedPointSize)
	ptCompressed = append(ptCompressed, yIsOdd+prefixCompressedEven)
	ptCompressed = append(ptCompressed, xFe.Bytes()...)

	return newRcvr().SetCompressedBytes(ptCompressed)
}

// SplitUncompressedPoint splits the SEC 1, Verson 2.0, Section 2.3.3
// uncompressed encoding of a point into the 32-byte big-endian byte
// encoding of the x-coordinate, and a uint64 indicating if the
// y-coordinate is odd.
func SplitUncompressedPoint(ptBytes []byte) ([]byte, uint64) {
	if len(ptBytes) != UncompressedPointSize {
		panic("secp256k1: invalid uncompressed point for split")
	}
	xBytes := ptBytes[1 : 1+CoordSize]
	yIsOdd := uint64(ptBytes[len(ptBytes)-1] & 1)

	return xBytes, yIsOdd
}

func xyOnCurve(x, y *field.Element) uint64 {
	return maybeYY(x).Equal(field.NewElement().Square(y))
}

func maybeYY(x *field.Element) *field.Element {
	yy := field.NewElement().Square(x)
	yy.Multiply(yy, x)
	yy.Add(yy, feB)
	return yy
}
