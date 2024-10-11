// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// +build no

package a

// want +1 `misplaced \+build comment`
// +build toolate

// want +1 `misplaced //go:build comment`
//go:build toolate

var _ = `
// +build notacomment
`
