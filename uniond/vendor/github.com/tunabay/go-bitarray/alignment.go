// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

// Alignment is used in some bitwise operations to specify whether the bits are
// left-aligned or right-aligned. The zero value is AlignLeft.
type Alignment bool

const (
	AlignLeft  Alignment = false
	AlignRight Alignment = true
)

// String returns the string representation of Alignment.
func (a Alignment) String() string {
	if a == AlignLeft {
		return "align-left"
	}
	return "align-right"
}
