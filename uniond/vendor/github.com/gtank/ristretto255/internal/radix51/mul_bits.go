// Copyright (c) 2019 George Tankersley. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// +build go1.12

package radix51

import "math/bits"

// madd64 multiples two 64-bit numbers and adds them to a split 128-bit accumulator.
func madd64(lo, hi, a, b uint64) (ol uint64, oh uint64) {
	oh, ol = bits.Mul64(a, b)
	var c uint64
	ol, c = bits.Add64(ol, lo, 0)
	oh, _ = bits.Add64(oh, hi, c)
	return
}
