package exec_test

import (
	"fmt"
	"log"
	"math/big"

	"github.com/mmcloughlin/addchain/alg/ensemble"
	"github.com/mmcloughlin/addchain/alg/exec"
)

func Example() {
	// Target number: 2²⁵⁵ - 21.
	n := new(big.Int)
	n.SetString("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeb", 16)

	// Default ensemble of algorithms.
	algorithms := ensemble.Ensemble()

	// Use parallel executor.
	ex := exec.NewParallel()
	results := ex.Execute(n, algorithms)

	// Output best result.
	best := 0
	for i, r := range results {
		if r.Err != nil {
			log.Fatal(r.Err)
		}
		if len(results[i].Program) < len(results[best].Program) {
			best = i
		}
	}
	r := results[best]
	fmt.Printf("best: %d\n", len(r.Program))
	fmt.Printf("algorithm: %s\n", r.Algorithm)

	// Output:
	// best: 266
	// algorithm: opt(runs(continued_fractions(dichotomic)))
}
