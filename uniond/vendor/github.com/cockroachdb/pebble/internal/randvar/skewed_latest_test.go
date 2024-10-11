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
	"fmt"
	"sort"
	"testing"

	"github.com/stretchr/testify/require"
)

func dumpSamples(x []int) {
	sort.Ints(x)

	max := x[len(x)-1]
	step := max / 20
	if step == 0 {
		step = 1
	}
	index := 0
	count := 0
	for i := 0; i <= max; i += step {
		count = 0
		for index < len(x) {
			if x[index] >= i+step {
				break
			}
			index++
			count++
		}
		fmt.Printf("[%3d-%3d) ", i, i+step)
		for j := 0; j < count; j++ {
			if j%50 == 0 {
				fmt.Printf("%c", '∎')
			}
		}
		fmt.Println()
	}
}

func TestSkewedLatest(t *testing.T) {
	rng := NewRand()
	z, err := NewSkewedLatest(0, 99, 0.99)
	require.NoError(t, err)

	x := make([]int, 10000)
	for i := range x {
		x[i] = int(z.Uint64(rng))
	}

	if testing.Verbose() {
		dumpSamples(x)
	}
}
