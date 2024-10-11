// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package c

import (
	"./a"
	"./b"
)

type c[V any] = struct {
	G b.B[[3]V]
}

var S struct{ F int } = a.B{}
var T struct{ F int } = a.F()

var U a.A[string] = (*string)(nil)
var V a.A[int] = (*int)(nil)

var W b.B[string] = struct{ F *[]string }{}
var X b.B[int] = struct{ F *[]int }{}

var Y c[string] = struct{ G struct{ F *[][3]string } }{}
var Z c[int] = struct{ G struct{ F *[][3]int } }{}
