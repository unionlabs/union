// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build ignore

// The printf command applies the printf checker to the specified
// packages of Go source code.
//
// Run with:
//
//	$ go run ./go/analysis/passes/printf/main.go -- packages...
package main

import (
	"golang.org/x/tools/go/analysis/passes/printf"
	"golang.org/x/tools/go/analysis/singlechecker"
)

func main() { singlechecker.Main(printf.Analyzer) }
