// Copyright (c) 2016 The Go Authors. All rights reserved.
// Copyright (c) 2019-2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//   * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//    * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//    * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package x25519

import (
	"bytes"
	"crypto/rand"
	"fmt"
	"testing"

	xcurve "golang.org/x/crypto/curve25519"

	"github.com/oasisprotocol/curve25519-voi/primitives/ed25519"
)

const expectedHex = "89161fde887b2b53de549af483940106ecc114d6982daa98256de23bdf77661a"

// result of the curve25519 scalarmult ((|255| * basepoint) * basepoint)... 1024 times
var curved25519Expected = [32]byte{
	0xac, 0xce, 0x24, 0xb1, 0xd4, 0xa2, 0x36, 0x21,
	0x15, 0xe2, 0x3e, 0x84, 0x3c, 0x23, 0x2b, 0x5f,
	0x95, 0x6c, 0xc0, 0x7b, 0x95, 0x82, 0xd7, 0x93,
	0xd5, 0x19, 0xb6, 0xf1, 0xfb, 0x96, 0xd6, 0x04,
}

func TestScalarBaseMult(t *testing.T) {
	csk := [2][32]byte{
		{255},
	}

	for i := 0; i < 1024; i++ {
		ScalarBaseMult(&csk[(i&1)^1], &csk[i&1])
	}

	if !bytes.Equal(curved25519Expected[:], csk[0][:]) {
		t.Fatal("scalarmult ((|255| * basepoint) * basepoint)... 1024 did not match")
	}
}

func TestX25519(t *testing.T) {
	t.Run("voi", testX25519)
}

func testX25519(t *testing.T) {
	t.Run("Basepoint", testX25519Basepoint)
	t.Run("LowOrderPoints", testX25519LowOrderPoints)
	t.Run("NonCanonicalPoint", testX25519NonCanonicalPoint)
	t.Run("TestVectors", func(t *testing.T) {
		testTestVectors(t, func(dst, scalar, point *[32]byte) {
			out, err := X25519(scalar[:], point[:])
			if err != nil {
				t.Fatal(err)
			}
			copy(dst[:], out)
		})
	})
}

func testX25519Basepoint(t *testing.T) {
	x := make([]byte, 32)
	x[0] = 1

	for i := 0; i < 200; i++ {
		var err error
		x, err = X25519(x, Basepoint)
		if err != nil {
			t.Fatal(err)
		}
	}

	result := fmt.Sprintf("%x", x)
	if result != expectedHex {
		t.Errorf("incorrect result: got %s, want %s", result, expectedHex)
	}
}

func testX25519LowOrderPoints(t *testing.T) {
	scalar := make([]byte, ScalarSize)
	if _, err := rand.Read(scalar); err != nil {
		t.Fatal(err)
	}
	for i, p := range lowOrderPoints {
		out, err := X25519(scalar, p)
		if err == nil {
			t.Errorf("%d: expected error, got nil", i)
		}
		if out != nil {
			t.Errorf("%d: expected nil output, got %x", i, out)
		}
	}
}

func testX25519NonCanonicalPoint(t *testing.T) {
	scalar := []byte{
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x40,
	}
	myPointH := []byte{
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	}
	myPointL := []byte{
		0x12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	}
	outH, err := X25519(scalar, myPointH)
	if err != nil {
		t.Errorf("problem deserializing myPointH %s", err)
	}
	outL, err := X25519(scalar, myPointL)
	if err != nil {
		t.Errorf("problem deserializing myPointL %s", err)
	}
	if !bytes.Equal(outH, outL) {
		t.Errorf("X25519(scalar, nonCanonical) != X25519(scalar, canonical)")
	}
}

func testTestVectors(t *testing.T, scalarMult func(dst, scalar, point *[32]byte)) {
	for _, tv := range testVectors {
		var got [32]byte
		scalarMult(&got, &tv.In, &tv.Base)
		if !bytes.Equal(got[:], tv.Expect[:]) {
			t.Logf("    in = %x", tv.In)
			t.Logf("  base = %x", tv.Base)
			t.Logf("   got = %x", got)
			t.Logf("expect = %x", tv.Expect)
			t.Fail()
		}
	}
}

func TestScalarMult(t *testing.T) {
	t.Run("voi", testScalarMult)
}

func testScalarMult(t *testing.T) {
	t.Run("HighBitIgnored", testHighBitIgnored)
	t.Run("TestVectors", func(t *testing.T) {
		testTestVectors(t, ScalarMult)
	})
}

// testHighBitIgnored tests the following requirement in RFC 7748:
//
//	When receiving such an array, implementations of X25519 (but not X448) MUST
//	mask the most significant bit in the final byte.
//
// Regression test for issue #30095.
func testHighBitIgnored(t *testing.T) {
	var s, u [32]byte
	_, _ = rand.Read(s[:])
	_, _ = rand.Read(u[:])

	var hi0, hi1 [32]byte

	u[31] &= 0x7f
	ScalarMult(&hi0, &s, &u)

	u[31] |= 0x80
	ScalarMult(&hi1, &s, &u)

	if !bytes.Equal(hi0[:], hi1[:]) {
		t.Errorf("high bit of group point should not affect result")
	}
}

func TestX25519Conversion(t *testing.T) {
	public, private, _ := ed25519.GenerateKey(rand.Reader)

	xPrivate := EdPrivateKeyToX25519(private)
	xPublic, err := X25519(xPrivate, Basepoint)
	if err != nil {
		t.Errorf("X25519(xPrivate, Basepoint): %v", err)
	}

	xPublic2, ok := EdPublicKeyToX25519(public)
	if !ok {
		t.Errorf("EdPublicKeyToX25519(public): failed")
	}

	if !bytes.Equal(xPublic, xPublic2) {
		t.Errorf("Values didn't match: curve25519 produced %x, conversion produced %x", xPublic, xPublic2)
	}
}

func BenchmarkScalarBaseMult(b *testing.B) {
	b.Run("voi", func(b *testing.B) { benchScalarBaseMult(b, ScalarBaseMult) })
	b.Run("xcrypto", func(b *testing.B) { benchScalarBaseMult(b, xcurve.ScalarBaseMult) })
}

func benchScalarBaseMult(b *testing.B, scalarBaseMult func(dst, scalar *[32]byte)) {
	var in, out [32]byte
	in[0] = 1

	b.ReportAllocs()
	b.SetBytes(32)
	for i := 0; i < b.N; i++ {
		scalarBaseMult(&out, &in)
	}
}

func BenchmarkScalarMult(b *testing.B) {
	b.Run("voi", func(b *testing.B) { benchScalarMult(b, ScalarMult) })
	b.Run("xcrypto", func(b *testing.B) {
		benchScalarMult(b, xcurve.ScalarMult) //nolint:staticcheck
	})
}

func benchScalarMult(b *testing.B, scalarMult func(dst, scalar, in *[32]byte)) {
	b.ResetTimer()

	var in, scalar, out [32]byte
	in[0] = 1
	scalar[0] = 1

	b.ReportAllocs()
	b.SetBytes(32)
	for i := 0; i < b.N; i++ {
		scalarMult(&out, &scalar, &in)
	}
}

func TestX25519DiffieHellman(t *testing.T) {
	// Alice
	privateKeyAliceRaw := [32]byte{0x77, 0x07, 0x6d, 0x0a, 0x73, 0x18, 0xa5, 0x7d, 0x3c, 0x16, 0xc1, 0x72, 0x51, 0xb2, 0x66, 0x45, 0xdf, 0x4c, 0x2f, 0x87, 0xeb, 0xc0, 0x99, 0x2a, 0xb1, 0x77, 0xfb, 0xa5, 0x1d, 0xb9, 0x2c, 0x2a}
	publicKeyAliceRaw := [32]byte{0x85, 0x20, 0xf0, 0x09, 0x89, 0x30, 0xa7, 0x54, 0x74, 0x8b, 0x7d, 0xdc, 0xb4, 0x3e, 0xf7, 0x5a, 0x0d, 0xbf, 0x3a, 0x0d, 0x26, 0x38, 0x1a, 0xf4, 0xeb, 0xa4, 0xa9, 0x8e, 0xaa, 0x9b, 0x4e, 0x6a}

	privateKeyAlice := (PrivateKey)(privateKeyAliceRaw)
	publicKeyAlice := privateKeyAlice.Public()

	if !bytes.Equal(publicKeyAlice[:], publicKeyAliceRaw[:]) {
		t.Errorf("invalid public key: %x != %x", *publicKeyAlice, publicKeyAliceRaw)
	}

	// Bob
	privateKeyBobRaw := [32]byte{0x5d, 0xab, 0x08, 0x7e, 0x62, 0x4a, 0x8a, 0x4b, 0x79, 0xe1, 0x7f, 0x8b, 0x83, 0x80, 0x0e, 0xe6, 0x6f, 0x3b, 0xb1, 0x29, 0x26, 0x18, 0xb6, 0xfd, 0x1c, 0x2f, 0x8b, 0x27, 0xff, 0x88, 0xe0, 0xeb}
	publicKeyBobRaw := [32]byte{0xde, 0x9e, 0xdb, 0x7d, 0x7b, 0x7d, 0xc1, 0xb4, 0xd3, 0x5b, 0x61, 0xc2, 0xec, 0xe4, 0x35, 0x37, 0x3f, 0x83, 0x43, 0xc8, 0x5b, 0x78, 0x67, 0x4d, 0xad, 0xfc, 0x7e, 0x14, 0x6f, 0x88, 0x2b, 0x4f}

	privateKeyBob := (PrivateKey)(privateKeyBobRaw)
	publicKeyBob := privateKeyBob.Public()

	if !bytes.Equal(publicKeyBob[:], publicKeyBobRaw[:]) {
		t.Errorf("invalid public key: %x != %x", *publicKeyBob, publicKeyBobRaw)
	}

	// Diffie-Hellman
	sharedSecretRaw := [32]byte{0x4a, 0x5d, 0x9d, 0x5b, 0xa4, 0xce, 0x2d, 0xe1, 0x72, 0x8e, 0x3b, 0xf4, 0x80, 0x35, 0x0f, 0x25, 0xe0, 0x7e, 0x21, 0xc9, 0x47, 0xd1, 0x9e, 0x33, 0x76, 0xf0, 0x9b, 0x3c, 0x1e, 0x16, 0x17, 0x42}

	sharedSecretAlice := privateKeyAlice.DiffieHellman(publicKeyBob)
	sharedSecretBob := privateKeyBob.DiffieHellman(publicKeyAlice)

	if !bytes.Equal(sharedSecretAlice[:], sharedSecretBob[:]) {
		t.Errorf("mismatched shared secrets: %x != %x", *sharedSecretAlice, *sharedSecretBob)
	}
	if !bytes.Equal(sharedSecretAlice[:], sharedSecretRaw[:]) {
		t.Errorf("invalid shared secrets: %x != %x", *sharedSecretAlice, sharedSecretRaw)
	}
}

func TestX25519GenerateKey(t *testing.T) {
	pub, priv, err := GenerateKey(nil)
	if err != nil {
		t.Errorf("failed to generate key pair: %s", err)
	}

	var expected [32]byte
	ScalarBaseMult(&expected, (*[ScalarSize]byte)(priv))
	if !bytes.Equal(pub[:], expected[:]) {
		t.Errorf("generated key pair doesn't match: %s", err)
	}
}
