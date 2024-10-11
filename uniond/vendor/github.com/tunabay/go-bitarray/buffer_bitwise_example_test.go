// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBuffer_ToggleBitAt() {
	ba := bitarray.MustParse("110010")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf)
	buf.ToggleBitAt(1)
	buf.ToggleBitAt(3)
	buf.ToggleBitAt(5)
	fmt.Println(buf)

	// Output:
	// 110010
	// 100111
}

func ExampleBuffer_ToggleBitsAt() {
	ba := bitarray.MustParse("11110000")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf)
	buf.ToggleBitsAt(2, 4)
	fmt.Println(buf)

	// Output:
	// 11110000
	// 11001100
}

func ExampleBuffer_AndAt() {
	ba := bitarray.MustParse("11110000")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf)
	buf.AndAt(2, bitarray.MustParse("0110"))
	fmt.Println(buf)

	// Output:
	// 11110000
	// 11010000
}

func ExampleBuffer_OrAt() {
	ba := bitarray.MustParse("11110000")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf)
	buf.OrAt(2, bitarray.MustParse("0110"))
	fmt.Println(buf)

	// Output:
	// 11110000
	// 11111000
}

func ExampleBuffer_XorAt() {
	ba := bitarray.MustParse("11110000")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf)
	buf.XorAt(2, bitarray.MustParse("0110"))
	fmt.Println(buf)

	// Output:
	// 11110000
	// 11101000
}

func ExampleBuffer_LeadingZeros() {
	ba := bitarray.MustParse("11110000")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Printf("%b: %d\n", buf, buf.LeadingZeros())
	buf.ToggleBitsAt(0, 2)
	fmt.Printf("%b: %d\n", buf, buf.LeadingZeros())

	// Output:
	// 11110000: 0
	// 00110000: 2
}

func ExampleBuffer_TrailingZeros() {
	ba := bitarray.MustParse("11110000")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Printf("%b: %d\n", buf, buf.TrailingZeros())
	buf.ToggleBitsAt(6, 2)
	fmt.Printf("%b: %d\n", buf, buf.TrailingZeros())

	// Output:
	// 11110000: 4
	// 11110011: 0
}

func ExampleBuffer_OnesCount() {
	ba := bitarray.MustParse("00111100")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Printf("%b: %d\n", buf, buf.OnesCount())
	buf.ToggleBitsAt(0, 6)
	fmt.Printf("%b: %d\n", buf, buf.OnesCount())

	// Output:
	// 00111100: 4
	// 11000000: 2
}
