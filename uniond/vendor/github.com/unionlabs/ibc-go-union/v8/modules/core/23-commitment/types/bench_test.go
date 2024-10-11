package types

import (
	"testing"
)

func BenchmarkMerkleProofEmpty(b *testing.B) {
	b.ReportAllocs()
	var mk MerkleProof
	for i := 0; i < b.N; i++ {
		if !mk.Empty() {
			b.Fatal("supposed to be empty")
		}
	}
}
