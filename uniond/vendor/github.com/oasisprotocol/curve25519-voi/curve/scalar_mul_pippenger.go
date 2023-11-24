// Copyright (c) 2019 Oglev Andreev. All rights reserved.
// Copyright (c) 2020-2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
// TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
// PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
// TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package curve

import "github.com/oasisprotocol/curve25519-voi/curve/scalar"

func edwardsMultiscalarMulPippengerVartime(out *EdwardsPoint, scalars []*scalar.Scalar, points []*EdwardsPoint) *EdwardsPoint {
	switch supportsVectorizedEdwards {
	case true:
		return edwardsMultiscalarMulPippengerVartimeVector(out, nil, nil, scalars, points)
	default:
		return edwardsMultiscalarMulPippengerVartimeGeneric(out, nil, nil, scalars, points)
	}
}

func expandedEdwardsMultiscalarMulPippengerVartime(out *EdwardsPoint, staticScalars []*scalar.Scalar, staticPoints []*ExpandedEdwardsPoint, dynamicScalars []*scalar.Scalar, dynamicPoints []*EdwardsPoint) *EdwardsPoint {
	// There is no actual precomputed Pippenger's implementation,
	// but pretending that there is one saves memory and time
	// when we have to fall back to the non-precomputed version.
	points := make([]*EdwardsPoint, 0, len(staticPoints))
	for _, point := range staticPoints {
		points = append(points, &point.point)
	}

	switch supportsVectorizedEdwards {
	case true:
		return edwardsMultiscalarMulPippengerVartimeVector(out, staticScalars, points, dynamicScalars, dynamicPoints)
	default:
		return edwardsMultiscalarMulPippengerVartimeGeneric(out, staticScalars, points, dynamicScalars, dynamicPoints)
	}
}

func edwardsMultiscalarMulPippengerVartimeGeneric(out *EdwardsPoint, staticScalars []*scalar.Scalar, staticPoints []*EdwardsPoint, dynamicScalars []*scalar.Scalar, dynamicPoints []*EdwardsPoint) *EdwardsPoint {
	size := len(staticScalars) + len(dynamicScalars)

	// Digit width in bits. As digit width grows,
	// number of point additions goes down, but amount of
	// buckets and bucket additions grows exponentially.
	var w uint
	switch {
	case size < 500:
		w = 6
	case size < 800:
		w = 7
	default:
		w = 8
	}

	maxDigit := 1 << w
	digitsCount := scalar.ToRadix2wSizeHint(w)
	bucketsCount := maxDigit / 2 // digits are signed+centered hence 2^w/2, excluding 0-th bucket.

	// Collect optimized scalars and points in buffers for repeated access
	// (scanning the whole set per digit position).
	optScalars := make([][43]int8, 0, size)
	for _, scalars := range [][]*scalar.Scalar{staticScalars, dynamicScalars} {
		for _, scalar := range scalars {
			optScalars = append(optScalars, scalar.ToRadix2w(w))
		}
	}

	optPoints, off := make([]projectiveNielsPoint, size), 0
	for _, points := range [][]*EdwardsPoint{staticPoints, dynamicPoints} {
		for i, point := range points {
			optPoints[i+off].SetEdwards(point)
		}
		off += len(points)
	}

	// Prepare 2^w/2 buckets.
	// buckets[i] corresponds to a multiplication factor (i+1).
	//
	// No need to initialize the buckets since calculateColumn initializes
	// them as needed as the first thing in the routine.
	buckets := make([]EdwardsPoint, bucketsCount)

	calculateColumn := func(idx int) EdwardsPoint {
		// Clear the buckets when processing another digit.
		for i := 0; i < bucketsCount; i++ {
			buckets[i].Identity()
		}

		// Iterate over pairs of (point, scalar)
		// and add/sub the point to the corresponding bucket.
		// Note: if we add support for precomputed lookup tables,
		// we'll be adding/subtracting point premultiplied by `digits[i]` to buckets[0].
		var tmp completedPoint
		for i := 0; i < size; i++ {
			digit := int16(optScalars[i][idx])
			if digit > 0 {
				b := uint(digit - 1)
				buckets[b].setCompleted(tmp.AddEdwardsProjectiveNiels(&buckets[b], &optPoints[i]))
			} else if digit < 0 {
				b := uint(-digit - 1)
				buckets[b].setCompleted(tmp.SubEdwardsProjectiveNiels(&buckets[b], &optPoints[i]))
			}
		}

		// Add the buckets applying the multiplication factor to each bucket.
		// The most efficient way to do that is to have a single sum with two running sums:
		// an intermediate sum from last bucket to the first, and a sum of intermediate sums.
		//
		// For example, to add buckets 1*A, 2*B, 3*C we need to add these points:
		//   C
		//   C B
		//   C B A   Sum = C + (C+B) + (C+B+A)

		bucketsIntermediateSum := buckets[bucketsCount-1]
		bucketsSum := buckets[bucketsCount-1]
		for i := int((bucketsCount - 1) - 1); i >= 0; i-- {
			bucketsIntermediateSum.Add(&bucketsIntermediateSum, &buckets[i])
			bucketsSum.Add(&bucketsSum, &bucketsIntermediateSum)
		}

		return bucketsSum
	}

	// Take the high column as an initial value to avoid wasting time doubling
	// the identity element.
	sum := calculateColumn(int(digitsCount - 1))
	for i := int(digitsCount-1) - 1; i >= 0; i-- {
		var sumMul EdwardsPoint
		p := calculateColumn(i)
		sum.Add(sumMul.mulByPow2(&sum, w), &p)
	}

	return out.Set(&sum)
}

func edwardsMultiscalarMulPippengerVartimeVector(out *EdwardsPoint, staticScalars []*scalar.Scalar, staticPoints []*EdwardsPoint, dynamicScalars []*scalar.Scalar, dynamicPoints []*EdwardsPoint) *EdwardsPoint {
	size := len(staticScalars) + len(dynamicScalars)

	var w uint
	switch {
	case size < 500:
		w = 6
	case size < 800:
		w = 7
	default:
		w = 8
	}

	maxDigit := 1 << w
	digitsCount := scalar.ToRadix2wSizeHint(w)
	bucketsCount := maxDigit / 2

	optScalars := make([][43]int8, 0, size)
	for _, scalars := range [][]*scalar.Scalar{staticScalars, dynamicScalars} {
		for _, scalar := range scalars {
			optScalars = append(optScalars, scalar.ToRadix2w(w))
		}
	}

	optPoints, off := make([]cachedPoint, size), 0
	for _, points := range [][]*EdwardsPoint{staticPoints, dynamicPoints} {
		for i, point := range points {
			var ep extendedPoint
			optPoints[i+off].SetExtended(ep.SetEdwards(point))
		}
		off += len(points)
	}

	buckets := make([]extendedPoint, bucketsCount)

	calculateColumn := func(idx int) extendedPoint {
		for i := 0; i < bucketsCount; i++ {
			buckets[i].Identity()
		}

		for i := 0; i < size; i++ {
			digit := int16(optScalars[i][idx])
			if digit > 0 {
				b := uint(digit - 1)
				buckets[b].AddExtendedCached(&buckets[b], &optPoints[i])
			} else if digit < 0 {
				b := uint(-digit - 1)
				buckets[b].SubExtendedCached(&buckets[b], &optPoints[i])
			}
		}

		bucketsIntermediateSum := buckets[bucketsCount-1]
		bucketsSum := buckets[bucketsCount-1]
		for i := int((bucketsCount - 1) - 1); i >= 0; i-- {
			var cp cachedPoint
			bucketsIntermediateSum.AddExtendedCached(&bucketsIntermediateSum, cp.SetExtended(&buckets[i]))
			bucketsSum.AddExtendedCached(&bucketsSum, cp.SetExtended(&bucketsIntermediateSum))
		}

		return bucketsSum
	}

	sum := calculateColumn(int(digitsCount - 1))
	for i := int(digitsCount-1) - 1; i >= 0; i-- {
		var (
			sumMul extendedPoint
			cp     cachedPoint
		)
		ep := calculateColumn(i)
		sum.AddExtendedCached(sumMul.MulByPow2(&sum, w), cp.SetExtended(&ep))
	}

	return out.setExtended(&sum)
}
