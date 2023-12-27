// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package constants

const (
	// oneIf64Bit is 1 on 64-bit platforms and 0 on 32-bit platforms.
	oneIf64Bit = ^uint(0) >> 63

	// MaxUint32OrInt returns min(MaxUint32, MaxInt), i.e
	// - MaxUint32 on 64-bit platforms;
	// - MaxInt on 32-bit platforms.
	// It is used when slices are limited to Uint32 on 64-bit platforms (the
	// length limit for slices is naturally MaxInt on 32-bit platforms).
	MaxUint32OrInt = (1<<31)<<oneIf64Bit - 1
)
