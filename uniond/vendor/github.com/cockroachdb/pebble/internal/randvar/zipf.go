// Copyright 2017 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License. See the AUTHORS file
// for names of contributors.
//
// ZipfGenerator implements the Incrementing Zipfian Random Number Generator from
// [1]: "Quickly Generating Billion-Record Synthetic Databases"
// by Gray, Sundaresan, Englert, Baclawski, and Weinberger, SIGMOD 1994.

package randvar

import (
	"math"
	"sync"

	"github.com/cockroachdb/errors"
	"golang.org/x/exp/rand"
)

const (
	// See https://github.com/brianfrankcooper/YCSB/blob/f886c1e7988f8f4965cb88a1fe2f6bad2c61b56d/core/src/main/java/com/yahoo/ycsb/generator/ScrambledZipfianGenerator.java#L33-L35
	defaultMax   = 10000000000
	defaultTheta = 0.99
	defaultZetaN = 26.46902820178302
)

// Zipf is a random number generator that generates random numbers from a Zipf
// distribution. Unlike rand.Zipf, this generator supports incrementing the max
// parameter without performing an expensive recomputation of the underlying
// hidden parameters, which is a pattern used in [1] for efficiently generating
// large volumes of Zipf-distributed records for synthetic data. Second,
// rand.Zipf only supports theta <= 1, we suppose all values of theta.
type Zipf struct {
	// Supplied constants.
	theta float64
	min   uint64
	// Internally computed constants.
	alpha, zeta2 float64
	halfPowTheta float64
	// Mutable state.
	mu struct {
		sync.RWMutex
		max   uint64
		eta   float64
		zetaN float64
	}
}

// NewDefaultZipf constructs a new Zipf generator with the default parameters.
func NewDefaultZipf() (*Zipf, error) {
	return NewZipf(1, defaultMax, defaultTheta)
}

// NewZipf constructs a new Zipf generator with the given parameters.  Returns
// an error if the parameters are outside the accepted range.
func NewZipf(min, max uint64, theta float64) (*Zipf, error) {
	if min > max {
		return nil, errors.Errorf("min %d > max %d", errors.Safe(min), errors.Safe(max))
	}
	if theta < 0.0 || theta == 1.0 {
		return nil, errors.New("0 < theta, and theta != 1")
	}

	z := &Zipf{
		min:   min,
		theta: theta,
	}
	z.mu.max = max

	// Compute hidden parameters.
	z.zeta2 = computeZetaFromScratch(2, theta)
	z.halfPowTheta = 1.0 + math.Pow(0.5, z.theta)
	z.mu.zetaN = computeZetaFromScratch(max+1-min, theta)
	z.alpha = 1.0 / (1.0 - theta)
	z.mu.eta = (1 - math.Pow(2.0/float64(z.mu.max+1-z.min), 1.0-theta)) / (1.0 - z.zeta2/z.mu.zetaN)
	return z, nil
}

// computeZetaIncrementally recomputes zeta(max, theta), assuming that sum =
// zeta(oldMax, theta). Returns zeta(max, theta), computed incrementally.
func computeZetaIncrementally(oldMax, max uint64, theta float64, sum float64) float64 {
	if max < oldMax {
		panic("unable to decrement max!")
	}
	for i := oldMax + 1; i <= max; i++ {
		sum += 1.0 / math.Pow(float64(i), theta)
	}
	return sum
}

// The function zeta computes the value
// zeta(n, theta) = (1/1)^theta + (1/2)^theta + (1/3)^theta + ... + (1/n)^theta
func computeZetaFromScratch(n uint64, theta float64) float64 {
	if n == defaultMax && theta == defaultTheta {
		// Precomputed value, borrowed from ScrambledZipfianGenerator.java. This is
		// quite slow to calculate from scratch due to the large n value.
		return defaultZetaN
	}
	return computeZetaIncrementally(0, n, theta, 0.0)
}

// IncMax increments max and recomputes the internal values that depend on
// it. Returns an error if the recomputation failed.
func (z *Zipf) IncMax(delta int) {
	z.mu.Lock()
	oldMax := z.mu.max
	z.mu.max += uint64(delta)
	z.mu.zetaN = computeZetaIncrementally(oldMax+1-z.min, z.mu.max+1-z.min, z.theta, z.mu.zetaN)
	z.mu.eta = (1 - math.Pow(2.0/float64(z.mu.max+1-z.min), 1.0-z.theta)) / (1.0 - z.zeta2/z.mu.zetaN)
	z.mu.Unlock()
}

// Max returns the max.
func (z *Zipf) Max() uint64 {
	z.mu.Lock()
	defer z.mu.Unlock()
	return z.mu.max
}

// Uint64 draws a new value between min and max, with probabilities according
// to the Zipf distribution.
func (z *Zipf) Uint64(rng *rand.Rand) uint64 {
	u := rng.Float64()
	z.mu.RLock()
	uz := u * z.mu.zetaN
	var result uint64
	if uz < 1.0 {
		result = z.min
	} else if uz < z.halfPowTheta {
		result = z.min + 1
	} else {
		spread := float64(z.mu.max + 1 - z.min)
		result = z.min + uint64(spread*math.Pow(z.mu.eta*u-z.mu.eta+1.0, z.alpha))
	}
	z.mu.RUnlock()
	return result
}
