// Package opt implements generic optimizations that remove redundancy from addition chains.
package opt

import (
	"fmt"
	"math/big"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/alg"
)

// Algorithm applies chain optimization to the result of a wrapped algorithm.
type Algorithm struct {
	Algorithm alg.ChainAlgorithm
}

func (a Algorithm) String() string {
	return fmt.Sprintf("opt(%s)", a.Algorithm)
}

// FindChain delegates to the wrapped algorithm, then runs Optimize on the result.
func (a Algorithm) FindChain(n *big.Int) (addchain.Chain, error) {
	c, err := a.Algorithm.FindChain(n)
	if err != nil {
		return nil, err
	}

	opt, err := Optimize(c)
	if err != nil {
		return nil, err
	}

	return opt, nil
}

// Optimize aims to remove redundancy from an addition chain.
func Optimize(c addchain.Chain) (addchain.Chain, error) {
	// Build program for c with all possible options at each step.
	ops := make([][]addchain.Op, len(c))
	for k := 1; k < len(c); k++ {
		ops[k] = c.Ops(k)
	}

	// Count how many times each index is used where it is the only available Op.
	counts := make([]int, len(c))
	for k := 1; k < len(c); k++ {
		if len(ops[k]) != 1 {
			continue
		}
		for _, i := range ops[k][0].Operands() {
			counts[i]++
		}
	}

	// Now, try to remove the positions which are never the only available op.
	remove := []int{}
	for k := 1; k < len(c)-1; k++ {
		if counts[k] > 0 {
			continue
		}

		// Prune places k is used.
		for l := k + 1; l < len(c); l++ {
			ops[l] = pruneuses(ops[l], k)

			// If this list now only has one element, the operands in it are now
			// indispensable.
			if len(ops[l]) == 1 {
				for _, i := range ops[l][0].Operands() {
					counts[i]++
				}
			}
		}

		// Mark k for deletion.
		remove = append(remove, k)
	}

	// Perform removals.
	pruned := addchain.Chain{}
	for i, x := range c {
		if len(remove) > 0 && remove[0] == i {
			remove = remove[1:]
			continue
		}
		pruned = append(pruned, x)
	}

	return pruned, nil
}

// pruneuses removes any uses of i from the list of operations.
func pruneuses(ops []addchain.Op, i int) []addchain.Op {
	filtered := ops[:0]
	for _, op := range ops {
		if !op.Uses(i) {
			filtered = append(filtered, op)
		}
	}
	return filtered
}
