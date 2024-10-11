// Copyright (c) 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package edwards25519

import (
	"crypto/rand"
	"encoding/hex"
	"testing"
	"testing/quick"
)

// TestBytesMontgomery tests the SetBytesWithClamping+BytesMontgomery path
// equivalence to curve25519.X25519 for basepoint scalar multiplications.
//
// Note that you can't actually implement X25519 with this package because
// there is no SetBytesMontgomery, and it would not be possible to implement
// it properly: points on the twist would get rejected, and the Scalar returned
// by SetBytesWithClamping does not preserve its cofactor-clearing properties.
//
// Disabled to avoid the golang.org/x/crypto module dependency.
/* func TestBytesMontgomery(t *testing.T) {
       f := func(scalar [32]byte) bool {
               s := NewScalar().SetBytesWithClamping(scalar[:])
               p := (&Point{}).ScalarBaseMult(s)
               got := p.BytesMontgomery()
               want, _ := curve25519.X25519(scalar[:], curve25519.Basepoint)
               return bytes.Equal(got, want)
       }
       if err := quick.Check(f, nil); err != nil {
               t.Error(err)
       }
} */

func TestBytesMontgomerySodium(t *testing.T) {
	// Generated with libsodium.js 1.0.18
	// crypto_sign_keypair().publicKey
	publicKey := "3bf918ffc2c955dc895bf145f566fb96623c1cadbe040091175764b5fde322c0"
	p, err := (&Point{}).SetBytes(decodeHex(publicKey))
	if err != nil {
		t.Fatal(err)
	}
	// crypto_sign_ed25519_pk_to_curve25519(publicKey)
	want := "efc6c9d0738e9ea18d738ad4a2653631558931b0f1fde4dd58c436d19686dc28"
	if got := hex.EncodeToString(p.BytesMontgomery()); got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestBytesMontgomeryInfinity(t *testing.T) {
	p := NewIdentityPoint()
	want := "0000000000000000000000000000000000000000000000000000000000000000"
	if got := hex.EncodeToString(p.BytesMontgomery()); got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestMultByCofactor(t *testing.T) {
	lowOrderBytes := "26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc85"
	lowOrder, err := (&Point{}).SetBytes(decodeHex(lowOrderBytes))
	if err != nil {
		t.Fatal(err)
	}

	if p := (&Point{}).MultByCofactor(lowOrder); p.Equal(NewIdentityPoint()) != 1 {
		t.Errorf("expected low order point * cofactor to be the identity")
	}

	f := func(scalar [64]byte) bool {
		s, _ := NewScalar().SetUniformBytes(scalar[:])
		p := (&Point{}).ScalarBaseMult(s)
		p8 := (&Point{}).MultByCofactor(p)
		checkOnCurve(t, p8)

		// 8 * p == (8 * s) * B
		reprEight := [32]byte{8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}
		scEight, _ := (&Scalar{}).SetCanonicalBytes(reprEight[:])
		s.Multiply(s, scEight)
		pp := (&Point{}).ScalarBaseMult(s)
		if p8.Equal(pp) != 1 {
			return false
		}

		// 8 * p == 8 * (lowOrder + p)
		pp.Add(p, lowOrder)
		pp.MultByCofactor(pp)
		if p8.Equal(pp) != 1 {
			return false
		}

		// 8 * p == p + p + p + p + p + p + p + p
		pp.Set(NewIdentityPoint())
		for i := 0; i < 8; i++ {
			pp.Add(pp, p)
		}
		return p8.Equal(pp) == 1
	}
	if err := quick.Check(f, nil); err != nil {
		t.Error(err)
	}
}

func TestScalarInvert(t *testing.T) {
	invertWorks := func(xInv Scalar, x notZeroScalar) bool {
		xInv.Invert((*Scalar)(&x))
		var check Scalar
		check.Multiply((*Scalar)(&x), &xInv)

		return check.Equal(scOne) == 1 && isReduced(xInv.Bytes())
	}

	if err := quick.Check(invertWorks, quickCheckConfig(32)); err != nil {
		t.Error(err)
	}

	randomScalar := *dalekScalar
	randomInverse := NewScalar().Invert(&randomScalar)
	var check Scalar
	check.Multiply(&randomScalar, randomInverse)

	if check.Equal(scOne) == 0 || !isReduced(randomInverse.Bytes()) {
		t.Error("inversion did not work")
	}

	zero := NewScalar()
	if xx := NewScalar().Invert(zero); xx.Equal(zero) != 1 {
		t.Errorf("inverting zero did not return zero")
	}
}

func TestMultiScalarMultMatchesBaseMult(t *testing.T) {
	multiScalarMultMatchesBaseMult := func(x, y, z Scalar) bool {
		var p, q1, q2, q3, check Point

		p.MultiScalarMult([]*Scalar{&x, &y, &z}, []*Point{B, B, B})

		q1.ScalarBaseMult(&x)
		q2.ScalarBaseMult(&y)
		q3.ScalarBaseMult(&z)
		check.Add(&q1, &q2).Add(&check, &q3)

		checkOnCurve(t, &p, &check, &q1, &q2, &q3)
		return p.Equal(&check) == 1
	}

	if err := quick.Check(multiScalarMultMatchesBaseMult, quickCheckConfig(32)); err != nil {
		t.Error(err)
	}
}

func TestVarTimeMultiScalarMultMatchesBaseMult(t *testing.T) {
	varTimeMultiScalarMultMatchesBaseMult := func(x, y, z Scalar) bool {
		var p, q1, q2, q3, check Point

		p.VarTimeMultiScalarMult([]*Scalar{&x, &y, &z}, []*Point{B, B, B})

		q1.ScalarBaseMult(&x)
		q2.ScalarBaseMult(&y)
		q3.ScalarBaseMult(&z)
		check.Add(&q1, &q2).Add(&check, &q3)

		checkOnCurve(t, &p, &check, &q1, &q2, &q3)
		return p.Equal(&check) == 1
	}

	if err := quick.Check(varTimeMultiScalarMultMatchesBaseMult, quickCheckConfig(32)); err != nil {
		t.Error(err)
	}
}

func BenchmarkMultiScalarMultSize8(t *testing.B) {
	var p Point
	x := dalekScalar

	for i := 0; i < t.N; i++ {
		p.MultiScalarMult([]*Scalar{x, x, x, x, x, x, x, x},
			[]*Point{B, B, B, B, B, B, B, B})
	}
}

func BenchmarkScalarAddition(b *testing.B) {
	var rnd [128]byte
	rand.Read(rnd[:])
	s1, _ := (&Scalar{}).SetUniformBytes(rnd[0:64])
	s2, _ := (&Scalar{}).SetUniformBytes(rnd[64:128])
	t := &Scalar{}

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		t.Add(s1, s2)
	}
}

func BenchmarkScalarMultiplication(b *testing.B) {
	var rnd [128]byte
	rand.Read(rnd[:])
	s1, _ := (&Scalar{}).SetUniformBytes(rnd[0:64])
	s2, _ := (&Scalar{}).SetUniformBytes(rnd[64:128])
	t := &Scalar{}

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		t.Multiply(s1, s2)
	}
}

func BenchmarkScalarInversion(b *testing.B) {
	var rnd [64]byte
	rand.Read(rnd[:])
	s1, _ := (&Scalar{}).SetUniformBytes(rnd[0:64])

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		s1.Invert(s1)
	}
}
