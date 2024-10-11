package contfrac

import (
	"math/big"
	"testing"

	"github.com/mmcloughlin/addchain/alg"
	"github.com/mmcloughlin/addchain/alg/algtest"
	"github.com/mmcloughlin/addchain/internal/bigints"
)

func TestAlgorithms(t *testing.T) {
	for _, strategy := range Strategies {
		suite := algtest.SequenceAlgorithmSuite{
			Algorithm:          NewAlgorithm(strategy),
			AcceptsLargeInputs: strategy.Singleton(),
		}
		t.Run(suite.Algorithm.String(), suite.Tests())
	}
}

func TestBinaryStrategy(t *testing.T) {
	a := alg.AsChainAlgorithm(NewAlgorithm(BinaryStrategy{}))
	n := big.NewInt(87)
	expect := bigints.Int64s(1, 2, 4, 5, 10, 20, 21, 42, 43, 86, 87)
	algtest.AssertChainAlgorithmGenerates(t, a, n, expect)
}

func TestCoBinaryStrategy(t *testing.T) {
	a := alg.AsChainAlgorithm(NewAlgorithm(CoBinaryStrategy{}))
	n := big.NewInt(87)
	expect := bigints.Int64s(1, 2, 3, 5, 10, 11, 21, 22, 43, 44, 87)
	algtest.AssertChainAlgorithmGenerates(t, a, n, expect)
}
