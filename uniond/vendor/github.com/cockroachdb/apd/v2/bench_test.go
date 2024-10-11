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
// permissions and limitations under the License.

package apd

import (
	"bytes"
	"fmt"
	"math/rand"
	"testing"
)

// runBenches benchmarks a given function on random decimals on combinations of
// three parameters:
//
//   precision:    desired output precision
//   inScale:      the scale of the input decimal: the absolute value will be between
//                 10^inScale and 10^(inScale+1)
//   inNumDigits:  number of digits in the input decimal; if negative the number
//                 will be negative and the number of digits are the absolute value.
func runBenches(
	b *testing.B, precision, inScale, inNumDigits []int, fn func(*testing.B, *Context, *Decimal),
) {
	for _, p := range precision {
		ctx := BaseContext.WithPrecision(uint32(p))
		for _, s := range inScale {
			for _, d := range inNumDigits {
				numDigits := d
				negative := false
				if d < 0 {
					numDigits = -d
					negative = true
				}
				if numDigits > p {
					// Skip cases where we have more digits than the desired precision.
					continue
				}

				// Generate some random numbers with the given number of digits.
				nums := make([]Decimal, 20)
				for i := range nums {
					var buf bytes.Buffer
					if negative {
						buf.WriteByte('-')
					}
					buf.WriteByte('1' + byte(rand.Intn(9)))
					for j := 1; j < numDigits; j++ {
						buf.WriteByte('0' + byte(rand.Intn(10)))
					}
					if _, _, err := nums[i].SetString(buf.String()); err != nil {
						b.Fatal(err)
					}
					nums[i].Exponent = int32(s - numDigits)
				}
				b.Run(
					fmt.Sprintf("P%d/S%d/D%d", p, s, d),
					func(b *testing.B) {
						for i := 0; i <= b.N; i++ {
							fn(b, ctx, &nums[i%len(nums)])
						}
					},
				)
			}
		}
	}
}

func BenchmarkExp(b *testing.B) {
	precision := []int{5, 10, 100}
	scale := []int{-4, -1, 2}
	digits := []int{-100, -10, -2, 2, 10, 100}
	runBenches(
		b, precision, scale, digits,
		func(b *testing.B, ctx *Context, x *Decimal) {
			if _, err := ctx.Exp(&Decimal{}, x); err != nil {
				b.Fatal(err)
			}
		},
	)
}

func BenchmarkLn(b *testing.B) {
	precision := []int{2, 10, 100}
	scale := []int{-100, -10, -2, 2, 10, 100}
	digits := []int{2, 10, 100}
	runBenches(
		b, precision, scale, digits,
		func(b *testing.B, ctx *Context, x *Decimal) {
			if _, err := ctx.Ln(&Decimal{}, x); err != nil {
				b.Fatal(err)
			}
		},
	)
}
