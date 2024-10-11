package bigints

import (
	"math/big"
	"math/rand"
	"testing"
	"time"

	"github.com/mmcloughlin/addchain/internal/bigint"
)

func TestContainsSorted(t *testing.T) {
	const n = 256

	// Generate random sorted array.
	r := rand.New(rand.NewSource(time.Now().UnixNano()))
	xs := make([]*big.Int, n)
	for i := range xs {
		xs[i] = bigint.RandBits(r, 256)
	}

	Sort(xs)

	// Confirm every element is found.
	for _, x := range xs {
		if found := ContainsSorted(x, xs); !found {
			t.FailNow()
		}
	}
}
