package opt

import (
	"testing"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/internal/bigints"
)

func TestOptimize(t *testing.T) {
	cases := []struct {
		Input  addchain.Chain
		Expect addchain.Chain
	}{
		// Sub-optimal case. Should remove 3 or 4.
		{
			Input:  bigints.Int64s(1, 2, 3, 4, 5),
			Expect: bigints.Int64s(1, 2, 4, 5),
		},
		// Optimal case. Should do nothing.
		{
			Input:  bigints.Int64s(1, 2, 4, 5),
			Expect: bigints.Int64s(1, 2, 4, 5),
		},
		// Two removals are possible in this case.
		{
			Input:  bigints.Int64s(1, 2, 3, 4, 6, 7, 8, 14, 20, 24),
			Expect: bigints.Int64s(1, 2, 4, 6, 7, 14, 20, 24),
		},
	}
	for _, c := range cases {
		// Verify both are possible.
		if err := c.Input.Validate(); err != nil {
			t.Fatal(err)
		}
		if err := c.Expect.Validate(); err != nil {
			t.Fatal(err)
		}

		// Run optimization.
		got, err := Optimize(c.Input)
		if err != nil {
			t.Fatal(err)
		}

		// Require that the result is still valid, and at least as good as the expected.
		if err := got.Produces(c.Input.End()); err != nil {
			t.Fatal("optimization result does not end in the same value")
		}

		if len(c.Expect) < len(got) {
			t.Fatalf("Optimize(%v) = %v; expect %v", c.Input, got, c.Expect)
		}
	}
}
