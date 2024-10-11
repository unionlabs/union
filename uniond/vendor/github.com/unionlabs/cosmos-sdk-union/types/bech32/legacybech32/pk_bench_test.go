package legacybech32

import (
	"math/rand"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
)

func BenchmarkAccAddressString(b *testing.B) {
	b.ReportAllocs()
	pkBz := make([]byte, ed25519.PubKeySize)
	pk := &ed25519.PubKey{Key: pkBz}
	a := pk.Address()
	pk2 := make([]byte, ed25519.PubKeySize)
	for i := 1; i < ed25519.PubKeySize; i++ {
		pk2[i] = byte(i)
	}
	a2 := pk.Address()
	var str, str2 string
	for i := 0; i < b.N; i++ {
		str = a.String()
		str2 = a2.String()
	}
	require.NotEmpty(b, str)
	require.NotEmpty(b, str2)
}

func BenchmarkMarshalPubKey(b *testing.B) {
	b.ReportAllocs()
	pkBz := make([]byte, ed25519.PubKeySize)
	pk := &ed25519.PubKey{Key: pkBz}
	rng := rand.New(rand.NewSource(time.Now().Unix()))

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		b.StopTimer()
		rng.Read(pk.Key)
		b.StartTimer()

		_, err := MarshalPubKey(ConsPK, pk)
		require.NoError(b, err)
	}
}

func BenchmarkGetPubKeyFromBech32(b *testing.B) {
	b.ReportAllocs()
	pkBz := make([]byte, ed25519.PubKeySize)
	pk := &ed25519.PubKey{Key: pkBz}
	rng := rand.New(rand.NewSource(time.Now().Unix()))

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		b.StopTimer()
		rng.Read(pk.Key)

		pkStr, err := MarshalPubKey(ConsPK, pk)
		require.NoError(b, err)

		b.StartTimer()
		pk2, err := UnmarshalPubKey(ConsPK, pkStr)
		require.NoError(b, err)
		require.Equal(b, pk, pk2)
	}
}
