package secp256k1

import (
	"io"
	"testing"

	"github.com/cosmos/cosmos-sdk/crypto/keys/internal/benchmarking"
	"github.com/cosmos/cosmos-sdk/crypto/types"
)

func BenchmarkKeyGeneration(b *testing.B) {
	b.ReportAllocs()
	benchmarkKeygenWrapper := func(reader io.Reader) types.PrivKey {
		priv := genPrivKey()
		return &PrivKey{Key: priv}
	}
	benchmarking.BenchmarkKeyGeneration(b, benchmarkKeygenWrapper)
}

func BenchmarkSigning(b *testing.B) {
	b.ReportAllocs()
	priv := GenPrivKey()
	benchmarking.BenchmarkSigning(b, priv)
}

func BenchmarkVerification(b *testing.B) {
	b.ReportAllocs()
	priv := GenPrivKey()
	benchmarking.BenchmarkVerification(b, priv)
}
