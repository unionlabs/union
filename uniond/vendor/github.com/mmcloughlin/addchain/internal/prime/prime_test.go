package prime

import (
	"testing"

	"github.com/mmcloughlin/addchain/internal/polynomial"
)

func TestSolinas(t *testing.T) {
	// The "Goldilocks" prime.
	p := NewSolinas(polynomial.Polynomial{{A: -1, N: 0}, {A: -1, N: 1}, {A: 1, N: 2}}, 224)

	if p.Bits() != 448 {
		t.FailNow()
	}

	x := p.Int()
	decimal := "726838724295606890549323807888004534353641360687318060281490199180612328166730772686396383698676545930088884461843637361053498018365439"
	if x.String() != decimal {
		t.FailNow()
	}

	if p.String() != "2^448-2^224-1" {
		t.FailNow()
	}
}
