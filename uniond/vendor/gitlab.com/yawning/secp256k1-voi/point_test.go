// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secp256k1

import (
	"bytes"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"gitlab.com/yawning/secp256k1-voi/internal/field"
	"gitlab.com/yawning/secp256k1-voi/internal/helpers"
)

const randomTestIters = 1000

func TestPoint(t *testing.T) {
	t.Run("Uninitialized", func(t *testing.T) {
		var p Point
		require.Panics(t, func() { assertPointsValid(&p) })
	})
	t.Run("S11n", testPointS11n)
	t.Run("Add", testPointAdd)
	t.Run("Double", testPointDouble)
	t.Run("Subtract", testPointSubtract)
	t.Run("ScalarMult", testPointScalarMult)
	testPointMultiScalarMult(t)
	t.Run("ScalarBaseMult", testPointScalarBaseMult)
	t.Run("DoubleScalarMultBasepointVartime", testPointDoubleScalarMultBasepointVartime)

	t.Run("GLV/Split", testScalarSplit)
}

func testPointS11n(t *testing.T) {
	t.Run("G compressed", func(t *testing.T) {
		gCompressed := helpers.MustBytesFromHex("0279BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798")

		p, err := NewPointFromBytes(gCompressed)
		require.NoError(t, err, "NewPointFromBytes(gCompressed)")
		requirePointDeepEquals(t, NewGeneratorPoint(), p, "G decompressed")

		gBytes := p.CompressedBytes()
		require.Equal(t, gCompressed, gBytes, "G re-compressed")
	})
	t.Run("G uncompressed", func(t *testing.T) {
		gUncompressed := helpers.MustBytesFromHex("0479BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8")

		p, err := NewPointFromBytes(gUncompressed)
		require.NoError(t, err, "NewPointFromBytes(gUncompressed)")
		requirePointDeepEquals(t, NewGeneratorPoint(), p, "G")

		gBytes := p.UncompressedBytes()
		require.Equal(t, gUncompressed, gBytes, "G")
	})
	t.Run("Identity", func(t *testing.T) {
		secIDBytes := []byte{prefixIdentity}

		idBytes := NewIdentityPoint().CompressedBytes()
		require.Equal(t, secIDBytes, idBytes, "Identity")
		p, err := NewPointFromBytes(idBytes)
		require.NoError(t, err, "NewPointFromBytes(idCompressed)")
		requirePointDeepEquals(t, NewIdentityPoint(), p, "NewPointFromBytes(idCompressed)")

		idBytes = NewIdentityPoint().UncompressedBytes()
		require.Equal(t, secIDBytes, idBytes, "Identity")
		p, err = NewPointFromBytes(idBytes)
		require.NoError(t, err, "NewPointFromBytes(idUncompressed)")
		requirePointDeepEquals(t, NewIdentityPoint(), p, "NewPointFromBytes(idCompressed)")

		_, err = newRcvr().SetBytes([]byte{69})
		require.ErrorIs(t, err, errInvalidPrefix, "SetBytes(69)")
	})
	t.Run("NewPointFromCoords", func(t *testing.T) {
		p, err := NewPointFromCoords((*[CoordSize]byte)(feGX.Bytes()), (*[CoordSize]byte)(feGY.Bytes()))
		require.NoError(t, err, "NewPointFromCoords(gX, gY)")

		requirePointEquals(t, NewGeneratorPoint(), p, "NewPointFromCoords(gX, gY)")
	})
	t.Run("XBytes", func(t *testing.T) {
		g := NewGeneratorPoint()
		b, err := g.XBytes()
		require.NoError(t, err, "g.XBytes()")

		require.EqualValues(t, feGX.Bytes(), b, "g.XBytes()")

		_, err = NewIdentityPoint().XBytes()
		require.Error(t, err, "Identity.XBytes()")
	})
	t.Run("Invalid/Compressed", func(t *testing.T) {
		p := newRcvr().DebugMustRandomize()
		pBytes := p.CompressedBytes()

		p2, err := NewIdentityPoint().SetCompressedBytes(pBytes)
		require.NoError(t, err, "SetCompressedBytes(pCompressed)")
		requirePointEquals(t, p, p2, "p decompressed")

		b := pBytes[:len(pBytes)-1]

		p2, err = NewIdentityPoint().SetCompressedBytes(b)
		require.Nil(t, p2, "SetCompressedBytes(truncated)")
		require.ErrorIs(t, err, errInvalidEncoding, "SetCompressedBytes(truncated)")

		b = bytes.Clone(pBytes)
		b[0] = 69

		p2, err = NewIdentityPoint().SetCompressedBytes(b)
		require.Nil(t, p2, "SetCompressedBytes(badPrefix)")
		require.ErrorIs(t, err, errInvalidPrefix, "SetCompressedBytes(badPrefix)")
	})
	t.Run("Invalid/Uncompressed", func(t *testing.T) {
		p := newRcvr().DebugMustRandomize()
		pBytes := p.UncompressedBytes()

		p2, err := NewIdentityPoint().SetUncompressedBytes(pBytes)
		require.NoError(t, err, "SetUncompressedBytes(pUncompressed)")
		requirePointEquals(t, p, p2, "p uncompressed")

		b := pBytes[:len(pBytes)-1]

		p2, err = NewIdentityPoint().SetUncompressedBytes(b)
		require.Nil(t, p2, "SetUncompressedBytes(truncated)")
		require.ErrorIs(t, err, errInvalidEncoding, "SetUncompressedBytes(truncated)")

		b = bytes.Clone(pBytes)
		b[0] = 23

		p2, err = NewIdentityPoint().SetUncompressedBytes(b)
		require.Nil(t, p2, "SetUncompressedBytes(badPrefix)")
		require.ErrorIs(t, err, errInvalidPrefix, "SetUncompressedBytes(badPrefix)")
	})
}

func testPointAdd(t *testing.T) {
	t.Run("a + 0", func(t *testing.T) {
		a := newRcvr().DebugMustRandomize()
		id := NewIdentityPoint()

		aID := newRcvr().Add(a, id)
		idA := newRcvr().Add(id, a)

		requirePointEquals(t, a, aID, "a + 0 = a")
		requirePointEquals(t, a, idA, "0 + a = a")
	})
	t.Run("a + a", func(t *testing.T) {
		a := newRcvr().DebugMustRandomize()

		sum := newRcvr().Add(a, a)
		product := newRcvr().Double(a)

		requirePointEquals(t, product, sum, "2 * a = a + a")
	})
	t.Run("a + b", func(t *testing.T) {
		a := newRcvr().DebugMustRandomize()
		b := newRcvr().DebugMustRandomize()

		ab := newRcvr().Add(a, b)
		ba := newRcvr().Add(b, a)

		requirePointEquals(t, ab, ba, "a + b = b + a")
	})
}

func testPointDouble(t *testing.T) {
	t.Run("2 * 0", func(t *testing.T) {
		id := NewIdentityPoint()

		product := newRcvr().Double(id)

		requirePointEquals(t, id, product, "0 = 2 * 0")
	})
	t.Run("2 * a", func(t *testing.T) {
		a := newRcvr().DebugMustRandomize()

		product := newRcvr().Double(a)
		sum := newRcvr().Add(a, a)

		requirePointEquals(t, sum, product, "a + a = 2 * a")
	})
}

func testPointSubtract(t *testing.T) {
	t.Run("a - 0", func(t *testing.T) {
		a := newRcvr().DebugMustRandomize()
		negA := newRcvr().Negate(a)
		id := NewIdentityPoint()

		aID := newRcvr().Subtract(a, id)
		idA := newRcvr().Subtract(id, a)

		requirePointEquals(t, a, aID, "a - 0 = a")
		requirePointEquals(t, negA, idA, "0 - a = -a")
	})
	t.Run("a - a", func(t *testing.T) {
		a := newRcvr().DebugMustRandomize()

		diff := newRcvr().Subtract(a, a)

		requirePointEquals(t, NewIdentityPoint(), diff, "0 = a - a")
	})
	t.Run("a - b", func(t *testing.T) {
		a := newRcvr().DebugMustRandomize()
		b := newRcvr().DebugMustRandomize()
		negB := newRcvr().Negate(b)

		ab := newRcvr().Subtract(a, b)
		aNegB := newRcvr().Add(a, negB)

		requirePointEquals(t, ab, aNegB, "a - b = a + (-b)")
	})
}

func testPointScalarMult(t *testing.T) {
	t.Run("0 * G", func(t *testing.T) {
		g := NewGeneratorPoint()
		s := NewScalar()

		q := newRcvr().ScalarMult(s, g)
		qv := newRcvr().scalarMultVartimeGLV(s, g) // Special case

		require.EqualValues(t, 1, q.IsIdentity(), "0 * G == id, got %+v", q)
		require.EqualValues(t, 1, qv.IsIdentity(), "0 * G == id, got %+v", q)
	})
	t.Run("1 * G", func(t *testing.T) {
		g := NewGeneratorPoint()

		q := newRcvr().ScalarMult(scOne, g)

		requirePointEquals(t, g, q, "1 * G = G")
	})
	t.Run("2 * G", func(t *testing.T) {
		g := NewGeneratorPoint()
		s := NewScalarFromUint64(2)

		q := newRcvr().ScalarMult(s, g)
		g.Double(g)

		requirePointEquals(t, g, q, "2 * G = G + G")
	})
	t.Run("KAT/libsecp256k1", func(t *testing.T) {
		// Known answer test stolen from libsecp256k1 (`ecmult_const_random_mult`)
		aUncompressed := helpers.MustBytesFromHex("04" + "6d98654457ff52b8cf1b81265b802a5ba97f9263b1e880449335132591bc450a535c59f7325e5d2bc391fbe83c12787c337e4a98e82a90110123ba37dd769c7d")
		a, err := NewPointFromBytes(aUncompressed)
		require.NoError(t, err, "NewPointFromBytes(aUncompressed)")

		xnBytes := helpers.MustBytesFromHex("649d4f77c4242df77f2079c914530327a31b876ad2d8ce2a2236d5c6d7b2029b")
		xn, err := NewScalarFromCanonicalBytes((*[32]byte)(xnBytes))
		require.NoError(t, err, "NewScalarFromCanonicalBytes(xnBytes)")

		bUncompressed := helpers.MustBytesFromHex("04" + "237736844d209dc7098a786f20d06fcd070a38bfc11ac651030043191e2a8786ed8c3b8ec06dd57bd06ea66e45492b0fb84e4e1bfb77e21f96baae2a63dec956")
		bExpected, err := NewPointFromBytes(bUncompressed)
		require.NoError(t, err, "NewPointFromBytes(bUncompressed)")

		aXn := newRcvr().ScalarMult(xn, a)
		aXnV := newRcvr().scalarMultVartimeGLV(xn, a)

		requirePointEquals(t, bExpected, aXn, "xn * a == b")
		requirePointEquals(t, bExpected, aXnV, "xn * a (vartime) == b")
	})
	t.Run("Consistency", func(t *testing.T) {
		var s Scalar
		check := newRcvr().DebugMustRandomize()
		p1 := NewPointFrom(check)
		p2 := NewPointFrom(check)
		for i := 0; i < randomTestIters; i++ {
			s.DebugMustRandomizeNonZero()
			check := check.scalarMultTrivial(&s, check)
			p1.ScalarMult(&s, p1)
			p2.scalarMultVartimeGLV(&s, p2)

			requirePointEquals(t, check, p1, fmt.Sprintf("[%d]: s * check (trivial) == s * p1 (ct)", i))
			requirePointEquals(t, p1, p2, fmt.Sprintf("[%d]: s * p1 (ct) == s * p2 (vartime)", i))

			p1.DebugMustRandomizeZ()
			p2.DebugMustRandomizeZ()
		}
	})
}

func testPointScalarBaseMult(t *testing.T) {
	t.Run("0 * G", func(t *testing.T) {
		s := NewScalar()

		q := newRcvr().ScalarBaseMult(s)

		require.EqualValues(t, 1, q.IsIdentity(), "0 * G == id, got %+v", q)
	})
	t.Run("1 * G", func(t *testing.T) {
		g := NewGeneratorPoint()

		q := newRcvr().ScalarBaseMult(scOne)

		requirePointEquals(t, g, q, "1 * G == G")
	})
	t.Run("2 * G", func(t *testing.T) {
		g := NewGeneratorPoint()
		s := NewScalarFromUint64(2)

		q := newRcvr().ScalarBaseMult(s)
		g.Double(g)

		requirePointEquals(t, g, q, "2 * G = G + G")
	})
	t.Run("Consistency", func(t *testing.T) {
		var s Scalar
		check, p1, p2, p3, g := newRcvr(), newRcvr(), newRcvr(), newRcvr(), NewGeneratorPoint()
		for i := 0; i < randomTestIters; i++ {
			s.DebugMustRandomizeNonZero()
			check.scalarMultTrivial(&s, g)
			p1.ScalarBaseMult(&s)
			p2.scalarBaseMultVartime(&s)
			p3.ScalarMult(&s, g)

			g.DebugMustRandomizeZ()

			requirePointEquals(t, check, p1, fmt.Sprintf("[%d]: s * G (trivial) != s * G (ct)", i))
			requirePointEquals(t, p1, p2, fmt.Sprintf("[%d]: s * G (ct) != s * G (vartime)", i))
			requirePointEquals(t, p1, p3, fmt.Sprintf("[%d]: s * G (ct) != s * G (generic, ct)", i))
		}
	})
}

func testPointDoubleScalarMultBasepointVartime(t *testing.T) {
	t.Run("Consistency", func(t *testing.T) {
		var u1, u2 Scalar
		check, tmp, p, p1, g := newRcvr(), newRcvr(), newRcvr(), newRcvr(), NewGeneratorPoint()

		for i := 0; i < randomTestIters; i++ {
			u1.DebugMustRandomizeNonZero()
			u2.DebugMustRandomizeNonZero()
			p.DebugMustRandomize()

			tmp.scalarMultTrivial(&u1, g)
			check.scalarMultTrivial(&u2, p)
			check.Add(tmp, check)

			p.DebugMustRandomizeZ()
			p1.DoubleScalarMultBasepointVartime(&u1, &u2, p)

			g.DebugMustRandomizeZ()

			requirePointEquals(t, check, p1, fmt.Sprintf("[%d]: u1 * G + u2 * P (trivial) != u1 * G + u2 * P (one-shot)", i))
		}
	})
}

func (v *Point) DebugMustRandomize() *Point {
	for {
		s := NewScalar().DebugMustRandomizeNonZero()
		if s.IsZero() != 0 {
			continue
		}
		return v.ScalarBaseMult(s)
	}
}

func (v *Point) DebugMustRandomizeZ() *Point {
	assertPointsValid(v)

	if v.IsIdentity() != 0 {
		return v
	}
	for {
		rndFactor := field.NewElement().DebugMustRandomizeNonZero()

		v.x.Multiply(&v.x, rndFactor)
		v.y.Multiply(&v.y, rndFactor)
		v.z.Multiply(&v.z, rndFactor)
		return v
	}
}

func requirePointDeepEquals(t *testing.T, expected, actual *Point, descr string) {
	assertPointsValid(expected, actual)
	require.Equal(t, expected.x.Bytes(), actual.x.Bytes(), "%s X (%x %x)", descr, expected.x.Bytes(), expected.x.Bytes())
	require.Equal(t, expected.y.Bytes(), actual.y.Bytes(), "%s Y (%x %x)", descr, expected.y.Bytes(), expected.y.Bytes())
	require.Equal(t, expected.z.Bytes(), actual.z.Bytes(), "%s Z (%x %x)", descr, expected.z.Bytes(), expected.z.Bytes())
	require.EqualValues(t, 1, expected.Equal(actual), descr) // For good measure.
}

func requirePointEquals(t *testing.T, expected, actual *Point, descr string) {
	assertPointsValid(expected, actual)
	require.EqualValues(t, 1, expected.Equal(actual), descr)

	expectedScaled := newRcvr().rescale(expected)
	actualScaled := newRcvr().rescale(actual)
	requirePointDeepEquals(t, expectedScaled, actualScaled, descr)
}

func (v *Point) scalarMultTrivial(s *Scalar, p *Point) *Point {
	// This is slow but trivially correct, and is used for
	// cross-checking results of the more sophisticated
	// implementations.

	// Constant-time double and add (decreasing index).
	//
	// From https://bearssl.org/constanttime.html:
	// 1. Set Q = 0 (the “point at infinity”)
	// 2. Compute Q ← 2·Q
	// 3. If next bit of n is set, then add P to Q
	// 4. Loop to step 2 until end of multiplier is reached
	q, id, addend := NewIdentityPoint(), NewIdentityPoint(), newRcvr()
	sBits := s.Bits()
	for i := len(sBits) - 1; i >= 0; i-- {
		if i != len(sBits)-1 {
			q.Double(q)
		}
		addend.ConditionalSelect(id, p, uint64(sBits[i]))

		q.Add(q, addend)
	}

	return v.Set(q)
}

func BenchmarkPoint(b *testing.B) {
	// Yes, this is a scalar op, but it's sole purpose is
	// to make point multiplication faster.
	b.Run("GLV/SplitScalar", func(b *testing.B) {
		s := NewScalar().DebugMustRandomizeNonZero()
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			_, _ = s.splitGLV()
		}
	})
	b.Run("GLV/ScalarMult", func(b *testing.B) {
		q := NewGeneratorPoint()
		s := NewScalar().DebugMustRandomizeNonZero()
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			q.ScalarMult(s, q)
		}
	})
	b.Run("GLV/ScalarMultVartime", func(b *testing.B) {
		q := NewGeneratorPoint()
		s := NewScalar()
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			// Continually re-randomize the scalar, as execution time
			// varies depending on the value and we want the typical
			// time.
			b.StopTimer()
			s.DebugMustRandomizeNonZero()
			b.StartTimer()

			q.scalarMultVartimeGLV(s, q)
		}
	})

	b.Run("Add", func(b *testing.B) {
		p := NewGeneratorPoint()
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			p.Add(p, p)
		}
	})
	b.Run("Add/Mixed", func(b *testing.B) {
		p, g := NewGeneratorPoint(), NewGeneratorPoint()
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			p.addMixed(p, &g.x, &g.y)
		}
	})
	b.Run("Double", func(b *testing.B) {
		p := NewGeneratorPoint()
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			p.Double(p)
		}
	})
	b.Run("ScalarBaseMult", func(b *testing.B) {
		var s Scalar
		q := NewGeneratorPoint()
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			b.StopTimer()
			s.DebugMustRandomizeNonZero()
			b.StartTimer()

			q.ScalarBaseMult(&s)
		}
	})
	b.Run("ScalarBaseMult/Vartime", func(b *testing.B) {
		var s Scalar
		q := NewGeneratorPoint()
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			b.StopTimer()
			s.DebugMustRandomizeNonZero()
			b.StartTimer()

			q.scalarBaseMultVartime(&s)
		}
	})
	benchPointMultiScalarMult(b)
	b.Run("DoubleScalarMultBasepointVartime", func(b *testing.B) {
		var s1, s2 Scalar
		q := NewGeneratorPoint()
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			b.StopTimer()
			s1.DebugMustRandomizeNonZero()
			s2.DebugMustRandomizeNonZero()
			b.StartTimer()

			q.DoubleScalarMultBasepointVartime(&s1, &s2, q)
		}
	})
	b.Run("s11n/UncompressedBytes", func(b *testing.B) {
		p := NewGeneratorPoint()

		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			_ = p.UncompressedBytes()
		}
	})
	b.Run("s11n/CompressedBytes", func(b *testing.B) {
		p := NewGeneratorPoint()

		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			_ = p.CompressedBytes()
		}
	})
}
