package types_test

import (
	"testing"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

var coinStrs = []string{
	"2000ATM",
	"5000AMX",
	"192XXX",
	"1e9BTC",
}

func BenchmarkParseCoin(b *testing.B) {
	var blankCoin sdk.Coin
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		for _, coinStr := range coinStrs {
			coin, err := sdk.ParseCoinNormalized(coinStr)
			if err != nil {
				b.Fatal(err)
			}
			if coin == blankCoin {
				b.Fatal("Unexpectedly returned a blank coin")
			}
		}
	}
}

func BenchmarkUintMarshal(b *testing.B) {
	values := []uint64{
		0,
		1,
		1 << 10,
		1<<10 - 3,
		1<<63 - 1,
		1<<32 - 7,
		1<<22 - 8,
	}

	var scratch [20]byte
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		for _, value := range values {
			u := sdkmath.NewUint(value)
			n, err := u.MarshalTo(scratch[:])
			if err != nil {
				b.Fatal(err)
			}
			b.SetBytes(int64(n))
		}
	}
}

func BenchmarkIntMarshal(b *testing.B) {
	values := []int64{
		0,
		1,
		1 << 10,
		1<<10 - 3,
		1<<63 - 1,
		1<<32 - 7,
		1<<22 - 8,
	}

	var scratch [20]byte
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		for _, value := range values {
			in := sdkmath.NewInt(value)
			n, err := in.MarshalTo(scratch[:])
			if err != nil {
				b.Fatal(err)
			}
			b.SetBytes(int64(n))
		}
	}
}
