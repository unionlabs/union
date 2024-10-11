/*
 *
 * Copyright 2019 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package wrr

import (
	"errors"
	"math"
	"math/rand"
	"strconv"
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/internal/grpctest"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

const iterCount = 10000

func equalApproximate(a, b float64) error {
	opt := cmp.Comparer(func(x, y float64) bool {
		delta := math.Abs(x - y)
		mean := math.Abs(x+y) / 2.0
		return delta/mean < 0.05
	})
	if !cmp.Equal(a, b, opt) {
		return errors.New(cmp.Diff(a, b))
	}
	return nil
}

func testWRRNext(t *testing.T, newWRR func() WRR) {
	tests := []struct {
		name    string
		weights []int64
	}{
		{
			name:    "1-1-1",
			weights: []int64{1, 1, 1},
		},
		{
			name:    "1-2-3",
			weights: []int64{1, 2, 3},
		},
		{
			name:    "5-3-2",
			weights: []int64{5, 3, 2},
		},
		{
			name:    "17-23-37",
			weights: []int64{17, 23, 37},
		},
		{
			name:    "no items",
			weights: []int64{},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			w := newWRR()
			if len(tt.weights) == 0 {
				if next := w.Next(); next != nil {
					t.Fatalf("w.Next returns non nil value:%v when there is no item", next)
				}
				return
			}

			var sumOfWeights int64
			for i, weight := range tt.weights {
				w.Add(i, weight)
				sumOfWeights += weight
			}

			results := make(map[int]int)
			for i := 0; i < iterCount; i++ {
				results[w.Next().(int)]++
			}

			wantRatio := make([]float64, len(tt.weights))
			for i, weight := range tt.weights {
				wantRatio[i] = float64(weight) / float64(sumOfWeights)
			}
			gotRatio := make([]float64, len(tt.weights))
			for i, count := range results {
				gotRatio[i] = float64(count) / iterCount
			}

			for i := range wantRatio {
				if err := equalApproximate(gotRatio[i], wantRatio[i]); err != nil {
					t.Errorf("%v not equal %v", i, err)
				}
			}
		})
	}
}

func (s) TestRandomWRRNext(t *testing.T) {
	testWRRNext(t, NewRandom)
}

func (s) TestEdfWrrNext(t *testing.T) {
	testWRRNext(t, NewEDF)
}

func BenchmarkRandomWRRNext(b *testing.B) {
	for _, n := range []int{100, 500, 1000} {
		b.Run("equal-weights-"+strconv.Itoa(n)+"-items", func(b *testing.B) {
			w := NewRandom()
			sumOfWeights := n
			for i := 0; i < n; i++ {
				w.Add(i, 1)
			}
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				for i := 0; i < sumOfWeights; i++ {
					w.Next()
				}
			}
		})
	}

	var maxWeight int64 = 1024
	for _, n := range []int{100, 500, 1000} {
		b.Run("random-weights-"+strconv.Itoa(n)+"-items", func(b *testing.B) {
			w := NewRandom()
			var sumOfWeights int64
			for i := 0; i < n; i++ {
				weight := rand.Int63n(maxWeight + 1)
				w.Add(i, weight)
				sumOfWeights += weight
			}
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				for i := 0; i < int(sumOfWeights); i++ {
					w.Next()
				}
			}
		})
	}

	itemsNum := 200
	heavyWeight := int64(itemsNum)
	lightWeight := int64(1)
	heavyIndices := []int{0, itemsNum / 2, itemsNum - 1}
	for _, heavyIndex := range heavyIndices {
		b.Run("skew-weights-heavy-index-"+strconv.Itoa(heavyIndex), func(b *testing.B) {
			w := NewRandom()
			var sumOfWeights int64
			for i := 0; i < itemsNum; i++ {
				var weight int64
				if i == heavyIndex {
					weight = heavyWeight
				} else {
					weight = lightWeight
				}
				sumOfWeights += weight
				w.Add(i, weight)
			}
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				for i := 0; i < int(sumOfWeights); i++ {
					w.Next()
				}
			}
		})
	}
}

func init() {
	r := rand.New(rand.NewSource(0))
	randInt63n = r.Int63n
}
