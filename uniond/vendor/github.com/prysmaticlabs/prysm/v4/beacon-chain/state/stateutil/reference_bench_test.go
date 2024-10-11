package stateutil

import (
	"math"
	"testing"
)

func BenchmarkReference_MinusRef(b *testing.B) {
	ref := &Reference{
		refs: math.MaxUint64,
	}
	for i := 0; i < b.N; i++ {
		ref.MinusRef()
	}
}
