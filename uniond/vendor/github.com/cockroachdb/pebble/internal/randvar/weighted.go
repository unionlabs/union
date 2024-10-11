// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package randvar

import "golang.org/x/exp/rand"

// Weighted is a random number generator that generates numbers in the range
// [0,len(weights)-1] where the probability of i is weights(i)/sum(weights).
type Weighted struct {
	rng     *rand.Rand
	sum     float64
	weights []float64
}

// NewWeighted returns a new weighted random number generator.
func NewWeighted(rng *rand.Rand, weights ...float64) *Weighted {
	var sum float64
	for i := range weights {
		sum += weights[i]
	}
	return &Weighted{
		rng:     ensureRand(rng),
		sum:     sum,
		weights: weights,
	}
}

// Int returns a random number in the range [0,len(weights)-1] where the
// probability of i is weights(i)/sum(weights).
func (w *Weighted) Int() int {
	p := w.rng.Float64() * w.sum
	for i, weight := range w.weights {
		if p <= weight {
			return i
		}
		p -= weight
	}
	return len(w.weights) - 1
}
