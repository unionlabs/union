package simulation

import "math/rand"

// TransitionMatrix is _almost_ a left stochastic matrix.  It is technically
// not one due to not normalizing the column values.  In the future, if we want
// to find the steady state distribution, it will be quite easy to normalize
// these values to get a stochastic matrix.  Floats aren't currently used as
// the default due to non-determinism across architectures
type TransitionMatrix interface {
	NextState(r *rand.Rand, i int) int
}
