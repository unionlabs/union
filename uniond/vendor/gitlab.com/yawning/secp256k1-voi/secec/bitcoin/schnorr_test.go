// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package bitcoin

import (
	"bytes"
	"crypto"
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"encoding/csv"
	"fmt"
	"io"
	"os"
	"testing"

	"github.com/stretchr/testify/require"

	"gitlab.com/yawning/secp256k1-voi"
	"gitlab.com/yawning/secp256k1-voi/internal/helpers"
	"gitlab.com/yawning/secp256k1-voi/secec"
)

const testMessage = "It draws in a lot of charlatans who are trying to create various sorts of exchanges or whatever it may be. It's something where people who are of less than stellar character see an opportunity to clip people who are trying to get rich because their neighbor's getting rich buying this stuff that neither one of them understands. It will come to a bad ending."

func TestSchnorr(t *testing.T) {
	privNist, err := ecdsa.GenerateKey(elliptic.P521(), rand.Reader)
	require.NoError(t, err, "GenerateKey - P521")
	pubNist := privNist.Public()

	t.Run("Integration", func(t *testing.T) {
		priv, err := GenerateSchnorrKey()
		require.NoError(t, err, "GenerateSchnorrKey")

		signer := crypto.Signer(priv)

		pub := priv.PublicKey()

		preHashedMsg, err := PreHashSchnorrMessage(
			"secp256k1-voi/BIP0340/test",
			[]byte(testMessage),
		)
		require.NoError(t, err, "PreHashSchnorrMessage")

		d := priv.d

		sig, err := signer.Sign(nil, preHashedMsg, nil)
		require.NoError(t, err, "Sign")

		ok := pub.Verify(preHashedMsg, sig)
		require.True(t, ok, "Verify")
		ok = verifySchnorrSelf(d, pub.Bytes(), preHashedMsg, sig)
		require.True(t, ok, "VerifySelf")

		tmp := bytes.Clone(sig)
		tmp[0] ^= 0x69
		ok = pub.Verify(preHashedMsg, tmp)
		require.False(t, ok, "Verify - Corrupted sig")
		ok = verifySchnorrSelf(d, pub.Bytes(), preHashedMsg, tmp)
		require.False(t, ok, "VerifySelf - Corrupted sig")

		ok = pub.Verify(tmp, sig[:17])
		require.False(t, ok, "Verify - Truncated sig")
		ok = verifySchnorrSelf(d, pub.Bytes(), preHashedMsg, sig[:15])
		require.False(t, ok, "VerifySelf - Truncated sig")

		tmp = bytes.Clone(preHashedMsg)
		tmp[0] ^= 0x69
		ok = pub.Verify(tmp, sig)
		require.False(t, ok, "Verify - Corrupted msg")
		ok = verifySchnorrSelf(d, pub.Bytes(), tmp, sig)
		require.False(t, ok, "VerifySelf - Corrupted msg")

		_, err = PreHashSchnorrMessage("", []byte(testMessage))
		require.ErrorIs(t, err, errInvalidDomainSep, "PreHashSchnorrMessage - no domain sep")

		require.False(t, priv.Equal(privNist), "priv.Equal(privNist)")
		require.False(t, pub.Equal(pubNist), "pub.Equal(pubNist)")

		require.Panics(t, func() {
			_, _ = priv.Sign(secec.RFC6979SHA256(), preHashedMsg, nil)
		})

		_, err = NewSchnorrPrivateKey([]byte("super sekrit key"))
		require.Error(t, err, "NewSchnorrPrivateKey(not a key)")
	})

	t.Run("TestVectors", testSchnorrKAT)

	t.Run("PublicKey/Invalid", func(t *testing.T) {
		k, err := NewSchnorrPublicKey([]byte{0x45, 0x45, 0x45, 0x45})
		require.Nil(t, k, "NewSchnorrPublicKey - truncated")
		require.ErrorIs(t, err, errInvalidPublicKey, "NewSchnorrPublicKey - truncated")

		k, err = NewSchnorrPublicKeyFromPoint(secp256k1.NewIdentityPoint())
		require.Nil(t, k, "NewSchnorrPublicKeyFromPoint - identity")
		require.ErrorIs(t, err, errAIsInfinity, "NewSchnorrPublicKeyFromPoint - identity")

		require.PanicsWithValue(t, errAIsUninitialized, func() {
			new(SchnorrPublicKey).Bytes()
		}, "uninitialized.Bytes()")
	})

	t.Run("BadRNG", func(t *testing.T) {
		priv, err := GenerateSchnorrKey()
		require.NoError(t, err, "GenerateSchnorrKey")

		badSig, err := priv.Sign(newBadReader(7), []byte("any message"), nil)
		require.Nil(t, badSig, "SignSchnorr - badReader")
		require.ErrorIs(t, err, errEntropySource, "SignSchnorr - badReader")
	})
}

func BenchmarkSchnorr(b *testing.B) {
	preHashedMsg, err := PreHashSchnorrMessage(
		"secp256k1-voi/BIP0340/bench",
		[]byte(testMessage),
	)
	require.NoError(b, err, "PreHashSchnorrMessage")

	randomPriv, err := GenerateSchnorrKey()
	require.NoError(b, err)

	randomPub := randomPriv.PublicKey()
	randomSig, err := randomPriv.Sign(rand.Reader, preHashedMsg, nil)
	require.NoError(b, err)

	b.Run("Sign", func(b *testing.B) {
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			_, _ = randomPriv.Sign(rand.Reader, preHashedMsg, nil)
		}
	})
	b.Run("Verify", func(b *testing.B) {
		b.ReportAllocs()
		b.ResetTimer()

		for i := 0; i < b.N; i++ {
			ok := randomPub.Verify(preHashedMsg, randomSig)
			require.True(b, ok)
		}
	})
}

func testSchnorrKAT(t *testing.T) {
	f, err := os.Open("testdata/bip-0340-test-vectors.csv")
	require.NoError(t, err, "Open")
	defer f.Close()

	rd := csv.NewReader(f)
	records, err := rd.ReadAll()
	require.NoError(t, err, "cvs.ReadAll")

	records = records[1:] // Skip the header

	const (
		fieldIndex              = 0
		fieldSecretKey          = 1
		fieldPublicKey          = 2
		fieldAuxRand            = 3
		fieldMessage            = 4
		fieldSignature          = 5
		fieldVerificationResult = 6
		fieldComment            = 7

		resultPass = "TRUE"
		resultFail = "FALSE"
	)

	badPublicKeyTests := map[int]bool{
		5:  true,
		14: true,
	}

	for i, vec := range records {
		n := fmt.Sprintf("TestCase/%s", vec[fieldIndex])
		t.Run(n, func(t *testing.T) {
			if comment := vec[fieldComment]; comment != "" {
				t.Logf("%s", comment)
			}

			shouldPass := vec[fieldVerificationResult] == resultPass

			pkBytes := helpers.MustBytesFromHex(vec[fieldPublicKey])
			pk, err := NewSchnorrPublicKey(pkBytes)
			if badPublicKeyTests[i] {
				// Public key deserialziation failure ends the test case
				// since the API doesn't allow invalid public keys.
				require.False(t, shouldPass)
				require.Error(t, err, "NewSchnorrPublicKey")
				return
			}
			require.NoError(t, err, "NewSchnorrPublicKey")
			pkPoint := pk.Point()

			msgBytes := helpers.MustBytesFromHex(vec[fieldMessage])
			sigBytes := helpers.MustBytesFromHex(vec[fieldSignature])

			sigOk := pk.Verify(msgBytes, sigBytes)
			require.EqualValues(t, shouldPass, sigOk, "pk.Verify")

			// If there isn't a secret key provided, we're done.
			skStr := vec[fieldSecretKey]
			if skStr == "" || !shouldPass {
				return
			}
			skBytes := helpers.MustBytesFromHex(skStr)

			sk, err := NewSchnorrPrivateKey(skBytes)
			require.NoError(t, err, "NewSchnorrPrivateKey")
			require.EqualValues(t, skBytes, sk.Bytes(), "sk.Bytes() == skBytes")
			require.EqualValues(t, skBytes, sk.Scalar().Bytes(), "sk.Scalar().Bytes() == skBytes")
			require.True(t, pk.Equal(sk.Public()), "pk.Equal(sk.Public())")

			ecdsaSk, err := secec.NewPrivateKey(skBytes)
			require.NoError(t, err, "NewPrivateKey - ECDSA")

			derivedSk := NewSchnorrPrivateKeyFromECDSA(ecdsaSk)
			require.True(t, sk.Equal(derivedSk))

			derivedPk, err := NewSchnorrPublicKeyFromPoint(ecdsaSk.PublicKey().Point())
			require.NoError(t, err, "NewSchnorrPublicKeyFromPoint")

			require.True(t, derivedPk.Equal(pk), "derivedPk.Equal(pk)")
			require.EqualValues(t, pk.Bytes(), derivedPk.Bytes(), "pk.Bytes() == deriviedPk.Bytes()")
			require.EqualValues(t, 1, pkPoint.Equal(derivedPk.point), "pk.Point() == derivedPk.Point()")

			derivedPk = NewSchnorrPublicKeyFromECDSA(ecdsaSk.PublicKey())
			require.True(t, derivedPk.Equal(pk), "derivedPk.Equal(pk) - FromECDSA")

			skPubKey := sk.PublicKey()
			require.EqualValues(t, pk.Bytes(), skPubKey.Bytes(), "pk.Bytes() == sk.pk.Bytes()")
			require.EqualValues(t, 1, pkPoint.Equal(skPubKey.point), "pk.Point() == sk.pk.Point()")

			auxRandBytes := (*[schnorrEntropySize]byte)(helpers.MustBytesFromHex(vec[fieldAuxRand]))

			derivedSig, err := signSchnorr(auxRandBytes, sk, msgBytes)
			require.NoError(t, err, "signSchnorr")
			require.EqualValues(t, sigBytes, derivedSig)
		})
	}
}

func newBadReader(n int64) io.Reader {
	return &io.LimitedReader{
		R: rand.Reader,
		N: n,
	}
}
