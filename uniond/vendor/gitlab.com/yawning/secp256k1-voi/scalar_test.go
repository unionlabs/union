// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secp256k1

import (
	"crypto/rand"
	"encoding/hex"
	"testing"

	"github.com/stretchr/testify/require"

	fiat "gitlab.com/yawning/secp256k1-voi/internal/fiat/secp256k1montgomeryscalar"
	"gitlab.com/yawning/secp256k1-voi/internal/helpers"
)

var scOne = NewScalar().One()

func (s *Scalar) String() string {
	x := hex.EncodeToString(s.Bytes())
	return x
}

func TestScalar(t *testing.T) {
	nStr := "0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141" // N

	// N = fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141
	geqN := [][]byte{
		helpers.MustBytesFromHex(nStr), // N
		helpers.MustBytesFromHex("0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364142"), // N+1
		helpers.MustBytesFromHex("0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364143"), // N+2
		helpers.MustBytesFromHex("0xffffffffffffffffffffffffffffffffbaaedce6af48a03bbfd25e8cd0364141"), // N+2^128
	}
	geqNReduced := []*Scalar{
		NewScalarFromUint64(0),
		NewScalarFromUint64(1),
		NewScalarFromUint64(2),
		newScalarFromCanonicalHex("0x100000000000000000000000000000000"),
	}
	t.Run("SetBytes", func(t *testing.T) {
		for i, raw := range geqN {
			s, didReduce := NewScalar().SetBytes((*[ScalarSize]byte)(raw))
			require.EqualValues(t, 1, didReduce, "[%d]: didReduce SetBytes(largerThanN)", i)
			require.EqualValues(t, 1, geqNReduced[i].Equal(s), "[%d]: SetBytes(largerThanN)", i)
		}
	})
	t.Run("SetCanonicalBytes", func(t *testing.T) {
		for i, raw := range geqN {
			s, err := NewScalar().SetCanonicalBytes((*[ScalarSize]byte)(raw))
			require.ErrorIs(t, err, errNonCanonicalEncoding, "[%d]: SetCanonicalBytes(largerThanN)", i)
			require.Nil(t, s, "[%d]: SetCanonicalBytes(largerThanN)", i)
		}
	})

	t.Run("Sum", func(t *testing.T) {
		// Test the empty case.
		s := NewScalar().Sum()
		require.EqualValues(t, 1, s.IsZero())

		scThree := NewScalarFromUint64(3)
		s.Sum(scOne, scOne, scOne)
		require.EqualValues(t, 1, scThree.Equal(s))
	})

	t.Run("Product", func(t *testing.T) {
		// Test the empty case.
		s := NewScalar().Product()
		require.EqualValues(t, 1, scOne.Equal(s))

		scTwo, scThree, scSix := NewScalarFromUint64(2), NewScalarFromUint64(3), NewScalarFromUint64(6)
		s.Product(scTwo, scThree)
		require.EqualValues(t, 1, scSix.Equal(s))
	})

	t.Run("IsGreaterThanHalfN", func(t *testing.T) {
		// N/2 = 7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0
		leqHalfN := []*Scalar{
			newScalarFromCanonicalHex("0x7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0"), // N/2
			newScalarFromCanonicalHex("0x7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b209f"), // N/2-1
		}
		for i, s := range leqHalfN {
			isGt := s.IsGreaterThanHalfN()
			require.EqualValues(t, 0, isGt, "[%d]: (leq).IsGreaterThanHalfN", i)
		}

		gtHalfN := []*Scalar{
			newScalarFromCanonicalHex("0x7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a1"), // N/2+1
			newScalarFromCanonicalHex("0x7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a2"), // N/2+2
		}
		for i, s := range gtHalfN {
			isGt := s.IsGreaterThanHalfN()
			require.EqualValues(t, 1, isGt, "[%d]: (gt).IsGreaterThanHalfN", i)
		}
	})

	t.Run("Zero", func(t *testing.T) {
		s := NewScalar().DebugMustRandomizeNonZero()
		require.EqualValues(t, 0, s.IsZero(), "(rand).IsZero()")

		s.Zero()
		require.EqualValues(t, 1, s.IsZero(), "(rand.Zero()).IsZero()")
	})

	// Interal: "Why are you doing that" assertion tests.
	require.Panics(t, func() { newScalarFromCanonicalHex(nStr) })
	require.Panics(t, func() {
		s := NewScalarFromUint64(69)
		s.pow2k(s, 0)
	})
}

func BenchmarkScalar(b *testing.B) {
	b.Run("Invert/addchain", func(b *testing.B) {
		s := NewScalar().DebugMustRandomizeNonZero()
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			s.Invert(s)
		}
	})
}

func (s *Scalar) DebugMustRandomizeNonZero() *Scalar {
	var b [ScalarSize]byte
	for {
		if _, err := rand.Read(b[:]); err != nil {
			panic("scalar: entropy source failure")
		}
		if _, err := s.SetCanonicalBytes(&b); err != nil {
			continue
		}
		if s.IsZero() == 1 {
			continue
		}
		return s
	}
}

// Bits returns the bit representation of `s` in LSB->MSB order.
func (s *Scalar) Bits() [ScalarSize * 8]byte {
	var nm fiat.NonMontgomeryDomainFieldElement
	fiat.FromMontgomery(&nm, &s.m)

	// XXX: This is gross, and I'm probably overcomplicating things,
	// and MSB->LSB order is probably easier to work with, and I might
	// as well produce output that is sized for the window used.

	var dst [ScalarSize * 8]byte
	for il, l := range nm { // For each 64-bit limb, least to most significant
		lOff := il * 64
		for ib := 0; ib < 8; ib++ { // For each 8-bit bytes, least to most significant
			off := lOff + ib*8
			b := byte(l >> (ib * 8))
			for i := 0; i < 8; i++ { // For each bit, least to most significant
				dst[off+i] = (b >> (i & 7)) & 1
			}
		}
	}

	return dst
}
