// Package rand provides random addition chain generators.
package rand

import (
	"fmt"
	"math/big"
	"math/rand"
	"time"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/alg"
	"github.com/mmcloughlin/addchain/internal/bigint"
	"github.com/mmcloughlin/addchain/internal/bigints"
)

// Generator can generate random chains.
type Generator interface {
	GenerateChain() (addchain.Chain, error)
	String() string
}

// AddsGenerator generates a random chain by making N random adds.
type AddsGenerator struct {
	N int
}

func (a AddsGenerator) String() string {
	return fmt.Sprintf("random_adds(%d)", a.N)
}

// GenerateChain generates a random chain based on N random adds.
func (a AddsGenerator) GenerateChain() (addchain.Chain, error) {
	c := addchain.New()
	for len(c) < a.N {
		i, j := rand.Intn(len(c)), rand.Intn(len(c))
		sum := new(big.Int).Add(c[i], c[j])
		c = bigints.InsertSortedUnique(c, sum)
	}
	return c, nil
}

// SolverGenerator generates random N-bit values and uses an algorithm to build
// a chain for them.
type SolverGenerator struct {
	N         uint
	Algorithm alg.ChainAlgorithm
	rand      *rand.Rand
}

// NewSolverGenerator constructs a solver generator based on n-bit targets solved with a.
func NewSolverGenerator(n uint, a alg.ChainAlgorithm) SolverGenerator {
	return SolverGenerator{
		N:         n,
		Algorithm: a,
		rand:      rand.New(rand.NewSource(time.Now().UnixNano())),
	}
}

func (s SolverGenerator) String() string {
	return fmt.Sprintf("random_solver(%d,%s)", s.N, s.Algorithm)
}

// GenerateChain generates a random n-bit value and builds a chain for it using
// the configured chain algorithm.
func (s SolverGenerator) GenerateChain() (addchain.Chain, error) {
	target := bigint.RandBits(s.rand, s.N)
	return s.Algorithm.FindChain(target)
}
