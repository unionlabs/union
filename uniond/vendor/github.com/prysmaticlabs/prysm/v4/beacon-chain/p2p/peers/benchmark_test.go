package peers

import (
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
)

func Benchmark_retrieveIndicesFromBitfield(b *testing.B) {
	bv := bitfield.NewBitvector64()
	for i := uint64(0); i < bv.Len(); i++ {
		bv.SetBitAt(i, true)
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		indicesFromBitfield(bv)
	}
}
