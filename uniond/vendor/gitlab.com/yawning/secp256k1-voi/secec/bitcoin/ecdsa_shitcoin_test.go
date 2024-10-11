// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package bitcoin

import (
	"crypto/sha256"
	"testing"

	"github.com/stretchr/testify/require"

	"gitlab.com/yawning/secp256k1-voi/secec"
)

func TestECDSA(t *testing.T) {
	priv, err := secec.GenerateKey()
	require.NoError(t, err, "GenerateKey")

	pub := priv.PublicKey()

	msgHash := sha256.Sum256([]byte(testMessage))
	hBytes := msgHash[:]

	r, s, _, err := priv.SignRaw(nil, hBytes)
	require.NoError(t, err, "Sign")

	// I have no idea what this sighash thing is, except that the
	// verification routine demands it exists, and it has no impact
	// on if the signature is valid or not.
	sigBytes := secec.BuildASN1Signature(r, s)
	ok := VerifyASN1(pub, hBytes, sigBytes)
	require.False(t, ok, "Verify - no sighash")

	// We always create signatures with s <= n/2.
	sigBytes = append(sigBytes, 69)
	ok = VerifyASN1(pub, hBytes, sigBytes)
	require.True(t, ok, "Verify - normal sig")

	// Negate s so s > n/2.
	s.Negate(s)
	sigBytes = secec.BuildASN1Signature(r, s)

	// Verifying with SEC semantics should succeed.
	ok = pub.Verify(hBytes, sigBytes, nil)
	require.True(t, ok, "Verify - ECDSA large S")

	// Verifying with shitcoin semantics should fail.
	sigBytes = append(sigBytes, 69)
	ok = VerifyASN1(pub, hBytes, sigBytes)
	require.False(t, ok, "Verify - large S")
}
