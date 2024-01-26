// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"errors"
)

// BreakIteration is used as a return value from IterateFunc to indicate the
// current iteration is to be terminated without error.
//nolint:revive,stylecheck,errname // this is not an error
var BreakIteration = errors.New("break iteration")

// IterateFunc is the type of the function called by Iterate to process each bit
// of a BitArray. The i argument contains the offset from the beginning. The b
// argument is the actual bit value, 0 or 1.
//
// If the function returns the special error BreakIteration, Iterate breaks the
// current iteration without error. Otherwise, if the function returns a non-nil
// error, Iterate stops the current iteration and returns that error.
type IterateFunc func(i, b int) error

// Iterate iterates calling the function fn for each bit in order from the
// beginning of the bit array. Iterate returns an error only if the function fn
// returns an error that is not BreakIteration. Otherwise, it returns nil after
// calling fn for the last bit.
func (ba *BitArray) Iterate(fn IterateFunc) error {
	switch {
	case ba.IsZero():
		return nil
	case ba.b == nil:
		for i := 0; i < ba.nBits; i++ {
			if err := fn(i, 0); err != nil {
				if errors.Is(err, BreakIteration) {
					return nil
				}
				return err
			}
		}
		return nil
	}
	for i := 0; i < ba.nBits; i++ {
		b := int(ba.b[i>>3] >> (7 - i&7) & 1)
		if err := fn(i, b); err != nil {
			if errors.Is(err, BreakIteration) {
				return nil
			}
			return err
		}
	}

	return nil
}
