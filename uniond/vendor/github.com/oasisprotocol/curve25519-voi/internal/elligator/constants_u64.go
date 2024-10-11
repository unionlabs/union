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

//go:build (amd64 || arm64 || ppc64le || ppc64 || s390x || force64bit) && !force32bit

package elligator

import "github.com/oasisprotocol/curve25519-voi/internal/field"

var (
	// A = 486662
	constMONTGOMERY_A = field.NewElement51(486662, 0, 0, 0, 0)

	// NEG_A = -A
	constMONTGOMERY_NEG_A = field.NewElement51(
		2251799813198567,
		2251799813685247,
		2251799813685247,
		2251799813685247,
		2251799813685247,
	)

	// A_SQUARED = A^2
	constMONTGOMERY_A_SQUARED = field.NewElement51(236839902244, 0, 0, 0, 0)

	// SQRT_NEG_A_PLUS_TWO = sqrt(-(A+2))
	constMONTGOMERY_SQRT_NEG_A_PLUS_TWO = field.NewElement51(
		1693982333959686,
		608509411481997,
		2235573344831311,
		947681270984193,
		266558006233600,
	)

	// U_FACTOR = -2 * sqrt(-1)
	constMONTGOMERY_U_FACTOR = field.NewElement51(
		1066188786548365,
		1781982046572228,
		36570682222399,
		269194373326530,
		720847714518980,
	)

	// V_FACTOR = sqrt(U_FACTOR)
	constMONTGOMERY_V_FACTOR = field.NewElement51(
		533094393274174,
		2016890930128738,
		18285341111199,
		134597186663265,
		1486323764102114,
	)
)
