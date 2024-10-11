// Copyright (c) 2016-2019 isis agora lovecruft. All rights reserved.
// Copyright (c) 2016-2019 Henry de Valence. All rights reserved.
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

import (
	"strconv"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
)

var benchMultiscalarSizes = []int{1, 2, 4, 8, 16, 32, 64, 128, 256, 384, 512, 768, 1024}

func BenchmarkEdwards(b *testing.B) {
	b.Run("Compress", benchEdwardsCompress)
	b.Run("Decompress", benchEdwardsDecompress)
	b.Run("Mul", benchEdwardsMul)
	b.Run("BasepointTable/New", benchEdwardsBasepointTableNew)
	b.Run("BasepointTable/Mul", benchEdwardsBasepointTableMul)
	b.Run("DoubleScalarMulBasepointVartime", benchEdwardsDoubleScalarMulBasepointVartime)
	b.Run("TripleScalarMulBasepointVartime", benchEdwardsTripleScalarMulBasepointVartime)
	b.Run("MultiscalarMul", benchEdwardsMultiscalarMul)
	b.Run("MultiscalarMulVartime", benchEdwardsMultiscalarMulVartime)
	b.Run("Window", benchEdwardsWindow)
}

func benchEdwardsCompress(b *testing.B) {
	var compressed CompressedEdwardsY
	for i := 0; i < b.N; i++ {
		compressed.SetEdwardsPoint(ED25519_BASEPOINT_POINT)
	}
}

func benchEdwardsDecompress(b *testing.B) {
	var decompressed EdwardsPoint
	for i := 0; i < b.N; i++ {
		if _, err := decompressed.SetCompressedY(ED25519_BASEPOINT_COMPRESSED); err != nil {
			b.Fatalf("FromCompressedY(): %v", err)
		}
	}
}

func benchEdwardsMul(b *testing.B) {
	s := scalar.New().Invert(scalar.NewFromUint64(897987897))

	b.ResetTimer()

	var tmp EdwardsPoint
	for i := 0; i < b.N; i++ {
		tmp.Mul(ED25519_BASEPOINT_POINT, s)
	}
}

func benchEdwardsBasepointTableNew(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = NewEdwardsBasepointTable(ED25519_BASEPOINT_POINT)
	}
}

func benchEdwardsBasepointTableMul(b *testing.B) {
	s := scalar.New().Invert(scalar.NewFromUint64(897987897))

	b.ResetTimer()

	var tmp EdwardsPoint
	for i := 0; i < b.N; i++ {
		tmp.MulBasepoint(ED25519_BASEPOINT_TABLE, s)
	}
	_ = tmp // Shut up compiler.
}

func benchEdwardsDoubleScalarMulBasepointVartime(b *testing.B) {
	A := newTestBenchRandomPoint(b)

	b.ReportAllocs()
	b.ResetTimer()

	var tmp EdwardsPoint
	for i := 0; i < b.N; i++ {
		b.StopTimer()
		aScalar, bScalar := newTestBenchRandomScalar(b), newTestBenchRandomScalar(b)
		b.StartTimer()

		tmp.DoubleScalarMulBasepointVartime(aScalar, A, bScalar)
	}
}

func benchEdwardsTripleScalarMulBasepointVartime(b *testing.B) {
	A := newTestBenchRandomPoint(b)
	C := newTestBenchRandomPoint(b)

	b.ReportAllocs()
	b.ResetTimer()

	var tmp EdwardsPoint
	for i := 0; i < b.N; i++ {
		b.StopTimer()
		aScalar, bScalar := newTestBenchRandomScalar(b), newTestBenchRandomScalar(b)
		b.StartTimer()

		tmp.TripleScalarMulBasepointVartime(aScalar, A, bScalar, C)
	}
}

func benchEdwardsMultiscalarMulIter(b *testing.B, n int, isVartime bool) {
	points := newBenchRandomPoints(b, n)

	b.ResetTimer()

	var tmp EdwardsPoint
	for i := 0; i < b.N; i++ {
		b.StopTimer()
		scalars := newTestBenchRandomScalars(b, n)
		b.StartTimer()

		if isVartime {
			tmp.MultiscalarMulVartime(scalars, points)
		} else {
			tmp.MultiscalarMul(scalars, points)
		}
	}
}

func benchEdwardsMultiscalarMul(b *testing.B) {
	for _, n := range benchMultiscalarSizes {
		b.Run(strconv.Itoa(n), func(b *testing.B) {
			b.ReportAllocs()
			benchEdwardsMultiscalarMulIter(b, n, false)
		})
	}
}

func benchEdwardsMultiscalarMulVartime(b *testing.B) {
	for _, n := range benchMultiscalarSizes {
		b.Run(strconv.Itoa(n), func(b *testing.B) {
			b.ReportAllocs()
			benchEdwardsMultiscalarMulIter(b, n, true)
		})
	}
}

func benchEdwardsWindow(b *testing.B) {
	b.Run("newAffineNielsPointNafLookupTable", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			_ = newAffineNielsPointNafLookupTable(ED25519_BASEPOINT_POINT)
		}
	})
	b.Run("unpackEdwardsBasepointTable", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			_ = unpackEdwardsBasepointTable()
		}
	})
	b.Run("unpackAffineNielsPointNafLookupTable", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			_ = unpackAffineNielsPointNafLookupTable(packedAffineOddMultiplesOfBasepoint)
		}
	})

	if supportsVectorizedEdwards {
		b.Run("newCachedPointNafLookupTable8", func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				_ = newCachedPointNafLookupTable8(ED25519_BASEPOINT_POINT)
			}
		})
	}
}

func BenchmarkExpandedEdwards(b *testing.B) {
	b.Run("New", benchExpandedEdwardsNew)
	b.Run("DoubleScalarMulBasepointVartime", benchExpandedEdwardsDoubleScalarMulBasepointVartime)
	b.Run("TripleScalarMulBasepointVartime", benchExpandedEdwardsTripleScalarMulBasepointVartime)
	b.Run("MultiscalarMulVartime", benchExpandedEdwardsMultiscalarMulVartime)
}

func benchExpandedEdwardsNew(b *testing.B) {
	p := newTestBenchRandomPoint(b)

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = NewExpandedEdwardsPoint(p)
	}
}

func benchExpandedEdwardsTripleScalarMulBasepointVartime(b *testing.B) {
	A := NewExpandedEdwardsPoint(newTestBenchRandomPoint(b))
	C := newTestBenchRandomPoint(b)

	b.ReportAllocs()
	b.ResetTimer()

	var tmp EdwardsPoint
	for i := 0; i < b.N; i++ {
		b.StopTimer()
		aScalar, bScalar := newTestBenchRandomScalar(b), newTestBenchRandomScalar(b)
		b.StartTimer()

		tmp.ExpandedTripleScalarMulBasepointVartime(aScalar, A, bScalar, C)
	}
}

func benchExpandedEdwardsDoubleScalarMulBasepointVartime(b *testing.B) {
	A := NewExpandedEdwardsPoint(newTestBenchRandomPoint(b))

	b.ReportAllocs()
	b.ResetTimer()

	var tmp EdwardsPoint
	for i := 0; i < b.N; i++ {
		b.StopTimer()
		aScalar, bScalar := newTestBenchRandomScalar(b), newTestBenchRandomScalar(b)
		b.StartTimer()

		tmp.ExpandedDoubleScalarMulBasepointVartime(aScalar, A, bScalar)
	}
}

func benchExpandedEdwardsMultiscalarMulVartime(b *testing.B) {
	for _, n := range benchMultiscalarSizes {
		points := newBenchRandomPoints(b, n)
		prePoints := make([]*ExpandedEdwardsPoint, 0, n)
		for _, point := range points {
			prePoints = append(prePoints, NewExpandedEdwardsPoint(point))
		}

		b.Run(strconv.Itoa(n), func(b *testing.B) {
			b.ReportAllocs()

			b.ResetTimer()

			var tmp EdwardsPoint
			for i := 0; i < b.N; i++ {
				b.StopTimer()
				scalars := newTestBenchRandomScalars(b, n)
				b.StartTimer()

				tmp.ExpandedMultiscalarMulVartime(scalars, prePoints, nil, nil)
			}
		})
	}
}

func BenchmarkRistretto(b *testing.B) {
	b.Run("Compress", benchRistrettoCompress)
	b.Run("Decompress", benchRistrettoDecompress)
}

func benchRistrettoCompress(b *testing.B) {
	var compressed CompressedRistretto
	for i := 0; i < b.N; i++ {
		compressed.SetRistrettoPoint(RISTRETTO_BASEPOINT_POINT)
	}
}

func benchRistrettoDecompress(b *testing.B) {
	var decompressed RistrettoPoint
	for i := 0; i < b.N; i++ {
		if _, err := decompressed.SetCompressed(RISTRETTO_BASEPOINT_COMPRESSED); err != nil {
			b.Fatalf("FromRistrettoPoint(): %v", err)
		}
	}
}

func BenchmarkMontgomery(b *testing.B) {
	b.Run("Mul", benchMontgomeryMul)
}

func benchMontgomeryMul(b *testing.B) {
	s := scalar.New().Invert(scalar.NewFromUint64(897987897))

	b.ResetTimer()

	var tmp MontgomeryPoint
	for i := 0; i < b.N; i++ {
		tmp.Mul(X25519_BASEPOINT, s)
	}
}

func newBenchRandomPoints(b *testing.B, n int) []*EdwardsPoint {
	v := make([]*EdwardsPoint, 0, n)
	for i := 0; i < n; i++ {
		v = append(v, newTestBenchRandomPoint(b))
	}
	return v
}
