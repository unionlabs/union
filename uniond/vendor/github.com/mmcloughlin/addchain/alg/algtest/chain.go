package algtest

import (
	"math"
	"math/big"
	"math/rand"
	"testing"

	"github.com/mmcloughlin/addchain/alg"
	"github.com/mmcloughlin/addchain/internal/bigint"
	"github.com/mmcloughlin/addchain/internal/prime"
	"github.com/mmcloughlin/addchain/internal/test"
)

// ChainAlgorithm applies a generic test suite to the algorithm a.
func ChainAlgorithm(t *testing.T, a alg.ChainAlgorithm) {
	suite := ChainAlgorithmSuite(a)
	suite(t)
}

// ChainAlgorithmSuite builds a generic test suite function for the algorithm a.
func ChainAlgorithmSuite(a alg.ChainAlgorithm) func(t *testing.T) {
	return func(t *testing.T) {
		t.Run("powers_of_two", checkPowersOfTwo(a, 100))
		t.Run("binary_runs", checkBinaryRuns(a, 32))
		t.Run("random_int64", checkRandomInt64s(a))
		t.Run("primes", checkPrimes(a))
	}
}

func checkPowersOfTwo(a alg.ChainAlgorithm, e uint) func(t *testing.T) {
	return func(t *testing.T) {
		n := big.NewInt(1)
		for i := uint(0); i <= e; i++ {
			AssertChainAlgorithmProduces(t, a, n)
			n.Lsh(n, 1)
		}
	}
}

func checkBinaryRuns(a alg.ChainAlgorithm, n uint) func(t *testing.T) {
	return func(t *testing.T) {
		for i := uint(1); i <= n; i++ {
			r := bigint.Pow2(i)
			r.Sub(r, bigint.One())
			AssertChainAlgorithmProduces(t, a, r)
		}
	}
}

func checkRandomInt64s(a alg.ChainAlgorithm) func(t *testing.T) {
	return test.Trials(func(t *testing.T) bool {
		r := rand.Int63n(math.MaxInt64)
		n := big.NewInt(r)
		AssertChainAlgorithmProduces(t, a, n)
		return true
	})
}

func checkPrimes(a alg.ChainAlgorithm) func(t *testing.T) {
	// Prepare primes in a random order.
	ps := []*big.Int{}
	for _, p := range prime.Distinguished {
		ps = append(ps, p.Int())
	}
	rand.Shuffle(len(ps), func(i, j int) { ps[i], ps[j] = ps[j], ps[i] })

	return test.Trials(func(t *testing.T) bool {
		AssertChainAlgorithmProduces(t, a, ps[0])
		ps = ps[1:]
		return len(ps) > 0
	})
}
