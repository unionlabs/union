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

import (
	"testing"

	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
)

func testEdwardsMultiscalarMulPippengerVartime(t *testing.T) {
	n := 512
	x := scalar.New().Invert(scalar.NewFromUint64(2128506))
	y := scalar.New().Invert(scalar.NewFromUint64(4443282))

	points := make([]*EdwardsPoint, 0, n)
	for i := 0; i < n; i++ {
		tmp := scalar.NewFromUint64(1 + uint64(i))

		var point EdwardsPoint
		point.Mul(ED25519_BASEPOINT_POINT, tmp)
		points = append(points, &point)
	}

	scalars := make([]*scalar.Scalar, 0, n)
	for i := 0; i < n; i++ {
		tmp := scalar.New().Mul(scalar.NewFromUint64(uint64(i)), y)
		tmp.Add(x, tmp)
		scalars = append(scalars, tmp)
	}

	premultiplied := make([]*EdwardsPoint, 0, n)
	for i := 0; i < n; i++ {
		var point EdwardsPoint
		point.Mul(points[i], scalars[i])
		premultiplied = append(premultiplied, &point)
	}

	for n > 0 {
		var control EdwardsPoint
		control.Sum(premultiplied[:n])

		var subject EdwardsPoint
		edwardsMultiscalarMulPippengerVartime(&subject, scalars[:n], points[:n])

		if subject.Equal(&control) == 0 {
			t.Fatalf("multiscalarMulPippengerVartime(scalars[:%d], points[:%d] != control (Got: %v)", n, n, subject)
		}

		n = n / 2
	}
}
