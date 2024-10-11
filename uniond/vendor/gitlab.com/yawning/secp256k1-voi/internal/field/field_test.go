// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package field

import (
	"bytes"
	"testing"

	"github.com/stretchr/testify/require"

	"gitlab.com/yawning/secp256k1-voi/internal/helpers"
)

func BenchmarkField(b *testing.B) {
	b.Run("Invert/addchain", func(b *testing.B) {
		fe := NewElement().DebugMustRandomizeNonZero()
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			fe.Invert(fe)
		}
	})
}

func TestElement(t *testing.T) {
	pPlusOneStr := "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc30"
	geqP := [][]byte{
		helpers.MustBytesFromHex("0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f"), // P
		helpers.MustBytesFromHex(pPlusOneStr), // P+1
		helpers.MustBytesFromHex("0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc31"), // P+2
		helpers.MustBytesFromHex("0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc2f"), // P+2^32
	}
	geqPReduced := []*Element{
		NewElementFromUint64(0),
		NewElementFromUint64(1),
		NewElementFromUint64(2),
		NewElementFromUint64(0x100000000),
	}
	t.Run("SetBytes", func(t *testing.T) {
		for i, raw := range geqP {
			fe, didReduce := NewElement().SetBytes((*[ElementSize]byte)(raw))
			require.EqualValues(t, 1, didReduce, "[%d]: didReduce SetBytes(largerThanP)", i)
			require.EqualValues(t, 1, geqPReduced[i].Equal(fe), "[%d]: SetBytes(largerThanP)", i)
		}
	})
	t.Run("SetCanonicalBytes", func(t *testing.T) {
		for i, raw := range geqP {
			fe, err := NewElement().SetCanonicalBytes((*[ElementSize]byte)(raw))
			require.Nil(t, fe, "[%d]: SetCanonicalBytes(largerThanN)", i)
			require.ErrorIs(t, err, errNonCanonicalEncoding, "[%d]: SetCanonicalBytes(largerThanN)", i)
		}
	})
	t.Run("MustSetCanonicalBytes", func(t *testing.T) {
		for i, raw := range geqP {
			require.Panics(t, func() {
				NewElement().MustSetCanonicalBytes((*[ElementSize]byte)(raw))
			},
				"[%d]: SetCanonicalBytes(largerThanN)", i,
			)
		}
	})
	t.Run("SetWideBytes", func(t *testing.T) {
		huge := bytes.Repeat([]byte{0xff}, 64)                           // 2^512-1
		hugeReduced := NewElementFromCanonicalHex("0x1000007a2000e90a0") // From sage
		fe := NewElement().SetWideBytes(huge)
		require.EqualValues(t, 1, hugeReduced.Equal(fe), "SetWideBytes(huge)")

		for i, raw := range geqP {
			fe.SetWideBytes(raw)
			require.EqualValues(t, 1, geqPReduced[i].Equal(fe), "[%d]: SetWideBytes(largerThanP)", i)
		}

		require.Panics(t, func() {
			NewElement().SetWideBytes([]byte("not all that wide"))
		})
		require.Panics(t, func() {
			tooHuge := append([]byte{0xff}, huge...)
			NewElement().SetWideBytes(tooHuge)
		})
	})

	t.Run("String", func(t *testing.T) {
		// This is only exposed because it was useful for debugging.
		fe := NewElement().DebugMustRandomizeNonZero()
		fe2 := NewElement().MustSetCanonicalBytes((*[ElementSize]byte)(helpers.MustBytesFromHex(fe.String())))
		require.EqualValues(t, fe, fe2, "fe.String should roundtrip")
	})
	t.Run("Constants/c2", func(t *testing.T) {
		shouldBeNegZ := NewElement().Square(feC2)
		negZ := NewElementFromUint64(11)
		require.EqualValues(t, negZ, shouldBeNegZ, "c2 is sqrt(negZ)")
	})
	t.Run("Invert/zero", func(t *testing.T) {
		// Check that the exceptional case `1/0` returns `0`.
		//
		// The current method guarantees this property, but other
		// things (eg: h2c) absolutely rely on it, so make sure we
		// don't inadvertently break it.
		shouldBeZero := NewElement().Invert(&feZero)
		require.EqualValues(t, 1, shouldBeZero.IsZero(), "Invert(0) is 0")
	})

	// Interal: "Why are you doing that" assertion tests.
	require.Panics(t, func() { NewElementFromCanonicalHex(pPlusOneStr) })
	require.Panics(t, func() {
		fe := NewElementFromUint64(69)
		fe.Pow2k(fe, 0)
	})
}
