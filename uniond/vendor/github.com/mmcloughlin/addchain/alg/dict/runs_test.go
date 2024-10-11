package dict

import (
	"math/big"
	"testing"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/internal/bigint"
)

func TestRunsChain(t *testing.T) {
	cases := []addchain.Chain{
		// Basic case.
		addchain.Int64s(1, 2, 4, 6),

		// The following induces a bug in an earlier incorrect implementation. Note
		// that 9 and 12 are formed with 1+8 and 3+8, therfore both using 8 as the
		// larger of the two operands. This means that shifts of Ones(8) are required
		// on two occasions, causing a duplicate if you're not careful.
		addchain.Int64s(1, 2, 4, 8, 9, 12),
	}
	for _, lengths := range cases {
		runs := []*big.Int{}
		for _, length := range lengths {
			runs = append(runs, bigint.Ones(uint(length.Uint64())))
		}

		got, err := RunsChain(lengths)
		if err != nil {
			t.Fatal(err)
		}

		if err := got.Superset(runs); err != nil {
			t.Fatal(err)
		}
	}
}
