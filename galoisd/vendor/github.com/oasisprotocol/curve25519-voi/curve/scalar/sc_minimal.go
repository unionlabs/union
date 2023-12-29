// Copyright (c) 2016 The Go Authors. All rights reserved.
// Copyright (c) 2019-2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//   * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//    * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//    * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package scalar

import "encoding/binary"

// order is the order of Curve25519 in little-endian form.
var order = func() [4]uint64 {
	var orderBytes [ScalarSize]byte
	_ = BASEPOINT_ORDER.ToBytes(orderBytes[:])

	var ret [4]uint64
	for i := range ret {
		ret[i] = binary.LittleEndian.Uint64(orderBytes[i*8 : (i+1)*8])
	}

	return ret
}()

// ScMinimalVartime returns true if the given byte-encoded scalar is
// less than the order of the curve, in variable-time.
//
// This method is intended for verification applications, and is
// significantly faster than deserializing the scalar and calling
// IsCanonical.
func ScMinimalVartime(scalar []byte) bool {
	if scalar[31]&240 == 0 {
		// 4 most significant bits unset, succeed fast
		return true
	}
	if scalar[31]&224 != 0 {
		// Any of the 3 most significant bits set, fail fast
		return false
	}

	// 4th most significant bit set (unlikely), actually check vs order
	for i := 3; ; i-- {
		v := binary.LittleEndian.Uint64(scalar[i*8:])
		if v > order[i] {
			return false
		} else if v < order[i] {
			break
		} else if i == 0 {
			return false
		}
	}

	return true
}
