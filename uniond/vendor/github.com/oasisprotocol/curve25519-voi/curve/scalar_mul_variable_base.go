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

import "github.com/oasisprotocol/curve25519-voi/curve/scalar"

func edwardsMul(out, point *EdwardsPoint, scalar *scalar.Scalar) *EdwardsPoint {
	switch supportsVectorizedEdwards {
	case true:
		return edwardsMulVector(out, point, scalar)
	default:
		return edwardsMulGeneric(out, point, scalar)
	}
}

func edwardsMulGeneric(out, point *EdwardsPoint, scalar *scalar.Scalar) *EdwardsPoint {
	// Construct a lookup table of [P,2P,3P,4P,5P,6P,7P,8P]
	lookupTable := newProjectiveNielsPointLookupTable(point)
	// Setting s = scalar, compute
	//
	//    s = s_0 + s_1*16^1 + ... + s_63*16^63,
	//
	// with `-8 <= s_i < 8` for `0 <= i < 63` and `-8 <= s_63 <= 8`.
	scalarDigits := scalar.ToRadix16()
	// Compute s*P as
	//
	//    s*P = P*(s_0 +   s_1*16^1 +   s_2*16^2 + ... +   s_63*16^63)
	//    s*P =  P*s_0 + P*s_1*16^1 + P*s_2*16^2 + ... + P*s_63*16^63
	//    s*P = P*s_0 + 16*(P*s_1 + 16*(P*s_2 + 16*( ... + P*s_63)...))
	//
	// We sum right-to-left.

	// Unwrap first loop iteration to save computing 16*identity
	var (
		tmp3 EdwardsPoint
		tmp2 projectivePoint
		tmp1 completedPoint
	)
	tmp3.Identity()
	tmp := lookupTable.Lookup(scalarDigits[63])
	tmp1.AddEdwardsProjectiveNiels(&tmp3, &tmp)
	// Now tmp1 = s_63*P in P1xP1 coords
	for i := 62; i >= 0; i-- {
		tmp2.SetCompleted(&tmp1) // tmp2 =    (prev) in P2 coords
		tmp1.Double(&tmp2)       // tmp1 =  2*(prev) in P1xP1 coords
		tmp2.SetCompleted(&tmp1) // tmp2 =  2*(prev) in P2 coords
		tmp1.Double(&tmp2)       // tmp1 =  4*(prev) in P1xP1 coords
		tmp2.SetCompleted(&tmp1) // tmp2 =  4*(prev) in P2 coords
		tmp1.Double(&tmp2)       // tmp1 =  8*(prev) in P1xP1 coords
		tmp2.SetCompleted(&tmp1) // tmp2 =  8*(prev) in P2 coords
		tmp1.Double(&tmp2)       // tmp1 = 16*(prev) in P1xP1 coords
		tmp3.setCompleted(&tmp1) // tmp3 = 16*(prev) in P3 coords
		tmp = lookupTable.Lookup(scalarDigits[i])
		tmp1.AddEdwardsProjectiveNiels(&tmp3, &tmp)
		// Now tmp1 = s_i*P + 16*(prev) in P1xP1 coords
	}
	return out.setCompleted(&tmp1)
}

func edwardsMulVector(out, point *EdwardsPoint, scalar *scalar.Scalar) *EdwardsPoint {
	lookupTable := newCachedPointLookupTable(point)

	scalarDigits := scalar.ToRadix16()
	var (
		q   extendedPoint
		tmp cachedPoint
	)
	q.Identity()
	for i := 63; i >= 0; i-- {
		q.MulByPow2(&q, 4)
		tmp = lookupTable.Lookup(scalarDigits[i])
		q.AddExtendedCached(&q, &tmp)
	}
	return out.setExtended(&q)
}
