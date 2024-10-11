// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build ignore
// +build ignore

package main

import (
	"golang.org/x/tools/go/analysis/singlechecker"
	"golang.org/x/tools/go/analysis/passes/stdversion"
)

func main() { singlechecker.Main(stdversion.Analyzer) }
