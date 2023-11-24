// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package manifest

import "fmt"

const (
	// 3 bits are necessary to represent level values from 0-6.
	levelBits = 3
	levelMask = (1 << levelBits) - 1
	// invalidSublevel denotes an invalid or non-applicable sublevel.
	invalidSublevel = -1
)

// Level encodes a level and optional sublevel for use in log and error
// messages. The encoding has the property that Level(0) ==
// L0Sublevel(invalidSublevel).
type Level uint32

func makeLevel(level, sublevel int) Level {
	return Level(((sublevel + 1) << levelBits) | level)
}

// LevelToInt returns the int representation of a Level
func LevelToInt(l Level) int {
	return int(l) & levelMask
}

// L0Sublevel returns a Level representing the specified L0 sublevel.
func L0Sublevel(sublevel int) Level {
	if sublevel < 0 {
		panic(fmt.Sprintf("invalid L0 sublevel: %d", sublevel))
	}
	return makeLevel(0, sublevel)
}

func (l Level) String() string {
	level := int(l) & levelMask
	sublevel := (int(l) >> levelBits) - 1
	if sublevel != invalidSublevel {
		return fmt.Sprintf("L%d.%d", level, sublevel)
	}
	return fmt.Sprintf("L%d", level)
}
