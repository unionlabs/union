package address

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/address"
)

func BenchmarkCodecWithCache(b *testing.B) {
	cdc, err := NewCachedBech32Codec("cosmos", cacheOptions)
	require.NoError(b, err)
	bytesToString(b, cdc)
}

func BenchmarkCodecWithoutCache(b *testing.B) {
	cdc := Bech32Codec{Bech32Prefix: "cosmos"}
	bytesToString(b, cdc)
}

func bytesToString(b *testing.B, cdc address.Codec) {
	b.Helper()
	addresses, err := generateAddresses(10)
	require.NoError(b, err)

	b.Helper()
	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_, err := cdc.BytesToString(addresses[i%len(addresses)])
		require.NoError(b, err)
	}
}
