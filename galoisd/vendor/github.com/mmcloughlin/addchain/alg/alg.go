// Package alg provides base types for addition chain and addition sequence search algorithms.
package alg

import (
	"math/big"

	"github.com/mmcloughlin/addchain"
)

// ChainAlgorithm is a method of generating an addition chain for a target integer.
type ChainAlgorithm interface {
	// FindChain generates an addition chain ending at target.
	FindChain(target *big.Int) (addchain.Chain, error)

	// String returns a name for the algorithm.
	String() string
}

// SequenceAlgorithm is a method of generating an addition sequence for a set of
// target values.
type SequenceAlgorithm interface {
	// FindSequence generates an addition chain containing every element of targets.
	FindSequence(targets []*big.Int) (addchain.Chain, error)

	// String returns a name for the algorithm.
	String() string
}

// AsChainAlgorithm adapts a sequence algorithm to a chain algorithm. The
// resulting algorithm calls the sequence algorithm with a singleton list
// containing the target.
func AsChainAlgorithm(s SequenceAlgorithm) ChainAlgorithm {
	return asChainAlgorithm{s}
}

type asChainAlgorithm struct {
	SequenceAlgorithm
}

// FindChain calls FindSequence with a singleton list containing the target.
func (a asChainAlgorithm) FindChain(target *big.Int) (addchain.Chain, error) {
	return a.FindSequence([]*big.Int{target})
}
