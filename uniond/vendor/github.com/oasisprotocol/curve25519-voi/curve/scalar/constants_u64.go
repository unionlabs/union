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

package scalar

// `L` is the order of base point, i.e. 2^252 + 27742317777372353535851937790883648493.
var constL unpackedScalar = unpackedScalar{
	0x0002631a5cf5d3ed,
	0x000dea2f79cd6581,
	0x000000000014def9,
	0x0000000000000000,
	0x0000100000000000,
}

// `R` = R % L where R = 2^260.
var constR unpackedScalar = unpackedScalar{
	0x000f48bd6721e6ed,
	0x0003bab5ac67e45a,
	0x000fffffeb35e51b,
	0x000fffffffffffff,
	0x00000fffffffffff,
}

// `RR` = (R^2) % L where R = 2^260.
var constRR = unpackedScalar{
	0x0009d265e952d13b,
	0x000d63c715bea69f,
	0x0005be65cb687604,
	0x0003dceec73d217f,
	0x000009411b7c309a,
}

// `L` * `LFACTOR` = -1 (mod 2^52).
const constLFACTOR uint64 = 0x51da312547e1b
