// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBitArray_Reverse() {
	ba := bitarray.MustParse("1100-1111 0000-1010")

	fmt.Printf("% b\n", ba.Reverse())

	// Output:
	// 01010000 11110011
}

func ExampleBitArray_ShiftLeft() {
	ba := bitarray.MustParse("1100-1111 0000-1010 11")

	fmt.Printf("% b\n", ba.ShiftLeft(1))
	fmt.Printf("% b\n", ba.ShiftLeft(8))
	fmt.Printf("% b\n", ba.ShiftLeft(-5))
	fmt.Printf("% b\n", ba.ShiftLeft(0))

	// Output:
	// 10011110 00010101 10
	// 00001010 11000000 00
	// 00000110 01111000 01
	// 11001111 00001010 11
}

func ExampleBitArray_RotateLeft() {
	ba := bitarray.MustParse("1100-1111 0000-1010 11")

	fmt.Printf("% b\n", ba.RotateLeft(1))
	fmt.Printf("% b\n", ba.RotateLeft(8))
	fmt.Printf("% b\n", ba.RotateLeft(-5))
	fmt.Printf("% b\n", ba.RotateLeft(0))

	// Output:
	// 10011110 00010101 11
	// 00001010 11110011 11
	// 01011110 01111000 01
	// 11001111 00001010 11
}
