// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package field

import (
	"math/bits"

	fiat "gitlab.com/yawning/secp256k1-voi/internal/fiat/secp256k1montgomery"
	"gitlab.com/yawning/secp256k1-voi/internal/helpers"
)

var (
	scTwo192 = NewElementFromCanonicalHex("0x1000000000000000000000000000000000000000000000000") // 2^192 mod m
	scTwo384 = NewElementFromCanonicalHex("0x1000003d100000000000000000000000000000000")         // 2^384 mod m (from sage)
)

// SetWideBytes sets `fe = src % p`, where `src` is a big-endian encoding
// of `fe` with a length in the range `[32,64]`-bytes, and returns `fe`.
// This routine only exists to implement certain standards that require
// this.  In practice, p is close enough to `2^256-1` such that this is
// largely unnecessary.
func (fe *Element) SetWideBytes(src []byte) *Element {
	// An alternative way to do this would be something like
	// 14.3.4 "Reduction methods for moduli of special form" from
	// "Handbook of Applied Cryptography" by Menezes, Oorschot, and
	// Vanstone.
	//
	// The existence of this routine is stupid, and it is only
	// for h2c.

	sLen := len(src)
	switch {
	case sLen < ElementSize:
		// Ironically this is implemented as "setShortBytes", but
		// aside from as a helper for our wide-reduction, there is
		// no reason to ever call it.
		panic("secp256k1/internal/field: wide element too short")
	case sLen == ElementSize:
		// When possible, call the simpler routine.
		fe.SetBytes((*[ElementSize]byte)(src))
		return fe
	case sLen <= WideElementSize:
		// Use Frank Denis' trick, as documented by Filippo Valsorda
		// at https://words.filippo.io/dispatches/wide-reduction/
		//
		// "I represent the value as a+b*2^192+c*2^384"

		// Zero extend to 512-bits.
		var src512 [WideElementSize]byte
		copy(src512[WideElementSize-sLen:], src)

		fe.setShortBytes(src512[40:])                  // a
		b := NewElement().setShortBytes(src512[16:40]) // b
		c := NewElement().setShortBytes(src512[:16])   // c
		fe.Add(fe, b.Multiply(b, scTwo192))
		fe.Add(fe, c.Multiply(c, scTwo384))

		return fe
	default:
		panic("secp256k1/internal/field: wide element too large")
	}
}

func (fe *Element) setShortBytes(src []byte) *Element {
	// Invariant: sLen < ElementSize, so src < n.
	sLen := len(src)
	if sLen >= ElementSize {
		panic("internal/field: short element too wide")
	}

	// Zero extend to 256-bits.
	var src256 [ElementSize]byte
	copy(src256[ElementSize-sLen:], src)

	// Unchecked set (s < n).
	sat := helpers.BytesToSaturated(&src256)
	return fe.uncheckedSetSaturated(&sat)
}

func reduceSaturated(dst, src *[4]uint64) uint64 {
	// Assume that the reduction is needed, and calclate
	// reduced = src - n.  This is fine because src will never
	// be >= 2n.
	var (
		reduced [4]uint64
		borrow  uint64
	)
	reduced[0], borrow = bits.Sub64(src[0], mSat[0], borrow)
	reduced[1], borrow = bits.Sub64(src[1], mSat[1], borrow)
	reduced[2], borrow = bits.Sub64(src[2], mSat[2], borrow)
	reduced[3], borrow = bits.Sub64(src[3], mSat[3], borrow)

	// if borrow == 0, src >= n
	// if borrow == 1, src < n (no reduction needed)
	didReduce := helpers.Uint64IsZero(borrow)

	fiat.Selectznz(dst, fiat.Uint64ToUint1(didReduce), src, &reduced)

	return didReduce
}
