// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"errors"
)

// ErrIllegalExpression is an error thrown when a string representation of
// bit array is not a legal format.
var ErrIllegalExpression = errors.New("illegal bit array expression")

// ErrFractionalBitsBeforeEOF is an error thrown when a byte-oriented reading
// method reaches io.EOF but there are still fractional bits less than 8 bits
// that cannot be read in bytes.
var ErrFractionalBitsBeforeEOF = errors.New("fractional bits before EOF")
