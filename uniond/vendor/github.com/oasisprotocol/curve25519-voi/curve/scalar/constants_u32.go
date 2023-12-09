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

// `L` is the order of base point, i.e. 2^252 + 27742317777372353535851937790883648493.
var constL unpackedScalar = unpackedScalar{
	0x1cf5d3ed, 0x009318d2, 0x1de73596, 0x1df3bd45,
	0x0000014d, 0x00000000, 0x00000000, 0x00000000,
	0x00100000,
}

// `R` = R % L where R = 2^261.
var constR unpackedScalar = unpackedScalar{
	0x114df9ed, 0x1a617303, 0x0f7c098c, 0x16793167,
	0x1ffd656e, 0x1fffffff, 0x1fffffff, 0x1fffffff,
	0x000fffff,
}

// `RR` = (R^2) % L where R = 2^261.
var constRR = unpackedScalar{
	0x0b5f9d12, 0x1e141b17, 0x158d7f3d, 0x143f3757,
	0x1972d781, 0x042feb7c, 0x1ceec73d, 0x1e184d1e,
	0x0005046d,
}

// `L` * `LFACTOR` = -1 (mod 2^29)
const constLFACTOR uint32 = 0x12547e1b
