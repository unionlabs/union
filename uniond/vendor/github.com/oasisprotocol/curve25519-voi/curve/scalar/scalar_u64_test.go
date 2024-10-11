// Copyright (c) 2016-2019 isis agora lovecruft. All rights reserved.
// Copyright (c) 2016-2019 Henry de Valence. All rights reserved.
// Copyright (c) 2020-2021 Oasis Labs Inc.  All rights reserved.
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
		0x000fffffffffffff, 0x000fffffffffffff, 0x000fffffffffffff, 0x000fffffffffffff,
		0x00001fffffffffff,
	},

	// x^2 = 3078544782642840487852506753550082162405942681916160040940637093560259278169 mod l
	"XX": {
		0x0001668020217559, 0x000531640ffd0ec0, 0x00085fd6f9f38a31, 0x000c268f73bb1cf4,
		0x000006ce65046df0,
	},

	// x^2 = 2912514428060642753613814151688322857484807845836623976981729207238463947987*R mod l in Montgomery form
	"XX_MONT": {
		0x000c754eea569a5c, 0x00063b6ed36cb215, 0x0008ffa36bf25886, 0x000e9183614e7543,
		0x0000061db6c6f26f,
	},

	// y = 6145104759870991071742105800796537629880401874866217824609283457819451087098
	"Y": {
		0x000b75071e1458fa, 0x000bf9d75e1ecdac, 0x000433d2baf0672b, 0x0005fffcc11fad13,
		0x00000d96018bb825,
	},

	// x*y = 36752150652102274958925982391442301741
	"XY": {
		0x000ee6d76ba7632d, 0x000ed50d71d84e02, 0x00000000001ba634, 0x0000000000000000,
		0x0000000000000000,
	},

	// x*y = 3783114862749659543382438697751927473898937741870308063443170013240655651591*R mod l in Montgomery form
	"XY_MONT": {
		0x0006d52bf200cfd5, 0x00033fb1d7021570, 0x000f201bc07139d8, 0x0001267e3e49169e,
		0x000007b839c00268,
	},

	// a = 2351415481556538453565687241199399922945659411799870114962672658845158063753
	"A": {
		0x0005236c07b3be89, 0x0001bc3d2a67c0c4, 0x000a4aa782aae3ee, 0x0006b3f6e4fec4c4,
		0x00000532da9fab8c,
	},

	// b = 4885590095775723760407499321843594317911456947580037491039278279440296187236
	"B": {
		0x000d3fae55421564, 0x000c2df24f65a4bc, 0x0005b5587d69fb0b, 0x00094c091b013b3b,
		0x00000acd25605473,
	},

	// a+b = 0
	// a-b = 4702830963113076907131374482398799845891318823599740229925345317690316127506
	"AB": {
		0x000a46d80f677d12, 0x0003787a54cf8188, 0x0004954f0555c7dc, 0x000d67edc9fd8989,
		0x00000a65b53f5718,
	},

	// c = (2^512 - 1) % l = 1627715501170711445284395025044413883736156588369414752970002579683115011840
	"C": {
		0x000611e3449c0f00, 0x000a768859347a40, 0x0007f5be65d00e1b, 0x0009a3dceec73d21,
		0x00000399411b7c30,
	},
}
