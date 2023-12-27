// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build !cgo
// +build !cgo

package manual

// Provides versions of New and Free when cgo is not available (e.g. cross
// compilation).

// New allocates a slice of size n.
func New(n int) []byte {
	return make([]byte, n)
}

// Free frees the specified slice.
func Free(b []byte) {
}
