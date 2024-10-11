// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secec

import (
	"bytes"
	"crypto"
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/sha256"
	"testing"

	"github.com/stretchr/testify/require"

	"gitlab.com/yawning/secp256k1-voi"
	"gitlab.com/yawning/secp256k1-voi/internal/helpers"
)

const testMessage = "Most lawyers couldn’t recognize a Ponzi scheme if they were having dinner with Charles Ponzi."

var testMessageHash = hashMsgForTests([]byte(testMessage))

func hashMsgForTests(b []byte) []byte {
	h := sha256.Sum256(b)
	return h[:]
}

func TestSecec(t *testing.T) {
	privNist, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	require.NoError(t, err, "GenerateKey - P256")
	pubNist := privNist.Public()

	// Basic integration tests.  Wycheproof will take care of most of
	// the in-depth testing.
	t.Run("ECDH", func(t *testing.T) {
		alicePriv, err := GenerateKey()
		require.NoError(t, err, "GenerateKey - Alice")
		alicePubBytes := alicePriv.PublicKey().Bytes()

		bobPriv, err := GenerateKey()
		require.NoError(t, err, "GenerateKey - Bob")
		bobPubBytes := bobPriv.PublicKey().Bytes()

		bobPub, err := NewPublicKey(bobPubBytes)
		require.NoError(t, err, "NewPublicKey - Bob")

		alicePub, err := NewPublicKey(alicePubBytes)
		require.NoError(t, err, "NewPublicKey - Alice")

		aliceX, err := alicePriv.ECDH(bobPub)
		require.NoError(t, err, "ECDH - Alice")

		bobX, err := bobPriv.ECDH(alicePub)
		require.NoError(t, err, "ECDH - Bob")

		require.EqualValues(t, aliceX, bobX, "shared secrets should match")
	})
	t.Run("ECDSA", func(t *testing.T) {
		priv, err := GenerateKey()
		require.NoError(t, err, "GenerateKey")

		signer := crypto.Signer(priv)

		pub := priv.PublicKey()

		sig, err := signer.Sign(rand.Reader, testMessageHash, nil)
		require.NoError(t, err, "Sign")

		ok := pub.Verify(testMessageHash, sig, nil)
		require.True(t, ok, "Verify")

		tmp := bytes.Clone(sig)
		tmp[0] ^= 0x69
		ok = pub.Verify(testMessageHash, tmp, nil)
		require.False(t, ok, "Verify - Corrupted sig")

		tmp = bytes.Clone(testMessageHash)
		tmp[0] ^= 0x69
		ok = pub.Verify(tmp, sig, nil)
		require.False(t, ok, "Verify - Corrupted h")

		ok = pub.Verify(testMessageHash[:5], sig, nil)
		require.False(t, ok, "Verify - Truncated h")

		// Test the various encodings.
		r, s, v, err := priv.SignRaw(RFC6979SHA256(), testMessageHash)
		require.NoError(t, err, "SignRaw")

		ok = pub.VerifyRaw(testMessageHash, r, s)
		require.True(t, ok, "VerifyRaw")

		opts := &ECDSAOptions{
			Hash:       crypto.SHA256,
			Encoding:   EncodingCompact,
			SelfVerify: true,
		}
		compactSig, err := priv.Sign(RFC6979SHA256(), testMessageHash, opts)
		require.NoError(t, err, "Sign - Compact")
		require.EqualValues(t, BuildCompactSignature(r, s), compactSig)

		ok = pub.Verify(testMessageHash, compactSig, opts)
		require.True(t, ok, "Verify - Compact")

		compR, compS, err := ParseCompactSignature(compactSig)
		require.NoError(t, err, "ParseCompactSignature")
		require.EqualValues(t, 1, r.Equal(compR))
		require.EqualValues(t, 1, s.Equal(compS))

		opts.Encoding = EncodingCompactRecoverable
		recoverableSig, err := priv.Sign(RFC6979SHA256(), testMessageHash, opts)
		require.NoError(t, err, "Sign - CompactRecoverable")
		require.EqualValues(t, compactSig, recoverableSig[:CompactSignatureSize])
		require.EqualValues(t, v, recoverableSig[CompactSignatureSize])

		// Test some pathological cases.
		var zero secp256k1.Scalar
		err = verify(nil, pub, testMessageHash, &zero, s)
		require.ErrorIs(t, err, errInvalidRorS, "verify - Zero r")
		err = verify(nil, pub, testMessageHash, r, &zero)
		require.ErrorIs(t, err, errInvalidRorS, "verify - Zero s")

		badSig, err := priv.Sign(rand.Reader, testMessageHash[:30], nil)
		require.Nil(t, badSig, "Sign - Truncated hash")
		require.ErrorIs(t, err, errInvalidDigest, "Sign - Truncated hash")

		opts.Encoding = EncodingASN1
		ok = pub.Verify(testMessageHash[:30], sig, opts)
		require.False(t, ok, "Verify - Truncated hash")

		badSig, err = priv.Sign(rand.Reader, testMessageHash, crypto.SHA512)
		require.Nil(t, badSig, "Sign - Truncated hash")
		require.ErrorIs(t, err, errInvalidDigest, "Sign - Truncated hash, opts")

		opts.Encoding = EncodingCompactRecoverable + 1
		badSig, err = priv.Sign(rand.Reader, testMessageHash, opts)
		require.Nil(t, badSig, "Sign - Bad encoding")
		require.ErrorIs(t, err, errInvalidEncoding, "Sign - Bad encoding")

		ok = pub.Verify(testMessageHash, sig, opts)
		require.False(t, ok, "Verify - Bad encoding")

		_, _, err = ParseCompactSignature(compactSig[:15])
		require.ErrorIs(t, err, errInvalidCompactSig, "ParseCompactSignature - truncated")

		badCompactSig := BuildCompactSignature(&zero, s)
		_, _, err = ParseCompactSignature(badCompactSig)
		require.ErrorIs(t, err, errInvalidScalar, "ParseCompactSignature - Zero r")
		badCompactSig = BuildCompactSignature(r, &zero)
		_, _, err = ParseCompactSignature(badCompactSig)
		require.ErrorIs(t, err, errInvalidScalar, "ParseCompactSignature - Zero s")

		require.False(t, priv.Equal(privNist), "priv.Equal(privNist)")
		require.False(t, pub.Equal(pubNist), "pub.Equal(pubNist)")

		pubUntyped := priv.Public()
		require.True(t, pub.Equal(pubUntyped), "pub.Equal(pubUntyped)")
	})
	t.Run("ECDSA/Recover", func(t *testing.T) {
		priv, err := GenerateKey()
		require.NoError(t, err, "GenerateKey")

		opts := &ECDSAOptions{
			Encoding: EncodingCompactRecoverable,
		}
		sig, err := priv.Sign(rand.Reader, testMessageHash, opts)
		require.NoError(t, err, "Sign")

		pub := priv.PublicKey()

		ok := pub.Verify(testMessageHash, sig, opts)
		require.True(t, ok, "Verify")

		tmp := bytes.Clone(sig)
		tmp[64] = 27 // Fuck your unregistered securities.
		ok = pub.Verify(testMessageHash, tmp, opts)
		require.False(t, ok, "Verify - Bad sig (Rec ID)")

		r, s, v, err := ParseCompactRecoverableSignature(sig)
		require.NoError(t, err, "ParseCompactRecoverableSignature")

		q, err := RecoverPublicKey(testMessageHash, r, s, v)
		require.NoError(t, err, "RecoverPublicKey")
		require.True(t, pub.Equal(q))

		// Test some pathological cases.
		var zero secp256k1.Scalar
		_, _, _, err = ParseCompactRecoverableSignature(sig[:CompactSignatureSize])
		require.ErrorIs(t, err, errInvalidCompactSig, "ParseCompactRecoverableSignature - truncated")
		badSig := BuildCompactRecoverableSignature(&zero, s, v)
		_, _, _, err = ParseCompactRecoverableSignature(badSig)
		require.ErrorIs(t, err, errInvalidScalar, "ParseCompactRecoverableSignature - Zero r")
		badSig = BuildCompactRecoverableSignature(r, &zero, v)
		_, _, _, err = ParseCompactRecoverableSignature(badSig)
		require.ErrorIs(t, err, errInvalidScalar, "ParseCompactRecoverableSignature - Zero s")

		_, err = RecoverPublicKey(testMessageHash, &zero, s, v)
		require.ErrorIs(t, err, errInvalidRorS, "RecoverPublicKey - Zero r")
		_, err = RecoverPublicKey(testMessageHash, r, &zero, v)
		require.ErrorIs(t, err, errInvalidRorS, "RecoverPublicKey - Zero s")
		_, err = RecoverPublicKey(testMessageHash, r, s, v+27)
		require.Error(t, err, "RecoverPublicKey - Bad recovery ID")
		_, err = RecoverPublicKey(testMessageHash[:31], r, s, v)
		require.ErrorIs(t, err, errInvalidDigest, "RecoverPublicKey - Truncated h")
	})
	t.Run("ECDSA/K", testEcdsaK)
	t.Run("PrivateKey/Invalid", func(t *testing.T) {
		for _, v := range [][]byte{
			[]byte("trucated"),
			helpers.MustBytesFromHex("0000000000000000000000000000000000000000000000000000000000000000"), // N+1
			helpers.MustBytesFromHex("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364142"), // N+1
		} {
			k, err := NewPrivateKey(v)
			require.Nil(t, k, "NewPrivateKey(%x)", v)
			require.ErrorIs(t, err, errInvalidPrivateKey, "NewPrivateKey(%x)", v)
		}
	})
	t.Run("PublicKey/Invalid", func(t *testing.T) {
		k, err := NewPublicKey([]byte{0x00})
		require.Nil(t, k, "NewPublicKey - identity")
		require.ErrorIs(t, err, errAIsInfinity, "NewPublicKey - identity")

		k, err = NewPublicKeyFromPoint(secp256k1.NewIdentityPoint())
		require.Nil(t, k, "NewPublicKeyFromPoint - identity")
		require.ErrorIs(t, err, errAIsInfinity, "NewPublicKeyFromPoint - identity")

		require.PanicsWithValue(t, errAIsUninitialized, func() {
			new(PublicKey).Bytes()
		}, "uninitialized.Bytes()")
	})
	t.Run("PublicKey/Polarity", func(t *testing.T) {
		var (
			gotOdd, gotEven bool
			i               int
		)
		for gotOdd == false && gotEven == false {
			priv, err := GenerateKey()
			require.NoError(t, err, "GenerateKey")

			pub := priv.PublicKey()

			isOdd := pub.point.IsYOdd() == 1
			gotOdd = gotOdd || isOdd
			gotEven = gotEven || (!isOdd)

			require.Equal(t, pub.Point().CompressedBytes(), pub.CompressedBytes())

			i++
		}
		t.Logf("%d iters to see both odd and even Y", i+1)
	})
	t.Run("Internal/sampleRandomScalar", func(t *testing.T) {
		// All-zero entropy source should cause the rejection sampling
		// to give up, because it keeps generating scalars that are 0.
		sc, err := sampleRandomScalar(newZeroReader())
		require.Nil(t, sc, "sampleRandomScalar - zeroReader")
		require.ErrorIs(t, err, errRejectionSampling, "sampleRandomScalar - zeroReader")

		// Broken (non-functional) entropy source should just fail.
		sc, err = sampleRandomScalar(newBadReader(13))
		require.Nil(t, sc, "sampleRandomScalar - badReader")
		require.ErrorIs(t, err, errEntropySource, "sampleRandomScalar - badReader")
	})
}

func BenchmarkSecec(b *testing.B) {
	randomPriv, err := GenerateKey()
	require.NoError(b, err)
	randomPrivateBytes := randomPriv.Scalar().Bytes()

	randomPriv2, err := GenerateKey()
	require.NoError(b, err)
	randomPub := randomPriv2.PublicKey()
	randomPublicBytes := randomPub.Bytes()

	randomSig, err := randomPriv2.Sign(rand.Reader, testMessageHash, nil)
	require.NoError(b, err)

	randomR, randomS, randomRecID, err := randomPriv2.SignRaw(rand.Reader, testMessageHash)
	require.NoError(b, err)

	b.Run("GenerateKey", func(b *testing.B) {
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			_, err := GenerateKey()
			require.NoError(b, err)
		}
	})
	b.Run("NewPrivateKey", func(b *testing.B) {
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			_, err := NewPrivateKey(randomPrivateBytes)
			require.NoError(b, err)
		}
	})
	b.Run("NewPublicKey", func(b *testing.B) {
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			_, err := NewPublicKey(randomPublicBytes)
			require.NoError(b, err)
		}
	})
	b.Run("PrivateKey", func(b *testing.B) {
		b.Run("Bytes", func(b *testing.B) {
			b.ReportAllocs()
			b.ResetTimer()

			for i := 0; i < b.N; i++ {
				_ = randomPriv.Bytes()
			}
		})
		b.Run("ECDH", func(b *testing.B) {
			b.ReportAllocs()
			b.ResetTimer()

			for i := 0; i < b.N; i++ {
				_, _ = randomPriv.ECDH(randomPub)
			}
		})
		b.Run("Sign", func(b *testing.B) {
			b.ReportAllocs()
			b.ResetTimer()

			for i := 0; i < b.N; i++ {
				_, _ = randomPriv.Sign(rand.Reader, testMessageHash, nil)
			}
		})
		b.Run("Sign/RFC6979SHA256", func(b *testing.B) {
			b.ReportAllocs()
			b.ResetTimer()

			for i := 0; i < b.N; i++ {
				_, _ = randomPriv.Sign(RFC6979SHA256(), testMessageHash, nil)
			}
		})
		b.Run("Sign/Paranoid", func(b *testing.B) {
			opts := &ECDSAOptions{
				SelfVerify: true,
			}
			b.ReportAllocs()
			b.ResetTimer()

			for i := 0; i < b.N; i++ {
				_, _ = randomPriv.Sign(rand.Reader, testMessageHash, opts)
			}
		})
	})
	b.Run("PublicKey", func(b *testing.B) {
		b.Run("Bytes", func(b *testing.B) {
			b.ReportAllocs()
			b.ResetTimer()

			for i := 0; i < b.N; i++ {
				_ = randomPub.Bytes()
			}
		})
		b.Run("Verify", func(b *testing.B) {
			b.ReportAllocs()
			b.ResetTimer()

			for i := 0; i < b.N; i++ {
				ok := randomPub.Verify(testMessageHash, randomSig, nil)
				require.True(b, ok)
			}
		})
		b.Run("Recover", func(b *testing.B) {
			b.ReportAllocs()
			b.ResetTimer()

			for i := 0; i < b.N; i++ {
				_, err := RecoverPublicKey(testMessageHash, randomR, randomS, randomRecID)
				require.NoError(b, err)
			}
		})
	})
}
