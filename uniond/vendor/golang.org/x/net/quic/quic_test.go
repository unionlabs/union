// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"testing"
)

func testSides(t *testing.T, name string, f func(*testing.T, connSide)) {
	if name != "" {
		name += "/"
	}
	t.Run(name+"server", func(t *testing.T) { f(t, serverSide) })
	t.Run(name+"client", func(t *testing.T) { f(t, clientSide) })
}

func testStreamTypes(t *testing.T, name string, f func(*testing.T, streamType)) {
	if name != "" {
		name += "/"
	}
	t.Run(name+"bidi", func(t *testing.T) { f(t, bidiStream) })
	t.Run(name+"uni", func(t *testing.T) { f(t, uniStream) })
}

func testSidesAndStreamTypes(t *testing.T, name string, f func(*testing.T, connSide, streamType)) {
	if name != "" {
		name += "/"
	}
	t.Run(name+"server/bidi", func(t *testing.T) { f(t, serverSide, bidiStream) })
	t.Run(name+"client/bidi", func(t *testing.T) { f(t, clientSide, bidiStream) })
	t.Run(name+"server/uni", func(t *testing.T) { f(t, serverSide, uniStream) })
	t.Run(name+"client/uni", func(t *testing.T) { f(t, clientSide, uniStream) })
}
