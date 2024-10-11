package ante_test

import (
	crand "crypto/rand"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256r1"
)

// This benchmark is used to asses the ante.Secp256k1ToR1GasFactor value
func BenchmarkSig(b *testing.B) {
	require := require.New(b)
	msg := cRandBytes(1000)

	skK := secp256k1.GenPrivKey()
	pkK := skK.PubKey()
	skR, _ := secp256r1.GenPrivKey()
	pkR := skR.PubKey()

	sigK, err := skK.Sign(msg)
	require.NoError(err)
	sigR, err := skR.Sign(msg)
	require.NoError(err)
	b.ResetTimer()

	b.Run("secp256k1", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			ok := pkK.VerifySignature(msg, sigK)
			require.True(ok)
		}
	})

	b.Run("secp256r1", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			ok := pkR.VerifySignature(msg, sigR)
			require.True(ok)
		}
	})
}

// randBytes generates a random byte slice of the specified length.
//
// It takes an integer parameter representing the number of bytes to generate.
// Returns a byte slice.
func randBytes(numBytes int) []byte {
	b := make([]byte, numBytes)
	_, err := crand.Read(b)
	if err != nil {
		panic(err)
	}
	return b
}

// This only uses the OS's randomness
func cRandBytes(numBytes int) []byte {
	return randBytes(numBytes)
}
