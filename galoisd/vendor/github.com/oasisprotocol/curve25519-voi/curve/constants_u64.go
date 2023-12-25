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

//go:build (amd64 || arm64 || ppc64le || ppc64 || s390x || force64bit) && !force32bit

package curve

import "github.com/oasisprotocol/curve25519-voi/internal/field"

var (
	// ED25519_BASEPOINT_POINT is the Ed25519 basepoint as an EdwardsPoint.
	ED25519_BASEPOINT_POINT = newEdwardsPoint(
		field.NewElement51(
			1738742601995546,
			1146398526822698,
			2070867633025821,
			562264141797630,
			587772402128613,
		),
		field.NewElement51(
			1801439850948184,
			1351079888211148,
			450359962737049,
			900719925474099,
			1801439850948198,
		),
		field.NewElement51(1, 0, 0, 0, 0),
		field.NewElement51(
			1841354044333475,
			16398895984059,
			755974180946558,
			900171276175154,
			1821297809914039,
		),
	)

	// The 8-torsion subgroup (E[8]).
	EIGHT_TORSION = eightTorsionInnerDocHidden

	eightTorsionInnerDocHidden = [8]*EdwardsPoint{
		newEdwardsPoint(
			field.NewElement51(0, 0, 0, 0, 0),
			field.NewElement51(1, 0, 0, 0, 0),
			field.NewElement51(1, 0, 0, 0, 0),
			field.NewElement51(0, 0, 0, 0, 0),
		),
		newEdwardsPoint(
			field.NewElement51(
				358744748052810,
				1691584618240980,
				977650209285361,
				1429865912637724,
				560044844278676,
			),
			field.NewElement51(
				84926274344903,
				473620666599931,
				365590438845504,
				1028470286882429,
				2146499180330972,
			),
			field.NewElement51(1, 0, 0, 0, 0),
			field.NewElement51(
				1448326834587521,
				1857896831960481,
				1093722731865333,
				1677408490711241,
				1915505153018406,
			),
		),
		newEdwardsPoint(
			field.NewElement51(
				533094393274173,
				2016890930128738,
				18285341111199,
				134597186663265,
				1486323764102114,
			),
			field.NewElement51(0, 0, 0, 0, 0),
			field.NewElement51(1, 0, 0, 0, 0),
			field.NewElement51(0, 0, 0, 0, 0),
		),
		newEdwardsPoint(
			field.NewElement51(
				358744748052810,
				1691584618240980,
				977650209285361,
				1429865912637724,
				560044844278676,
			),
			field.NewElement51(
				2166873539340326,
				1778179147085316,
				1886209374839743,
				1223329526802818,
				105300633354275,
			),
			field.NewElement51(1, 0, 0, 0, 0),
			field.NewElement51(
				803472979097708,
				393902981724766,
				1158077081819914,
				574391322974006,
				336294660666841,
			),
		),
		newEdwardsPoint(
			field.NewElement51(0, 0, 0, 0, 0),
			field.NewElement51(
				2251799813685228,
				2251799813685247,
				2251799813685247,
				2251799813685247,
				2251799813685247,
			),
			field.NewElement51(1, 0, 0, 0, 0),
			field.NewElement51(0, 0, 0, 0, 0),
		),
		newEdwardsPoint(
			field.NewElement51(
				1893055065632419,
				560215195444267,
				1274149604399886,
				821933901047523,
				1691754969406571,
			),
			field.NewElement51(
				2166873539340326,
				1778179147085316,
				1886209374839743,
				1223329526802818,
				105300633354275,
			),
			field.NewElement51(1, 0, 0, 0, 0),
			field.NewElement51(
				1448326834587521,
				1857896831960481,
				1093722731865333,
				1677408490711241,
				1915505153018406,
			),
		),
		newEdwardsPoint(
			field.NewElement51(
				1718705420411056,
				234908883556509,
				2233514472574048,
				2117202627021982,
				765476049583133,
			),
			field.NewElement51(0, 0, 0, 0, 0),
			field.NewElement51(1, 0, 0, 0, 0),
			field.NewElement51(0, 0, 0, 0, 0),
		),
		newEdwardsPoint(
			field.NewElement51(
				1893055065632419,
				560215195444267,
				1274149604399886,
				821933901047523,
				1691754969406571,
			),
			field.NewElement51(
				84926274344903,
				473620666599931,
				365590438845504,
				1028470286882429,
				2146499180330972,
			),
			field.NewElement51(1, 0, 0, 0, 0),
			field.NewElement51(
				803472979097708,
				393902981724766,
				1158077081819914,
				574391322974006,
				336294660666841,
			),
		),
	}
)

// The value of minus one, equal to `-Element.One()`.
var constMINUS_ONE = field.NewElement51(
	2251799813685228,
	2251799813685247,
	2251799813685247,
	2251799813685247,
	2251799813685247,
)

// Edwards `d` value, equal to `-121665/121666 mod p`.
var constEDWARDS_D = field.NewElement51(
	929955233495203,
	466365720129213,
	1662059464998953,
	2033849074728123,
	1442794654840575,
)

// Edwards `2*d` value, equal to `2*(-121665/121666) mod p`.
var constEDWARDS_D2 = field.NewElement51(
	1859910466990425,
	932731440258426,
	1072319116312658,
	1815898335770999,
	633789495995903,
)

// One minus edwards `d` value squared, equal to `(1 - (-121665/121666) mod p) pow 2`.
var constONE_MINUS_EDWARDS_D_SQUARED = field.NewElement51(
	1136626929484150,
	1998550399581263,
	496427632559748,
	118527312129759,
	45110755273534,
)

// Edwards `d` value minus one squared, equal to `(((-121665/121666) mod p) - 1) pow 2`.
var constEDWARDS_D_MINUS_ONE_SQUARED = field.NewElement51(
	1507062230895904,
	1572317787530805,
	683053064812840,
	317374165784489,
	1572899562415810,
)

/// `= sqrt(a*d - 1)`, where `a = -1 (mod p)`, `d` are the Edwards curve parameters.
var constSQRT_AD_MINUS_ONE = field.NewElement51(
	2241493124984347,
	425987919032274,
	2207028919301688,
	1220490630685848,
	974799131293748,
)

// `= 1/sqrt(a-d)`, where `a = -1 (mod p)`, `d` are the Edwards curve parameters.
var constINVSQRT_A_MINUS_D = field.NewElement51(
	278908739862762,
	821645201101625,
	8113234426968,
	1777959178193151,
	2118520810568447,
)

// `[2^128]B`
var constB_SHL_128 = newEdwardsPoint(
	field.NewElement51(
		1694250497969519,
		2055043727391392,
		1405737588602752,
		386731588847155,
		1260687722930339,
	),
	field.NewElement51(
		1728883392172401,
		153276466962872,
		1116166591938235,
		1393379381570381,
		2065441269379165,
	),
	field.NewElement51(
		1207379401816749,
		171747509335926,
		1694497998968523,
		946669555871190,
		125787048234050,
	),
	field.NewElement51(
		810795410810128,
		1897850844826044,
		1781033826520412,
		744123856001677,
		1165149454940150,
	),
)
