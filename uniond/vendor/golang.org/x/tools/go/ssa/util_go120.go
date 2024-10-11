// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.20
// +build go1.20

package ssa

import (
	"go/ast"
	"go/token"
)

func init() {
	rangePosition = func(rng *ast.RangeStmt) token.Pos { return rng.Range }
}
