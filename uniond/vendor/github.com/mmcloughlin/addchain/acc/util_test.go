package acc

import (
	"testing"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/alg/contfrac"
	"github.com/mmcloughlin/addchain/alg/dict"
	"github.com/mmcloughlin/addchain/internal/test"
	"github.com/mmcloughlin/addchain/rand"
)

// CheckRandom runs the check function against randomly generated chain programs.
func CheckRandom(t *testing.T, check func(t *testing.T, p addchain.Program)) {
	gs := []rand.Generator{
		rand.AddsGenerator{N: 10},
		rand.NewSolverGenerator(
			160,
			dict.NewAlgorithm(
				dict.SlidingWindow{K: 5},
				contfrac.NewAlgorithm(contfrac.DichotomicStrategy{}),
			),
		),
	}

	for _, g := range gs {
		g := g // scopelint
		t.Run(g.String(), test.Trials(func(t *testing.T) bool {
			c, err := g.GenerateChain()
			if err != nil {
				t.Fatal(err)
			}

			p, err := c.Program()
			if err != nil {
				t.Fatal(err)
			}

			check(t, p)

			return true
		}))
	}
}
