// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package fastrand

import _ "unsafe" // required by go:linkname

// Uint32 returns a lock free uint32 value.
//
//go:linkname Uint32 runtime.fastrand
func Uint32() uint32

// Uint32n returns a lock free uint32 value in the interval [0, n).
//
//go:linkname Uint32n runtime.fastrandn
func Uint32n(n uint32) uint32
