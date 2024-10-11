package secp256k1

import (
	"bytes"
	"math/big"
	"testing"

	secp256k1 "github.com/btcsuite/btcd/btcec/v2"
	"github.com/stretchr/testify/require"
)

func Test_genPrivKey(t *testing.T) {
	empty := make([]byte, 0, 32)
	oneB := big.NewInt(1).Bytes()
	onePadded := make([]byte, 32)
	copy(onePadded[32-len(oneB):32], oneB)
	t.Logf("one padded: %v, len=%v", onePadded, len(onePadded))

	validOne := append(empty, onePadded...)
	tests := []struct {
		name        string
		notSoRand   []byte
		shouldPanic bool
	}{
		{"empty bytes (panics because 1st 32 bytes are zero and 0 is not a valid field element)", empty, true},
		{"curve order: N", secp256k1.S256().N.Bytes(), true},
		{"valid because 0 < 1 < N", validOne, false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.shouldPanic {
				require.Panics(t, func() {
					genPrivKey(bytes.NewReader(tt.notSoRand))
				})
				return
			}
			got := genPrivKey(bytes.NewReader(tt.notSoRand))
			fe := new(big.Int).SetBytes(got[:])
			require.Less(t, fe.Cmp(secp256k1.S256().N), 0, "expected %v to be less than %v", fe, secp256k1.S256().N)
			require.Greater(t, fe.Sign(), 0)
		})
	}
}

// Ensure that signature verification works, and that
// non-canonical signatures fail.
// Note: run with CGO_ENABLED=0 or go test -tags !cgo.
func TestSignatureVerificationAndRejectUpperS(t *testing.T) {
	msg := []byte("We have lingered long enough on the shores of the cosmic ocean.")
	for i := 0; i < 500; i++ {
		priv := GenPrivKey()
		sigStr, err := priv.Sign(msg)
		require.NoError(t, err)
		var r secp256k1.ModNScalar
		r.SetByteSlice(sigStr[:32])
		var s secp256k1.ModNScalar
		s.SetByteSlice(sigStr[32:64])
		require.False(t, s.IsOverHalfOrder())

		pub := priv.PubKey()
		require.True(t, pub.VerifySignature(msg, sigStr))

		// malleate:
		var s256 secp256k1.ModNScalar
		s256.SetByteSlice(secp256k1.S256().N.Bytes())
		s.Negate().Add(&s256)
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
