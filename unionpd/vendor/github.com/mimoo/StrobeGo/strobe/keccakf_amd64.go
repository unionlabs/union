// Copyright 2015 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// +build amd64,!appengine,!gccgo

package strobe

// This function is implemented in keccakf_amd64.s.

//go:noescape

func keccakF1600(a *[25]uint64, nr int)
