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

//go:build (386 || arm || mips || mipsle || wasm || mips64le || mips64 || riscv64 || loong64 || force32bit) && !force64bit

package elligator

import "github.com/oasisprotocol/curve25519-voi/internal/field"

var (
	// A = 486662
	constMONTGOMERY_A = field.NewElement2625(486662, 0, 0, 0, 0, 0, 0, 0, 0, 0)

	// NEG_A = -A
	constMONTGOMERY_NEG_A = field.NewElement2625(
		66622183, 33554431, 67108863, 33554431, 67108863, 33554431, 67108863, 33554431, 67108863,
		33554431,
	)

	// A_SQUARED = A^2
	constMONTGOMERY_A_SQUARED = field.NewElement2625(12721188, 3529, 0, 0, 0, 0, 0, 0, 0, 0)

	// SQRT_NEG_A_PLUS_TWO = sqrt(-(A+2))
	constMONTGOMERY_SQRT_NEG_A_PLUS_TWO = field.NewElement2625(
		54885894, 25242303, 55597453, 9067496, 51808079, 33312638, 25456129, 14121551, 54921728,
		3972023,
	)

	// U_FACTOR = -2 * sqrt(-1)
	constMONTGOMERY_U_FACTOR = field.NewElement2625(
		65191565, 15887450, 48352964, 26553601, 42329919, 544945, 50292418, 4011308, 66455492,
		10741467,
	)

	// V_FACTOR = sqrt(U_FACTOR)
	constMONTGOMERY_V_FACTOR = field.NewElement2625(
		32595774, 7943725, 57730914, 30054016, 54719391, 272472, 25146209, 2005654, 66782178,
		22147949,
	)
)
