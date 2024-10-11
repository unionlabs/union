// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build ignore

// The unusedwrite command runs the unusedwrite analyzer
// on the specified packages.
package main

import (
	"golang.org/x/tools/go/analysis/passes/unusedwrite"
	"golang.org/x/tools/go/analysis/singlechecker"
)

func main() { singlechecker.Main(unusedwrite.Analyzer) }
