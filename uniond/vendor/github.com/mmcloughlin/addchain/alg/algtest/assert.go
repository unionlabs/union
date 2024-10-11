package algtest

import (
	"math/big"
	"reflect"
	"testing"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/alg"
)

// AssertChainAlgorithmGenerates asserts that the algorithm generates the expected chain for n.
func AssertChainAlgorithmGenerates(t *testing.T, a alg.ChainAlgorithm, n *big.Int, expect addchain.Chain) {
	c, err := a.FindChain(n)
	if err != nil {
		t.Fatal(err)
	}
	if err := c.Validate(); err != nil {
		t.Error(err)
	}
	if !reflect.DeepEqual(expect, c) {
		t.Fatalf("got %v; expect %v", c, expect)
	}
}

// AssertChainAlgorithmProduces verifies that a returns a valid chain for n.
func AssertChainAlgorithmProduces(t *testing.T, a alg.ChainAlgorithm, n *big.Int) addchain.Chain {
	c, err := a.FindChain(n)
	if err != nil {
		t.Fatal(err)
	}
	err = c.Produces(n)
	if err != nil {
		t.Log(c)
		t.Fatal(err)
	}
	return c
}

// AssertSequenceAlgorithmProduces verifies that a returns a valid chain containing targets.
func AssertSequenceAlgorithmProduces(t *testing.T, a alg.SequenceAlgorithm, targets []*big.Int) addchain.Chain {
	c, err := a.FindSequence(targets)
	if err != nil {
		t.Fatal(err)
	}
	err = c.Superset(targets)
	if err != nil {
		t.Log(c)
		t.Fatal(err)
	}
	return c
}
