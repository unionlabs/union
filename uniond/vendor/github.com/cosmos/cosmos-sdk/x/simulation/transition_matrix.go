package simulation

import (
	"fmt"
	"math/rand"

	"github.com/cosmos/cosmos-sdk/types/simulation"
)

// TransitionMatrix is _almost_ a left stochastic matrix.  It is technically
// not one due to not normalizing the column values.  In the future, if we want
// to find the steady state distribution, it will be quite easy to normalize
// these values to get a stochastic matrix.  Floats aren't currently used as
// the default due to non-determinism across architectures
type TransitionMatrix struct {
	weights [][]int
	// total in each column
	totals []int
	n      int
}

// CreateTransitionMatrix creates a transition matrix from the provided weights.
// TODO: Provide example usage
func CreateTransitionMatrix(weights [][]int) (simulation.TransitionMatrix, error) {
	n := len(weights)
	for i := 0; i < n; i++ {
		if len(weights[i]) != n {
			return TransitionMatrix{},
				fmt.Errorf("transition matrix: non-square matrix provided, error on row %d", i)
		}
	}

	totals := make([]int, n)

	for row := 0; row < n; row++ {
		for col := 0; col < n; col++ {
			totals[col] += weights[row][col]
		}
	}

	return TransitionMatrix{weights, totals, n}, nil
}

// NextState returns the next state randomly chosen using r, and the weightings
// provided in the transition matrix.
func (t TransitionMatrix) NextState(r *rand.Rand, i int) int {
	randNum := r.Intn(t.totals[i])
	for row := 0; row < t.n; row++ {
		if randNum < t.weights[row][i] {
			return row
		}

		randNum -= t.weights[row][i]
	}
	// This line should never get executed
	return -1
}

// GetMemberOfInitialState takes an initial array of weights, of size n.
// It returns a weighted random number in [0,n).
func GetMemberOfInitialState(r *rand.Rand, weights []int) int {
	n := len(weights)
	total := 0

	for i := 0; i < n; i++ {
		total += weights[i]
	}

	randNum := r.Intn(total)

	for state := 0; state < n; state++ {
		if randNum < weights[state] {
			return state
		}

		randNum -= weights[state]
	}
	// This line should never get executed
	return -1
}
