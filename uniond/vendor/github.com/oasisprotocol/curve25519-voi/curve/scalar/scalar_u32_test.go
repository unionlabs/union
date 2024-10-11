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

package scalar

// Having a way to declare file scoped globals will be really nice,
// since the packed tests also use some of these variable names.
var unpackedTestConstants = map[string]*unpackedScalar{
	// Note: x is 2^253-1 which is slightly larger than the largest scalar produced by
	// this implementation (l-1), and should verify there are no overflows for valid scalars
	//
	// x = 2^253-1 = 14474011154664524427946373126085988481658748083205070504932198000989141204991
	// x = 7237005577332262213973186563042994240801631723825162898930247062703686954002 mod l
	// x = 5147078182513738803124273553712992179887200054963030844803268920753008712037*R mod l in Montgomery form
	"X": {
		0x1fffffff, 0x1fffffff, 0x1fffffff, 0x1fffffff,
		0x1fffffff, 0x1fffffff, 0x1fffffff, 0x1fffffff,
		0x001fffff,
	},

	// x^2 = 3078544782642840487852506753550082162405942681916160040940637093560259278169 mod l
	"XX": {
		0x00217559, 0x000b3401, 0x103ff43b, 0x1462a62c,
		0x1d6f9f38, 0x18e7a42f, 0x09a3dcee, 0x008dbe18,
		0x0006ce65,
	},

	// x^2 = 2912514428060642753613814151688322857484807845836623976981729207238463947987*R mod l in Montgomery form
	"XX_MONT": {
		0x152b4d2e, 0x0571d53b, 0x1da6d964, 0x188663b6,
		0x1d1b5f92, 0x19d50e3f, 0x12306c29, 0x0c6f26fe,
		0x00030edb,
	},

	// y = 6145104759870991071742105800796537629880401874866217824609283457819451087098
	"Y": {
		0x1e1458fa, 0x165ba838, 0x1d787b36, 0x0e577f3a,
		0x1d2baf06, 0x1d689a19, 0x1fff3047, 0x117704ab,
		0x000d9601,
	},

	// x*y = 36752150652102274958925982391442301741
	"XY": {
		0x0ba7632d, 0x017736bb, 0x15c76138, 0x0c69daa1,
		0x000001ba, 0x00000000, 0x00000000, 0x00000000,
		0x00000000,
	},

	// x*y = 3783114862749659543382438697751927473898937741870308063443170013240655651591*R mod l in Montgomery form
	"XY_MONT": {
		0x077b51e1, 0x1c64e119, 0x02a19ef5, 0x18d2129e,
		0x00de0430, 0x045a7bc8, 0x04cfc7c9, 0x1c002681,
		0x000bdc1c,
	},

	// a = 2351415481556538453565687241199399922945659411799870114962672658845158063753
	"A": {
		0x07b3be89, 0x02291b60, 0x14a99f03, 0x07dc3787,
		0x0a782aae, 0x16262525, 0x0cfdb93f, 0x13f5718d,
		0x000532da,
	},

	// b = 4885590095775723760407499321843594317911456947580037491039278279440296187236
	"B": {
		0x15421564, 0x1e69fd72, 0x093d9692, 0x161785be,
		0x1587d69f, 0x09d9dada, 0x130246c0, 0x0c0a8e72,
		0x000acd25,
	},

	// a+b = 0
	// a-b = 4702830963113076907131374482398799845891318823599740229925345317690316127506
	"AB": {
		0x0f677d12, 0x045236c0, 0x09533e06, 0x0fb86f0f,
		0x14f0555c, 0x0c4c4a4a, 0x19fb727f, 0x07eae31a,
		0x000a65b5,
	},

	// c = (2^512 - 1) % l = 1627715501170711445284395025044413883736156588369414752970002579683115011840
	"C": {
		0x049c0f00, 0x00308f1a, 0x0164d1e9, 0x1c374ed1,
		0x1be65d00, 0x19e90bfa, 0x08f73bb1, 0x036f8613,
		0x00039941,
	},
}
