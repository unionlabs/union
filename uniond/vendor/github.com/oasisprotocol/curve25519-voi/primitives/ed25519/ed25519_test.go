// Copyright (c) 2016 The Go Authors. All rights reserved.
// Copyright (c) 2019 Oasis Labs Inc. All rights reserved.
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

package ed25519

import (
	"bufio"
	"bytes"
	"compress/gzip"
	"crypto"
	stded "crypto/ed25519"
	"crypto/rand"
	"crypto/sha512"
	"os"
	"strings"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/internal/testhelpers"
	"github.com/oasisprotocol/curve25519-voi/internal/zeroreader"
)

func TestStdLib(t *testing.T) {
	// Tests mostly shamelessly stolen from the standard library.
	t.Run("SignVerify", testSignVerify)
	t.Run("SignVerify/Hashed", testSignVerifyHashed)
	t.Run("SignVerify/AddedRandomness", testSignVerifyAddedRandomness)
	t.Run("SignVerify/SelfVerify", testSignVerifySelfVerify)
	t.Run("CryptoSigner", testCryptoSigner)
	t.Run("CryptoSigner/Hashed", testCryptoSignerHashed)
	t.Run("Equal", testEqual)
	t.Run("Golden", testGolden)
	t.Run("Malleability", testMalleability)
}

func testSignVerify(t *testing.T) {
	var zero zeroreader.ZeroReader
	public, private, _ := GenerateKey(zero)

	message := []byte("test message")
	sig := Sign(private, message)
	if !Verify(public, message, sig) {
		t.Errorf("valid signature rejected")
	}

	wrongMessage := []byte("wrong message")
	if Verify(public, wrongMessage, sig) {
		t.Errorf("signature of different message accepted")
	}
}

func testSignVerifyHashed(t *testing.T) {
	key := testhelpers.MustUnhex(t, "833fe62409237b9d62ec77587520911e9a759cec1d19755b7da901b96dca3d42ec172b93ad5e563bf4932c70e1245034c35467ef2efd4d64ebf819683467e2bf")
	expectedSig := testhelpers.MustUnhex(t, "98a70222f0b8121aa9d30f813d683f809e462b469c7ff87639499bb94e6dae4131f85042463c2a355a2003d062adf5aaa10b8c61e636062aaad11c2a26083406")

	private := PrivateKey(key)
	hash := sha512.Sum512([]byte("abc"))
	sig, _ := private.Sign(nil, hash[:], crypto.SHA512)
	if !bytes.Equal(sig, expectedSig) {
		t.Error("signature doesn't match test vector")
	}

	opts := &Options{
		Hash: crypto.SHA512,
	}
	if !VerifyWithOptions(key[32:], hash[:], sig, opts) {
		t.Errorf("valid signature rejected")
	}
	wrongHash := sha512.Sum512([]byte("wrong message"))
	if VerifyWithOptions(key[32:], wrongHash[:], sig, opts) {
		t.Errorf("signature of different message accepted")
	}
}

func testSignVerifyAddedRandomness(t *testing.T) {
	var zero zeroreader.ZeroReader
	public, private, _ := GenerateKey(zero)

	msg := []byte("Personal weapons are what raised mankind out of the mud, and the rifle is the queen of personal weapons.")

	opts := &Options{
		AddedRandomness: true,
	}
	sig, err := private.Sign(rand.Reader, msg, opts)
	if err != nil {
		t.Fatalf("failed to sign with added entropy: %v", err)
	}

	if !Verify(public, msg, sig) {
		t.Errorf("valid signature rejected")
	}

	sig2 := Sign(private, msg)
	if bytes.Equal(sig, sig2) {
		t.Errorf("standard signature matches entropy added signature")
	}
}

func testSignVerifySelfVerify(t *testing.T) {
	var zero zeroreader.ZeroReader
	public, private, _ := GenerateKey(zero)

	msg := []byte("Of course, if you wish, you can spend them fighting for a lost cause, but you know that you've lost.")

	opts := &Options{
		SelfVerify: true,
	}
	sig, err := private.Sign(nil, msg, opts)
	if err != nil {
		t.Fatalf("failed to sign with self-verify: %v", err)
	}

	if !Verify(public, msg, sig) {
		t.Errorf("valid signature rejected")
	}
}

func testCryptoSigner(t *testing.T) {
	var zero zeroreader.ZeroReader
	public, private, _ := GenerateKey(zero)

	signer := crypto.Signer(private)

	publicInterface := signer.Public()
	public2, ok := publicInterface.(PublicKey)
	if !ok {
		t.Fatalf("expected PublicKey from Public() but got %T", publicInterface)
	}

	if !bytes.Equal(public, public2) {
		t.Errorf("public keys do not match: original:%x vs Public():%x", public, public2)
	}

	message := []byte("message")
	var noHash crypto.Hash
	signature, err := signer.Sign(zero, message, noHash)
	if err != nil {
		t.Fatalf("error from Sign(): %s", err)
	}

	signature2, err := signer.Sign(zero, message, &Options{})
	if err != nil {
		t.Fatalf("error from Sign(&Options{}): %s", err)
	}

	if !bytes.Equal(signature, signature2) {
		t.Errorf("signatures do not match: Sign(noHash):%x vs Sign(&Options{}):%x", signature, signature2)
	}

	if !Verify(public, message, signature) {
		t.Errorf("Verify failed on signature from Sign()")
	}
}

func testCryptoSignerHashed(t *testing.T) {
	var zero zeroreader.ZeroReader
	public, private, _ := GenerateKey(zero)

	signer := crypto.Signer(private)

	publicInterface := signer.Public()
	public2, ok := publicInterface.(PublicKey)
	if !ok {
		t.Fatalf("expected PublicKey from Public() but got %T", publicInterface)
	}

	if !bytes.Equal(public, public2) {
		t.Errorf("public keys do not match: original:%x vs Public():%x", public, public2)
	}

	message := []byte("message")
	var noHash crypto.Hash
	signature, err := signer.Sign(zero, message, noHash)
	if err != nil {
		t.Fatalf("error from Sign(): %s", err)
	}

	if !Verify(public, message, signature) {
		t.Errorf("Verify failed on signature from Sign()")
	}
}

func testEqual(t *testing.T) {
	public, private, _ := GenerateKey(rand.Reader)

	if !public.Equal(public) {
		t.Errorf("public key is not equal to itself: %q", public)
	}
	if !public.Equal(crypto.Signer(private).Public()) {
		t.Errorf("private.Public() is not Equal to public: %q", public)
	}
	if !private.Equal(private) {
		t.Errorf("private key is not equal to itself: %q", private)
	}

	otherPub, otherPriv, _ := GenerateKey(rand.Reader)
	if public.Equal(otherPub) {
		t.Errorf("different public keys are Equal")
	}
	if private.Equal(otherPriv) {
		t.Errorf("different private keys are Equal")
	}
}

func testGolden(t *testing.T) {
	// sign.input.gz is a selection of test cases from
	// https://ed25519.cr.yp.to/python/sign.input
	testDataZ, err := os.Open("testdata/sign.input.gz")
	if err != nil {
		t.Fatal(err)
	}
	defer testDataZ.Close()
	testData, err := gzip.NewReader(testDataZ)
	if err != nil {
		t.Fatal(err)
	}
	defer testData.Close()

	scanner := bufio.NewScanner(testData)
	lineNo := 0

	for scanner.Scan() {
		lineNo++

		line := scanner.Text()
		parts := strings.Split(line, ":")
		if len(parts) != 5 {
			t.Fatalf("bad number of parts on line %d", lineNo)
		}

		privBytes := testhelpers.MustUnhex(t, parts[0])
		pubKey := testhelpers.MustUnhex(t, parts[1])
		msg := testhelpers.MustUnhex(t, parts[2])
		sig := testhelpers.MustUnhex(t, parts[3])
		// The signatures in the test vectors also include the message
		// at the end, but we just want R and S.
		sig = sig[:SignatureSize]

		if l := len(pubKey); l != PublicKeySize {
			t.Fatalf("bad public key length on line %d: got %d bytes", lineNo, l)
		}

		var priv [PrivateKeySize]byte
		copy(priv[:], privBytes)
		copy(priv[32:], pubKey)

		sig2 := Sign(priv[:], msg)
		if !bytes.Equal(sig, sig2[:]) {
			t.Errorf("different signature result on line %d: %x vs %x", lineNo, sig, sig2)
		}

		if !Verify(pubKey, msg, sig2) {
			t.Errorf("signature failed to verify on line %d", lineNo)
		}

		priv2 := NewKeyFromSeed(priv[:32])
		if !bytes.Equal(priv[:], priv2) {
			t.Errorf("recreating key pair gave different private key on line %d: %x vs %x", lineNo, priv[:], priv2)
		}

		if pubKey2 := priv2.Public().(PublicKey); !bytes.Equal(pubKey, pubKey2) {
			t.Errorf("recreating key pair gave different public key on line %d: %x vs %x", lineNo, pubKey, pubKey2)
		}

		if seed := priv2.Seed(); !bytes.Equal(priv[:32], seed) {
			t.Errorf("recreating key pair gave different seed on line %d: %x vs %x", lineNo, priv[:32], seed)
		}
	}

	if err := scanner.Err(); err != nil {
		t.Fatalf("error reading test data: %s", err)
	}
}

func testMalleability(t *testing.T) {
	// https://tools.ietf.org/html/rfc8032#section-5.1.7 adds an additional test
	// that s be in [0, order). This prevents someone from adding a multiple of
	// order to s and obtaining a second valid signature for the same message.
	msg := []byte{0x54, 0x65, 0x73, 0x74}
	sig := []byte{
		0x7c, 0x38, 0xe0, 0x26, 0xf2, 0x9e, 0x14, 0xaa, 0xbd, 0x05, 0x9a,
		0x0f, 0x2d, 0xb8, 0xb0, 0xcd, 0x78, 0x30, 0x40, 0x60, 0x9a, 0x8b,
		0xe6, 0x84, 0xdb, 0x12, 0xf8, 0x2a, 0x27, 0x77, 0x4a, 0xb0, 0x67,
		0x65, 0x4b, 0xce, 0x38, 0x32, 0xc2, 0xd7, 0x6f, 0x8f, 0x6f, 0x5d,
		0xaf, 0xc0, 0x8d, 0x93, 0x39, 0xd4, 0xee, 0xf6, 0x76, 0x57, 0x33,
		0x36, 0xa5, 0xc5, 0x1e, 0xb6, 0xf9, 0x46, 0xb3, 0x1d,
	}
	publicKey := []byte{
		0x7d, 0x4d, 0x0e, 0x7f, 0x61, 0x53, 0xa6, 0x9b, 0x62, 0x42, 0xb5,
		0x22, 0xab, 0xbe, 0xe6, 0x85, 0xfd, 0xa4, 0x42, 0x0f, 0x88, 0x34,
		0xb1, 0x08, 0xc3, 0xbd, 0xae, 0x36, 0x9e, 0xf5, 0x49, 0xfa,
	}

	if Verify(publicKey, msg, sig) {
		t.Fatal("non-canonical signature accepted")
	}
}

func BenchmarkGenerateKey(b *testing.B) {
	var zero zeroreader.ZeroReader
	b.Run("voi", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			if _, _, err := GenerateKey(zero); err != nil {
				b.Fatal(err)
			}
		}
	})
	b.Run("stdlib", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			if _, _, err := stded.GenerateKey(zero); err != nil {
				b.Fatal(err)
			}
		}
	})
}

func BenchmarkNewKeyFromSeed(b *testing.B) {
	seed := make([]byte, SeedSize)
	b.Run("voi", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			_ = NewKeyFromSeed(seed)
		}
	})
	b.Run("stdlib", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			_ = stded.NewKeyFromSeed(seed)
		}
	})
}

func BenchmarkSigning(b *testing.B) {
	var zero zeroreader.ZeroReader
	_, priv, err := GenerateKey(zero)
	if err != nil {
		b.Fatal(err)
	}
	message := []byte("Hello, world!")

	b.Run("voi", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			Sign(priv, message)
		}
	})
	b.Run("stdlib", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			stded.Sign(stded.PrivateKey(priv), message)
		}
	})
}

func BenchmarkVerification(b *testing.B) {
	var zero zeroreader.ZeroReader
	pub, priv, err := GenerateKey(zero)
	if err != nil {
		b.Fatal(err)
	}
	message := []byte("Hello, world!")
	signature := Sign(priv, message)

	optsStdLib := &Options{
		Verify: VerifyOptionsStdLib,
	}

	b.Run("voi", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			if !Verify(pub, message, signature) {
				b.Fatalf("verification failed")
			}
		}
	})
	b.Run("voi_stdlib", func(b *testing.B) {
		// Benchmark with the StdLib profile to get a better comparison.
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			if !VerifyWithOptions(pub, message, signature, optsStdLib) {
				b.Fatalf("verification failed")
			}
		}
	})
	b.Run("stdlib", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			if !stded.Verify(stded.PublicKey(pub), message, signature) {
				b.Fatalf("verification failed")
			}
		}
	})
}

func BenchmarkExpanded(b *testing.B) {
	var zero zeroreader.ZeroReader
	pub, priv, err := GenerateKey(zero)
	if err != nil {
		b.Fatal(err)
	}

	b.Run("NewExpandedPublicKey", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			_, _ = NewExpandedPublicKey(pub)
		}
	})
	b.Run("Verification", func(b *testing.B) {
		message := []byte("Hello, world!")
		signature := Sign(priv, message)

		expPub, err := NewExpandedPublicKey(pub)
		if err != nil {
			b.Fatalf("NewExpandedPublicKey: %v", err)
		}

		optsStdLib := &Options{
			Verify: VerifyOptionsStdLib,
		}
		b.Run("voi", func(b *testing.B) {
			b.ReportAllocs()
			for i := 0; i < b.N; i++ {
				if !VerifyExpandedWithOptions(expPub, message, signature, optionsDefault) {
					b.Fatalf("verification failed")
				}
			}
		})
		b.Run("voi_stdlib", func(b *testing.B) {
			b.ReportAllocs()
			for i := 0; i < b.N; i++ {
				if !VerifyExpandedWithOptions(expPub, message, signature, optsStdLib) {
					b.Fatalf("verification failed")
				}
			}
		})
	})
	b.Run("VerifyBatchOnly", func(b *testing.B) {
		for _, n := range benchBatchSizes {
			doBenchVerifyBatchOnly(b, n, true)
		}
	})
}
