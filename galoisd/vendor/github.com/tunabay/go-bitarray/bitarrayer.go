// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

// BitArrayer is an interface implemented by any type that can be treated as a
// BitArray. Within this package, BitArray itself, Builder and Buffer implement
// this interface.
//
// BitArray returns the value of itself converted to a BitArray. Note that for
// non-immutable types, multiple calls may return different values. It is legal
// to return nil to represent an empty BitArray, and it should be treated the
// same as a zero-length BitArray.
type BitArrayer interface {
	BitArray() *BitArray
}
