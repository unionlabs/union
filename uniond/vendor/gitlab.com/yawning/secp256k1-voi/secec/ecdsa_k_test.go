// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secec

import (
	"crypto/rand"
	"crypto/sha256"
	"encoding/csv"
	"encoding/hex"
	"fmt"
	"io"
	"math/big"
	"os"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"

	"gitlab.com/yawning/secp256k1-voi"
	"gitlab.com/yawning/secp256k1-voi/internal/helpers"
)

func testEcdsaK(t *testing.T) {
	// Test that a broken entropy source will degrade to deterministic,
	// but secure signatures.
	//
	// To be honest, this can just be done by signing 2 messages with
	// a busted RNG, and checking that r is different, but this is
	// more fun, and it helps me fill in gaps in the scalar arithmetic
	// test coverage.
	//
	// Note: There are more subtle catastrophic failure cases than the
	// one tested for here, involving bias in the distribution of k,
	// but that's also really difficult to test for.

	msg1 := []byte("This is Fail(TM). But it's not Epic(TM) yet...")
	msg1Hash := hashMsgForTests(msg1)

	msg2 := []byte("With private keys you can SIGN THINGS")
	msg2Hash := hashMsgForTests(msg2)

	t.Run("BadExample", func(t *testing.T) {
		// As a demonstration, here is a "test" key, and 2 signatures
		// generated with the same k, over 2 different messages.

		testKeyScalar := mustScalarFromHex(t, "000000000000000000000000"+"E5C4D0A8249A6F27E5E0C9D534F4DA15223F42AD")
		testKey, err := NewPrivateKeyFromScalar(testKeyScalar)
		require.NoError(t, err, "newPrivateKeyFromScalar")

		badKBytes := sha256.Sum256([]byte("chosen by fair dice roll. guaranteed to be random."))

		// Signature 1 (testKey, badK, msg1)
		//
		// Note: Coincidentally generating this requires also not doing
		// the condititional negate since s > n/2.  This doesn't matter
		// since the adversary is capable of negating s as required.

		r1 := mustScalarFromHex(t, "317365e5fada9ddf645d224952c398b3bfa5dcb4d11803213ee6565639ad25be")
		s1 := mustScalarFromHex(t, "c69a9505efb9a417b5f59f62ad7cd8140947b2e2189fb7ef111a8206d2ed8aa5")
		sigOk := testKey.PublicKey().VerifyRaw(msg1Hash, r1, s1)
		require.True(t, sigOk, "sig1 ok")

		// Signature 2 (testKey, badK, msg2)

		r2 := mustScalarFromHex(t, "317365e5fada9ddf645d224952c398b3bfa5dcb4d11803213ee6565639ad25be")
		s2 := mustScalarFromHex(t, "14577cbf24e320e45c14efe63b4190e2e00f9936102f00d67cb5e79113ef5a9b")
		sigOk = testKey.PublicKey().VerifyRaw(msg2Hash, r2, s2)
		require.True(t, sigOk, "sig2 ok")

		// So the adversary has (msg1, r1, s2), and (msg2, r2, s2).
		//
		// Note: If k is reused, r will be the same between multiple signatures.

		require.EqualValues(t, r1.Bytes(), r2.Bytes(), "r1 == r2")
		require.NotEqualValues(t, s1.Bytes(), s2.Bytes(), "s1 != s2")

		// Convert the message hashes to scalars (z1, z2), per ECDSA.

		z1, _ := hashToScalar(msg1Hash)
		z2, _ := hashToScalar(msg2Hash)

		// Recover k via `k = (z - z')/(s - s')`

		zDiff := secp256k1.NewScalar().Subtract(z1, z2)
		sDiff := secp256k1.NewScalar().Subtract(s1, s2)
		sDiff.Invert(sDiff)
		recoveredK := secp256k1.NewScalar().Multiply(zDiff, sDiff)
		require.EqualValues(t, badKBytes[:], recoveredK.Bytes(), "k == recoveredK")

		// Recover d via `d = (sk - z)/r`

		skSubZ := secp256k1.NewScalar().Multiply(s1, recoveredK)
		skSubZ.Subtract(skSubZ, z1)
		rInv := secp256k1.NewScalar().Invert(r1)
		recoveredD := secp256k1.NewScalar().Multiply(skSubZ, rInv)
		require.EqualValues(t, testKey.Scalar().Bytes(), recoveredD.Bytes(), "d == recoveredD")
		// ... And now you're sad, because that's the private key.
	})
	t.Run("MitigateDebianAndSony", func(t *testing.T) {
		// Use a different "test" key.
		testKeyScalar := mustScalarFromHex(t, "000000000000000000000000"+"14B022E892CF8614A44557DB095C928DE9B89970")
		testKey, err := NewPrivateKeyFromScalar(testKeyScalar)
		require.NoError(t, err, "newPrivateKeyFromScalar")

		// Signature 1 (testKey, all 0 entropy, msg1)
		//
		// Do it twice, to verify that signatures with no entropy are
		// deterministic.

		r1, s1, _, err := testKey.SignRaw(newZeroReader(), msg1Hash)
		require.NoError(t, err, "k1.SignRaw(zeroReader, msg1)")
		sigOk := testKey.PublicKey().VerifyRaw(msg1Hash, r1, s1)
		require.True(t, sigOk, "sig1 ok")

		r1check, s1check, _, err := testKey.SignRaw(newZeroReader(), msg1Hash)
		require.NoError(t, err, "SignRaw(zeroReader, msg1) - again")

		require.EqualValues(t, r1.Bytes(), r1check.Bytes(), "r1 != r1check")
		require.EqualValues(t, s1.Bytes(), s1check.Bytes(), "s1 != s1check")

		// Signature 2 (testKey, all 0 entropy, msg2)

		r2, s2, _, err := testKey.SignRaw(newZeroReader(), msg2Hash)
		require.NoError(t, err, "k1.SignRaw(zeroReader, msg2)")
		sigOk = testKey.PublicKey().VerifyRaw(msg2Hash, r2, s2)
		require.True(t, sigOk, "sig2 ok")

		// The mitigation used is to use a CSPRNG seeded with the
		// private key, entropy, and message digest to sample k.
		// So even with no entropy and a fixed private key, r
		// should be different.
		//
		// In theory the entropy input can be omitted all together,
		// and our construct will provide the equivalent behavior
		// to what was proposed in RFC 6979, but current thought
		// is that adding entropy is better.

		require.NotEqualValues(t, r1.Bytes(), r2.Bytes(), "r1 != r2")

		// Generate another "test" key.

		testKeyScalar2Bytes := sha256.Sum256([]byte("MD_Update(&m,buf,j);  /* purify complains */"))
		testKeyScalar2, err := secp256k1.NewScalarFromCanonicalBytes(&testKeyScalar2Bytes)
		require.NoError(t, err, "NewScalarFromCanonicalBytes")
		testKey2, err := NewPrivateKeyFromScalar(testKeyScalar2)
		require.NoError(t, err, "newPrivateKeyFromScalar")

		// Signature 3 (testKey2, all 0 entropy, msg1)

		r3, s3, _, err := testKey2.SignRaw(newZeroReader(), msg1Hash)
		require.NoError(t, err, "k2.Sign(zeroReader, msg1)")
		sigOk = testKey2.PublicKey().VerifyRaw(msg1Hash, r3, s3)
		require.True(t, sigOk, "sig3 ok")

		// Likewise, even with no entropy, using a different private key
		// to sign the same message, r should be different.

		require.NotEqualValues(t, r1.Bytes(), r3.Bytes(), "r1 != r3")

		// Signature 4 (testKey, actual entropy, msg1)

		r4, s4, _, err := testKey.SignRaw(rand.Reader, msg1Hash)
		require.NoError(t, err, "k1.Sign(rand.Reader, msg1")
		sigOk = testKey.PublicKey().VerifyRaw(msg1Hash, r4, s4)
		require.True(t, sigOk, "sig4 ok")

		require.NotEqualValues(t, r1.Bytes(), r4.Bytes(), "r1 != r4")
		require.NotEqualValues(t, s1.Bytes(), s4.Bytes(), "s1 != s4")

		// With actual entropy, using the same private key
		// to sign the same message, should result in a non-deterministic
		// signature.

		// Signature 5 (testKey, no entropy source specified, msg1)

		r5, s5, _, err := testKey.SignRaw(nil, msg1Hash)
		require.NoError(t, err, "k1.Sign(nil, msg1")
		sigOk = testKey.PublicKey().VerifyRaw(msg1Hash, r5, s5)
		require.True(t, sigOk, "sig5 ok")

		require.NotEqualValues(t, r1.Bytes(), r5.Bytes(), "r1 != r5")
		require.NotEqualValues(t, s1.Bytes(), s5.Bytes(), "s1 != s5")
		require.NotEqualValues(t, r4.Bytes(), r5.Bytes(), "r4 != r5")
		require.NotEqualValues(t, s4.Bytes(), s5.Bytes(), "s4 != s5")

		// Not specifying the entropy source results in non-deterministic
		// signatures using the system entropy source.
	})

	testKeyScalarBytes := sha256.Sum256([]byte("It's a proprietary strategy. I can't go into it in great detail."))
	testKeyScalar, err := secp256k1.NewScalarFromCanonicalBytes(&testKeyScalarBytes)
	require.NoError(t, err, "NewScalarFromCanonicalBytes")
	testKey, err := NewPrivateKeyFromScalar(testKeyScalar)
	require.NoError(t, err, "newPrivateKeyFromScalar")

	t.Run("MitigateDebianAndSony/BadRng", func(t *testing.T) {
		e, err := hashToScalar(msg1Hash)
		require.NoError(t, err, "hashToScalar")

		rng, err := mitigateDebianAndSony(newBadReader(13), domainSepECDSA, testKey, e)
		require.Nil(t, rng, "mitigateDebianAndSony - badReader")
		require.ErrorIs(t, err, errEntropySource, "mitigateDebianAndSony - badReader")

		badSig, err := testKey.Sign(newBadReader(27), msg1Hash, nil)
		require.Nil(t, badSig, "Sign - badReader")
		require.ErrorIs(t, err, errEntropySource, "Sign - badReader")
	})

	t.Run("RFC6979/SHA256/TestVectors", testRFC6979KAT)
	t.Run("RFC6979/SHA256/DRBG", func(t *testing.T) {
		// Since it is vanishingly unlikely that more than 1 read
		// will ever be done from the DRBG under normal circumstances,
		// compare several reads against values generated by a
		// known-good implementation so that the state update gets
		// validated.
		//
		// Note: For those that are curious, the "known-good" one is
		// mine, that I wrote for oasis-core.
		x := testKeyScalar
		e, _ := hashToScalar(msg1Hash)

		var b [secp256k1.ScalarSize]byte
		rd := newDrbgRFC6979(x, e)
		for _, expected := range [][]byte{
			helpers.MustBytesFromHex("98b1853bf3b2798395bffd1ac98f8abaf3e0e3666268f70541890f5c884111cd"),
			helpers.MustBytesFromHex("6f52ef0ec8d7e821316fca6780a791df875b03c73405bf4f63321c07c98ace6e"),
			helpers.MustBytesFromHex("bf6133b75a1a9220e989ad9b765f859a8502257ac5b8d3914329374034f03ce0"),
		} {
			n, err := rd.Read(b[:])
			require.EqualValues(t, secp256k1.ScalarSize, n)
			require.NoError(t, err)

			require.EqualValues(t, expected, b[:])
		}

		require.Panics(t, func() {
			_, _ = rd.Read(b[:5])
		})
	})
}

func testRFC6979KAT(t *testing.T) {
	f, err := os.Open("testdata/secp256k1_rfc6979_sha256.csv")
	require.NoError(t, err, "Open")
	defer f.Close()

	rd := csv.NewReader(f)
	rd.Comment = '#'
	records, err := rd.ReadAll()
	require.NoError(t, err, "cvs.ReadAll")

	const (
		fieldPrivKey   = 0
		fieldMessage   = 1
		fieldSignature = 2
	)

	for i, vec := range records {
		n := fmt.Sprintf("TestCase/%d", i)
		t.Run(n, func(t *testing.T) {
			privInt, ok := new(big.Int).SetString(vec[fieldPrivKey], 10)
			require.True(t, ok, "big.Int().SetString")

			var privBytes [PrivateKeySize]byte
			privInt.FillBytes(privBytes[:])

			privKey, err := NewPrivateKey(privBytes[:])
			require.NoError(t, err, "NewPrivateKey")

			sig, err := privKey.Sign(RFC6979SHA256(), hashMsgForTests([]byte(vec[fieldMessage])), nil)
			require.NoError(t, err, "Sign")

			require.Equal(t, vec[fieldSignature], strings.ToUpper(hex.EncodeToString(sig)), "Sign - RFC6979")
		})
	}
}

func mustScalarFromHex(t *testing.T, x string) *secp256k1.Scalar {
	b := helpers.MustBytesFromHex(x)
	s, err := secp256k1.NewScalarFromCanonicalBytes((*[secp256k1.ScalarSize]byte)(b))
	require.NoError(t, err, "NewScalarFromCanonicalBytes")
	return s
}

type zeroReader struct{}

func (zr zeroReader) Read(b []byte) (int, error) {
	for i := range b {
		b[i] = 0
	}
	return len(b), nil
}

func newZeroReader() io.Reader {
	return zeroReader{}
}

func newBadReader(n int64) io.Reader {
	return &io.LimitedReader{
		R: rand.Reader,
		N: n,
	}
}
