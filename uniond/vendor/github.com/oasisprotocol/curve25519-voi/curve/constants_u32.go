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

//go:build (386 || arm || mips || mipsle || wasm || mips64le || mips64 || riscv64 || loong64 || force32bit) && !force64bit

package curve

import "github.com/oasisprotocol/curve25519-voi/internal/field"

var (
	// ED25519_BASEPOINT_POINT is the Ed25519 basepoint as an EdwardsPoint.
	ED25519_BASEPOINT_POINT = newEdwardsPoint(
		field.NewElement2625(
			52811034, 25909283, 16144682, 17082669, 27570973, 30858332, 40966398, 8378388, 20764389,
			8758491,
		),
		field.NewElement2625(
			40265304, 26843545, 13421772, 20132659, 26843545, 6710886, 53687091, 13421772, 40265318,
			26843545,
		),
		field.NewElement2625(1, 0, 0, 0, 0, 0, 0, 0, 0, 0),
		field.NewElement2625(
			28827043, 27438313, 39759291, 244362, 8635006, 11264893, 19351346, 13413597, 16611511,
			27139452,
		),
	)

	// The 8-torsion subgroup (E8]).
	EIGHT_TORSION = eightTorsionInnerDocHidden

	eightTorsionInnerDocHidden = [8]*EdwardsPoint{
		newEdwardsPoint(
			field.NewElement2625(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
			field.NewElement2625(1, 0, 0, 0, 0, 0, 0, 0, 0, 0),
			field.NewElement2625(1, 0, 0, 0, 0, 0, 0, 0, 0, 0),
			field.NewElement2625(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
		),
		newEdwardsPoint(
			field.NewElement2625(
				21352778, 5345713, 4660180, 25206575, 24143089, 14568123, 30185756, 21306662, 33579924,
				8345318,
			),
			field.NewElement2625(
				6952903, 1265500, 60246523, 7057497, 4037696, 5447722, 35427965, 15325401, 19365852,
				31985330,
			),
			field.NewElement2625(1, 0, 0, 0, 0, 0, 0, 0, 0, 0),
			field.NewElement2625(
				41846657, 21581751, 11716001, 27684820, 48915701, 16297738, 20670665, 24995334,
				3541542, 28543251,
			),
		),
		newEdwardsPoint(
			field.NewElement2625(
				32595773, 7943725, 57730914, 30054016, 54719391, 272472, 25146209, 2005654, 66782178,
				22147949,
			),
			field.NewElement2625(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
			field.NewElement2625(1, 0, 0, 0, 0, 0, 0, 0, 0, 0),
			field.NewElement2625(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
		),
		newEdwardsPoint(
			field.NewElement2625(
				21352778, 5345713, 4660180, 25206575, 24143089, 14568123, 30185756, 21306662, 33579924,
				8345318,
			),
			field.NewElement2625(
				60155942, 32288931, 6862340, 26496934, 63071167, 28106709, 31680898, 18229030,
				47743011, 1569101,
			),
			field.NewElement2625(1, 0, 0, 0, 0, 0, 0, 0, 0, 0),
			field.NewElement2625(
				25262188, 11972680, 55392862, 5869611, 18193162, 17256693, 46438198, 8559097, 63567321,
				5011180,
			),
		),
		newEdwardsPoint(
			field.NewElement2625(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
			field.NewElement2625(
				67108844, 33554431, 67108863, 33554431, 67108863, 33554431, 67108863, 33554431,
				67108863, 33554431,
			),
			field.NewElement2625(1, 0, 0, 0, 0, 0, 0, 0, 0, 0),
			field.NewElement2625(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
		),
		newEdwardsPoint(
			field.NewElement2625(
				45756067, 28208718, 62448683, 8347856, 42965774, 18986308, 36923107, 12247769,
				33528939, 25209113,
			),
			field.NewElement2625(
				60155942, 32288931, 6862340, 26496934, 63071167, 28106709, 31680898, 18229030,
				47743011, 1569101,
			),
			field.NewElement2625(1, 0, 0, 0, 0, 0, 0, 0, 0, 0),
			field.NewElement2625(
				41846657, 21581751, 11716001, 27684820, 48915701, 16297738, 20670665, 24995334,
				3541542, 28543251,
			),
		),
		newEdwardsPoint(
			field.NewElement2625(
				34513072, 25610706, 9377949, 3500415, 12389472, 33281959, 41962654, 31548777, 326685,
				11406482,
			),
			field.NewElement2625(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
			field.NewElement2625(1, 0, 0, 0, 0, 0, 0, 0, 0, 0),
			field.NewElement2625(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
		),
		newEdwardsPoint(
			field.NewElement2625(
				45756067, 28208718, 62448683, 8347856, 42965774, 18986308, 36923107, 12247769,
				33528939, 25209113,
			),
			field.NewElement2625(
				6952903, 1265500, 60246523, 7057497, 4037696, 5447722, 35427965, 15325401, 19365852,
				31985330,
			),
			field.NewElement2625(1, 0, 0, 0, 0, 0, 0, 0, 0, 0),
			field.NewElement2625(
				25262188, 11972680, 55392862, 5869611, 18193162, 17256693, 46438198, 8559097, 63567321,
				5011180,
			),
		),
	}
)

// The value of minus one, equal to `-Element.One()`.
var constMINUS_ONE = field.NewElement2625(
	67108844, 33554431, 67108863, 33554431, 67108863, 33554431, 67108863, 33554431, 67108863, 33554431,
)

// Edwards `d` value, equal to `-121665/121666 mod p`.
var constEDWARDS_D = field.NewElement2625(
	56195235, 13857412, 51736253, 6949390, 114729, 24766616, 60832955, 30306712, 48412415, 21499315,
)

// Edwards `2*d` value, equal to `2*(-121665/121666) mod p`.
var constEDWARDS_D2 = field.NewElement2625(
	45281625, 27714825, 36363642, 13898781, 229458, 15978800, 54557047, 27058993, 29715967, 9444199,
)

// One minus edwards `d` value squared, equal to `(1 - (-121665/121666) mod p) pow 2`.
var constONE_MINUS_EDWARDS_D_SQUARED = field.NewElement2625(
	6275446, 16937061, 44170319, 29780721, 11667076, 7397348, 39186143, 1766194, 42675006, 672202,
)

// Edwards `d` value minus one squared, equal to `(((-121665/121666) mod p) - 1) pow 2`.
var constEDWARDS_D_MINUS_ONE_SQUARED = field.NewElement2625(
	15551776, 22456977, 53683765, 23429360, 55212328, 10178283, 40474537, 4729243, 61826754, 23438029,
)

/// `= sqrt(a*d - 1)`, where `a = -1 (mod p)`, `d` are the Edwards curve parameters.
var constSQRT_AD_MINUS_ONE = field.NewElement2625(
	24849947, 33400850, 43495378, 6347714, 46036536, 32887293, 41837720, 18186727, 66238516, 14525638,
)

// `= 1/sqrt(a-d)`, where `a = -1 (mod p)`, `d` are the Edwards curve parameters.
var constINVSQRT_A_MINUS_D = field.NewElement2625(
	6111466, 4156064, 39310137, 12243467, 41204824, 120896, 20826367, 26493656, 6093567, 31568420,
)

// `[2^128]B`
var constB_SHL_128 = newEdwardsPoint(
	field.NewElement2625(
		51875183, 25246299, 56523424, 30622537, 27113344, 20947122, 49940019, 5762749,
		65396899, 18785710,
	),
	field.NewElement2625(
		7524721, 25762370, 22913464, 2283997, 20512443, 16632178, 51604301, 20762970,
		19558493, 30777473,
	),
	field.NewElement2625(
		5946029, 17991355, 21559158, 2559237, 55383755, 25249987, 43576790, 14106475,
		5491778, 1874373,
	),
	field.NewElement2625(
		7496976, 12081793, 24201660, 28280181, 9440604, 26539472, 35330701, 11088309,
		56571894, 17362079,
	),
)
