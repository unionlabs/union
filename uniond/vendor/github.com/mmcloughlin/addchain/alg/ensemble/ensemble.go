// Package ensemble provides a collection of addition chain algorithms intended
// for target integers of cryptographic interest.
package ensemble

import (
	"github.com/mmcloughlin/addchain/alg"
	"github.com/mmcloughlin/addchain/alg/contfrac"
	"github.com/mmcloughlin/addchain/alg/dict"
	"github.com/mmcloughlin/addchain/alg/heuristic"
	"github.com/mmcloughlin/addchain/alg/opt"
)

// Ensemble is a convenience for building an ensemble of chain algorithms intended for large integers.
func Ensemble() []alg.ChainAlgorithm {
	// Choose sequence algorithms.
	seqalgs := []alg.SequenceAlgorithm{
		heuristic.NewAlgorithm(heuristic.UseFirst(
			heuristic.Halving{},
			heuristic.DeltaLargest{},
		)),
		heuristic.NewAlgorithm(heuristic.UseFirst(
			heuristic.Halving{},
			heuristic.Approximation{},
		)),
	}

	for _, strategy := range contfrac.Strategies {
		if strategy.Singleton() {
			seqalgs = append(seqalgs, contfrac.NewAlgorithm(strategy))
		}
	}

	// Build decomposers.
	decomposers := []dict.Decomposer{}
	for k := uint(4); k <= 128; k *= 2 {
		decomposers = append(decomposers, dict.SlidingWindow{K: k})
	}

	decomposers = append(decomposers, dict.RunLength{T: 0})
	for t := uint(16); t <= 128; t *= 2 {
		decomposers = append(decomposers, dict.RunLength{T: t})
	}

	for k := uint(2); k <= 8; k++ {
		decomposers = append(decomposers, dict.Hybrid{K: k, T: 0})
		for t := uint(16); t <= 64; t *= 2 {
			decomposers = append(decomposers, dict.Hybrid{K: k, T: t})
		}
	}

	// Build dictionary algorithms for every combination.
	as := []alg.ChainAlgorithm{}
	for _, decomp := range decomposers {
		for _, seqalg := range seqalgs {
			a := dict.NewAlgorithm(decomp, seqalg)
			as = append(as, a)
		}
	}

	// Add the runs algorithms.
	for _, seqalg := range seqalgs {
		as = append(as, dict.NewRunsAlgorithm(seqalg))
	}

	// Wrap in an optimization layer.
	for i, a := range as {
		as[i] = opt.Algorithm{Algorithm: a}
	}

	return as
}
