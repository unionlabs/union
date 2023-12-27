// Copyright (c) 2016-2017 isis agora lovecruft. All rights reserved.
// Copyright (c) 2016-2017 Henry de Valence. All rights reserved.
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

package subtle

import "crypto/subtle"

func ConstantTimeCompareByte(a, b byte) int {
	return subtle.ConstantTimeByteEq(a, b)
}

func ConstantTimeCompareBytes(a, b []byte) int {
	return subtle.ConstantTimeCompare(a, b)
}

func ConstantTimeSelectByte(choice int, a, b byte) byte {
	return byte(subtle.ConstantTimeSelect(choice, int(a), int(b)))
}

func ConstantTimeSelectUint64(choice int, a, b uint64) uint64 {
	mask := uint64(-choice)
	return b ^ (mask & (a ^ b))
}

func ConstantTimeSwapUint64(choice int, a, b *uint64) {
	mask := uint64(-choice)
	t := mask & (*a ^ *b)
	*a ^= t
	*b ^= t
}

func ConstantTimeSelectUint32(choice int, a, b uint32) uint32 {
	mask := uint32(-choice)
	return b ^ (mask & (a ^ b))
}

func ConstantTimeSwapUint32(choice int, a, b *uint32) {
	mask := uint32(-choice)
	t := mask & (*a ^ *b)
	*a ^= t
	*b ^= t
}
