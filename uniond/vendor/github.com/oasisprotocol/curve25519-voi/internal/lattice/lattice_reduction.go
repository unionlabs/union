// Copyright (c) 2020 Jack Grigg. All rights reserved.
// Copyright (c) 2021 Oasis Labs Inc. All rights reserved.
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

package lattice

import "github.com/oasisprotocol/curve25519-voi/curve/scalar"

var constELL_LOWER_HALF = newInt128(0x14def9dea2f79cd6, 0x5812631a5cf5d3ed)

// FindShortVector finds a "short" non-zero vector `(d_0, d_1)` such that
// `d_0 = d_1 k mod ell`. `d_0` and `d_1` may be negative.
//
// Implements Algorithm 4 from [Pornin 2020](https://eprint.iacr.org/2020/454).
func FindShortVector(k *scalar.Scalar) (Int128, Int128) {
	N_u_512 := ellSquared()

	N_v_512 := (&int512{}).Mul(k, k)
	N_v_512 = N_v_512.Add(N_v_512, i512One)

	p_512 := (&int512{}).Mul(scalar.BASEPOINT_ORDER, k)

	// The target bit-length of `N_v` for the vector to be considered short.
	const T = 254 // len(ell) + 1

	u_0, u_1 := constELL_LOWER_HALF, i128Zero
	v_0, v_1 := newInt128FromScalar(k), i128One

	// Like described in the paper under "Implementation Notes and Benchmarks",
	// this is implemented in two passes, one with 512-bit integers, and one
	// with 384-bit integers, leveraging the fact that N_u, N_v and p will
	// continually decrease in size.
	//
	// One would think that the duplication of the algorithm could be
	// eliminated by defining a bigInt interface, and they would be right.
	// However doing so comes at the cost of 6 heap allocations, and a 25%
	// performance penalty, at which point we are better off not bothering
	// to shrink the representations in the first place.

	//
	// Pass 1: N_u, N_v, p are stored in 512-bit integers.
	//

	for {
		if N_u_512.PositiveLt(N_v_512) { // N_u < N_v
			u_0, v_0 = v_0, u_0
			u_1, v_1 = v_1, u_1
			N_u_512, N_v_512 = N_v_512, N_u_512
		}

		// N_u is the largest out of N_u, N_v, and p.  N_u and N_v are
		// always positive, however p can be negative.  So if N_u fits
		// into 383 bits, then it is safe to store everything in a
		// 384-bit integer, since |p| <= N_u and provisions are made
		// as part of the check for the sign bit.
		if N_u_512.SafeToShrink() {
			break
		}

		len_N_v := N_v_512.BitLen()
		if len_N_v <= T {
			return v_0, v_1
		}

		var s uint
		if len_p := p_512.BitLen(); len_p > len_N_v {
			s = len_p - len_N_v
		}

		if !p_512.IsNegative() {
			u_0 = u_0.sub(v_0.shl(s))
			u_1 = u_1.sub(v_1.shl(s))

			N_u_512.AddShifted(N_u_512, N_v_512, 2*s)
			N_u_512.SubShifted(N_u_512, p_512, s+1)
			p_512.SubShifted(p_512, N_v_512, s)
		} else {
			u_0 = u_0.add(v_0.shl(s))
			u_1 = u_1.add(v_1.shl(s))

			N_u_512.AddShifted(N_u_512, N_v_512, 2*s)
			N_u_512.AddShifted(N_u_512, p_512, s+1)
			p_512.AddShifted(p_512, N_v_512, s)
		}
	}

	//
	// Pass 2: N_u, N_v, and p are store in 384-bit integers.
	//

	N_u_384 := (&int384{}).FromInt512(N_u_512)
	N_v_384 := (&int384{}).FromInt512(N_v_512)
	p_384 := (&int384{}).FromInt512(p_512)

	for {
		if N_u_384.PositiveLt(N_v_384) {
			u_0, v_0 = v_0, u_0
			u_1, v_1 = v_1, u_1
			N_u_384, N_v_384 = N_v_384, N_u_384
		}

		len_N_v := N_v_384.BitLen()
		if len_N_v <= T {
			return v_0, v_1
		}

		var s uint
		if len_p := p_384.BitLen(); len_p > len_N_v {
			s = len_p - len_N_v
		}

		if !p_384.IsNegative() {
			u_0 = u_0.sub(v_0.shl(s))
			u_1 = u_1.sub(v_1.shl(s))

			N_u_384.AddShifted(N_u_384, N_v_384, 2*s)
			N_u_384.SubShifted(N_u_384, p_384, s+1)
			p_384.SubShifted(p_384, N_v_384, s)
		} else {
			u_0 = u_0.add(v_0.shl(s))
			u_1 = u_1.add(v_1.shl(s))

			N_u_384.AddShifted(N_u_384, N_v_384, 2*s)
			N_u_384.AddShifted(N_u_384, p_384, s+1)
			p_384.AddShifted(p_384, N_v_384, s)
		}
	}
}
