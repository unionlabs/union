// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// This file contains tests for the stringintconv checker.

package a

type A string

type B = string

type C int

type D = uintptr

func StringTest() {
	var (
		i int
		j rune
		k byte
		l C
		m D
		n = []int{0, 1, 2}
		o struct{ x int }
	)
	const p = 0
	// First time only, assert the complete message:
	_ = string(i) // want `^conversion from int to string yields a string of one rune, not a string of digits$`
	_ = string(j)
	_ = string(k)
	_ = string(p)    // want `...from untyped int to string...`
	_ = A(l)         // want `...from C \(int\) to A \(string\)...`
	_ = B(m)         // want `...from (uintptr|D \(uintptr\)) to B \(string\)...`
	_ = string(n[1]) // want `...from int to string...`
	_ = string(o.x)  // want `...from int to string...`
}
