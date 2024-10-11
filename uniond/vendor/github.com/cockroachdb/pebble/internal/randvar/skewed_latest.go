// Copyright 2018 The Cockroach Authors.
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

package randvar

import (
	"sync"

	"golang.org/x/exp/rand"
)

// SkewedLatest is a random number generator that generates numbers in
// the range [min, max], but skews it towards max using a zipfian
// distribution.
type SkewedLatest struct {
	mu struct {
		sync.RWMutex
		max  uint64
		zipf *Zipf
	}
}

// NewDefaultSkewedLatest constructs a new SkewedLatest generator with the
// default parameters.
func NewDefaultSkewedLatest() (*SkewedLatest, error) {
	return NewSkewedLatest(1, defaultMax, defaultTheta)
}

// NewSkewedLatest constructs a new SkewedLatest generator with the given
// parameters. It returns an error if the parameters are outside the accepted
// range.
func NewSkewedLatest(min, max uint64, theta float64) (*SkewedLatest, error) {
	z := &SkewedLatest{}
	z.mu.max = max
	zipf, err := NewZipf(0, max-min, theta)
	if err != nil {
		return nil, err
	}
	z.mu.zipf = zipf
	return z, nil
}

// IncMax increments max.
func (z *SkewedLatest) IncMax(delta int) {
	z.mu.Lock()
	z.mu.zipf.IncMax(delta)
	z.mu.max += uint64(delta)
	z.mu.Unlock()
}

// Max returns max.
func (z *SkewedLatest) Max() uint64 {
	z.mu.Lock()
	defer z.mu.Unlock()
	return z.mu.zipf.Max()
}

// Uint64 returns a random Uint64 between min and max, where keys near max are
// most likely to be drawn.
func (z *SkewedLatest) Uint64(rng *rand.Rand) uint64 {
	z.mu.RLock()
	result := z.mu.max - z.mu.zipf.Uint64(rng)
	z.mu.RUnlock()
	return result
}
