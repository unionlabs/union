package stateutil_test

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stateutil"
)

func BenchmarkUint64ListRootWithRegistryLimit(b *testing.B) {
	balances := make([]uint64, 100000)
	for i := 0; i < len(balances); i++ {
		balances[i] = uint64(i)
	}
	b.Run("100k balances", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			_, err := stateutil.Uint64ListRootWithRegistryLimit(balances)
			if err != nil {
				b.Fatal(err)
			}
		}
	})
}
