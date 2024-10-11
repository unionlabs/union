//go:build !libsecp256k1_sdk
// +build !libsecp256k1_sdk

package secp256k1

import (
	"testing"

	secp256k1_dcrd "github.com/decred/dcrd/dcrec/secp256k1/v4"
	"github.com/stretchr/testify/require"
)

// Ensure that signature verification works, and that
// non-canonical signatures fail.
// Note: run with CGO_ENABLED=0 or go test -tags !cgo.
func TestSignatureVerificationAndRejectUpperS(t *testing.T) {
	msg := []byte("We have lingered long enough on the shores of the cosmic ocean.")
	for i := 0; i < 500; i++ {
		priv := GenPrivKey()
		sigStr, err := priv.Sign(msg)
		require.NoError(t, err)
		var r secp256k1_dcrd.ModNScalar
		r.SetByteSlice(sigStr[:32])
		var s secp256k1_dcrd.ModNScalar
		s.SetByteSlice(sigStr[32:64])
		require.False(t, s.IsOverHalfOrder())

		pub := priv.PubKey()
		require.True(t, pub.VerifySignature(msg, sigStr))

		// malleate:
		var S256 secp256k1_dcrd.ModNScalar
		S256.SetByteSlice(secp256k1_dcrd.S256().N.Bytes())
		s.Negate().Add(&S256)
		require.True(t, s.IsOverHalfOrder())

		rBytes := r.Bytes()
		sBytes := s.Bytes()
		malSigStr := make([]byte, 64)
		copy(malSigStr[32-len(rBytes):32], rBytes[:])
		copy(malSigStr[64-len(sBytes):64], sBytes[:])
		require.False(t, pub.VerifySignature(msg, malSigStr),
			"VerifyBytes incorrect with malleated & invalid S. sig=%v, key=%v",
			malSigStr,
			priv,
		)
	}
}

func TestConstantTimePubKeyGeneration(t *testing.T) {
	for i := 0; i < 500; i++ {
		pk := GenPrivKey().PubKey()
		require.NotNil(t, pk)
	}
}

// Legacy generation code
func TestNonConstantTimePubKeyGeneration(t *testing.T) {
	for i := 0; i < 500; i++ {
		privKey := GenPrivKey()
		nonConstantTimePk := secp256k1_dcrd.PrivKeyFromBytes(privKey.Key).PubKey().SerializeCompressed() // Legacy functionality from pubkey
		pk := &PubKey{Key: nonConstantTimePk}
		require.NotNil(t, pk)
	}
}
